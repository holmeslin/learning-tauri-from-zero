# store：儲存設定

## 本集目標

學會用 store plugin 把設定存成一個檔案，下次開 App 還讀得到，並知道 `autoSave` 與 `save()` 的差別。

## 正文

### 一個會存檔的 key-value 表

設定這類小資料（暱稱、主題、上次開的檔案），不需要資料庫。store plugin 給你一張 key-value 表：`set` 存、`get` 取，內容存成 JSON 檔，放在 App 的資料目錄（第 6 集）。

### 範例：記住暱稱

`.plugin(...)` 的寫法跟第 1 集一樣，只是 store 要用 `Builder::new().build()` 建出來，不是 `init()`。permission 是 `store:default`，它放行 store 全部的操作：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
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
    "store:default"
  ]
}
```

頁面一個輸入框、一個儲存按鈕、一個顯示目前暱稱的段落：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>store</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>store</h1>
      <div class="row">
        <input id="name-input" placeholder="暱稱" />
        <button id="save-btn" type="button">儲存</button>
      </div>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { load } from "@tauri-apps/plugin-store";

const inputEl: HTMLInputElement | null = document.querySelector("#name-input");
const msgEl: HTMLElement | null = document.querySelector("#msg");

async function main() {
  if (!inputEl || !msgEl) {
    return;
  }
  try {
    const store = await load("settings.json");
    const saved = await store.get<string>("name");
    msgEl.textContent = "還沒有暱稱";
    if (saved) {
      msgEl.textContent = `暱稱：${saved}`;
    }
    document.querySelector("#save-btn")?.addEventListener("click", async () => {
      await store.set("name", inputEl.value);
      msgEl.textContent = `暱稱：${inputEl.value}`;
    });
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
}

main();
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.39s
     Running `target/debug/tauri-app`
```

段落一開始寫「還沒有暱稱」。輸入 `Ada` 按「儲存」，段落變成「暱稱：Ada」。到終端機執行 `cat "$HOME/Library/Application Support/tw.example.tauri-app/settings.json"`，看 macOS 上的檔案：

```text
{
  "name": "Ada"
}
```

按 `Ctrl+C` 結束 `npm run tauri dev`，再執行一次，段落一開始就寫「暱稱：Ada」：資料在檔案裡，App 重開也在。

`load("settings.json")` 的路徑從 App 的資料目錄（第 6 集）算起。檔案與資料夾在第一次 `set` 之後才出現，資料夾由 plugin 自己建，不用先 `mkdir`。`get<string>("name")` 的 `<string>` 是告訴 TypeScript 值的型別；找不到這個 key 會得到 `undefined`，所以範例先用 `if (saved)` 檢查。

### `autoSave` 與 `save()`

`set` 只改記憶體裡的表。預設的 `autoSave` 會在最後一次修改的 100 毫秒之後，把整張表寫進檔案，所以範例沒有呼叫 `save()` 也存得下來。不想讓它自動存，在 `load` 的第二個參數關掉，要存的時候自己呼叫 `save()`（下面的片段寫在 `async` 函式裡）：

```ts,ignore
const store = await load("settings.json", { autoSave: false });
await store.set("name", "Ada");
await store.save();
```

### 沒有 permission 的時候

少了 `store:default`，`load()` 會被擋。範例把錯誤顯示在段落上，格式跟第 2 集看到的一樣，最後一段列出放行它的 permission：

```text
出錯了：store.load not allowed. Permissions associated with this command: store:allow-load, store:default
```

## 重點整理

- store 是一張會存成 JSON 檔的 key-value 表，檔案放在 App 的資料目錄；Rust 端用 `tauri_plugin_store::Builder::new().build()` 註冊，permission 是 `store:default`。
- `load("檔名")` 取得 store，`get<T>(key)` 讀（沒有就是 `undefined`），`set(key, value)` 寫。
- 預設的 `autoSave` 在修改後 100 毫秒自動存檔；`load(檔名, { autoSave: false })` 關掉之後，要自己呼叫 `save()`。
