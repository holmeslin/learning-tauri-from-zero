# 看 log 與常見錯誤

## 本集目標

出問題時，知道該去終端機還是 Console 找訊息，並能把前面章節出現過的錯誤訊息對照到原因。

## 正文

### 先問：訊息在哪一側

Tauri App 有兩條輸出，看錯地方就什麼都看不到：

| 想找的訊息 | 看哪裡 |
| --- | --- |
| Rust 的 `println!`、`log::info!`、編譯錯誤、`panic` | 執行 `npm run tauri dev` 的終端機 |
| 前端的 `console.log`、沒接住的錯誤、被拒絕的 `invoke` | 開發者工具的 Console（第 1 章第 8 集） |
| log plugin 寫的日誌 | 終端機與日誌檔；加上 `TargetKind::Webview` 與 `attachConsole()` 也會進 Console（第 7 章第 10 集） |

被拒絕的 `invoke`，如果 `catch` 接住並顯示在畫面上，就看畫面，沒接住才會進 Console。找不到訊息時，先換另一邊找。

### 錯誤訊息對照表

下面是前面章節出現過的錯誤。「前端」是被拒絕的 `invoke`（Promise 被 reject），「終端機」是 Rust 這一側。表裡只列訊息的關鍵字，名字會依你的程式而不同：

| 訊息的關鍵字 | 出現在 | 原因與先檢查什麼 |
| --- | --- | --- |
| `Command … not found` | 前端 | 名字拼錯（含底線），或沒放進 `generate_handler!`（第 3 章第 1 集） |
| `missing required key familyName` | 前端 | 參數的鍵要用 `camelCase`；`invalid type …` 是型別不對（第 3 章第 3 集） |
| `state not managed for field` | 前端 | 忘了 `.manage()`，或 `State<T>` 的 `T` 跟 `manage` 的型別不同（第 4 章第 1 集） |
| `… not allowed. Permissions associated with this command` | 前端 | 缺 permission，把冒號後面列的其中一項加進 `capabilities/default.json`（第 7 章第 2 集） |
| `plugin dialog not found` | 前端 | 漏了 `.plugin(...)` 註冊（第 7 章第 1 集） |
| `forbidden path` | 前端 | permission 有了，路徑不在 scope 內（第 7 章第 5 集） |
| `cannot find macro` 接 `__cmd__…` | 終端機（編譯） | command 少了 `#[tauri::command]`（第 3 章第 1 集） |
| `macro import … is private`（`E0603`） | 終端機（編譯） | 模組裡的 command 沒加 `pub`（第 3 章第 10 集） |
| `is already being managed`、`state() called before manage()` | 終端機（panic） | 同一個型別 `manage` 了兩次，或 `state()` 找不到狀態（第 4 章第 1、5 集） |
| `Failed to setup app` | 終端機（panic） | `setup` 回傳了 `Err`，冒號後面是原因（第 4 章第 4 集） |

### 自己寫出來的 panic

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

`thread` 與 `called` 開頭的兩行是主要的線索：哪條執行緒、哪個檔案的哪一行（`src/lib.rs:3:25`）、為什麼（`unwrap()` 碰到 `Err`）。照著位置去改。

最後一行是 macOS 上看到的結果：App 直接結束，視窗消失，`tauri dev` 也跟著結束，前端什麼錯誤都沒收到。其他平台沒有驗證。要避免，就別在 command 裡對使用者的輸入 `unwrap()`，改回傳 `Result`（第 3 章第 6 集）：失敗會變成前端 Promise 的拒絕，App 不會倒。

### 看呼叫的路徑：`RUST_BACKTRACE=1`

panic 訊息只指出出事的那一行。如果是在函式庫深處出事，加上環境變數 `RUST_BACKTRACE=1`，讓它印出完整的呼叫路徑（這是 macOS 上跑的寫法）：

```bash
RUST_BACKTRACE=1 npm run tauri dev
```

`RUST_BACKTRACE=1 npm run tauri dev` 的輸出（節錄）：

```text
stack backtrace:
...
   4: tauri_app_lib::parse_age
             at ./src/lib.rs:3:25
   5: tauri_app_lib::run::{closure#0}::{closure#0}
             at ./src/lib.rs:1:1
...
```

找開頭是自己 crate 名字（`tauri_app_lib::`）的那幾行，那是你的程式；它上面是出事的地方，下面是誰呼叫了它。

## 重點整理

- Rust 的輸出、編譯錯誤與 `panic` 在終端機；前端的輸出與錯誤在 Console；log plugin 的日誌兩邊都能去。
- 被拒絕的 `invoke` 與 Rust 的錯誤長得不一樣：對照表找關鍵字，先判斷是前端、編譯還是執行時的 panic。
- command 裡對輸入 `unwrap()`，失敗就 panic；macOS 上整個 App 會直接結束。會失敗的地方改回傳 `Result`。
- panic 訊息看 `thread` 與 `called` 開頭的兩行；要看呼叫路徑就加 `RUST_BACKTRACE=1`。
