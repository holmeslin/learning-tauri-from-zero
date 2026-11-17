# 把 command 拆到模組

## 本集目標

把 command 從 `lib.rs` 搬到自己的模組，並記住 Tauri 對模組裡的 command 的幾條額外規則。

## 正文

### 為什麼要拆

command 越寫越多，全塞在 `lib.rs` 會很擠。Rust 的做法你早就會了：用 `mod` 拆檔案。Tauri 只多幾條規則，這一集就是講這些規則。

### 範例：搬到 `commands.rs`

`lib.rs` 只留下組裝 App 的 `run()`，`greet` 與 `say_hello` 搬到新檔案 `src-tauri/src/commands.rs`。先照搬過去，看看會怎麼樣：

`src-tauri/src/lib.rs`：

```rust
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::say_hello
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src-tauri/src/commands.rs`（照搬，沒有 `pub`）：

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn say_hello() -> String {
    "哈囉，我是 Rust".to_string()
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
error[E0603]: macro import `__cmd__greet` is private
 --> src/lib.rs:8:23
  |
8 |             commands::greet,
  |                       ^^^^^ private macro import
  |
note: the macro import `__cmd__greet` is defined here...
 --> src/commands.rs:2:4
  |
2 | fn greet(name: &str) -> String {
  |    ^^^^^
note: ...and refers to the macro `__cmd__greet` which is defined here
 --> src/commands.rs:1:1
  |
1 | #[tauri::command]
  | ^^^^^^^^^^^^^^^^^
```

第 1 集看過 `#[tauri::command]` 會生出隱藏的 `__cmd__…` 巨集，`generate_handler!` 靠它找到 command。巨集的可見範圍跟函式一樣：函式是私有的，巨集也是私有的，`lib.rs` 就用不到。所以模組裡的 command 要加 `pub`：

`src-tauri/src/commands.rs`：

```rust
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn say_hello() -> String {
    "哈囉，我是 Rust".to_string()
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.42s
     Running `target/debug/tauri-app`
```

前端不用動：範本的 Greet 照常運作。前端用的 command 名字還是 `"greet"`，跟它放在哪個模組無關；寫成 `"commands::greet"` 反而會得到 `Command commands::greet not found`。

### 幾條規則

- 模組裡的 command 要 `pub`，在 `generate_handler!` 裡寫完整路徑，如 `commands::greet`。整個專案仍然只有一個 `generate_handler![...]`（第 1 集）。
- 反過來，`lib.rs` 裡的 command **不能**加 `pub`。例如把一個叫 `ping` 的 command 寫成 `pub fn ping()`，會得到這樣的錯誤（節錄）：

  ```text
  error[E0255]: the name `__cmd__ping` is defined multiple times
   --> src/lib.rs:2:8
  ```

  訊息只說名字重複定義，沒提 `pub`；看到它，先檢查 `lib.rs` 的 command 有沒有多寫 `pub`。
- command 的名字在整個專案裡不能重複，就算放在不同模組也一樣：兩個模組各有一個 `greet`，會得到 `E0428`（`__cmd__greet` 重複定義）。原因是 `pub` 的 command 生出來的巨集都放在整個 crate 的最上層，不分模組。
- 模組可以再分層，路徑照寫就好，如 `commands::greeting::greet`（`commands/mod.rs` 裡要 `pub mod greeting;`）。

## 重點整理

- 把 command 搬到模組，`lib.rs` 就只剩 `run()` 與一個 `generate_handler!`。
- 模組裡的 command 要 `pub`，登記時寫完整路徑（`commands::greet`）。
- `lib.rs` 裡的 command 不能 `pub`；command 名字全專案不能重複。
- 前端呼叫的名字只看函式名字，跟模組無關，搬家不影響前端。
