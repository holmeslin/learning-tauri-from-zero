# dialog：開檔對話框與訊息框

## 本集目標

學會用 dialog plugin 彈出系統原生的選檔、存檔對話框與訊息框，知道五個函式各自回傳什麼，並用它取代第 6 章在 macOS 上不能用的 `window.confirm()`。

## 正文

### 五個函式

dialog plugin 的前端套件是 `@tauri-apps/plugin-dialog`，這一集介紹五個函式，都回傳 Promise：

- `open(選項)`：選檔對話框。使用者選了檔案，回傳路徑（字串）；按「Cancel」回傳 `null`。選項的 `title` 是標題，`filters` 限制可以選的副檔名。加上 `multiple: true` 可以多選，回傳的就變成路徑陣列。
- `save(選項)`：存檔對話框，讓使用者決定檔名與位置。一樣回傳路徑，按「Cancel」回傳 `null`。選項的 `defaultPath` 可以給預設檔名。它只回傳路徑，不會寫檔案，寫檔是下一集 fs 的事。
- `message(訊息, 選項)`：只有「OK」一個按鈕，按下後回傳 `"Ok"`。選項用 `title` 設標題，`kind` 設圖示（`"info"`、`"warning"` 或 `"error"`）。
- `ask(訊息, 選項)`：「Yes」「No」兩個按鈕，回傳 `true` 或 `false`。
- `confirm(訊息, 選項)`：「OK」「Cancel」兩個按鈕，回傳 `true` 或 `false`。

### 範例

先照第 1 集的方式裝 plugin：

```bash
npm run tauri add dialog
```

它會在 `lib.rs` 註冊 plugin，並在 `capabilities/default.json` 加上 `dialog:default`。我們再多加一項 `core:window:allow-destroy`，最後一個範例要用，第 6 章第 6 集說過。兩個檔案現在是這樣：

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
    "dialog:default",
    "core:window:allow-destroy"
  ]
}
```

四個按鈕各呼叫一個函式，結果顯示在畫面上。`message` 第 1、2 集用過了，這裡不再放按鈕：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>dialog</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>dialog</h1>
      <div class="row">
        <button id="open-btn" type="button">選檔案</button>
        <button id="save-btn" type="button">存檔</button>
        <button id="ask-btn" type="button">問是或否</button>
        <button id="confirm-btn" type="button">確定或取消</button>
      </div>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask, confirm, open, save } from "@tauri-apps/plugin-dialog";

const msgEl: HTMLElement | null = document.querySelector("#msg");
if (!msgEl) {
  throw new Error("找不到 #msg");
}

document.querySelector("#open-btn")?.addEventListener("click", async () => {
  const path = await open({
    title: "選一個文字檔",
    filters: [{ name: "文字檔", extensions: ["txt", "md"] }],
  });
  msgEl.textContent = `open 回傳：${path}`;
});

document.querySelector("#save-btn")?.addEventListener("click", async () => {
  const path = await save({ defaultPath: "note.txt" });
  msgEl.textContent = `save 回傳：${path}`;
});

document.querySelector("#ask-btn")?.addEventListener("click", async () => {
  const yes = await ask("要刪除這個檔案嗎？", { kind: "warning" });
  msgEl.textContent = `ask 回傳：${yes}`;
});

document.querySelector("#confirm-btn")?.addEventListener("click", async () => {
  const ok = await confirm("要覆蓋原本的檔案嗎？");
  msgEl.textContent = `confirm 回傳：${ok}`;
});

getCurrentWindow().onCloseRequested(async (event) => {
  const leave = await confirm("確定要關閉視窗嗎？", { title: "關閉" });
  if (!leave) {
    event.preventDefault();
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s
     Running `target/debug/tauri-app`
```

按「選檔案」，彈出系統的選檔對話框，只能選 `.txt` 和 `.md`。選一個檔案，畫面顯示 `open 回傳：/Users/me/Downloads/hello.txt` 這樣的路徑；按「Cancel」，顯示 `open 回傳：null`。「存檔」的 `save` 同樣回傳你選的路徑或 `null`。「問是或否」按 Yes 顯示 `ask 回傳：true`，按 No 顯示 `false`。「確定或取消」按 OK 顯示 `confirm 回傳：true`，按 Cancel 顯示 `false`。

### 取代 `window.confirm()`

`main.ts` 最後一段，就是第 6 章第 6 集「關閉前確認」的原生版本。那一集提到 macOS 的 `window.confirm()` 不會彈出對話框，只好自己在頁面上畫確認列。現在改用 plugin 的 `confirm`（從 plugin 匯入，不是瀏覽器內建的同名函式），對話框由 Rust 這一側彈出，跟 WebView 的實作無關。

`onCloseRequested` 的函式可以是 `async`，Tauri 會等它跑完才決定要不要關：按「Cancel」就 `preventDefault()` 擋下關閉，按「OK」視窗就關掉。沒擋的話，關視窗要用 `destroy()`，所以 `core:window:allow-destroy` 還是要放行。

## 重點整理

- `open()` 與 `save()` 回傳選到的路徑，按取消回傳 `null`；`open()` 加 `multiple: true` 時回傳路徑陣列；`save()` 只給路徑，不寫檔案。
- `message()` 回傳 `"Ok"`；`ask()` 與 `confirm()` 回傳布林，差別只在按鈕是 Yes/No 還是 OK/Cancel。
- dialog 的 `confirm` 取代瀏覽器的 `window.confirm()`，在 `onCloseRequested` 裡 `await` 它，再決定要不要 `preventDefault()`。
