# 視窗設定：大小、標題與縮放

## 本集目標

學會在 `tauri.conf.json` 設定視窗的標題、大小、最小尺寸與位置，並讓視窗不能縮放。

## 正文

### 視窗寫在 `app.windows`

第 1 章第 6 集看過，`app.windows` 是一個陣列，每個元素是一個視窗，範本只有一個。這一集只改它。沒寫的欄位都有預設值：`title` 是 `Tauri App`、`width` 是 800、`height` 是 600，`label`（視窗的名字）是 `main`。`windows` 裡放兩個以上的視窗時，每個都要自己寫一個不重複的 `label`，第 4 集再細講。

### 標題、大小與位置

這份設定把視窗縮小，加上最小尺寸，並開在螢幕中央：

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
        "title": "視窗設定",
        "width": 600,
        "height": 400,
        "minWidth": 400,
        "minHeight": 300,
        "center": true
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

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.05s
     Running `target/debug/tauri-app`
```

- `title`：標題列的文字。
- `width`、`height`：視窗開啟時的大小。單位是邏輯像素（logical pixel），不是螢幕上真正的點。在縮放 2 倍的螢幕上，600×400 實際是 1200×800 個像素；設定檔一律寫邏輯像素，不用自己換算。
- `minWidth`、`minHeight`：使用者拖拉視窗邊緣時，縮到這個大小就停住。`maxWidth`、`maxHeight` 是反過來的上限。
- `center`：開在螢幕中央。不寫的話由作業系統決定位置，也可以用 `x`、`y` 指定左上角的位置。

### 不能縮放的視窗

有些視窗大小是固定的，例如小工具或「關於」視窗。`resizable` 設成 `false`，使用者就拉不動邊緣：

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
        "title": "視窗設定",
        "width": 600,
        "height": 400,
        "center": true,
        "resizable": false
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

`npm run tauri dev` 的輸出（改了 `tauri.conf.json` 之後）：

```text
        Info File src-tauri/tauri.conf.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.12s
     Running `target/debug/tauri-app`
```

視窗還是 600×400，但拖拉邊緣沒有反應，也不能最大化：在 macOS 上，標題列綠色的按鈕變成灰的。這時候 `minWidth`、`minHeight` 就用不到了，所以拿掉。

## 重點整理

- `app.windows` 的每個元素是一個視窗；沒寫的欄位用預設值（800×600，標題 `Tauri App`，label `main`）。
- `title`、`width`、`height` 設定標題與開啟時的大小，單位是邏輯像素。
- `minWidth`、`minHeight`（以及 `maxWidth`、`maxHeight`）限制使用者能縮放的範圍；`center` 開在螢幕中央，`x`、`y` 指定位置。
- `resizable: false` 讓視窗不能縮放，也不能最大化。
