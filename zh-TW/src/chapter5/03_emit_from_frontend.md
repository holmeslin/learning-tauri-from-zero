# 前端送事件給 Rust

## 本集目標

學會在前端用 `emit` 送事件，在 Rust 用 `listen` 接收，並知道 payload 到了 Rust 是 JSON 文字，得自己解析。

## 正文

### 反方向也行

事件不分方向：前端用 `emit` 送，Rust 用 `listen` 收。適合「通知一聲、不需要回答」的情況；要拿回結果，還是用 command（第 3 章）。

### 範例：前端喊加，Rust 算總和

前端 `emit("add", { amount: 5 })`，Rust 收到後把數字加進狀態，再 `emit("total", 總和)` 回報。Rust 的 `listen` 要用在 App（或 `AppHandle`）上，所以寫在 `setup` 裡（第 4 章第 4 集）：

`src-tauri/src/lib.rs`：

```rust
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{Emitter, Listener, Manager};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddRequest {
    amount: u32,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(0u32))
        .setup(|app| {
            let handle = app.handle().clone();
            app.listen("add", move |event| {
                let request = match serde_json::from_str::<AddRequest>(event.payload()) {
                    Ok(request) => request,
                    Err(error) => {
                        eprintln!("add 的 payload 不對：{error}");
                        return;
                    }
                };
                let state = handle.state::<Mutex<u32>>();
                let mut total = state.lock().unwrap();
                *total += request.amount;
                handle.emit("total", *total).unwrap();
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `app.listen("add", 函式)` 登記一個監聽函式。`Listener` trait 要先引入，`listen` 才叫得動。函式收到的 `event`，用 `event.payload()` 取資料。
- 函式要 `move` 進去，所以先複製一份 `AppHandle`。它不是 command，沒有人替它填 `State` 參數，要的狀態就用第 4 章第 5 集的 `handle.state()` 取。
- 前端 `emit` 送來的事件，函式是在非同步執行環境的工作執行緒上呼叫的（第 3 章第 8 集），不是主執行緒。
- 函式沒有回傳值，出錯不能用 `?` 丟出去，只能在函式裡自己處理。

### payload 是 JSON 文字

`event.payload()` 回傳 `&str`，內容是前端送來的 JSON 文字：

- 前端送數字 `5`，拿到文字 `5`。
- 前端送字串 `"hi"`，拿到的文字連雙引號都在：`"hi"`。
- 前端送物件 `{ amount: 5 }`，拿到 `{"amount":5}`。
- 前端沒給 payload，拿到文字 `null`。

所以要用 `serde_json::from_str` 轉回型別，這跟第 3 章的 `Deserialize` 是同一套（範本已經有 `serde_json`）。command 的參數不對，Tauri 會直接拒絕呼叫（第 3 章第 5 集）；事件沒有這一層檢查，前端送什麼就收什麼，格式不對就要自己處理，範例的 `Err` 分支就是在做這件事。

前端兩個按鈕負責 `emit`，一個 `listen` 顯示總和：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>前端送事件</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>前端送事件</h1>
      <div class="row">
        <button id="add1-btn" type="button">加 1</button>
        <button id="add10-btn" type="button">加 10</button>
      </div>
      <p id="total-msg">合計：0</p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { emit, listen } from "@tauri-apps/api/event";

const totalMsgEl: HTMLElement | null = document.querySelector("#total-msg");

listen<number>("total", (event) => {
  if (totalMsgEl) {
    totalMsgEl.textContent = `合計：${event.payload}`;
  }
});

document.querySelector("#add1-btn")?.addEventListener("click", () => {
  emit("add", { amount: 1 });
});

document.querySelector("#add10-btn")?.addEventListener("click", () => {
  emit("add", { amount: 10 });
});
```

`emit("事件名稱", payload)` 也是回傳 Promise，這裡不需要等它。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.58s
     Running `target/debug/tauri-app`
```

依序按「加 1」「加 10」，畫面變成「合計：1」，再變「合計：11」。

### 取消監聽與只聽一次

Rust 這邊也有同樣的兩個功能：`app.listen` 回傳 `EventId`，之後用 `app.unlisten(id)` 取消；只聽一次就用 `app.once(...)`。

### 跟 Electron 比一比

Electron 的 `ipcRenderer.send` 只會送到主行程，只有 `ipcMain.on` 收得到。Tauri 前端的 `emit` 是廣播，不是專門寄給 Rust 的信：Rust 的 `listen` 會收到，前端自己的 `listen("add", ...)` 也會收到。想限定接收者，下一集再說。

## 重點整理

- 前端用 `emit("名稱", payload)` 送，Rust 用 `app.listen("名稱", 函式)` 收，要 `use tauri::Listener`。
- `event.payload()` 是 JSON 文字，要自己用 `serde_json::from_str` 解析；格式不對只能在函式裡處理。
- 監聽函式不是 command：要狀態就用 `handle.state()`，沒有回傳值。
- `listen` 回傳 `EventId`，`unlisten(id)` 取消，`once` 只聽一次。
- 前端的 `emit` 是廣播，前端自己的監聽也會收到。
