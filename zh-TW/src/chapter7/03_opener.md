# opener：開網址與檔案

## 本集目標

學會用 `openUrl` 在系統瀏覽器開網址、用 `openPath` 用系統預設的程式開檔案，並知道 `openPath` 除了 permission，還要設定能開哪些路徑。

## 正文

### opener 做什麼

opener plugin 把網址或檔案交給作業系統處理：網址用預設瀏覽器開，檔案用預設程式開（`.txt` 就用文字編輯器）。範本已經裝好它，`opener:default` 也已經在 `capabilities/default.json` 裡，不用再跑 `tauri add`。前端套件 `@tauri-apps/plugin-opener` 有兩個函式：`openUrl(網址)` 和 `openPath(路徑)`，都回傳 Promise。

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

`capabilities/default.json` 沒動，`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s
     Running `target/debug/tauri-app`
```

按「開網址」，系統預設瀏覽器打開 `https://tauri.app`，畫面顯示「已交給系統開啟」。把網址改成 `ftp://example.com` 再按，畫面顯示：

```text
出錯了：Not allowed to open url ftp://example.com
```

在第二個輸入框填一個檔案的完整路徑，例如「下載」資料夾裡的 `hello.txt`（路徑依你的電腦而不同），按「開檔案」：

```text
出錯了：opener.open_path not allowed. Permissions associated with this command: opener:allow-open-path
```

### 兩層檢查：permission 與 scope

這兩個錯誤長得不一樣，因為被兩層檢查擋下（scope 第 14 章會正式介紹，第 5 集的 fs plugin 也會用到）：

- permission 管「能不能用這個函式」。`openPath` 沒被放行，所以得到第 2 集看過的 `not allowed`。
- scope 管「這個函式能碰哪些網址或路徑」。`opener:default` 放行了 `openUrl`，但只放行 `http://`、`https://`、`mailto:`、`tel:` 開頭的網址，`ftp://` 不在範圍內，所以得到 `Not allowed to open url`。

### 替 `openPath` 設定 scope

只加 `opener:allow-open-path` 還不夠：函式放行了，可是沒有任何路徑在範圍內，畫面會顯示 `Not allowed to open path …`。所以 permission 要寫成物件，把允許的路徑一起帶進去：

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

`identifier` 是 permission 的名稱，`allow` 是 scope，每一項 `path` 是一個允許的路徑。`$DOWNLOAD` 是 Tauri 內建的路徑變數，代表這台電腦的「下載」資料夾；`**` 代表底下所有的檔案和子資料夾。

`npm run tauri dev` 的輸出（改了 `capabilities/default.json` 之後）：

```text
        Info File src-tauri/capabilities/default.json changed. Rebuilding application...
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.71s
     Running `target/debug/tauri-app`
```

再試一次：下載資料夾裡的檔案會用預設程式打開，畫面顯示「已交給系統開啟」。換成下載資料夾以外的檔案，像 `/etc/hosts`，會得到 `Not allowed to open path /etc/hosts`。`$DOWNLOAD/**` 不包含資料夾本身，所以填「下載」資料夾自己的路徑也會被擋。

順帶一提，第 2 章第 2 集說過，範本裡 `target="_blank"` 的連結會交給系統瀏覽器開，就是 opener 做的：它在頁面裡放了一小段程式，攔下這類連結被按下的動作，改呼叫 `openUrl`。所以連結一樣受 permission 管。把 `opener:default` 拿掉，點範本的 logo 就開不了，Console 會出現缺少 `opener:allow-open-url` 的錯誤。

## 重點整理

- `openUrl(網址)` 用預設瀏覽器開網址，`openPath(路徑)` 用預設程式開檔案，都要 `await`。
- 錯誤訊息有兩種：`not allowed. Permissions associated…` 是缺 permission；`Not allowed to open url/path …` 是 permission 有了，但網址或路徑不在 scope 內。
- `opener:default` 放行 `openUrl`，只限 `http`、`https`、`mailto`、`tel`；`openPath` 要另外放行 `opener:allow-open-path`，並用 `allow` 列出允許的路徑，例如 `$DOWNLOAD/**`（不含資料夾本身）。
- 範本裡 `target="_blank"` 的連結也是靠 opener 打開，同樣要有 `opener:default`。
