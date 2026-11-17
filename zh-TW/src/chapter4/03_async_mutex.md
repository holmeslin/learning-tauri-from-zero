# async command 裡的狀態

## 本集目標

在 async command 裡讀寫狀態：知道為什麼會撞上兩個編譯錯誤，並改用 `tauri::async_runtime::Mutex` 解決。

## 正文

### 直接改成 async 會失敗

要等的工作該寫成 async command（第 3 章第 8 集）。把上一集的 `increment` 改一下：加一之前先等半秒，假裝在做耗時的事。等待用 `tokio::time::sleep`，要沿用第 3 章第 8 集加進專案的 tokio（`time` 功能），還沒加的話先執行 `cargo add --manifest-path src-tauri/Cargo.toml tokio@1 --features time`。照直覺，只多寫 `async` 與 `.await`：

```rust,ignore
#[tauri::command]
async fn slow_increment(state: State<'_, Mutex<Counter>>) -> u32 {
    let mut counter = state.lock().unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    counter.count += 1;
    counter.count
}
```

編譯一次會得到兩個錯誤（節錄）：

```text
error[E0277]: async commands that contain references as inputs must return a `Result`
  --> src/lib.rs:10:62
   |
10 | async fn slow_increment(state: State<'_, Mutex<Counter>>) -> u32 {
   |                                                              ^^^ the trait `_::AsyncCommandMustReturnResult` is not implemented for `u32`

error: future cannot be sent between threads safely
  --> src/lib.rs:9:1
   |
 9 | #[tauri::command]
   | ^^^^^^^^^^^^^^^^^ future returned by `slow_increment` is not `Send`
   = help: within `impl std::future::Future<Output = u32>`, the trait `std::marker::Send` is not implemented for `std::sync::MutexGuard<'_, Counter>`
```

第一個是第 3 章第 8 集預告過的：`State<'_, T>` 是借用的參數（`'_` 就是借用的生命週期），async command 的參數只要有借用，就必須回傳 `Result`。回傳型別改成 `Result<u32, String>` 就好。

第二個比較難懂。async command 在 Tauri 的工作執行緒上跑，睡醒之後可能被換到另一條執行緒繼續，所以整個 future 必須是 `Send`。可是 std 的 `MutexGuard` 不是 `Send`，而它在 `.await` 前後一直活著（`counter` 到函式結束才放掉），future 就不是 `Send` 了。

### 換成 async 的 `Mutex`

`tauri::async_runtime::Mutex` 就是 tokio 的 `Mutex`，Tauri 把它轉出來，直接用就好。它的 `lock()` 是 async 的，要寫 `.lock().await`；回傳的守衛是 `Send`，可以安心跨過 `.await`。它也沒有「中毒」的問題，所以 `.await` 之後直接是守衛，不用 `unwrap()`。

前端同時呼叫兩次（兩次 `invoke` 都不先 `await`，所以幾乎同時送出，量時間的做法跟第 3 章第 8 集一樣），看鎖怎麼運作：

`src-tauri/src/lib.rs`：

```rust
use std::time::Duration;
use tauri::State;
use tauri::async_runtime::Mutex;

struct Counter {
    count: u32,
}

#[tauri::command]
async fn slow_increment(state: State<'_, Mutex<Counter>>) -> Result<u32, String> {
    let mut counter = state.lock().await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    counter.count += 1;
    Ok(counter.count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(Counter { count: 0 }))
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
  const a = await first;
  const b = await second;
  if (resultMsgEl) {
    resultMsgEl.textContent = `兩次的結果是 ${a} 和 ${b}，共花了 ${Date.now() - start} 毫秒`;
  }
}

slowBtnEl?.addEventListener("click", incrementTwice);
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.74s
     Running `target/debug/tauri-app`
```

按下按鈕，顯示「兩次的結果是 1 和 2，共花了 1014 毫秒」（毫秒數會略有出入）。兩個呼叫同時進來，先搶到鎖的睡半秒、加一再放鎖，另一個在 `lock().await` 等著，輪到它再睡半秒，所以總共約一秒，而且結果是 1 和 2，不會兩個都是 1。等鎖的時候它只是暫停，沒有佔住執行緒，別的 command 照樣能跑。

### 沒有 `.await` 的話，std 的 `Mutex` 就夠了

鎖住的期間如果沒有 `.await`（例如先 `sleep`，睡醒才 `lock()`），守衛就不會跨過 `.await`，future 仍是 `Send`，async command 裡用上一集的 `std::sync::Mutex` 也行，而且比較輕量；只是 `State` 是借用的參數，還是得回傳 `Result`。只有「上了鎖之後還要 `.await`」才需要換成 async 的 `Mutex`。

要注意的是，`std::sync::Mutex<Counter>` 與 `tauri::async_runtime::Mutex<Counter>` 是兩個不同的型別。`manage` 管的是哪一個，`State` 就要寫同一個，否則 command 會收到第 1 集那個 `state not managed for field ...` 的拒絕。

## 重點整理

- async command 的參數有借用（包含 `State<'_, T>`）時，必須回傳 `Result`。
- std 的 `MutexGuard` 不是 `Send`，不能活著跨過 `.await`；上了鎖之後還要 `.await`，就用 `tauri::async_runtime::Mutex`。
- async 的 `Mutex` 用 `.lock().await` 上鎖，守衛直接可用，不用 `unwrap()`；等鎖不會佔住執行緒。
- `manage` 與 `State` 的 `Mutex` 要是同一個型別，不然會收到 `state not managed`。
