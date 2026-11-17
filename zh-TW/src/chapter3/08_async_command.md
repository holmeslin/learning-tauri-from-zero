# async command

## 本集目標

學會用 `async fn` 寫 command，並知道它跟一般 command 在哪條執行緒上跑、為什麼參數不能直接借用。

## 正文

### 同步與非同步，跑的地方不一樣

command 可以寫成一般的 `fn`，也可以寫成 `async fn`，前端的呼叫方式完全一樣，差別在 Rust 這一側：

- 沒有 `async` 的 command，在**主執行緒**（main thread）上執行，只有一條，別的同步 command 得排隊。
- `async fn` 的 command，丟給 Tauri 的非同步執行環境（runtime，底層是 tokio），在它的工作執行緒（worker thread）上執行。

所以要花時間的工作（等網路、等檔案）該寫成 `async fn`。下面實際量一次。`async fn` 裡的等待要用 `tokio::time::sleep`，得先把 tokio 加成自己的相依套件，並打開 `time` 功能（Tauri 自己用的 tokio 沒有打開它）：

```bash
cd src-tauri
cargo add tokio@1 --features time
```

### 範例：量兩次呼叫的時間

兩個 command 都睡一秒再回傳問候，一個同步、一個非同步：

`src-tauri/src/lib.rs`：

```rust
use std::time::Duration;

#[tauri::command]
fn slow_sync(name: &str) -> String {
    std::thread::sleep(Duration::from_secs(1));
    format!("你好，{name}！")
}

#[tauri::command]
async fn slow_async(name: String) -> String {
    tokio::time::sleep(Duration::from_secs(1)).await;
    format!("你好，{name}！")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![slow_sync, slow_async])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

前端有兩個按鈕，各自對同一個 command 連續呼叫兩次，量從頭到尾花了多久：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>async command</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>async command</h1>
      <div class="row">
        <button id="sync-btn" type="button">同步 ×2</button>
        <button id="async-btn" type="button">非同步 ×2</button>
      </div>
      <p id="time-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const syncBtnEl: HTMLButtonElement | null = document.querySelector("#sync-btn");
const asyncBtnEl: HTMLButtonElement | null = document.querySelector("#async-btn");
const timeMsgEl: HTMLElement | null = document.querySelector("#time-msg");

async function timeTwoCalls(command: string): Promise<void> {
  const start = performance.now();
  const first = invoke<string>(command, { name: "Ada" });
  const second = invoke<string>(command, { name: "Bob" });
  await first;
  await second;
  const seconds = ((performance.now() - start) / 1000).toFixed(1);
  if (timeMsgEl) {
    timeMsgEl.textContent = `兩次呼叫共花了 ${seconds} 秒`;
  }
}

syncBtnEl?.addEventListener("click", () => timeTwoCalls("slow_sync"));
asyncBtnEl?.addEventListener("click", () => timeTwoCalls("slow_async"));
```

`performance.now()` 是瀏覽器內建的計時函式，回傳毫秒；`toFixed(1)` 把數字四捨五入到小數點後一位。兩次 `invoke` 都不先 `await`：Promise 一建立就開始跑（第 2 章第 10 集），所以兩次呼叫幾乎同時送出。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.40s
     Running `target/debug/tauri-app`
```

按「同步 ×2」，顯示「兩次呼叫共花了 2.0 秒」：兩個 command 在主執行緒上排隊，一個接一個睡。按「非同步 ×2」，顯示 1.0 秒：兩個一起在等。

### async command 的參數不能借用

同步的 `slow_sync` 收 `&str` 沒問題，但 `slow_async` 的 `name` 是 `String`。如果寫成 `name: &str`，編譯器會擋下來（節錄）：

```text
error[E0277]: async commands that contain references as inputs must return a `Result`
  --> src/lib.rs:10:36
   |
10 | async fn slow_async(name: &str) -> String {
   |                                    ^^^^^^ the trait `AsyncCommandMustReturnResult` is not implemented for `std::string::String`
```

規則寫在訊息裡：`async` command 的參數若含有參考（`&str` 這種借用），就必須回傳 `Result`。最直接的解法是把參數改成 `String`，它擁有自己的資料，不是借用；或是維持借用，把回傳型別改成 `Result<String, String>`（第 6 集）。

### 不想改成 async 的話

同步函式想離開主執行緒，可以在屬性上加 `async`，`fn` 本身不用改。它就會跑在工作執行緒上，兩次呼叫一樣只花 1 秒左右：

```rust,ignore
#[tauri::command(async)]
fn slow_sync(name: &str) -> String {
    std::thread::sleep(Duration::from_secs(1));
    format!("你好，{name}！")
}
```

## 重點整理

- 沒有 `async` 的 command 在主執行緒執行，會排隊；`async fn` 在 Tauri 的非同步執行環境（tokio）執行，可以同時進行。
- 耗時的工作寫成 `async fn`，等待用 `.await`；tokio 的 `time` 功能要自己用 `cargo add tokio@1 --features time` 加。
- `async fn` 的參數含借用型別時，必須回傳 `Result`；最簡單是把參數改成 `String`。
- `#[tauri::command(async)]` 可以讓同步函式改到工作執行緒上跑。
