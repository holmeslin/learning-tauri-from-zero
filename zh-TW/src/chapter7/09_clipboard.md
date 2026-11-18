# clipboard-manager：剪貼簿

## 本集目標

學會用 clipboard-manager plugin 讀寫剪貼簿的文字，並知道它的 `default` 什麼都不放行，要自己加 `allow-write-text` 與 `allow-read-text`。

## 正文

### 範例：複製與貼上

兩個函式：`writeText(文字)` 把文字放進剪貼簿，`readText()` 讀出剪貼簿目前的文字。先照第 1 集的方式裝 plugin：

```bash
npm run tauri add clipboard-manager
```

它會在 `lib.rs` 用 `init()` 註冊 plugin，並在 `capabilities/default.json` 加上 `clipboard-manager:default`。但這個 `default` 是**空的**，一項都不放行。剪貼簿裡可能有密碼，所以要讀要寫，都由你明確決定。我們先不管它，看看會怎樣：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
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
    "clipboard-manager:default"
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
if (!inputEl || !msgEl) {
  throw new Error("找不到頁面上的元素");
}

document.querySelector("#copy-btn")?.addEventListener("click", async () => {
  try {
    await writeText(inputEl.value);
    msgEl.textContent = "已複製";
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});

document.querySelector("#paste-btn")?.addEventListener("click", async () => {
  try {
    msgEl.textContent = `剪貼簿裡是：${await readText()}`;
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

執行 `npm run tauri dev`，按「複製」，段落出現下面的錯誤（按「貼上」也一樣，只是缺的是 `clipboard-manager:allow-read-text`）：

```text
出錯了：clipboard-manager.write_text not allowed. Permissions associated with this command: clipboard-manager:allow-write-text
```

### 補上 permission

錯誤訊息最後一段說得很清楚：要寫就加 `clipboard-manager:allow-write-text`，要讀就加 `clipboard-manager:allow-read-text`。把空的 `clipboard-manager:default` 換成這兩項：

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

改完存檔，`tauri dev` 會自動重新編譯。按「複製」，段落顯示「已複製」。到終端機驗證，內容真的進了系統的剪貼簿（`pbpaste` 是 macOS 印出剪貼簿文字的指令）：

```bash
pbpaste
```

終端機印出「你好，剪貼簿」。反過來，在終端機把文字放進剪貼簿（`pbcopy` 是 macOS 上做這件事的指令）：

```bash
echo "來自終端機" | pbcopy
```

再按 app 的「貼上」，段落顯示「剪貼簿裡是：來自終端機」。

## 重點整理

- `writeText(文字)` 寫入、`readText()` 讀出，都回傳 Promise。Rust 端用 `tauri_plugin_clipboard_manager::init()` 註冊。
- clipboard-manager 的 `default` 不放行任何功能，要自己加 `clipboard-manager:allow-write-text` 與 `clipboard-manager:allow-read-text`。
- 缺 permission 時，錯誤訊息最後一段就是缺的那一項。
