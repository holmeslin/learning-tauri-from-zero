# 把第 11 章的待辦清單搬到手機

## 本集目標

不改第 11 章待辦清單的任何一行 Rust 或 TypeScript，把它搬到 Android 與 iOS，並看懂為什麼不用改。這一集的行動裝置部分都沒有實測。

## 正文

這一集接著第 11 章第 7 集的專案，不重新開始，也不重貼第 11 章的檔案。從第 11 章帶過來、這一集一個字都不改的檔案：

- `src-tauri/Cargo.toml`：第 11 章第 2 集加的 `rusqlite`（`bundled`）與 `thiserror`。
- `src-tauri/tauri.conf.json`：`identifier` 是 `tw.example.tauri-app`，視窗 480×640，標題「待辦清單」。
- `src-tauri/src/lib.rs`、`src-tauri/src/db.rs`：`run()`、四個 command 與資料庫函式。
- `src/main.ts`、`src/todos.ts`、`index.html`、`src/styles.css`：畫面。

新增的只有第 2 集產生的 `src-tauri/gen/android` 與 `src-tauri/gen/apple`。它們在專案資料夾裡，所以要在這個專案裡產生。

「不用改」是讀文件與原始碼得到的結論，不是在手機上跑出來的。下面看四個最可能出問題的地方。

### 入口與函式庫

`lib.rs` 的 `run()` 從範本到第 11 章都帶著 `#[cfg_attr(mobile, tauri::mobile_entry_point)]`，`[lib]` 也還是範本的樣子，第 4 集說明它們做什麼。這兩個不用動。

### SQLite 還是編得起來

第 11 章第 2 集的 `bundled`，讓 rusqlite 不找系統的 SQLite，而是用 `cc` crate 把 SQLite 的 C 原始碼編進 App。編給手機時，這段 C 程式碼也得用手機用的編譯器。

Android 這邊，Tauri CLI 編譯前會設好環境變數 `TARGET_CC`、`TARGET_CXX`、`TARGET_AR`，指向第 1 集裝的 NDK 裡的 clang 與 ar；`cc` 會讀 `TARGET_CC` 這類變數來挑編譯器。SQLite 的 build script（`libsqlite3-sys`）也有 Android 專屬的處理：多加一個編譯選項，原始碼的註解說理由是 Android 上 SQLite 不知道暫存檔該放哪。

iOS 這邊，build script 沒有 iOS 專屬的處理，走預設的路。iOS 編 C 程式碼時怎麼挑編譯器，我們沒有查證。

rusqlite 在手機上有沒有已知的問題，我們也沒有查證。

### 資料庫檔案放哪裡

`setup` 用 `app.path().app_data_dir()` 加上 `todos.db`，開檔前先 `create_dir_all`（第 11 章第 2 集）。這個函式兩個手機平台都有實作：

- Android：App 自己的私有資料目錄（原始碼呼叫的是 Android 的 `dataDir`）。
- iOS：跟 macOS 同一套算法（`dirs` crate），也就是 `$HOME/Library/Application Support/tw.example.tauri-app`。

手機與桌面各有一份 `todos.db`，兩邊的待辦不會互通。`identifier` 還是不要動，原因跟第 11 章第 7 集一樣；它在手機上怎麼變成套件名稱，第 2 集說過。

### 畫面第 11 章已經留好

第 11 章第 1 集設計畫面時，就是為了這一步：

- 單欄，沒有只在滑鼠移上去才出現的按鈕。`main` 的 `max-width: 480px` 只是上限，螢幕比 480 窄時，`main` 就用滿整個寬度。
- `index.html` 範本留下的 viewport `<meta>`（`width=device-width, initial-scale=1.0`）還在，它是手機決定頁面寬度的設定。
- `tauri.conf.json` 的 480×640 在手機上會被忽略（第 4 集）。Android 的視窗實作裡，設定標題也不做任何事，所以「待辦清單」不會顯示成視窗標題。
- 沒有選單、系統匣、全域快捷鍵（第 4 集說這些手機都沒有），也不開第二個視窗。`lib.rs` 只註冊範本的 opener plugin，這個專案沒用到它；它在兩個平台都有，但只能開網址（第 4 集）。

### 手機上的名字

App 在手機上的名字來自 `productName`：Android 的 `strings.xml` 與 iOS 的 Xcode 專案（`PRODUCT_NAME`）都是從它來的。第 11 章沒有改過 `productName`，所以名字會是 `tauri-app`，不是「待辦清單」。

要換的話改 `productName`，桌面安裝檔的名字也會跟著變（第 15 章第 3 集）。中文名字在手機上顯示得對不對，我們沒有查證，這一集不改。

### 在手機上跑

在這個專案資料夾，先照第 2 集產生 `gen/`，再照第 3 集啟動。目標是模擬器或實機都可以，選法在第 3 集：

```bash
npm run tauri android init
npm run tauri android dev
```

iOS 是同一組，換成 `ios`：

```bash
npm run tauri ios init
npm run tauri ios dev
```

跑起來之後，自己對一遍：

- 新增、勾選、刪除三個動作，用手指都做得到。
- 標題有沒有被狀態列蓋住。Android 範本的 `MainActivity.kt` 呼叫了 `enableEdgeToEdge()`，內容可能延伸到狀態列底下。網頁的常見做法是在 viewport 的 `content` 加 `viewport-fit=cover`，CSS 用 `padding-top: env(safe-area-inset-top)`。Tauri 的文件與原始碼都沒提這件事，在兩個平台上有沒有用，我們沒有查證。
- 把 App 關掉再開，待辦還在不在。在，表示 `todos.db` 寫進了上面那個資料夾。

### 這集實測了什麼

- 沒有實測：這一集所有行動裝置的內容。寫這一章的電腦沒有 Android 與 iOS 的開發工具，也沒有手機，上面的指令都沒有跑過，所以沒有貼輸出。
- 依據是 v2.tauri.app 的文件與鎖定版本的原始碼：tauri-cli 2.12.5、cargo-mobile2 0.22.5、`tauri` 2.12.6、tauri-runtime-wry 2.12.4、tao 0.37.2、`libsqlite3-sys` 0.40.0、`cc` 1.11.2、`dirs` 7。
- 這一集沒有貼 Rust 或 TypeScript，沒有東西讓檢查程式編譯。第 11 章的程式在第 11 章是用桌面版建置、測試過的，沒有為 Android 或 iOS 編譯過。
- 沒有查證：rusqlite 在手機上的已知問題、`env(safe-area-inset-top)` 在兩個平台的行為、中文 `productName` 的顯示、iOS 編譯 C 程式碼用的編譯器。

## 重點整理

- 依原始碼判斷，第 11 章的待辦清單搬到手機不用改 Rust 或 TypeScript：入口、`bundled` 的 SQLite、`app_data_dir()` 都有手機的實作。這是讀出來的結論，沒有實測。
- 第 11 章刻意做成單欄、不靠滑鼠、沒有桌面才有的功能；`tauri.conf.json` 的視窗大小在手機上會被忽略。
- 手機與桌面各有一份 `todos.db`；名字來自 `productName`，目前會是 `tauri-app`。
- 在這個專案資料夾先做第 2 集的 init，再用第 3 集的 `dev` 跑；上手機後要自己看標題有沒有被狀態列蓋住。
