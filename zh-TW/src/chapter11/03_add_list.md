# 新增與列出

## 本集目標

寫出新增與讀出待辦的資料庫函式與 command，讓前端能新增一則待辦、讀回整份清單並畫出來。

## 正文

### Rust 端：`Todo`、`list` 與 `add`

`db.rs` 多了三樣東西。`Todo` 是從資料庫讀出來的一列，要送給前端，所以要 `Serialize`；`list` 讀出所有待辦；`add` 新增一則。上一集的 `count` 用不到了，拿掉：

`src-tauri/src/db.rs`：

```rust
use crate::AppError;
use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct Todo {
    id: i64,
    title: String,
    done: bool,
}

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

pub fn list(conn: &Connection) -> Result<Vec<Todo>, AppError> {
    let mut stmt = conn.prepare("SELECT id, title, done FROM todos ORDER BY id")?;
    let todos = stmt
        .query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                done: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(todos)
}

pub fn add(conn: &Connection, title: &str) -> Result<(), AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::EmptyTitle);
    }
    conn.execute("INSERT INTO todos (title) VALUES (?1)", [title])?;
    Ok(())
}
```

先看 `list`：

- `conn.prepare(SQL)` 把 SQL 先編譯成一個預備好的語句（prepared statement），回傳 `Statement`。
- `stmt.query_map(參數, 函式)` 執行它，對每一列呼叫一次函式，把那一列 `row` 轉成你要的型別，這裡是 `Todo`。`row.get(0)` 取第 0 個欄位，也就是 `SELECT` 列出的第一個 `id`；`get(1)` 是 `title`，`get(2)` 是 `done`。欄位的型別由接收的地方決定：資料表的 `INTEGER` 變成 `i64`，`TEXT` 變成 `String`，存著 `0`、`1` 的 `INTEGER` 欄位可以直接變成 `bool`。
- `query_map` 回傳一個迭代器，每個元素都是 `Result<Todo>`（讀某一列可能失敗）。`collect::<Result<Vec<_>, _>>()` 把它們收成 `Result<Vec<Todo>>`：全部成功才是 `Ok`，有一個失敗就整個是 `Err`。這是 Rust 標準的 `collect` 用法。

`Todo` 的欄位沒有寫 `pub`：只有 `db.rs` 自己建立它，前端拿到的是序列化後的 JSON，別的模組不需要直接碰欄位。

再看 `add`：先 `trim()` 去掉頭尾的空白，只剩空白的標題不收，回傳 `AppError::EmptyTitle`。

### `?1`：不要把輸入拼進 SQL

`add` 的 SQL 裡寫的不是標題本身，而是 `?1`，標題放在第二個參數 `[title]`。`?1` 是參數的位置：它代表「後面傳進來的第 1 個參數」。多個參數就用 `?1`、`?2`，參數寫成 `params![a, b]`，下一集會用到。

為什麼不直接用 `format!` 把標題拼進 SQL？

```rust,ignore
// 不要這樣寫
let sql = format!("INSERT INTO todos (title) VALUES ('{title}')");
conn.execute(&sql, [])?;
```

使用者輸入的標題是 `Tom's list` 時，拼出來的 SQL 是 `VALUES ('Tom's list')`：中間的 `'` 提早結束了字串，SQL 變成語法錯誤。更糟的是，輸入刻意設計過的字串，可以改變這句 SQL 本來的意思，這叫 SQL injection。

用 `?1`，rusqlite 會把值另外交給 SQLite，值永遠只是資料，不會被當成 SQL 解讀，`'` 或任何字元都不會出事。所以：值一律走參數，不要拼字串。

### 兩個 command

`lib.rs` 加上 `AppError` 的新變體 `EmptyTitle`，以及兩個 command。`AppError` 的 `Serialize` 送出的是訊息字串，前端 `catch` 到的就是「標題不能是空的」。command 只做三件事：鎖住連線、交給 `db.rs`、回傳結果。這是第 9 章第 1 集的分法：Tauri 的東西留在 command，SQL 在 `db.rs`。

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
    #[error("標題不能是空的")]
    EmptyTitle,
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
fn list_todos(state: State<Mutex<Connection>>) -> Result<Vec<db::Todo>, AppError> {
    db::list(&state.lock().unwrap())
}

