# 在 `setup` 裡初始化

## 本集目標

學會用 `Builder::setup` 在 App 建好之後建立狀態，並知道 `setup` 什麼時候跑、失敗時會怎樣。

## 正文

### `Builder` 手上還沒有 App

前兩集的 `.manage(...)` 寫在 `Builder` 上。那時 App 還沒建出來，手上只有你事先寫好、算得出來的值。要用 App 的資訊（名稱、版本、設定檔的內容）決定初始值，就得等 App 建好。Tauri 留了一個時間點給你：`setup`。

### 範例：記下啟動的資訊

啟動時從 App 讀出名稱與版本，連同啟動的時間存成狀態，之後 command 就能回報「這個 App 跑了多久」：

`src-tauri/src/lib.rs`：

```rust
use std::time::Instant;
use tauri::{Manager, State};

struct Session {
    title: String,
    started_at: Instant,
}

#[tauri::command]
fn session_info(session: State<Session>) -> String {
    format!(
        "{}，已經執行 {} 秒",
        session.title,
        session.started_at.elapsed().as_secs()
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let info = app.package_info();
            app.manage(Session {
                title: format!("{} {}", info.name, info.version),
                started_at: Instant::now(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![session_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `setup` 收一個閉包，參數 `app` 是 `&mut App`。閉包在 App 啟動時執行一次，要回傳 `Ok(())`。
- `app.package_info()` 跟第 3 章第 9 集 `AppHandle` 上的 `package_info()` 一樣，給 App 的名稱與版本。
- `app.manage(值)` 的作用跟 `Builder` 上的 `.manage(...)` 一樣，只是現在寫在 App 上。`State<Session>` 的取用方式完全沒變。
- 方法 `manage` 定義在 `Manager` 這個 trait 上，要先 `use tauri::Manager`。

前端一個按鈕、一個放結果的段落：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>setup</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>setup</h1>
      <div class="row">
        <button id="info-btn" type="button">查看</button>
      </div>
      <p id="info-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const infoBtnEl: HTMLButtonElement | null = document.querySelector("#info-btn");
const infoMsgEl: HTMLElement | null = document.querySelector("#info-msg");

infoBtnEl?.addEventListener("click", async () => {
  const info = await invoke<string>("session_info");
  if (infoMsgEl) {
    infoMsgEl.textContent = info;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.49s
     Running `target/debug/tauri-app`
```

按「查看」，段落出現「tauri-app 0.1.0，已經執行 5 秒」，秒數是從 App 啟動算起，隔一陣子再按會變大。

### 漏了 `use tauri::Manager`

`app.manage(...)` 寫了卻沒有 `use tauri::Manager`，編譯器會說找不到方法（節錄）：

```text
error[E0599]: no method named `manage` found for mutable reference `&mut tauri::App` in the current scope
    --> src/lib.rs:24:17
     |
  24 |             app.manage(Session {
     |             ----^^^^^^
     |
     = help: items from traits can only be used if the trait is in scope
...
help: trait `Manager` which provides `manage` is implemented but not in scope; perhaps you want to import it
     |
   1 + use tauri::Manager;
```

### `setup` 什麼時候跑、失敗了會怎樣

- `setup` 在 App 啟動時、主執行緒上執行一次。前端的呼叫要由主執行緒收進來，它被 `setup` 佔著的時候，呼叫只能等，所以 command 看得到 `setup` 裡 `manage` 的狀態。反過來說，它會卡住主執行緒，不要在裡面做太久的事。
- 閉包回傳 `Result<(), Box<dyn Error>>`，裡面可以用 `?`。在最前面加一行 `let _n: u32 = "abc".parse()?;` 故意讓它失敗，App 就不會開起來，終端機出現：

  ```text
  thread 'main' (123456) panicked at ~/.cargo/registry/src/.../tauri-2.12.6/src/app.rs:1444:11:
  Failed to setup app: error encountered during setup hook: invalid digit found in string
  ```

## 重點整理

- 要用 App 的資訊建立狀態，就在 `Builder::setup` 的閉包裡 `app.manage(...)`；閉包要回傳 `Ok(())`。
- `manage` 等方法在 `Manager` trait 上，要 `use tauri::Manager`。
- `setup` 在啟動時於主執行緒跑一次，前端的呼叫會等它跑完；裡面出錯用 `?` 回傳，App 會停止啟動。
- 前端與 command 取用狀態的方式不變，仍是 `State<T>`。
