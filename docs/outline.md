# 從零開始學 Tauri — 全書目錄規劃（草案）

Tauri 2.12 · Rust 1.99（edition 2024）· Node LTS + npm · 前端 vanilla TypeScript + Vite · 繁中 · 給讀完《從零開始學 Rust》的人 · 一集一個小概念

讀者會 Rust（也會 Go），但沒寫過前端、沒用過 Tauri。Rust 語法不再教，要教的是前端基礎（第 2 章）與 Tauri 本身。寫作規則見 `docs/style-guide.md`，名詞見 `GLOSSARY.md`。本檔只標註各章需要特別注意的進度邊界。

> 這是草案，章節與集數在 PR 審閱時調整。定案後每一章開一個 GitHub issue。

# 第一部 基礎

## 第 1 章 第一個 Tauri App
01 Tauri 是什麼（WebView + Rust 行程）· 02 安裝前置需求（macOS、Windows、Linux）· 03 用 `create-tauri-app` 建專案 · 04 專案結構導覽（`src/` 與 `src-tauri/`）· 05 `npm run tauri dev` 與熱重載 · 06 `tauri.conf.json` 初探 · 07 讀懂範本的 `greet` · 08 開發者工具與除錯

邊界：第 7 集只「讀懂」範本：前端的語法第 2 章才教，`#[tauri::command]` 與 `invoke` 的寫法第 3 章才正式教。

## 第 2 章 前端速成
01 網頁由什麼組成（HTML 結構、CSS 外觀、JS/TS 行為）· 02 HTML 元素與屬性 · 03 CSS 選擇器與 flex 排版 · 04 TypeScript 是什麼（跟 JavaScript、跟 Rust 的型別比一比）· 05 `let`、`const` 與函式、箭頭函式 · 06 物件、陣列、`type` 與 `interface` · 07 DOM：`querySelector`、修改內容與 `console.log`（輸出到開發者工具的 Console）· 08 事件：`addEventListener` · 09 表單與輸入值 · 10 `Promise` 與 `async`/`await`（跟 Rust 的 `Future` 比一比）· 11 `import`/`export`、npm 與 Vite

邊界：本章只在第 1 章建好的專案的 `index.html`、`src/main.ts`、`src/styles.css` 裡寫前端，第 11 集可以在 `src/` 底下新增 `.ts` 模組檔；完全不碰 `src-tauri`，也不呼叫 `invoke`。不用框架，不另外安裝 npm 套件，範例用瀏覽器內建的 API 完成；範本本來就有的 npm 與 Vite 到第 11 集才說明。

## 第 3 章 Command：前端呼叫 Rust
01 `#[tauri::command]` 與 `generate_handler!` · 02 前端的 `invoke` · 03 傳參數（snake_case 與 camelCase 的對應）· 04 回傳值與 serde · 05 struct 當參數與回傳值（`Serialize`、`Deserialize`）· 06 回傳 `Result` · 07 自訂錯誤型別（thiserror 加 `Serialize`）· 08 async command · 09 在 command 裡拿到 `AppHandle` 與 `WebviewWindow` · 10 把 command 拆到模組

## 第 4 章 狀態管理
01 `manage` 與 `State` · 02 用 `Mutex` 包住可變狀態 · 03 async command 裡的狀態（`tauri::async_runtime::Mutex`）· 04 在 `setup` 裡初始化 · 05 從 `AppHandle` 取得狀態

## 第 5 章 事件與 Channel
01 Rust 送事件給前端：`emit` · 02 前端 `listen` 與取消監聽 · 03 前端送事件給 Rust · 04 只送給某個視窗 · 05 `Channel`：串流資料 · 06 長時間工作回報進度

## 第 6 章 視窗
01 視窗設定（大小、標題、可否縮放）· 02 從前端操作視窗（`getCurrentWindow`）· 03 從 Rust 開新視窗（`WebviewWindowBuilder`）· 04 多個視窗與 label · 05 無邊框與自訂標題列 · 06 視窗事件（關閉前確認）

## 第 7 章 Plugin
01 plugin 是什麼、怎麼加（`npm run tauri add`）· 02 permission 初探（`capabilities/default.json`）· 03 opener：開網址與檔案 · 04 dialog：開檔對話框與訊息框 · 05 fs：讀寫檔案與 scope · 06 路徑：App 的資料夾 · 07 store：儲存設定 · 08 notification · 09 clipboard-manager · 10 log

