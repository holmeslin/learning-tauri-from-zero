# Tauri 是什麼

## 本集目標

知道一個 Tauri App 由哪兩個部分組成、兩邊怎麼溝通，以及它為什麼不必把瀏覽器塞進安裝檔。

## 正文

### 一個 App，兩個部分

Tauri 讓你用網頁技術畫介面，其他事情都交給 Rust。跑起來的 App 是這個樣子：

```text
你的 App
├── 後端：一個 Rust 行程，管視窗、檔案、資料庫、系統功能
│       ▲
│       │ IPC（互相傳訊息）
│       ▼
└── 前端：視窗裡的網頁（WebView），用 HTML、CSS、TypeScript 畫介面
```

- **後端**是一個 Rust 行程，官方文件叫它 Core 行程。App 從這裡啟動，視窗由它建立，也只有它能讀寫檔案、連資料庫、呼叫系統功能。這本書裡你寫的 Rust 都在這一側。
- **前端**是視窗裡面的網頁，負責畫介面、接收使用者的操作。

### WebView：App 裡的瀏覽器

視窗裡的網頁是誰在顯示？答案是 WebView，也就是作業系統附的、可以嵌進任何 App 的瀏覽器元件。Tauri 直接借用各平台現成的那一個：

| 平台 | WebView |
| --- | --- |
| macOS | WKWebView（Safari 的 WebKit 引擎） |
| Windows | WebView2（Edge 的 Chromium 引擎） |
| Linux | WebKitGTK |

Tauri 不替你帶一個瀏覽器，所以 App 的安裝檔很小。這也是它最常被拿來和 Electron 比的地方。

### IPC：兩邊怎麼說話

前端和後端是兩個不同的行程，前端不能直接呼叫 Rust 的函式。它們靠 IPC（行程間通訊）互相傳訊息，而且所有訊息都先經過後端。這點很重要：後端可以決定哪些訊息放行、哪些擋下，第 14 章的安全性就建立在這上面。

前端要 Rust 做事的標準做法叫 command，從第 3 章開始學。這一章先看範本裡現成的一個。

### 跟 Electron 比一比

Electron 的做法是把整套 Chromium 和 Node.js 打包進每個 App，後端用 JavaScript 寫。Tauri 有兩個地方不同，照 Electron 的直覺想會出錯：

- **瀏覽器引擎隨平台而異。** Electron 自帶 Chromium，三個平台畫面一致。Tauri 用系統的 WebView：macOS 和 Linux 是 WebKit，Windows 是 Chromium 系的 WebView2，同一段 CSS 或 JavaScript 偶爾會在不同平台表現不同。所以「我的電腦上正常」不代表每個平台都正常，要在各平台測過。
- **App 裡沒有 Node.js。** 後端是 Rust，前端是單純的網頁環境。Node.js 只在開發時用來跑 Vite 和 npm，不會被打包進 App。前端程式裡沒有 `require("fs")` 這類東西，想讀檔案就要請 Rust 去做。

## 重點整理

- Tauri App 由一個 Rust 後端行程和一個顯示網頁的 WebView 前端組成。
- 後端管視窗、檔案、資料庫、系統功能；前端管介面。
- WebView 是系統附的：macOS 用 WKWebView，Windows 用 WebView2，Linux 用 WebKitGTK。Tauri 不打包瀏覽器，所以 App 小。
- 前後端是不同行程，靠 IPC 傳訊息，訊息都經過後端。
- 跟 Electron 不同：瀏覽器引擎隨平台而異，App 裡也沒有 Node.js。
