# 從前端操作視窗：`getCurrentWindow`

## 本集目標

學會在前端用 `getCurrentWindow()` 拿到目前的視窗並呼叫它的方法，也知道改動視窗的方法要先在 `capabilities/default.json` 放行。

## 正文

### 目前這個視窗

`@tauri-apps/api/window` 的 `getCurrentWindow()`，回傳代表「執行這段程式的視窗」的 `Window` 物件。它的方法跟 `invoke` 一樣要跟 Rust 那邊溝通，所以回傳 Promise（第 2 章第 10 集），要 `await`。

### 範例：改視窗的標題

輸入框打新的標題，按下按鈕就呼叫 `setTitle`。失敗的話，把錯誤顯示在畫面上，寫法跟第 3 章第 6 集的 `catch` 一樣：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>操作視窗</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>操作視窗</h1>
      <form class="row" id="title-form">
        <input id="title-input" placeholder="新的標題" />
        <button type="submit">改標題</button>
      </form>
      <p id="title-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();
const formEl: HTMLFormElement | null = document.querySelector("#title-form");
const inputEl: HTMLInputElement | null = document.querySelector("#title-input");
const msgEl: HTMLElement | null = document.querySelector("#title-msg");

formEl?.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!inputEl || !msgEl) {
    return;
  }
  try {
    await appWindow.setTitle(inputEl.value);
    msgEl.textContent = `標題改成了：${await appWindow.title()}`;
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s
     Running `target/debug/tauri-app`
```

輸入 `新標題` 按下按鈕，標題列沒變，段落出現：

```text
出錯了：window.set_title not allowed. Permissions associated with this command: core:window:allow-set-title
```

### 被 permission 擋下來

程式沒有錯，是 Tauri 擋的：前端呼叫視窗的功能，要先被允許。訊息最後一段就是答案：要有 `core:window:allow-set-title` 這項 permission。

範本的 `capabilities/default.json` 有一行 `core:default`，它放行的是「讀」：`title()`、`isMaximized()`、`innerSize()` 這類查詢都能直接用。會改動視窗的方法，像 `setTitle`、`minimize`、`close`，得一項一項加。把這一項加進去：

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-set-title",
    "opener:default"
  ]
}
```

`permissions` 是一份清單，每一項放行一組功能，把缺的那一項加進去就行；`windows` 寫著這份清單給哪個視窗用，`main` 是範本的視窗。permission 第 7 章會正式介紹，capability 完整的模型留到第 14 章。

`npm run tauri dev` 的輸出（改了 `capabilities/default.json` 之後）：

```text
        Info File src-tauri/capabilities/default.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.71s
     Running `target/debug/tauri-app`
```

再試一次，標題列變成 `新標題`，段落顯示「標題改成了：新標題」。`minimize()`、`close()` 這類會改動視窗的方法也一樣，沒放行就被擋，錯誤訊息的最後一段會寫出缺哪一項。

## 重點整理

- `getCurrentWindow()` 回傳目前視窗的 `Window` 物件，方法回傳 Promise，要 `await`。
- 範本的 `core:default` 對視窗的方法只放行查詢（`title()`、`isMaximized()` 等）；改動視窗的方法會被擋，錯誤訊息最後一段就是缺的 permission。
- 到 `src-tauri/capabilities/default.json` 的 `permissions` 加上 `core:window:allow-set-title` 這類項目；改完會重新編譯。
