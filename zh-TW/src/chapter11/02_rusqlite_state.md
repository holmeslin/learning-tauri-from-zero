# `rusqlite`（`bundled`）與 `State<Mutex<Connection>>`

## 本集目標

把 rusqlite 加進專案，在 `setup` 裡打開資料庫檔案、建好資料表，再放進 `State<Mutex<Connection>>`，讓 command 查得到資料。

## 正文

### 加上 rusqlite

rusqlite 是 Rust 的 SQLite 函式庫：用 Rust 的函式打開資料庫、執行 SQL、取回結果。《從零開始學 Rust》沒教過它，範本也沒有，先加進專案：

```bash
cargo add --manifest-path src-tauri/Cargo.toml rusqlite@0.43 --features bundled
```

`cargo add` 的輸出（節錄）：

```text
    Updating crates.io index
      Adding rusqlite v0.43 to dependencies
             Features as of v0.43.0:
             + bundled
```

`--features bundled` 是這一集的重點。rusqlite 本身只是「跟 SQLite 說話」的那一層，SQLite 本體是 C 寫的函式庫。沒有 `bundled` 的話，編譯時它會去找系統上已經裝好的 SQLite：macOS、Linux 通常有，Windows 通常沒有，而且每個人的版本不一樣。打開 `bundled`，SQLite 的 C 原始碼就跟著 rusqlite 一起下載，編譯時一起編進你的 App：不用請使用者安裝任何東西，大家用的也是同一個版本。

代價是第一次編譯要多花幾秒鐘編 C 程式，所以電腦上要有 C 編譯器，也就是第 1 章第 2 集裝的 Xcode 命令列工具、C++ Build Tools 或 `build-essential`。

### 打開資料庫，數有幾筆待辦

資料庫的程式放進新檔案 `db.rs`。`open` 打開（沒有就建立）資料庫檔案，再確保 `todos` 資料表存在；`count` 數資料表裡有幾列：

`src-tauri/src/db.rs`：

```rust
use crate::AppError;
use rusqlite::Connection;
use std::path::Path;

pub fn open(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0
        )",
    )?;
    Ok(conn)
}

pub fn count(conn: &Connection) -> Result<i64, AppError> {
    let n = conn.query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))?;
    Ok(n)
}
```

- `Connection` 是一條到資料庫的連線，所有操作都從它出發。`Connection::open(路徑)` 在那個路徑的檔案不存在時，會自己建立。
- `execute_batch` 執行一段不需要回傳結果的 SQL，可以一次放好幾個語句。`CREATE TABLE IF NOT EXISTS` 是「資料表還沒有才建立」，已經有就什麼都不做，所以每次啟動都跑一遍也安全。
- `query_row(SQL, 參數, 函式)` 執行 `SELECT`，取第一列。第二個參數是要代入 SQL 的參數，這個 SQL 沒有，傳空陣列 `[]`；函式收到那一列 `row`，`row.get(0)` 取第 0 個欄位。`COUNT(*)` 是 SQL 的內建函式，回傳列數。取出來的值要什麼型別，由接收的地方決定：這裡函式回傳 `Result<i64, _>`，所以 `n` 就是 `i64`。
- rusqlite 的錯誤是 `rusqlite::Error`，它不能序列化，所以包進自己的 `AppError`，做法跟第 3 章第 7 集一樣。`db.rs` 用 `crate::AppError` 找到定義在 `lib.rs` 的它；子模組看得到父模組的私有項目，`AppError` 不用加 `pub`。

`lib.rs` 在 `setup` 裡打開資料庫，放進 `State`，再寫一個 command 讓前端問「有幾筆」。路徑用第 7 章第 6 集的 `app_data_dir()`，它只算出路徑、不建資料夾，所以先 `create_dir_all`。`setup` 回傳的是 `Result`，打不開資料庫就用 `?` 往外丟，App 會停止啟動（第 4 章第 4 集）。

先照直覺，把 `Connection` 直接交給 `manage`：

`src-tauri/src/lib.rs`：

```rust
mod db;

use rusqlite::Connection;
use serde::{Serialize, Serializer};
use tauri::{Manager, State};

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("資料庫錯誤：{0}")]
    Db(#[from] rusqlite::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
fn todo_count(conn: State<Connection>) -> Result<i64, AppError> {
    db::count(&conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("todos.db"))?;
            app.manage(conn);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![todo_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

前端先只做一件事：問 Rust 有幾筆，寫在畫面上。畫面暫時簡化成一行字，下一集再換回完整的：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>待辦清單</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main>
      <h1>待辦清單</h1>
      <p id="status"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const statusEl: HTMLElement | null = document.querySelector("#status");

const start = async () => {
  const count = await invoke<number>("todo_count");
  if (statusEl) {
    statusEl.textContent = `資料庫裡有 ${count} 筆待辦`;
  }
};

start();
```

`npm run tauri dev` 的輸出（節錄）：