邊界：寫本章之前，先把本書用到的 plugin（Rust crate 與 npm 套件）加進 `tools/checkcode/template`，鎖住版本。permission 只講「沒加會被擋、加了就能用」，完整的權限模型在第 14 章。

## 第 8 章 選單、系統匣與快捷鍵
01 應用程式選單 · 02 處理選單事件 · 03 右鍵選單 · 04 系統匣圖示 · 05 系統匣選單 · 06 global-shortcut：全域快捷鍵 · 07 single-instance：只允許開一個

## 第 9 章 測試與除錯
01 把邏輯從 command 抽出來測 · 02 `tauri::test` 的 mock runtime · 03 前端 mock IPC（`@tauri-apps/api/mocks`）· 04 看 log 與常見錯誤

# 第二部 實戰

每章是一個完整的小專案，章內一集只做一個零件。順序由簡單到複雜。

## 第 10 章 實戰：筆記 App
01 規劃畫面與資料 · 02 筆記清單 · 03 新增與編輯 · 04 存到 App 資料夾（fs）· 05 匯入、匯出（dialog）· 06 錯誤處理與提示 · 07 收尾

## 第 11 章 實戰：SQLite 待辦清單
01 規劃資料表 · 02 rusqlite（`bundled`）與 `State<Mutex<Connection>>` · 03 新增與列出 · 04 完成與刪除 · 05 用事件通知畫面更新 · 06 資料庫結構升級（migration）· 07 收尾

邊界：資料庫只在 Rust 端用 rusqlite 存取，前端對資料庫的操作只透過 `invoke`，事件只負責通知；不使用 tauri-plugin-sql。

## 第 12 章 實戰：Markdown 編輯器
01 規劃 · 02 Rust 端轉 HTML（pulldown-cmark）· 03 即時預覽 · 04 開檔與存檔 · 05 自動儲存 · 06 未儲存就關閉的提醒 · 07 檔案關聯（雙擊 `.md` 開啟）· 08 收尾

## 第 13 章 實戰：系統匣小工具
01 規劃 · 02 沒有主視窗的 App · 03 系統匣選單 · 04 全域快捷鍵叫出視窗 · 05 設定視窗（store）· 06 開機自動啟動（autostart）· 07 收尾

# 第三部 上線

## 第 14 章 安全性
01 Tauri 的安全模型 · 02 capabilities 深入 · 03 permission 與 scope · 04 替自己的 command 定義 permission · 05 CSP · 06 isolation pattern

## 第 15 章 打包與安裝檔
01 `npm run tauri build`（含 `--debug`，以及用 `devtools` feature 在 release 版打開開發者工具）· 02 圖示（`tauri icon`）· 03 bundle 設定與版本號 · 04 macOS：`.app` 與 `.dmg` · 05 Windows：MSI 與 NSIS · 06 Linux：AppImage、deb、rpm · 07 一起打包資源檔

## 第 16 章 自動更新與程式碼簽章
01 updater plugin · 02 產生簽章金鑰 · 03 更新資訊檔 `latest.json` · 04 在 App 裡檢查與安裝更新 · 05 macOS 簽章與公證 · 06 Windows 簽章

邊界：簽章需要付費憑證的部分（Apple Developer、Windows 憑證）只說明流程，範例不依賴真的憑證。

## 第 17 章 GitHub Actions 跨平台發佈
01 tauri-action 簡介 · 02 三平台 matrix · 03 發佈成 GitHub Release 草稿 · 04 在 CI 裡簽章 · 05 一起上傳更新檔

## 第 18 章 行動裝置
01 前置需求（Android Studio、Xcode）· 02 `tauri android init` 與 `tauri ios init` · 03 在模擬器與手機上執行 · 04 桌面與行動的差異（`mobile_entry_point`、`cfg(mobile)`、不能用的 plugin）· 05 把第 11 章的待辦清單搬到手機 · 06 打包 APK、AAB 與 IPA

邊界：檢查程式只驗證桌面編譯。作者沒有行動裝置的開發工具，行動裝置的指令只說明、不貼輸出，依 v2.tauri.app 與原始碼查證，每集在「這集實測了什麼」寫明沒有跑。
