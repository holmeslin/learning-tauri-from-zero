# 視窗事件：關閉前確認

## 本集目標

學會用 `onCloseRequested` 在使用者要關視窗時攔下來，有未儲存的內容就先問一聲，並知道關閉的責任交給你之後，要放行 `destroy`。

## 正文

### 關閉是一個請求

使用者按下視窗的關閉鈕，視窗不會馬上消失。Tauri 先發出一個「要求關閉」的事件，前端用 `getCurrentWindow().onCloseRequested(處理函式)` 接。用法跟第 5 章的 `listen` 是同一套：函式收到 `event`，呼叫 `event.preventDefault()` 就能擋下，視窗繼續開著。沒有擋的話，視窗照常關閉。

### 範例：有未儲存的內容就先問

輸入框打字就當作有未儲存的內容，按「儲存」才算存好。使用者要關視窗時，如果有未儲存的內容，就擋下來，顯示確認列。`input` 事件是輸入框內容每次改變就發出，用法跟第 2 章第 8 集的 `click` 一樣；`hidden` 是 HTML 元素的屬性，設成 `true` 就把元素藏起來：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>關閉前確認</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>關閉前確認</h1>
      <div class="row">
        <input id="note-input" placeholder="隨便寫點東西" />
        <button id="save-btn" type="button">儲存</button>
      </div>
      <div id="confirm-bar" hidden>
        <p>還有沒儲存的內容，確定要關閉嗎？</p>
        <button id="discard-btn" type="button">不儲存，關閉</button>
        <button id="stay-btn" type="button">繼續編輯</button>
      </div>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();
const confirmBarEl: HTMLElement | null = document.querySelector("#confirm-bar");
let dirty = false;

document.querySelector("#note-input")?.addEventListener("input", () => {
  dirty = true;
});

document.querySelector("#save-btn")?.addEventListener("click", () => {
  dirty = false;
});

appWindow.onCloseRequested((event) => {
  if (dirty && confirmBarEl) {
    event.preventDefault();
    confirmBarEl.hidden = false;
  }
});

document.querySelector("#discard-btn")?.addEventListener("click", () => {
  appWindow.destroy();
});

document.querySelector("#stay-btn")?.addEventListener("click", () => {
  if (confirmBarEl) {
    confirmBarEl.hidden = true;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.87s
     Running `target/debug/tauri-app`
```

不打字直接按關閉鈕，視窗**沒有關**。打字之後按關閉鈕，確認列出現，按「不儲存，關閉」也關不掉。Console（第 1 章第 8 集的開發者工具）出現一則錯誤，訊息內容是：

```text
window.destroy not allowed. Permissions associated with this command: core:window:allow-destroy
```

### 關閉的責任交給你了

有了 `onCloseRequested`，Tauri 就不自己關視窗，等你的函式跑完：沒有 `preventDefault()` 的話，由 `@tauri-apps/api` 替你呼叫 `destroy()` 關掉。`destroy()` 是會改動視窗的方法，要先放行。範例裡「不儲存，關閉」也是呼叫它，所以兩條路都被擋下來。

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
    "opener:default"
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

現在不打字按關閉鈕，視窗直接關掉；打字之後按關閉鈕，出現確認列，「繼續編輯」收起確認列，「不儲存，關閉」關掉視窗；按了「儲存」之後，又能直接關閉。

### `destroy()` 與 `close()`

`close()` 等於使用者按了關閉鈕，會再發一次要求關閉的事件；`destroy()` 不問，直接關。所以在確認之後要用 `destroy()`：如果「不儲存，關閉」改呼叫 `close()`（它也要另外放行 `core:window:allow-close`），函式又會被叫一次，又擋下來，確認列永遠關不掉。

### 為什麼不用 `confirm()`

瀏覽器內建的 `window.confirm("確定嗎？")` 會彈出對話框，傳回 `true` 或 `false`。我在 macOS 上試過，Tauri 的視窗裡它不會彈出任何東西，立刻傳回 `false`，關閉就永遠被擋下來。所以這一集用頁面上的確認列。要原生的對話框，用 dialog plugin，第 7 章會正式介紹。

## 重點整理

- `getCurrentWindow().onCloseRequested(函式)` 在使用者要關視窗時被呼叫；`event.preventDefault()` 擋下關閉，沒有擋就照常關。
- 有了這個監聽，關視窗由 `destroy()` 完成，要在 `capabilities/default.json` 放行 `core:window:allow-destroy`，否則視窗關不掉，Console 出現缺少 permission 的錯誤。
- 確認之後用 `destroy()` 關，不要用 `close()`：`close()` 會再觸發一次要求關閉的事件。
- macOS 的 `window.confirm()` 不會彈出對話框，立刻傳回 `false`；確認要自己畫在頁面上，或之後用 dialog plugin。
