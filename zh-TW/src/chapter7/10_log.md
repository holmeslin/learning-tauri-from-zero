# log：記錄日誌

## 本集目標

學會用 log plugin 讓 Rust 與前端的日誌寫到同一個地方：終端機與日誌檔，也可以送回開發者工具的 Console。

## 正文

### 一個 plugin，兩端都能寫日誌

Rust 的日誌慣例是 `log` crate：它只定義 `error!`、`warn!`、`info!`、`debug!`、`trace!` 這幾個巨集，真正印到哪裡，由「記錄器」決定。log plugin 就是 Tauri 的記錄器，同時提供前端的 `info()`、`error()`，兩邊的訊息會進同一份日誌。

### 範例

先照第 1 集的方式裝 plugin：

```bash
npm run tauri add log
```

它會在 `lib.rs` 註冊 plugin，並在 `capabilities/default.json` 加上 `log:default`。permission 只管前端：少了它，前端的 `info()` 會被擋，Rust 端的 `log::info!` 不經過 permission，不受影響。Rust 端要寫 `log::info!`，還得自己加 `log` crate，在專案根目錄執行：

```bash
cargo add --manifest-path src-tauri/Cargo.toml log@0.4
```

接著改 `lib.rs`。`.level(...)` 設定最低等級，不設的話所有等級都印，連 Tauri 底層視窗函式庫的 TRACE 訊息也會印出來，一啟動就幾十行。`Target::new(TargetKind::Webview)` 多加一個輸出位置：前端的 Console。

`src-tauri/src/lib.rs`：

```rust
use tauri_plugin_log::{Target, TargetKind};

#[tauri::command]
fn log_from_rust() {
    log::info!("Rust 的 info");
    log::warn!("Rust 的 warn");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .target(Target::new(TargetKind::Webview))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            log::info!("app 啟動了");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![log_from_rust])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "log:default"
  ]
}
```

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>log</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>log</h1>
      <div class="row">
        <button id="log-btn" type="button">寫日誌</button>
      </div>
    </main>
  </body>
</html>
```

前端先呼叫 `attachConsole()`，把送來 Console 的日誌印出來。按鈕寫兩則前端的日誌，再叫 Rust 的 `log_from_rust`：

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { attachConsole, error, info } from "@tauri-apps/plugin-log";

attachConsole();

document.querySelector("#log-btn")?.addEventListener("click", async () => {
  await info("前端的 info");
  await error("前端的 error");
  await invoke("log_from_rust");
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
     Running `target/debug/tauri-app`
[2026-11-18][01:26:41][tauri_app_lib][INFO] app 啟動了
```

按下「寫日誌」，終端機多出四行：

```text
[2026-11-18][01:26:44][webview][INFO] 前端的 info
[2026-11-18][01:26:44][webview][ERROR] 前端的 error
[2026-11-18][01:26:44][tauri_app_lib][INFO] Rust 的 info
[2026-11-18][01:26:44][tauri_app_lib][WARN] Rust 的 warn
```

每行是 `[日期][時間][來源][等級] 訊息`，時間用 UTC，台灣的時間要加 8 小時。來源在 Rust 端是呼叫 `log::info!` 的模組（這裡是 crate `tauri_app_lib`），前端的是 `webview`。

### 日誌檔與 Console

同樣的內容也寫進日誌檔。路徑是第 6 集的 `appLogDir()`，檔名是 `productName` 加 `.log`。macOS 上，檔案的內容就是上面的五行：

```bash
cat "$HOME/Library/Logs/tw.example.tauri-app/tauri-app.log"
```

打開開發者工具（第 1 章第 8 集）的 Console，會看到按下按鈕之後的那四行。`app 啟動了` 不在其中，因為它在頁面載入、`attachConsole()` 執行之前就印過了。前端的日誌其實繞了一圈：先送到 Rust，再從 `TargetKind::Webview` 這個輸出位置回到 Console。少了這個輸出位置，或少了 `attachConsole()`，Console 就什麼都不會出現。

## 重點整理

- Rust 端用 `log` crate 的 `info!`、`warn!` 等巨集，要自己用 `cargo add --manifest-path src-tauri/Cargo.toml log@0.4` 加；前端用 `@tauri-apps/plugin-log` 的 `info()`、`error()`。
- `tauri_plugin_log::Builder::new().level(...).build()` 註冊記錄器；預設輸出到終端機與日誌檔（macOS 在 `~/Library/Logs/<identifier>/<productName>.log`），不設 `.level()` 會連 TRACE 都印。
- 加上 `Target::new(TargetKind::Webview)` 並在前端呼叫 `attachConsole()`，日誌也會出現在 Console。
- 前端的 `info()` 需要 `log:default`；Rust 端不需要 permission。
