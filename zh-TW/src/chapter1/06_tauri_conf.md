# `tauri.conf.json` 初探

## 本集目標

看懂 `src-tauri/tauri.conf.json` 的主要欄位，並改它的視窗標題與大小，看設定怎麼影響 App。

## 正文

### 範本的設定檔

`tauri.conf.json` 是 Tauri 的設定檔。範本裡的內容是這樣：

`src-tauri/tauri.conf.json`（範本原本的內容）：

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
      "csp": null
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

### 最上面的幾個欄位

- `$schema`：告訴編輯器這份 JSON 的格式，讓它能自動補全、檢查欄位。
- `productName`：App 給使用者看的名字，安裝檔和應用程式的名稱都從這裡來。
- `version`：App 的版本號。
- `identifier`：App 的唯一代號，格式是反向網域，只能有英文字母、數字、`-` 和 `.`，不能跟別的 App 重複。第 3 集建專案時輸入的就是它。

### `build`：前端怎麼接上來

- `beforeDevCommand`：`tauri dev` 啟動時先執行的指令，就是上一集輸出裡的 `BeforeDevCommand`，這裡用來啟動 Vite。
- `devUrl`：開發時視窗載入的網址。
- `beforeBuildCommand`：`tauri build` 打包前先執行的指令。`npm run build` 會檢查 TypeScript，再用 Vite 把前端打包進專案根目錄的 `dist/`。
- `frontendDist`：打包好的前端放在哪裡，路徑從 `src-tauri/` 算起，所以 `../dist` 就是剛剛的 `dist/`。

開發時視窗載入一個網址（`devUrl`），正式版則是把 `frontendDist` 資料夾的檔案放進 App 裡，不需要伺服器。

### `app`：視窗與安全

- `windows`：視窗的清單，範本只有一個，標題 `tauri-app`，寬 800、高 600。沒有特別指定的話，視窗的 label 預設是 `main`，`capabilities/default.json` 裡的 `"windows": ["main"]` 指的就是它。視窗的完整設定在第 6 章。
- `security.csp`：內容安全政策（CSP），範本設成 `null`，表示不限制。第 14 章會回來處理。
- `withGlobalTauri`：打開的話，前端可以用全域變數 `window.__TAURI__` 呼叫 Tauri 的功能。本書的前端程式都用 `import`，所以不會用到它。

### `bundle`：打包

整段是 `tauri build` 打包安裝檔時用的設定，`icon` 列的就是 `icons/` 裡的檔案。第 15 章再細講。

### 改改看

把視窗的標題與大小改掉，其他欄位維持原樣：

`src-tauri/tauri.conf.json`（改了視窗的標題、寬、高）：

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
        "title": "我的第一個 Tauri App",
        "width": 600,
        "height": 480
      }
    ],
    "security": {
      "csp": null
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

`tauri dev` 還開著的話，存檔後會看到：

`npm run tauri dev` 的輸出（改了 `tauri.conf.json` 之後）：

```text
        Info File src-tauri/tauri.conf.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.76s
     Running `target/debug/tauri-app`
```

設定檔是編譯時讀進程式的，所以改了它也會重新編譯、重啟。新的視窗標題是「我的第一個 Tauri App」，大小 600×480。

### 跟 Electron 比一比

Electron 的視窗是在主行程的程式碼裡，用 `new BrowserWindow({ width: 800, height: 600 })` 建出來的。Tauri 預設把視窗寫在設定檔，所以你在 `lib.rs` 裡找不到建視窗的那一行，是 App 啟動時照 `tauri.conf.json` 開的。想用 Rust 程式碼建視窗也可以，第 6 章會教。

## 重點整理

- `tauri.conf.json` 是 Tauri 的設定檔，在編譯時讀進程式，改了會觸發重新編譯。
- `identifier` 是 App 的唯一代號；`productName` 與 `version` 是給使用者看的名稱與版本。
- `build` 說明前端怎麼接上：開發用 `devUrl`，正式版用 `frontendDist`。
- `app.windows` 設定視窗的標題與大小，預設視窗的 label 是 `main`。
- `bundle` 是打包設定，`security.csp` 是安全設定，這兩塊留到第 15、14 章。
