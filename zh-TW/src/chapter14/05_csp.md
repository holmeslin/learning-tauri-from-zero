# CSP

## 本集目標

寫出一份剛好夠用的 CSP，看它擋下行內腳本與遠端圖片，並知道 `tauri dev` 載入 Vite 的頁面時套不到 CSP，要用 debug 版本才看得到效果。

## 正文

### CSP 管的是頁面自己

前四集的 permission 管的是「前端能不能呼叫 Rust」。CSP（Content Security Policy，內容安全政策）管的是另一件事：頁面自己能載入什麼、執行什麼。它是一份寫給 WebView 的規則，由 WebView 來執行，頁面想載入圖片、執行腳本、連網路，都先對照規則，不符合就擋下。

這補上了 permission 看不到的地方。第 12 章的預覽把 HTML 放進頁面，已經用兩道關卡擋過一次，CSP 是第二層：就算一段 `<img onerror="…">` 溜進來，WebView 也不會執行行內腳本（inline script）；就算腳本想把資料送到外面，也得先過 `connect-src` 這一關。

一份 CSP 由很多條指令（directive）組成，每一條管一類資源：`default-src` 是其他指令沒寫的資源都照它，`script-src` 管腳本，`img-src` 管圖片，`connect-src` 管 `fetch`、WebSocket 這類連線，`style-src` 管樣式。指令後面是允許的來源，常見的有 `'self'`（頁面自己的來源，單引號是語法的一部分）、`data:`、完整的網址，還有 `'none'`（全部不准）。

### 範例：模擬第 12 章的預覽

這個頁面模擬不可信的內容溜進預覽之後的樣子。第 12 章的預覽是用 `innerHTML` 放進 HTML 的，這裡改用 `createElement` 把同樣的東西一個一個做出來，放進 `#preview`：

- 第一張圖指向外部網址（這裡用 `127.0.0.1` 的一個位址代替，不需要真的有伺服器）。
- 第二張圖的來源是壞的，會觸發 `onerror`。`setAttribute("onerror", …)` 設的是 HTML 屬性，效果跟 HTML 裡寫 `<img onerror="…">` 一樣，叫行內事件處理（inline event handler）。它要是執行了，`#msg` 那一行「惡意程式沒有執行」就會被改掉。

程式也監聽 `securitypolicyviolation`：CSP 每擋下一樣東西，WebView 就發這個事件，`violatedDirective` 是違反的指令，`blockedURI` 是被擋下的東西，行內腳本會是 `inline`。最後呼叫範本的 `greet`（`lib.rs` 沒動），確認 IPC 還通，回應放進 `#greet-msg`；違規的紀錄放進 `#violations`。

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>CSP</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>CSP</h1>
      <div id="preview"></div>
      <p id="msg">惡意程式沒有執行</p>
      <p id="greet-msg"></p>
      <ul id="violations"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const previewEl: HTMLElement | null = document.querySelector("#preview");
const greetEl: HTMLElement | null = document.querySelector("#greet-msg");
const listEl: HTMLElement | null = document.querySelector("#violations");
if (!previewEl || !greetEl || !listEl) {
  throw new Error("找不到 #preview、#greet-msg 或 #violations");
}

document.addEventListener("securitypolicyviolation", (event) => {
  const item = document.createElement("li");
  item.textContent = `${event.violatedDirective}：${event.blockedURI}`;
  listEl.append(item);
});

const remoteImg = document.createElement("img");
remoteImg.src = "http://127.0.0.1:8123/pixel.png";
previewEl.append(remoteImg);

const brokenImg = document.createElement("img");
brokenImg.setAttribute(
  "onerror",
  "document.querySelector('#msg').textContent = '惡意程式執行了'",
);
brokenImg.src = "x";
previewEl.append(brokenImg);

const showGreeting = async () => {
  try {
    greetEl.textContent = await invoke<string>("greet", { name: "CSP" });
  } catch (error) {
    greetEl.textContent = `出錯了：${error}`;
  }
};
showGreeting();
```

先用範本的設定（`"csp": null`，不限制）跑：

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.33s
     Running `target/debug/tauri-app`
```

畫面顯示「惡意程式執行了」，下面的違規清單是空的。沒有 CSP，行內的 `onerror` 照跑。

### 設定 CSP

把 `app.security.csp` 換成一份 CSP。可以寫成一個字串，也可以像這樣寫成物件，一個指令一個欄位：

