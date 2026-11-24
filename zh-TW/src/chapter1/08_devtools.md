# 開發者工具與除錯

## 本集目標

打開視窗裡的開發者工具，並分清楚除錯時該看哪裡：Rust 的輸出和編譯錯誤在終端機，前端的輸出和錯誤在開發者工具裡。

## 正文

### 打開開發者工具

讓 `tauri dev` 開著，用下面任一種方式：

- 在視窗上按右鍵，選「Inspect Element」。選項的文字依平台與系統語言而不同，找名字裡有 Inspect 或「檢查」的那一個。
- 按快捷鍵：macOS 是 `Command + Option + I`，Windows 與 Linux 是 `Ctrl + Shift + I`。

這就是網頁開發者每天在用的那一套，只是它來自各平台的 WebView：macOS 是 Safari 的 Web Inspector，Windows 是 Edge 的開發者工具，Linux 是 WebKit 的 inspector。第 1 集說過引擎因平台而異，所以三個平台上的介面長得不太一樣，但功能大同小異。

目前只需要認識兩個分頁，中文介面可能會把名稱翻譯掉，認得英文原名就好：

- **Elements**：目前頁面實際的 HTML 結構，可以點選元素查看或暫時修改樣式。改動只在這次載入有效，重新載入就消失，不會寫回你的檔案。
- **Console**：前端的輸出與錯誤訊息。前端程式出錯時，訊息會出現在這裡。

開發者工具在開發模式（`tauri dev`）和 debug 版本（`npm run tauri build -- --debug`）都能開。一般的 release 版預設沒有，要在 `src-tauri/Cargo.toml` 替 `tauri` crate 打開 `devtools` 這個 feature 才有，第 15 章第 1 集會再提到。

### 前端的錯誤看 Console

來看一次前端出錯。把 `src/main.ts` 裡 `invoke("greet", ...` 的 `"greet"` 改成 `"greet2"`（Rust 那邊沒有這個 command），存檔，視窗會自動重新載入。打開 Console，輸入名字、按 Greet：段落沒有出現問候語，Console 多了一則錯誤，內容有 `Command greet2 not found`（前面的字樣依 WebView 而不同，例如 `Uncaught (in promise)`）。

這就是第 7 集說的「名字對不上，呼叫就會失敗」。要看這則錯誤得開 Console，終端機看不到。確認之後，把 `greet2` 改回 `greet`。

### Rust 的輸出看終端機

後端是 Rust，`println!` 會印在**終端機**，也就是你執行 `npm run tauri dev` 的那個視窗，不會出現在 Console。反過來，前端想印東西到 Console 要用 `console.log`，第 2 章第 7 集會教。所以除錯時要先想清楚問題出在哪一側，再決定看哪裡。

在 `greet` 裡加一行 `println!`：

`src-tauri/src/lib.rs`（加上 `println!`）：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("greet 被呼叫了，name = {name}");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

存檔後，Rust 會重新編譯、App 重啟：

`npm run tauri dev` 的輸出（重啟的部分）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.05s
     Running `target/debug/tauri-app`
```

在輸入框打 `Tauri` 並按 Greet，終端機會多一行：

```text
greet 被呼叫了，name = Tauri
```

### Rust 的編譯錯誤也在終端機

故意把 `println!` 裡的 `name` 打錯成 `nam`：

`src-tauri/src/lib.rs`（故意打錯）：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("greet 被呼叫了，name = {nam}");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
error[E0425]: cannot find value `nam` in this scope
 --> src/lib.rs:4:34
  |
4 |     println!("greet 被呼叫了，name = {nam}");
  |                                       ^^^
  |
help: a local variable with a similar name exists
  |
4 |     println!("greet 被呼叫了，name = {name}");
  |                                          +

For more information about this error, try `rustc --explain E0425`.
error: could not compile `tauri-app` (lib) due to 1 previous error
```

這跟你平常用 `cargo` 看到的錯誤一模一樣。App 的視窗會消失，因為舊的 App 已經被關掉了，新的又編不起來；`tauri dev` 本身不會結束，它繼續等你改檔案。把 `nam` 改回 `name` 存檔，就會重新編譯，視窗也會回來。

## 重點整理

- 在視窗按右鍵選 Inspect Element，或按 `Command + Option + I`（macOS）、`Ctrl + Shift + I`（Windows、Linux），打開開發者工具。開發模式與 debug 版本能開，release 版要開 `devtools` feature。
- Elements 看頁面結構與樣式，Console 看前端的輸出與錯誤。
- Rust 的 `println!` 與編譯錯誤在終端機；前端的輸出與錯誤在 Console。
- Rust 編譯失敗時，視窗會關掉、`tauri dev` 繼續等你修好檔案。