#[tauri::command]
fn add_todo(title: &str, state: State<Mutex<Connection>>) -> Result<(), AppError> {
    db::add(&state.lock().unwrap(), title)
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
        .invoke_handler(tauri::generate_handler![list_todos, add_todo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 前端：`todos.ts` 與畫面

前端呼叫 command 的地方，統一放在 `todos.ts`：每個 command 一個函式，`main.ts` 只認得這些函式，不直接寫 command 的名字。`Todo` 的欄位跟 Rust 的 `Todo` 一一對應。`addTodo` 傳的鍵 `title` 跟 Rust 參數 `title` 同名（第 3 章第 3 集）。

`src/todos.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

export interface Todo {
  id: number;
  title: string;
  done: boolean;
}

export function listTodos(): Promise<Todo[]> {
  return invoke<Todo[]>("list_todos");
}

export function addTodo(title: string): Promise<void> {
  return invoke<void>("add_todo", { title: title });
}
```

畫面換回完整的版本，`index.html` 與 `src/styles.css` 跟第 1 集一樣，只是清單改成空的，等程式來填：

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
      <form id="add-form">
        <input id="title-input" placeholder="要做什麼？" />
        <button type="submit">新增</button>
      </form>
      <ul id="todo-list"></ul>
      <p id="status"></p>
    </main>
  </body>
</html>
```

`src/styles.css`：

```css
body {
  margin: 0;
  padding: 16px;
  font-family: sans-serif;
}

main {
  max-width: 480px;
  margin: 0 auto;
}

#add-form {
  display: flex;
  gap: 8px;
}

#title-input {
  flex: 1;
  min-width: 0;
}

#todo-list {
  padding: 0;
  list-style: none;
}

#todo-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid #ddd;
}

#todo-list label {
  flex: 1;
}

.done {
  color: #888;
  text-decoration: line-through;
}
```

`src/main.ts` 做兩件事：啟動時讀出清單畫上去，按「新增」時呼叫 `addTodo`，成功了再讀一次。這一集清單只畫標題，勾選與刪除下一集才加：

`src/main.ts`：

```ts
import { Todo, addTodo, listTodos } from "./todos";

const formEl: HTMLFormElement | null = document.querySelector("#add-form");
const titleEl: HTMLInputElement | null = document.querySelector("#title-input");
const listEl: HTMLUListElement | null = document.querySelector("#todo-list");
const statusEl: HTMLElement | null = document.querySelector("#status");
if (!formEl || !titleEl || !listEl || !statusEl) {
  throw new Error("找不到必要的元素");
}

const show = (message: string) => {
  statusEl.textContent = message;
};

const render = (todos: Todo[]) => {
  listEl.textContent = "";
  for (const todo of todos) {
    const itemEl = document.createElement("li");
    itemEl.textContent = todo.title;
    listEl.append(itemEl);
  }
};

const refresh = async () => {
  try {
    render(await listTodos());
  } catch (error) {
    show(`讀取失敗：${error}`);
  }
};

formEl.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await addTodo(titleEl.value);
    titleEl.value = "";
    show("");
    await refresh();
  } catch (error) {
    show(`新增失敗：${error}`);
  }
});

refresh();
```

- `render` 跟第 10 章一樣，每次清空再整份重畫。
- `refresh` 向 Rust 要整份清單，交給 `render`。啟動時呼叫一次，新增成功後再呼叫一次。
- 新增失敗時，`invoke` 的 Promise 被拒絕（第 3 章第 6 集），`await` 把它當成錯誤丟出來，`catch` 接住，把原因放進狀態列。Rust 的 `AppError` 送到前端是字串，所以直接放進 `${error}` 就能顯示。
- 新增成功才清空輸入框；失敗時輸入的字還在，使用者可以直接修改。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.86s
     Running `target/debug/tauri-app`
```

視窗一開始是空的清單。輸入「買牛奶」（前後故意多打幾個空白），按「新增」，清單出現「買牛奶」，輸入框清空。輸入框空著或只有空白就按「新增」，狀態列出現「新增失敗：標題不能是空的」，清單不變。再輸入 `Tom's list`，也能新增，那個 `'` 沒有造成問題。

關掉 App 再開，兩則待辦還在。到終端機直接看資料庫（macOS 的路徑）：

```bash
sqlite3 -header -column "$HOME/Library/Application Support/tw.example.tauri-app/todos.db" "SELECT * FROM todos"
```

```text
id    title     done
--  ----------  ----
 1  買牛奶         0
 2  Tom's list     0
```

標題前後的空白已經在 `add` 裡去掉了，`done` 是預設的 `0`。

## 重點整理

- `prepare` 加 `query_map` 讀出多列資料：每一列用 `row.get(欄位編號)` 轉成需要的型別，再 `collect` 成 `Result<Vec<_>, _>`。
- 新增用 `conn.execute(SQL, 參數)`。SQL 裡用 `?1`、`?2` 代表參數的位置，值從參數傳進去，不要用 `format!` 拼進 SQL，否則 `'` 會弄壞 SQL，也有 SQL injection 的風險。
- command 只負責鎖住連線、呼叫 `db.rs`；錯誤用 `AppError` 回傳，前端 `catch` 到的是訊息字串，放進狀態列。
- 前端呼叫 command 的地方集中在 `todos.ts`，`main.ts` 只呼叫這些函式。
