# 從零開始學 Tauri

寫給讀完《從零開始學 Rust》的人的繁體中文 Tauri 教學書；本檔統一書籍編排與書中術語的用法。

## 書籍結構

**部**：
全書最上層的分組；第一部是基礎，第二部是實戰專案，第三部是上線（安全性、打包、更新、發佈、行動裝置）。
_Avoid_: 篇、卷

**章**：
一個主題的集合，例如「第 3 章 Command：前端呼叫 Rust」。
_Avoid_: 單元、課

**集**：
一個檔案，只教一個小概念，固定包含「本集目標」「正文」「重點整理」。
_Avoid_: 節、課、lesson

**跟 Electron 比一比**：
集裡選用的小節，只在 Tauri 跟 Electron 不一樣、照 Electron 直覺會出錯時出現。
_Avoid_: Electron 對照

**起始專案**：
`create-tauri-app` 的 vanilla-ts 範本產生的專案，也就是 `tools/checkcode/template/`；每一集都從它出發。
_Avoid_: 樣板專案、boilerplate

**路徑標籤**：
程式區塊前一行的「`路徑`：」，表示區塊是該檔案的完整內容。

**檢查點**：
「`指令` 的輸出：」加輸出區塊；checkcode 在這裡驗證專案的編譯結果。

**來源註記**：
沒實測的平台，在集裡註明內容的依據，只用三種：「依 Tauri 文件整理」「依原始碼整理」「依 Tauri 文件與原始碼整理」。寫到具體版本或專案時照實寫，例如「依 tauri-bundler 2.10 原始碼整理」。第 18 章的「依 v2.tauri.app 與原始碼查證」是另一個意思（內容查證過但沒有跑），不用這三種。
_Avoid_: 依文件整理、查原始碼整理、依 plugin 的原始碼與文件整理

## Tauri 術語

原則：跟 Tauri、Rust、Web 平台 API 名稱相同的詞保留英文；其他翻成台灣用語，第一次出現時附英文。

保留英文：command、plugin、capability、permission、scope、set（permission 的）、WebView、IPC、CSP、ACL、XSS、isolation、hook、label（視窗的）、`AppHandle`、`State`、`Channel`、`invoke`、`emit`、`listen`、payload、DOM、HTML、CSS、TypeScript、JavaScript、npm、Vite、crate、serde、JSON、AppImage、MSI、NSIS、deb、rpm、WiX、WebView2、bootstrapper、APK、AAB、IPA、tray（只在 API 名稱裡，正文寫「系統匣」）、GitHub Actions、tauri-action、workflow、job、step、action（step 套用的現成功能）、runner、matrix、secret、repo、commit、tag、token、release（GitHub 的）、workflow artifact（Actions 一次執行上傳、在那次執行的頁面下載的檔案）、Android Studio、Xcode、SDK、NDK、Gradle、target（Rust 的編譯目標，例如 `aarch64-linux-android`）、identifier（`tauri.conf.json` 的）、applicationId（Android 的）、Bundle ID（iOS 的，大小寫與空格照這樣寫）、mock（mock runtime、mock IPC：測試時換上去的假實作）、migration（資料表結構升級，第一次出現附「結構遷移」）、`Runtime`（Tauri 的 trait：建立視窗、跑事件迴圈的那一層，測試用 `MockRuntime`；跟 tokio 的非同步執行環境是兩回事，正文單獨寫 runtime 時，前後文要看得出是哪一個）。

| 書中用語 | 英文 | 不要用 |
| --- | --- | --- |
| 前端 | frontend | 前台 |
| 後端（Rust 端） | backend / core | 後台 |
| 視窗 | window | 窗口 |
| 選單 | menu | 菜單 |
| 右鍵選單 | context menu | 上下文菜單 |
| 系統匣 | system tray | 托盤、系統托盤 |
| 全域快捷鍵 | global shortcut | 全局快捷鍵 |
| 快捷鍵 | keyboard shortcut | 快速鍵 |
| 對話框 | dialog | 對話窗 |
| 通知 | notification | 推送 |
| 剪貼簿 | clipboard | 剪貼板 |
| 事件 | event | 消息 |
| 監聽 | listen | 偵聽 |
| payload（事件附帶的資料） | payload | 負載 |
| 熱重載 | hot reload | 熱更新 |
| 打包 | bundle | 捆綁 |
| 安裝檔 | installer | 安裝包 |
| 圖示（App 的） | icon | 圖標 |
| 磁碟映像檔（`.dmg`） | disk image | 光碟映像、映像檔案 |
| 開發者工具（產品名稱照原名，例如 Edge DevTools） | developer tools（devtools） | 開發人員工具、DevTools、控制台（那是 Console 分頁） |
| 自動更新 | auto update | 熱更新 |
| 程式碼簽章 | code signing | 代碼簽名 |
| 更新檔簽章 | updater signature（`.sig`） | 更新簽名 |
| 公證 | notarization（macOS） | 認證 |
| 憑證 | certificate | 證書 |
| 資源檔 | resource | 資源文件 |
| 設定檔 | configuration file | 配置文件 |
| 元素 | element（HTML 的） | 標籤（那是 tag） |
| 屬性 | attribute（HTML 的） | 特性 |
| 選擇器 | selector（CSS 的） | 選取器 |
| 非同步 | asynchronous | 異步 |
| 序列化 | serialize | 串行化 |
| 反序列化 | deserialize | 反串行化 |
| 行程 | process | 進程 |
| 執行緒 | thread | 線程 |
| 函式 | function | 函數 |
| 型別 | type | 類型 |
| 變數 | variable | 變量 |
| 預設 | default | 默認 |
| 相依套件 | dependency | 依賴 |
| 模擬器 | emulator / simulator | 仿真器 |
| 最小權限 | least privilege | 最小特權 |
| 指令（CSP 的） | directive | 指示 |
| 語意化版本 | semantic versioning（semver） | 語義化版本 |
| 草稿（release 的） | draft | 草案 |
| 附件（release 的檔案） | asset | 資產 |
| 一次執行（workflow 的） | run | 運行 |
| 實機 | physical device | 真機 |
| App（我們做的應用程式；程式碼、路徑與識別字照原樣，例如 `tauri-app`） | app | app（小寫） |
| 資料夾（含 App 的資料夾） | folder / data directory | 資料目錄 |
| 結束代碼 | exit code | 結束碼 |
| 手機版 | mobile version | 行動版 |
| 單元測試 | unit test | 單位測試 |
| 接線（command 登記、`manage` 狀態、參數鍵名這些把程式接上 Tauri 的設定） | wiring | 串接 |
| 非同步執行環境 | async runtime（tokio） | 運行時、非同步運行時 |
| 資料庫 | database | 數據庫 |
| 資料表 | table（SQLite 的；指它時寫全名，不單用「表」） | 數據表 |
| 欄位 | column | 字段 |
| 列 | row | 資料列 |
| 主鍵（資料表的；第 8 章快捷鍵的「主鍵」指修飾鍵以外的那個鍵，是另一回事） | primary key | 主索引 |
| 交易 | transaction | 事務 |
| 語句（SQL 的） | SQL statement | 敘述、陳述式 |
| 預備好的語句 | prepared statement | 預處理語句、預編譯語句 |
