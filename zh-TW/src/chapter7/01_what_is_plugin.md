# plugin 是什麼、怎麼加

## 本集目標

知道一個 plugin 由 Rust crate、npm 套件與 permission 三個部分組成，會用 `npm run tauri add` 把 plugin 裝進專案，並看懂它改了哪些檔案。

## 正文

### plugin 是什麼

Tauri 的核心只管視窗、事件與 command。要跟作業系統打交道的功能，像是對話框、讀寫檔案、系統通知、剪貼簿，都拆成一個個 plugin，用到哪個才裝哪個。範本已經裝了一個：opener。第 1 章第 7 集讀 `lib.rs` 時看到的 `.plugin(tauri_plugin_opener::init())`，就是在註冊它。

### 一個 plugin 有三個部分

以 opener 為例：

- Rust crate `tauri-plugin-opener`：功能真正執行的地方。列在 `src-tauri/Cargo.toml`，並在 `src-tauri/src/lib.rs` 用 `.plugin(...)` 註冊。
- npm 套件 `@tauri-apps/plugin-opener`：前端呼叫它用的函式，列在 `package.json`。背後做的事跟第 3 章的 `invoke` 一樣，只是替你包好了。
- permission `opener:default`：寫在 `src-tauri/capabilities/default.json`，放行前端呼叫這個 plugin。

少了任何一個，功能就不能用。

### 用 `npm run tauri add` 安裝

手動裝要改四個地方，`npm run tauri add` 一次做完。我們來裝 dialog plugin，它能彈出系統原生的對話框：

```bash
npm run tauri add dialog
```

`npm run tauri add dialog` 的輸出：

```text
        Info Installing Cargo dependency "tauri-plugin-dialog"...
        Info Installing NPM dependency "@tauri-apps/plugin-dialog@~2"...
       Added permission `dialog:default` to `default` at ~/tauri-app/src-tauri/capabilities/default.json
        Info Adding plugin to ~/tauri-app/src-tauri/src/lib.rs
        Info Running `cargo fmt`...
```

它做了四件事。`src-tauri/Cargo.toml` 多一行 `tauri-plugin-dialog = "2"`，`package.json` 多一行 `"@tauri-apps/plugin-dialog": "^2.8.4"`。`lib.rs` 多一行 `.plugin(tauri_plugin_dialog::init())`，`capabilities/default.json` 多一項 `dialog:default`。

後兩個檔案現在是這樣。`lib.rs` 沿用前幾章的寫法，沒有 `greet`。CLI 會把 `default.json` 的 `"windows"` 展開成多行，內容沒變，本書一律寫成範本原本的一行：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
    "dialog:default"
  ]
}
```

### 試試看

做一個按鈕，按下去彈出對話框。`message` 是 dialog plugin 的函式，顯示一則訊息，等使用者按了「OK」才結束，所以要 `await`。對話框的用法第 4 集再說，這裡只確認 plugin 真的能用：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>plugin</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>plugin</h1>
      <div class="row">
        <button id="hello-btn" type="button">打開對話框</button>
      </div>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { message } from "@tauri-apps/plugin-dialog";

const msgEl: HTMLElement | null = document.querySelector("#msg");
if (!msgEl) {
  throw new Error("找不到 #msg");
}

document.querySelector("#hello-btn")?.addEventListener("click", async () => {
  try {
    await message("dialog plugin 裝好了", { title: "plugin" });
    msgEl.textContent = "對話框關掉了";
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-plugin-dialog v2.8.4
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.80s
     Running `target/debug/tauri-app`
```

按下按鈕，彈出標題為 `plugin` 的系統對話框，按「OK」之後畫面出現「對話框關掉了」。

漏掉 `lib.rs` 的 `.plugin(tauri_plugin_dialog::init())` 的話，編譯照樣成功，但按下按鈕會得到 `出錯了：plugin dialog not found`：Rust 這一側沒有註冊叫 `dialog` 的 plugin。漏掉 permission 的情況，下一集再說。

### 跟 Electron 比一比

Electron 的主行程跑在 Node.js 上，要讀檔、彈對話框，直接呼叫 Node.js 與 Electron 的 API 就好。Tauri 的前端只是一般網頁，沒有 Node.js API，碰不到系統。想做這些事，只能呼叫 Rust 這一側的 plugin（或自己寫的 command），而且 plugin 要先裝、先註冊，還要有 permission 放行。前端能做的事，因此只限於你放行的那些。

## 重點整理

- plugin 把系統功能拆開，用到才裝；一個 plugin 有 Rust crate、npm 套件和 permission 三個部分。
- `npm run tauri add <名稱>` 會改 `Cargo.toml`、`package.json`、`lib.rs`（`.plugin(...)`）和 `capabilities/default.json`。
- 漏掉 `.plugin(...)` 的話，前端呼叫會得到 `plugin ... not found`。
- Tauri 的前端沒有 Node.js API，要碰系統只能透過 plugin 或 command。