`src-tauri/tauri.conf.json`：

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "tauri-app",
  "version": "0.1.0",
  "identifier": "tw.example.tauri-app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "tauri-app",
        "width": 800,
        "height": 600
      }
    ],
    "security": {
      "csp": {
        "default-src": "'self'",
        "img-src": "'self' data:",
        "connect-src": "ipc: http://ipc.localhost"
      }
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`npm run tauri dev` 的輸出（改了 `tauri.conf.json` 之後）：

```text
        Info File src-tauri/tauri.conf.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.46s
     Running `target/debug/tauri-app`
```

畫面還是「惡意程式執行了」，違規清單還是空的。CSP 好像沒有作用？

### `tauri dev` 的頁面套不到 CSP

CSP 是 Tauri 提供頁面時，附在回應上的。`tauri dev` 載入的是 Vite 開發伺服器的頁面（`devUrl`，第 1 章第 6 集），回應是 Vite 給的，Tauri 沒有經手，所以不會套上 CSP。`tauri.conf.json` 另有一個 `devCsp`，是開發模式專用的另一份，同樣只會套在 Tauri 自己提供的頁面上。要看效果，得用 Tauri 自己提供頁面的版本，也就是打包過的 App。這裡用 debug 版本（第 1 章第 8 集提過），`--no-bundle` 表示只編譯，不做安裝檔，打包是第 15 章的事：

```bash
npm run tauri build -- --debug --no-bundle
```

`npm run tauri build -- --debug --no-bundle` 的輸出（節錄）：

```text
     Running beforeBuildCommand `npm run build`
   Compiling tauri v2.12.6
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.84s
       Built application at: ~/tauri-app/src-tauri/target/debug/tauri-app
```

執行編譯出來的程式：

```bash
./src-tauri/target/debug/tauri-app
```

（Windows 是 `src-tauri\target\debug\tauri-app.exe`。）這次畫面顯示「惡意程式沒有執行」，`greet` 的問候語照常出現，違規清單多了兩行：

```text
img-src：http://127.0.0.1:8123/pixel.png
script-src-attr：inline
```

### 這份 CSP 在做什麼

- `default-src 'self'`：預設只准用頁面自己的來源。`script-src` 沒另外寫，也照這條，所以行內事件處理（`script-src-attr`）被擋，`onerror` 沒有跑。（違規清單上寫 `script-src-attr`，是在 macOS 的 WebView 實測的結果。`script-src-attr` 專管行內事件處理，沒寫的話依序回退到 `script-src`、`default-src`。別的平台的 WebView 引擎不一定一樣，清單上的指令名稱可能不同，我們沒有確認。）Tauri 打包時會替寫在 HTML 裡的 `<script>`、`<style>` 在 CSP 裡補上雜湊值（hash）或 nonce；用 `src` 載入的腳本來自頁面自己的來源，`'self'` 已經放行。所以不用為了自己的程式放寬。
- `img-src 'self' data:`：圖片只能來自自己或 `data:`。第 12 章第 2 集說過，預覽放行 `http`、`https` 的圖片，代價是打開別人的檔案時 App 會去連圖片的網址；有了這一行，WebView 在送出請求之前就擋下了。想放行網路上的圖片，就得自己在這裡加。
- `connect-src ipc: http://ipc.localhost`：`invoke` 走的是 Tauri 的 `ipc:` 協定（Windows 與 Android 是 `http://ipc.localhost`）。拿掉這一行重新打包，`greet` 一樣有回應，但違規清單會多一行 `connect-src：ipc://localhost/greet`：Tauri 的第一種連線方式被擋，才改用備援的 `postMessage`。能用，但先報一次錯才改道，所以要寫進去。

設了 CSP 之後，一定要用打包的版本再把每個功能跑一遍。太嚴的 CSP 只會讓打包版壞掉，`tauri dev` 一點事也沒有。也別為了省事寫 `'unsafe-inline'`、`'unsafe-eval'` 或 `*`，那等於把這道防線拆掉。

## 重點整理

- CSP 是寫給 WebView 的規則，管頁面能載入、執行什麼，補上 permission 管不到的地方。設定在 `tauri.conf.json` 的 `app.security.csp`，字串或物件都可以。
- 範本的 `null` 是不限制。一份夠用的起點是 `default-src 'self'`、`img-src 'self' data:`、`connect-src ipc: http://ipc.localhost`。
- 行內事件處理（`onerror`）與遠端圖片會被擋下，`securitypolicyviolation` 事件可以看到每一筆。
- `tauri dev` 載入 Vite 的頁面時套不到 CSP；用 `npm run tauri build -- --debug --no-bundle` 驗證。
- 不要用 `'unsafe-inline'`、`'unsafe-eval'` 或 `*` 放寬。
