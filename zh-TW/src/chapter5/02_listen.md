# 前端 `listen` 與取消監聽

## 本集目標

學會用 `listen` 與 `once` 收事件，用它回傳的函式取消監聽，並知道事件不會補發、重複 `listen` 會重複收。

## 正文

### `listen` 回傳什麼

上一集借用的 `listen("名稱", 函式)`，其實回傳一個 `Promise`（第 2 章第 10 集）：Tauri 登記好這個監聽之後，才算完成。`await` 它，拿到的是一個函式，型別叫 `UnlistenFn`。呼叫這個函式，監聽就取消，之後同名的事件不會再進來。

另一個函式 `once` 的用法一樣，只是只收下一個事件，收到之後自動取消。

### 範例：開始、停止、只聽一次

Rust 這一側先沿用上一集的做法，每秒送一個 `tick`，這次 payload 簡化成一個數字：

`src-tauri/src/lib.rs`：

```rust
use std::thread;
use std::time::Duration;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            thread::spawn(move || {
                let mut seconds: u32 = 0;
                loop {
                    thread::sleep(Duration::from_secs(1));
                    seconds += 1;
                    handle.emit("tick", seconds).unwrap();
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

頁面有三個按鈕，收到的事件一行一行記在清單裡：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>listen</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>listen</h1>
      <div class="row">
        <button id="start-btn" type="button">開始監聽</button>
        <button id="stop-btn" type="button">停止監聽</button>
        <button id="once-btn" type="button">只聽一次</button>
      </div>
      <ul id="log"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { listen, once, UnlistenFn } from "@tauri-apps/api/event";

const logEl: HTMLElement | null = document.querySelector("#log");
let unlisten: UnlistenFn | null = null;

function addLog(text: string): void {
  if (logEl) {
    const itemEl = document.createElement("li");
    itemEl.textContent = text;
    logEl.append(itemEl);
  }
}

document.querySelector("#start-btn")?.addEventListener("click", async () => {
  if (unlisten === null) {
    unlisten = await listen<number>("tick", (event) => {
      addLog(`收到第 ${event.payload} 秒`);
    });
  }
});

document.querySelector("#stop-btn")?.addEventListener("click", () => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
});

document.querySelector("#once-btn")?.addEventListener("click", () => {
  once<number>("tick", (event) => {
    addLog(`once 收到第 ${event.payload} 秒`);
  });
});
```

`UnlistenFn` 是型別，跟 `listen` 一起從同一個套件匯入。「開始監聽」把 `listen` 回傳的函式存進 `unlisten`，「停止監聽」呼叫它；呼叫就生效，不必 `await`。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.47s
     Running `target/debug/tauri-app`
```

視窗打開後等幾秒，再按「開始監聽」，清單每秒多一行。按「停止監聽」就不再增加；再按一次「開始監聽」會重新收。按「只聽一次」只會多一行 `once` 開頭的紀錄。

### 事件不會補發

Rust 從 App 啟動就在數秒，但清單的第一行不是「第 1 秒」，而是你按下按鈕之後收到的第一個事件。事件只會送給「送出的當下已經在聽」的監聽，沒有人聽就沒了，Tauri 不會留著等你，也不會補發。所以前端的 `listen` 要在 Rust 開始送之前就登記好。

### 每呼叫一次 `listen`，就多一個監聽

`listen` 不會檢查是不是已經聽過同樣的事件，呼叫幾次就登記幾個，每個都會被呼叫。範例的 `if (unlisten === null)` 就是在防這個：拿掉它，連按兩次「開始監聽」，每秒會多出兩行，而且 `unlisten` 只存得下第二次的函式，第一個監聽再也取消不了。

## 重點整理

- `listen` 回傳 `Promise<UnlistenFn>`；`await` 拿到函式，呼叫它就取消監聽。
- `once` 只收下一個事件，收到後自動取消。
- 事件不會補發：只有登記之後送出的事件收得到，所以要先 `listen`，再讓 Rust 開始送。
- 每呼叫一次 `listen` 就多一個監聽，重複登記會重複處理，也會讓舊的取消函式遺失。
