# `#[tauri::command]` 與 `generate_handler!`

## 本集目標

自己寫一個 command，登記到 `Builder` 上，並弄清楚 `#[tauri::command]` 與 `generate_handler!` 各負責什麼。

## 正文

### command 是開放給前端的函式

第 1 章說過，前端要請 Rust 做事，標準做法叫 command。一個 command 就是一個普通的 Rust 函式，加上兩個動作，前端才叫得動它：

1. 在函式上面寫 `#[tauri::command]`。
2. 把函式的名字交給 `generate_handler!`，登記到 `Builder` 上。

### 寫第一個 command

我們在範本的 `greet` 旁邊加一個 `say_hello`，不收參數，只回傳一句話：

`src-tauri/src/lib.rs`：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn say_hello() -> String {
    "哈囉，我是 Rust".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, say_hello])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.21s
     Running `target/debug/tauri-app`
```

### 兩個東西各做什麼

- `#[tauri::command]` 是一個屬性巨集（attribute macro）。函式本身原封不動留著，它另外替函式生出一段轉接程式：從前端送來的資料取出參數、呼叫函式、把回傳值送回前端。所以 command 不能少了它。
- `generate_handler!` 收一串 command 的名字，生出一個「處理器」（handler）：前端傳來一個名字，就找到同名函式的轉接程式去執行。
- `.invoke_handler(...)` 把這個處理器裝到 `Builder` 上。

所以前端呼叫時用的名字，就是函式的名字，一個字都不能差，像 `say_hello` 的底線也要照寫。

`invoke_handler` 只能呼叫一次：再呼叫一次，後面的會整個蓋掉前面的。所有 command 都要放進同一個 `generate_handler![...]`，用逗號隔開。

自己寫的 command 不必在 `capabilities` 裡另外設定 permission 就能呼叫（範本的 `capabilities/default.json` 不用動）。怎麼替自己的 command 加上限制，第 14 章會正式介紹。

### 在前端試試看

前端的 `invoke` 下一集才正式介紹，這裡先借範本現成的寫法試一下。打開 `src/main.ts`，把 `greet()` 裡呼叫 `invoke` 的那一段換成：

```ts,ignore
greetMsgEl.textContent = await invoke("say_hello");
```

存檔後在視窗按 Greet，下方會出現「哈囉，我是 Rust」。`say_hello` 不收參數，所以不必再傳 `{ name: ... }`。試完可以改回去。

### 少了 `#[tauri::command]` 會怎樣

把 `say_hello` 上面的 `#[tauri::command]` 拿掉，但還是把它放進 `generate_handler!`：

`src-tauri/src/lib.rs`（故意拿掉屬性）：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn say_hello() -> String {
    "哈囉，我是 Rust".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, say_hello])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
error: cannot find macro `__tauri_command_name_say_hello` in this scope
  --> src/lib.rs:15:25
   |
 2 | #[tauri::command]
   | ----------------- similarly named macro `__tauri_command_name_greet` defined here
...
15 |         .invoke_handler(tauri::generate_handler![greet, say_hello])
   |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: this error originates in the macro `tauri::generate_handler` (in Nightly builds, run with -Z macro-backtrace for more info)
help: a macro with a similar name exists
   |
15 -         .invoke_handler(tauri::generate_handler![greet, say_hello])
15 +         .invoke_handler(__tauri_command_name_greet)
   |

error: cannot find macro `__cmd__say_hello` in this scope
  --> src/lib.rs:15:57
   |
15 |         .invoke_handler(tauri::generate_handler![greet, say_hello])
   |                                                         ^^^^^^^^^

error: could not compile `tauri-app` (lib) due to 2 previous errors
```

錯誤提到的 `__cmd__say_hello` 就是屬性巨集生出來的轉接程式，`generate_handler!` 要用它，找不到就編不過。看到這種 `cannot find macro `__cmd__…``，先檢查那個函式有沒有加 `#[tauri::command]`；編譯器建議的改法（`help` 那段）不對，別照著改。加回屬性就好了。

反過來，寫了 `#[tauri::command]` 卻忘了放進 `generate_handler!`，編譯得過，只是那個函式沒人用，編譯器會警告它從未被使用（`never used`）。到了執行時，前端呼叫它會得到第 1 章第 8 集看過的 `Command … not found`。

## 重點整理

- command 是加了 `#[tauri::command]` 的 Rust 函式；前端用函式的名字呼叫它。
- `#[tauri::command]` 生出轉接程式，負責取參數、呼叫函式、送回回傳值；`generate_handler!` 依名字把前端的呼叫分派給對應的轉接程式。
- `invoke_handler` 只能呼叫一次，所有 command 放進同一個 `generate_handler![...]`。
- 缺 `#[tauri::command]` 會編譯失敗；缺登記則編得過，但執行時前端會得到 `Command … not found`。
