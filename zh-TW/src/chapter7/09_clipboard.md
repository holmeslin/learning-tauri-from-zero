# clipboard-manager：剪貼簿

## 本集目標

學會用 clipboard-manager plugin 讀寫剪貼簿的文字，並知道它的 `default` 什麼都不放行，要自己加 `allow-write-text` 與 `allow-read-text`。

## 正文

### 範例：複製與貼上

兩個函式：`writeText(文字)` 把文字放進剪貼簿，`readText()` 讀出剪貼簿目前的文字。Rust 端用 `init()` 註冊。`capabilities/default.json` 先不動，看看會怎樣：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
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
    <title>clipboard</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>clipboard</h1>
      <div class="row">
        <input id="text-input" value="你好，剪貼簿" />
        <button id="copy-btn" type="button">複製</button>
        <button id="paste-btn" type="button">貼上</button>
      </div>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

const inputEl: HTMLInputElement | null = document.querySelector("#text-input");
const msgEl: HTMLElement | null = document.querySelector("#msg");

document.querySelector("#copy-btn")?.addEventListener("click", async () => {
  if (!inputEl || !msgEl) {
    return;
  }
  try {
    await writeText(inputEl.value);
    msgEl.textContent = "已複製";
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});

document.querySelector("#paste-btn")?.addEventListener("click", async () => {
  if (!msgEl) {
    return;
  }
  try {
    msgEl.textContent = `剪貼簿裡是：${await readText()}`;
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.39s
     Running `target/debug/tauri-app`
```

按「複製」，段落出現：

```text
出錯了：clipboard-manager.write_text not allowed. Permissions associated with this command: clipboard-manager:allow-write-text
```

按「貼上」也一樣，缺的是 `clipboard-manager:allow-read-text`。

### 補上 permission

錯誤訊息最後一段說得很清楚：要寫就加 `clipboard-manager:allow-write-text`，要讀就加 `clipboard-manager:allow-read-text`。兩個都加：

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
    "clipboard-manager:allow-write-text",
    "clipboard-manager:allow-read-text"
  ]
}
```

`npm run tauri dev` 的輸出（改了 `capabilities/default.json` 之後）：

```text
        Info File src-tauri/capabilities/default.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.76s
     Running `target/debug/tauri-app`
```

按「複製」，段落顯示「已複製」。到終端機驗證，內容真的進了系統的剪貼簿（`pbpaste` 是 macOS 印出剪貼簿文字的指令）：

```bash
pbpaste
```

```text
你好，剪貼簿
```

反過來，在終端機執行 `echo "來自終端機" | pbcopy`（`pbcopy` 把文字放進剪貼簿），再按 App 的「貼上」，段落顯示「剪貼簿裡是：來自終端機」。

順帶一提，加 `clipboard-manager:default` 一次放行是沒有用的：clipboard-manager 的 `default` 是**空的**，一項都不放行。剪貼簿裡可能有密碼，要讀要寫，由你明確決定。

## 重點整理

- `writeText(文字)` 寫入、`readText()` 讀出，都回傳 Promise。Rust 端用 `tauri_plugin_clipboard_manager::init()` 註冊。
- clipboard-manager 的 `default` 不放行任何功能，要自己加 `clipboard-manager:allow-write-text` 與 `clipboard-manager:allow-read-text`。
- 缺 permission 時，錯誤訊息最後一段就是缺的那一項。
