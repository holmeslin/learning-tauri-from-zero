# dialog：開檔對話框與訊息框

## 本集目標

學會用 dialog plugin 彈出系統原生的選檔對話框與訊息框，知道四個函式各自回傳什麼，並用它取代第 6 章在 macOS 上不能用的 `window.confirm()`。

## 正文

### 四個函式

dialog plugin 的前端套件是 `@tauri-apps/plugin-dialog`，這一集用到四個函式，都回傳 Promise：

- `open(選項)`：選檔對話框。使用者選了檔案，回傳路徑（字串）；按「Cancel」回傳 `null`。選項的 `title` 是標題，`filters` 限制可以選的副檔名。加上 `multiple: true` 可以多選，回傳的就變成路徑陣列。
- `message(訊息, 選項)`：只有「OK」一個按鈕，按下後回傳 `"Ok"`。
- `ask(訊息, 選項)`：「Yes」「No」兩個按鈕，回傳 `true` 或 `false`。
- `confirm(訊息, 選項)`：「OK」「Cancel」兩個按鈕，回傳 `true` 或 `false`。

訊息框的選項可以用 `title` 設標題，`kind` 設圖示（`"info"`、`"warning"` 或 `"error"`）。

### 範例

先照第 1 集的方式裝 plugin：

```bash
npm run tauri add dialog
```

它會在 `lib.rs` 註冊 plugin，並在 `capabilities/default.json` 加上 `dialog:default`。我們再多加一項 `core:window:allow-destroy`，最後一個範例要用，第 6 章第 6 集說過：

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-destroy",
    "opener:default",
    "dialog:default"
  ]
}
```

四個按鈕各呼叫一個函式，結果顯示在畫面上：

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
        <button id="message-btn" type="button">訊息</button>
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
import { ask, confirm, message, open } from "@tauri-apps/plugin-dialog";

const msgEl: HTMLElement | null = document.querySelector("#msg");
if (!msgEl) {
  throw new Error("找不到 #msg");
}

document.querySelector("#open-btn")?.addEventListener("click", async () => {
  const path = await open({
    title: "選一個文字檔",
    filters: [{ name: "文字檔", extensions: ["txt", "md"] }],
  });
  msgEl.textContent = path === null ? "沒有選檔案" : `選了：${path}`;
});

document.querySelector("#message-btn")?.addEventListener("click", async () => {
  const result = await message("存檔完成", { title: "提示" });
  msgEl.textContent = `message 回傳：${result}`;
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
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s
     Running `target/debug/tauri-app`
```

按「選檔案」，彈出系統的選檔對話框，只能選 `.txt` 和 `.md`。選一個檔案，畫面顯示 `選了：/Users/me/Downloads/hello.txt` 這樣的路徑；按「Cancel」，顯示「沒有選檔案」。「訊息」顯示 `message 回傳：Ok`。「問是或否」按 Yes 顯示 `ask 回傳：true`，按 No 顯示 `false`。「確定或取消」按 OK 顯示 `confirm 回傳：true`，按 Cancel 顯示 `false`。

選到的路徑只是一個字串，怎麼讀檔案的內容是下一個 plugin 的事，第 5 集會正式介紹 fs。

### 取代 `window.confirm()`

`main.ts` 最後一段，就是第 6 章第 6 集「關閉前確認」的原生版本。那一集提到 macOS 的 `window.confirm()` 不會彈出對話框，只好自己在頁面上畫確認列。現在改用 dialog plugin 的 `confirm`：

- `onCloseRequested` 的函式可以是 `async`，Tauri 會等它跑完，才決定要不要關。
- 按視窗的關閉鈕，彈出「確定要關閉視窗嗎？」。按「Cancel」，`confirm` 回傳 `false`，`event.preventDefault()` 擋下關閉，視窗留著。按「OK」，沒有擋，視窗就關掉。
- 程式碼裡的 `confirm` 是從 plugin 匯入的，不是瀏覽器內建的同名函式。對話框由 Rust 這一側彈出，跟 WebView 的實作無關。
- 沒擋的話，關視窗要用 `destroy()`，所以 `core:window:allow-destroy` 還是要放行。

## 重點整理

- `open()` 回傳選到的路徑，按取消回傳 `null`；`multiple: true` 時回傳路徑陣列。
- `message()` 回傳 `"Ok"`；`ask()` 與 `confirm()` 回傳布林，差別只在按鈕是 Yes/No 還是 OK/Cancel。
- `dialog:default` 放行 `open`、`save` 和三種訊息框，不用逐個加。
- dialog 的 `confirm` 取代瀏覽器的 `window.confirm()`，在 `onCloseRequested` 裡 `await` 它，再決定要不要 `preventDefault()`。
