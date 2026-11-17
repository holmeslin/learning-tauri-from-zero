# async command 裡的狀態

## 本集目標

在 async command 裡讀寫狀態：知道為什麼直接改成 async 會編譯失敗，並改用 `tauri::async_runtime::Mutex` 解決。

## 正文

### 直接改成 async 會失敗

要等的工作該寫成 async command（第 3 章第 8 集）。把上一集的 `increment` 改一下：狀態簡化成一個 `u32`，加一之前先等半秒，假裝在做耗時的事。等待用 `tokio::time::sleep`，tokio 在第 3 章第 8 集已經加進專案了。照直覺，只多寫 `async` 與 `.await`；`async fn` 的參數要標生命週期，所以 `State` 寫成 `State<'_, …>`：

`src-tauri/src/lib.rs`：

```rust
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;

#[tauri::command]
async fn slow_increment(state: State<'_, Mutex<u32>>) -> u32 {
    let mut count = state.lock().unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    *count += 1;
    *count
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(0u32))
        .invoke_handler(tauri::generate_handler![slow_increment])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
error[E0277]: async commands that contain references as inputs must return a `Result`
 --> src/lib.rs:6:58
...
error: future cannot be sent between threads safely
   --> src/lib.rs:5:1
...
    = help: within `impl Future<Output = u32>`, the trait `std::marker::Send` is not implemented for `std::sync::MutexGuard<'_, u32>`
...
error: could not compile `tauri-app` (lib) due to 3 previous errors
```

一次會看到兩種錯誤（最後一行寫 3 個，是因為第二種被報了兩次）。第一種是第 3 章第 8 集預告過的：`State<'_, T>` 是借用的參數，async command 的參數只要有借用，就必須回傳 `Result`，所以回傳型別要改成 `Result<u32, String>`。

第二種比較難懂。async command 在工作執行緒上跑，睡醒之後可能被換到另一條執行緒繼續，所以整個 future 必須是 `Send`。可是 std 的 `MutexGuard` 不是 `Send`，而 `count` 在 `.await` 前後都活著，future 就不是 `Send` 了。

### 換成 async 的 `Mutex`

`tauri::async_runtime::Mutex` 就是 tokio 的 `Mutex`，Tauri 把它轉出來，直接用就好。它的 `lock()` 是 async 的，要寫 `.lock().await`。回傳的守衛是 `Send`，可以放心跨過 `.await`。它也沒有「中毒」的問題，`.await` 之後直接是守衛，不用 `unwrap()`。

前端同時呼叫兩次，看鎖怎麼運作。兩次 `invoke` 都不先 `await`，所以幾乎同時送出；量時間的做法跟第 3 章第 8 集一樣：

`src-tauri/src/lib.rs`：

```rust
use std::time::Duration;
use tauri::State;
use tauri::async_runtime::Mutex;

#[tauri::command]
async fn slow_increment(state: State<'_, Mutex<u32>>) -> Result<u32, String> {
    let mut count = state.lock().await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    *count += 1;
    Ok(*count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(0u32))
        .invoke_handler(tauri::generate_handler![slow_increment])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>async Mutex</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>async Mutex</h1>
      <div class="row">
        <button id="slow-btn" type="button">同時加兩次</button>
      </div>
      <p id="result-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const slowBtnEl: HTMLButtonElement | null = document.querySelector("#slow-btn");
const resultMsgEl: HTMLElement | null = document.querySelector("#result-msg");

async function incrementTwice(): Promise<void> {
  const start = Date.now();
  const first = invoke<number>("slow_increment");
  const second = invoke<number>("slow_increment");
  const firstCount = await first;
  const secondCount = await second;
  if (resultMsgEl) {
    resultMsgEl.textContent = `兩次的結果是 ${firstCount} 和 ${secondCount}，共花了 ${Date.now() - start} 毫秒`;
  }
}

slowBtnEl?.addEventListener("click", incrementTwice);
```

`npm run tauri dev` 的輸出（節錄）：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.74s
     Running `target/debug/tauri-app`
```

按下按鈕，顯示「兩次的結果是 1 和 2，共花了 1014 毫秒」（毫秒數會略有出入）。先搶到鎖的睡半秒、加一、放鎖；另一個在 `lock().await` 等著，輪到它再睡半秒，所以總共約一秒。結果是 1 和 2，不會兩個都是 1。等鎖的時候只是暫停，沒有佔住執行緒。

## 重點整理

- async command 的參數有借用（包含 `State<'_, T>`）時，必須回傳 `Result`。
- std 的 `MutexGuard` 不是 `Send`，不能活著跨過 `.await`；只有上了鎖之後還要 `.await`，才需要換成 `tauri::async_runtime::Mutex`；沒有 `.await` 的話，上一集的 std `Mutex` 就夠了。
- async 的 `Mutex` 用 `.lock().await` 上鎖，守衛直接可用，不用 `unwrap()`；等鎖不會佔住執行緒。
