# opener：開網址與檔案

## 本集目標

學會用 `openUrl` 在系統瀏覽器開網址、用 `openPath` 用系統預設的程式開檔案，並知道 `openPath` 要另外放行，還要列出能開哪些路徑。

## 正文

### opener 做什麼

opener plugin 把網址或檔案交給作業系統處理：網址用預設瀏覽器開，檔案用預設程式開（`.txt` 就用文字編輯器）。範本已經裝好它，`opener:default` 也已經在 `capabilities/default.json` 裡，不用再跑 `tauri add`。前端套件 `@tauri-apps/plugin-opener` 的 `openUrl(網址)` 和 `openPath(路徑)` 都回傳 Promise。

### 範例：開網址、開檔案

兩個輸入框各配一個按鈕。出錯的話，把錯誤顯示在畫面上，寫法跟第 6 章一樣：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>opener</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>opener</h1>
      <form class="row" id="url-form">
        <input id="url-input" value="https://tauri.app" />
        <button type="submit">開網址</button>
      </form>
      <form class="row" id="path-form">
        <input id="path-input" placeholder="檔案的完整路徑" />
        <button type="submit">開檔案</button>
      </form>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { openPath, openUrl } from "@tauri-apps/plugin-opener";

const urlEl: HTMLInputElement | null = document.querySelector("#url-input");
const pathEl: HTMLInputElement | null = document.querySelector("#path-input");
const msgEl: HTMLElement | null = document.querySelector("#msg");
if (!urlEl || !pathEl || !msgEl) {
  throw new Error("找不到頁面上的元素");
}

const report = async (action: Promise<void>) => {
  try {
    await action;
    msgEl.textContent = "已交給系統開啟";
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
};

document.querySelector("#url-form")?.addEventListener("submit", (event) => {
  event.preventDefault();
  report(openUrl(urlEl.value));
});

document.querySelector("#path-form")?.addEventListener("submit", (event) => {
  event.preventDefault();
  report(openPath(pathEl.value));
});
```

`capabilities/default.json` 沒動，還是範本的樣子。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s
     Running `target/debug/tauri-app`
```

按「開網址」，系統預設瀏覽器打開 `https://tauri.app`，畫面顯示「已交給系統開啟」。

在第二個輸入框填一個檔案的完整路徑，例如「下載」資料夾裡的 `hello.txt`（路徑依你的電腦而不同），按「開檔案」：

```text
出錯了：opener.open_path not allowed. Permissions associated with this command: opener:allow-open-path
```

### 放行 `openPath`

`opener:default` 沒有包含 `openPath`，所以要自己加 `opener:allow-open-path`。但光放行函式還不夠：`openPath` 碰的是你電腦上的檔案，所以還要列出允許開哪些路徑，這叫 scope，第 5 集會再遇到，第 14 章才會完整介紹。permission 因此要寫成物件，把允許的路徑一起帶進去：

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
    {
      "identifier": "opener:allow-open-path",
      "allow": [{ "path": "$DOWNLOAD/**" }]
    }
  ]
}
```

`identifier` 是 permission 的名稱，`allow` 列出允許的路徑：`$DOWNLOAD` 代表這台電腦的「下載」資料夾，`**` 代表底下所有的檔案。

改完存檔，`tauri dev` 會自動重新編譯。再試一次：下載資料夾裡的檔案會用預設程式打開，畫面顯示「已交給系統開啟」。其他地方的檔案還是會被擋，畫面顯示 `Not allowed to open path …`。

## 重點整理

- `openUrl(網址)` 用預設瀏覽器開網址，`openPath(路徑)` 用預設程式開檔案，都要 `await`。
- `opener:default` 放行 `openUrl`，不含 `openPath`。
- `openPath` 要另外放行 `opener:allow-open-path`，並用 `allow` 列出允許的路徑，例如 `$DOWNLOAD/**`；路徑不在清單內會得到 `Not allowed to open path …`。
- 路徑清單叫 scope，第 14 章會正式介紹。
