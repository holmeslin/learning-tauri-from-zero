# 從 `AppHandle` 取得狀態

## 本集目標

學會用 `state()` 從 `AppHandle` 取得狀態，讓沒有 `State` 參數的地方（例如背景執行緒）也能讀寫它。

## 正文

### 不是每段程式都是 command

`State` 參數只能寫在 command 上。可是 App 裡不是每段程式都是 command：`setup` 裡開出去的背景執行緒（這一集的範例）、之後的事件回呼（第 5 章）都不是，沒有人替它們填參數。這時只要手上有 `AppHandle`，就能向它要：`handle.state::<型別>()`，拿到的跟參數裡的 `State<T>` 是同一種東西。第 3 章第 9 集說過 `AppHandle` 可以 `clone()` 交給別處，正好用在這裡。`state()` 定義在 `Manager` trait 上，要 `use tauri::Manager`；`setup` 裡的 `app` 與第 3 章第 9 集的 `WebviewWindow` 也實作了它，同樣有 `state()`。

### 範例：背景執行緒每秒加一

`setup` 裡開一條執行緒，每秒替計數加一。command 這一側，`get_seconds` 用上一集的 `State`，`reset` 改用 `AppHandle`：

`src-tauri/src/lib.rs`：

```rust
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

struct Ticker {
    seconds: u32,
}

#[tauri::command]
fn get_seconds(state: State<Mutex<Ticker>>) -> u32 {
    state.lock().unwrap().seconds
}

#[tauri::command]
fn reset(app: AppHandle) {
    let state = app.state::<Mutex<Ticker>>();
    state.lock().unwrap().seconds = 0;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(Ticker { seconds: 0 }))
        .setup(|app| {
            let handle = app.handle().clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let state = handle.state::<Mutex<Ticker>>();
                    state.lock().unwrap().seconds += 1;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_seconds, reset])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `.manage(...)` 仍然寫在 `Builder` 上，所以執行緒開始跑的時候，狀態已經在了；`app.handle()` 拿到 `AppHandle` 的參考，`.clone()` 之後 `move` 進執行緒。
- `state::<Mutex<Ticker>>()` 的尖括號寫的是狀態的型別，狀態是依型別找的（第 1 集），不寫的話 Rust 不知道你要哪一個。
- 背景執行緒、`get_seconds`、`reset` 走三條不同的路，拿到的是同一個 `Mutex`。

前端兩個按鈕，一個讀取、一個歸零：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>AppHandle 與狀態</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>AppHandle 與狀態</h1>
      <div class="row">
        <button id="read-btn" type="button">讀取</button>
        <button id="reset-btn" type="button">歸零</button>
      </div>
      <p id="seconds-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const readBtnEl: HTMLButtonElement | null = document.querySelector("#read-btn");
const resetBtnEl: HTMLButtonElement | null = document.querySelector("#reset-btn");
const secondsMsgEl: HTMLElement | null = document.querySelector("#seconds-msg");

async function showSeconds(): Promise<void> {
  const seconds = await invoke<number>("get_seconds");
  if (secondsMsgEl) {
    secondsMsgEl.textContent = `背景執行緒已經數了 ${seconds} 秒`;
  }
}

readBtnEl?.addEventListener("click", showSeconds);
resetBtnEl?.addEventListener("click", async () => {
  await invoke("reset");
  await showSeconds();
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.62s
     Running `target/debug/tauri-app`
```

等幾秒按「讀取」，顯示「背景執行緒已經數了 8 秒」，再按一次數字變大。按「歸零」，顯示 0，之後又從 0 繼續往上數。

### `state` 找不到與 `try_state`

在 command 的參數裡，找不到狀態只是把呼叫拒絕（第 1 集）。但 `state()` 找不到會直接 panic。把 `.manage(...)` 那行拿掉，背景執行緒會印出下面這行，然後這條執行緒就結束了：

```text
thread '<unnamed>' (123456) panicked at ~/.cargo/registry/src/.../tauri-2.12.6/src/lib.rs:761:7:
state() called before manage() for std::sync::poison::mutex::Mutex<tauri_app_lib::Ticker>
```

不確定狀態在不在，就改用 `try_state()`，它回傳 `Option`：

```rust,ignore
if let Some(state) = handle.try_state::<Mutex<Ticker>>() {
    state.lock().unwrap().seconds += 1;
}
```

## 重點整理

- 沒有 `State` 參數可用的地方，用 `AppHandle` 加 `state::<型別>()` 取得狀態；要 `use tauri::Manager`。
- `AppHandle` 可以 `clone()` 後 `move` 進執行緒，背景工作就能讀寫同一份狀態。
- 狀態是依型別找的，`state::<T>()` 的 `T` 要跟 `manage` 的型別完全一致（包含 `Mutex`）。
- `state()` 找不到狀態會 panic，`try_state()` 回傳 `Option`。
