# 看 log 與常見錯誤

## 本集目標

出問題時，知道該去終端機還是 Console 找訊息、各看什麼線索，並能把前面章節出現過的錯誤訊息對照到原因。

## 正文

### 先問：訊息在哪一側

Tauri App 有兩個地方會出現訊息，看錯地方就什麼都看不到：

- **終端機**：執行 `npm run tauri dev` 的那個視窗。Rust 的 `println!`、`log::info!`、`panic` 與編譯錯誤都在這裡。
- **開發者工具的 Console**（第 1 章第 8 集）：前端的 `console.log`、沒接住的錯誤，以及被拒絕的 `invoke`。

找不到訊息，就先換另一邊找。log plugin 的日誌兩邊都能去（第 7 章第 10 集），這集不重講。

### 終端機：找 `error` 與 `panicked at`

`tauri dev` 的終端機同時有 Vite、cargo 與你的 `println!`，輸出一多，重點很容易被蓋過去。Rust 這一側出事，找兩種行：編譯失敗的是 `error` 開頭，執行時 panic 的是 `panicked at`，後面都跟著檔案與行號（例如 `src/lib.rs:3:25`），照著位置去改。

### Console：看被拒絕的 `invoke`

`invoke` 失敗時，Promise 會被拒絕，原因是 Rust 回傳的 `Err`，或 Tauri 自己產生的錯誤。程式有 `catch` 並顯示在畫面上，就看畫面；沒接住，Console 才會出現紅色的 `Uncaught (in promise)`（字樣依 WebView 而不同），後面那段就是原因。

### 錯誤訊息對照表

下面挑幾個前面章節出現過、最常碰到的錯誤。「前端」是被拒絕的 `invoke`，「終端機」是 Rust 這一側。表裡只列訊息的關鍵字，名字會依你的程式而不同：

| 訊息的關鍵字 | 出現在 | 原因與先檢查什麼 |
| --- | --- | --- |
| `Command … not found` | 前端 | 名字拼錯（含底線），或沒放進 `generate_handler!`（第 3 章第 1 集） |
| `missing required key familyName` | 前端 | 參數的鍵要用 `camelCase`；`invalid type …` 是型別不對（第 3 章第 3 集） |
| `invalid args … missing field` | 前端 | struct 的欄位名稱跟 serde 對不上，檢查有沒有 `rename_all = "camelCase"`（第 3 章第 5 集） |
| `state not managed for field` | 前端 | 忘了 `.manage()`，或 `State<T>` 的 `T` 跟 `manage` 的型別不同（第 4 章第 1 集） |
| `… not allowed. Permissions associated with this command` | 前端 | 缺 permission，把冒號後面列的其中一項加進 `capabilities/default.json`（第 7 章第 2 集） |
| `plugin dialog not found` | 前端 | 漏了 `.plugin(...)` 註冊（第 7 章第 1 集） |
| `forbidden path` | 前端 | permission 有了，路徑不在 scope 內（第 7 章第 5 集） |
| `a webview with label … already exists` | 前端 | 視窗的 label 重複，先用 `get_webview_window` 找找看（第 6 章第 4 集） |
| `cannot find macro` 接 `__cmd__…` | 終端機（編譯） | command 少了 `#[tauri::command]`（第 3 章第 1 集） |
| `macro import … is private`（`E0603`） | 終端機（編譯） | 模組裡的 command 沒加 `pub`（第 3 章第 10 集） |
| `async commands that contain references`（`E0277`） | 終端機（編譯） | async command 的參數借用了資料：改成 `String`，或回傳 `Result`（第 3 章第 8 集） |
| `is already being managed`、`state() called before manage()` | 終端機（panic） | 同一個型別 `manage` 了兩次，或 `state()` 找不到狀態（第 4 章第 1、5 集） |
| `Failed to setup app` | 終端機（panic） | `setup` 回傳了 `Err`，冒號後面是原因（第 4 章第 4 集） |

### 自己寫的 panic

表裡沒有的一種，是自己在 command 裡寫的 `unwrap()`。頁面一載入就呼叫 `parse_age`，傳進 `abc`：

`src-tauri/src/lib.rs`：

```rust
#[tauri::command]
fn parse_age(text: &str) -> u8 {
    text.trim().parse().unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![parse_age])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

invoke("parse_age", { text: "abc" });
```

`npm run tauri dev` 的輸出（macOS，節錄）：

```text
     Running `target/debug/tauri-app`

thread 'main' (123456) panicked at src/lib.rs:3:25:
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
fatal runtime error: failed to initiate panic, error 5, aborting
```

`thread` 與 `called` 開頭的兩行說明了哪條執行緒、哪個檔案的哪一行、為什麼（`unwrap()` 碰到 `Err`）。最後一行是 macOS 上同步 command 的結果：App 直接結束，視窗消失，`tauri dev` 也跟著結束，前端什麼錯誤都沒收到。

async command（第 3 章第 8 集）不一樣：它在 Tauri 的非同步執行環境裡跑，panic 只會結束那一個 task。終端機一樣印出 `panicked at`（執行緒名稱換成 `tokio-rt-worker`），App 還活著，只是前端那個 Promise 永遠不會有結果，既不成功也不被拒絕，在畫面上看起來就是沒反應。這兩種結果都只在 macOS 上驗證過，其他平台沒有。

要避免，就別在 command 裡對使用者的輸入 `unwrap()`，改回傳 `Result`（第 3 章第 6 集）：失敗會變成前端 Promise 的拒絕，App 不會倒。

panic 發生在函式庫深處時，位置只會指到那一行。要看完整的呼叫路徑，加上環境變數 `RUST_BACKTRACE=1`，找開頭是自己 crate 名字（`tauri_app_lib::`）的幾行，那是你的程式，它上面是出事的地方，下面是誰呼叫了它。macOS、Linux 的寫法：

```bash
RUST_BACKTRACE=1 npm run tauri dev
```

Windows 的 PowerShell 要寫成 `$env:RUST_BACKTRACE=1; npm run tauri dev`（沒有在 Windows 上實測）。

## 重點整理

- Rust 的輸出、編譯錯誤與 `panic` 在終端機，找 `error` 與 `panicked at` 開頭的行；前端的輸出與沒接住的錯誤在 Console，被拒絕的 `invoke` 要看 Console 錯誤訊息裡的原因。找不到就換另一邊。
- 對照表用關鍵字找，先判斷是前端被拒絕、編譯錯誤，還是執行時的 panic。
- command 裡對輸入 `unwrap()`，失敗就 panic：同步 command 在 macOS 上整個 App 會結束，async command 只結束那個 task，Promise 不會有結果。會失敗的地方改回傳 `Result`。
- 要看呼叫路徑就加 `RUST_BACKTRACE=1`。