```text
error[E0277]: `RefCell<rusqlite::inner_connection::InnerConnection>` cannot be shared between threads safely
   --> src/lib.rs:35:24
    |
 35 |             app.manage(conn);
    |                 ------ ^^^^ `RefCell<rusqlite::inner_connection::InnerConnection>` cannot be shared between threads safely
    |                 |
    |                 required by a bound introduced by this call
    |
    = help: within `Connection`, the trait `Sync` is not implemented for `RefCell<rusqlite::inner_connection::InnerConnection>`
    = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
note: required because it appears within the type `Connection`
   --> ~/.cargo/registry/src/.../rusqlite-0.43.0/src/lib.rs:366:12
    |
366 | pub struct Connection {
    |            ^^^^^^^^^^
note: required by a bound in `manage`
   --> ~/.cargo/registry/src/.../tauri-2.12.6/src/lib.rs:717:15
    |
715 |   fn manage<T>(&self, state: T) -> bool
    |      ------ required by a bound in this associated function
716 |   where
717 |     T: Send + Sync + 'static,
    |               ^^^^ required by this bound in `Manager::manage`
```

後面還有幾則同樣是 `E0277` 的錯誤，說的都是同一件事，連 command 的參數 `State<Connection>` 也一起被指出來。

### 為什麼要 `Mutex`

錯誤說的是：`Connection` 裡面用了 `RefCell`。`RefCell` 沒有任何鎖，只適合單一執行緒，所以 `Connection` 不是 `Sync`，不能讓好幾條執行緒同時拿著它的參考。

`manage` 要求狀態同時是 `Send` 與 `Sync`，因為 command 可能同時在不同的執行緒上跑（第 4 章第 2 集）。`Connection` 是 `Send`，可以整個搬到另一條執行緒，卻不是 `Sync`。所以要有人替它排隊，同一時間只讓一條執行緒用它。

解法跟第 4 章第 2 集一樣：包進 `std::sync::Mutex`。`Mutex<T>` 只要 `T` 是 `Send` 就是 `Sync`，`Connection` 符合。`manage` 與 `State` 的型別都改成 `Mutex<Connection>`，command 裡先 `lock()` 才拿得到連線：

`src-tauri/src/lib.rs`：

```rust
mod db;

use rusqlite::Connection;
use serde::{Serialize, Serializer};
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("資料庫錯誤：{0}")]
    Db(#[from] rusqlite::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
fn todo_count(state: State<Mutex<Connection>>) -> Result<i64, AppError> {
    db::count(&state.lock().unwrap())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("todos.db"))?;
            app.manage(Mutex::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![todo_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`state.lock().unwrap()` 拿到的鎖守衛，前面加 `&` 就能當 `&Connection` 傳給 `db::count`（跟第 9 章第 1 集的 `&mut state.lock().unwrap()` 同一個道理）。守衛在那一行結束時放掉，鎖也跟著解開。

`std::sync::Mutex` 就夠用：rusqlite 的函式都是同步的，這一章的 command 也都是同步的，鎖不會跨過 `.await`，用不到第 4 章第 3 集的 async `Mutex`。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling libsqlite3-sys v0.40.0
   Compiling rusqlite v0.43.0
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.82s
     Running `target/debug/tauri-app`
```

視窗出現「資料庫裡有 0 筆待辦」。`libsqlite3-sys` 就是 rusqlite 帶來的 SQLite 本體，第一次編譯會看到它。

### 資料庫檔案在哪

第一次啟動時，`setup` 建了資料夾、資料庫檔案和 `todos` 資料表。檔案就在 app 資料夾裡（macOS 的路徑，其他平台用 `appDataDir()` 查，第 7 章第 6 集）：

```bash
ls "$HOME/Library/Application Support/tw.example.tauri-app"
```

```text
todos.db
```

用 `sqlite3` 指令看資料表長什麼樣子（沒有這個指令的平台可以跳過）：

```bash
sqlite3 "$HOME/Library/Application Support/tw.example.tauri-app/todos.db" ".schema"
```

```text
CREATE TABLE todos (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0
        );
```

`.schema` 印出的是建立資料表的 SQL，連縮排都跟 `execute_batch` 那段字串一樣；`IF NOT EXISTS` 是建立時才用的，不會留下來。再重新啟動 App，檔案和資料表都已經在了，`CREATE TABLE IF NOT EXISTS` 什麼都不做。

## 重點整理

- rusqlite 是 Rust 的 SQLite 函式庫；`cargo add --manifest-path src-tauri/Cargo.toml rusqlite@0.43 --features bundled` 加進專案。`bundled` 把 SQLite 本體一起編進 App，不用使用者另外安裝。
- 資料庫檔案放在 `app_data_dir()` 底下，路徑算出來後要先 `create_dir_all`；`Connection::open` 沒有檔案時會自己建立，`CREATE TABLE IF NOT EXISTS` 讓每次啟動都能安全地確保資料表存在。
- `Connection` 是 `Send`、不是 `Sync`，直接 `manage` 會編譯失敗（`E0277`）；包進 `Mutex` 後放進 `State<Mutex<Connection>>`，command 先 `lock()` 再用。
- rusqlite 的函式與這一章的 command 都是同步的，用標準函式庫的 `Mutex` 就夠。
- 資料庫相關的函式放在 `db.rs`，只收 `&Connection`；command 留在 `lib.rs`。
