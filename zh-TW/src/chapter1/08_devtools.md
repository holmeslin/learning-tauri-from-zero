# 開發者工具與除錯

## 本集目標

打開視窗裡的開發者工具，並分清楚除錯時該看哪裡：Rust 的輸出和編譯錯誤在終端機，前端的輸出在開發者工具的 Console。

## 正文

### 打開開發者工具

讓 `tauri dev` 開著，用下面任一種方式：

- 在視窗上按右鍵，選「Inspect Element」（檢查元素）。各平台的選項文字略有不同。
- 按快捷鍵：macOS 是 `Command + Option + I`，Windows 與 Linux 是 `Ctrl + Shift + I`。

這就是網頁開發者每天在用的那一套，只是它來自各平台的 WebView：macOS 是 Safari 的 Web Inspector，Windows 是 Edge DevTools，Linux 是 WebKit 的 inspector。第 1 集說過引擎因平台而異，所以三個平台上的介面長得不太一樣，但功能大同小異。

目前只需要認識兩個分頁：

- **Elements**（元素）：目前頁面實際的 HTML 結構，可以點選元素查看或暫時修改樣式。改動只在這次載入有效，重新載入就消失，不會寫回你的檔案。
- **Console**（控制台）：前端的輸出與錯誤訊息。

開發者工具預設只在開發時（`tauri dev`）能開，正式版的 App 沒有，打包時（第 15 章）會再提到。

### 兩邊的輸出各去哪裡

後端是 Rust，`println!` 印在**終端機**，就是你執行 `npm run tauri dev` 的那個視窗。前端是網頁，要用 `console.log` 輸出，它印在開發者工具的 **Console**。`console.log` 就是前端版的 `println!`，第 2 章會正式介紹。

兩邊各加一行：

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

`src/main.ts`（加上 `console.log`）：

```ts
import { invoke } from "@tauri-apps/api/core";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    console.log("送出的名字：", greetInputEl.value);
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
```

存檔後，Rust 會重新編譯、App 重啟：

`npm run tauri dev` 的輸出（重啟的部分）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.05s
     Running `target/debug/tauri-app`
```

打開 Console 分頁，在輸入框打 `Tauri` 並按 Greet。終端機會多一行：

```text
greet 被呼叫了，name = Tauri
```

Console 則會出現 `送出的名字： Tauri`。同一次按鈕，兩邊各印各的。

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

### Rust 程式當掉的時候

想看 panic 的完整呼叫堆疊，跟平常一樣開 `RUST_BACKTRACE`：

```bash
RUST_BACKTRACE=1 npm run tauri dev
```

Windows 的 PowerShell 要分成兩行：先執行 `$env:RUST_BACKTRACE=1`，再執行 `npm run tauri dev`。

## 重點整理

- 在視窗按右鍵選 Inspect Element，或按 `Command + Option + I`（macOS）、`Ctrl + Shift + I`（Windows、Linux），打開開發者工具。
- Elements 看頁面結構與樣式，Console 看前端的輸出與錯誤。
- Rust 的 `println!` 與編譯錯誤在終端機；前端的 `console.log` 在 Console。
- Rust 編譯失敗時，視窗會關掉、`tauri dev` 繼續等你修好檔案。
- 用 `RUST_BACKTRACE=1` 看 panic 的完整堆疊。
