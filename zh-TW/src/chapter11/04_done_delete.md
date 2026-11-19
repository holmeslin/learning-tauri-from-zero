# 完成與刪除

## 本集目標

加上「勾選完成」與「刪除」：資料庫端用 `UPDATE` 與 `DELETE`，畫面上每一列變成勾選框加刪除按鈕。

## 正文

### 資料庫端：`UPDATE` 與 `DELETE`

`db.rs` 多兩個函式，對應第 1 集的 `UPDATE` 與 `DELETE`：

`src-tauri/src/db.rs`：

```rust
use crate::AppError;
use rusqlite::{Connection, params};
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

pub fn set_done(conn: &Connection, id: i64, done: bool) -> Result<(), AppError> {
    let changed = conn.execute(
        "UPDATE todos SET done = ?1 WHERE id = ?2",
        params![done, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(id));
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), AppError> {
    let changed = conn.execute("DELETE FROM todos WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(AppError::NotFound(id));
    }
    Ok(())
}
```

- `set_done` 有兩個參數，所以用 `params![done, id]`：`?1` 是第一個（`done`），`?2` 是第二個（`id`）。`bool` 會被存成 `0` 或 `1`。只有一個參數時，直接寫 `[id]` 就好。
- `execute` 回傳「受影響的列數」。`WHERE id = ?2` 找不到那一列時，SQLite 不會報錯，只是改了 0 列。要是不檢查，前端會以為成功了，其實什麼都沒發生。所以 0 列就回傳 `AppError::NotFound`，把「找不到」明確告訴前端。

### 兩個 command

`AppError` 多一個變體 `NotFound(i64)`，`#[error("找不到編號 {0} 的待辦")]` 的 `{0}` 是變體裡的第一個欄位，也就是那個 `id`。兩個 command 跟上一集一樣，只是鎖住連線、交給 `db.rs`：

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
    #[error("找不到編號 {0} 的待辦")]
    NotFound(i64),
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

#[tauri::command]
fn set_done(id: i64, done: bool, state: State<Mutex<Connection>>) -> Result<(), AppError> {
    db::set_done(&state.lock().unwrap(), id, done)
}

#[tauri::command]
fn delete_todo(id: i64, state: State<Mutex<Connection>>) -> Result<(), AppError> {
    db::delete(&state.lock().unwrap(), id)
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
        .invoke_handler(tauri::generate_handler![
            list_todos,
            add_todo,
            set_done,
            delete_todo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`set_done` 這個 command 跟 `db::set_done` 同名，沒有衝突：一個在 `lib.rs`，一個在 `db` 模組裡，呼叫時寫 `db::` 就分得開。

### 前端：勾選框與刪除按鈕

`todos.ts` 多兩個函式。傳給 `invoke` 的鍵 `id`、`done` 跟 Rust 參數同名：

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

export function setDone(id: number, done: boolean): Promise<void> {
  return invoke<void>("set_done", { id: id, done: done });
}

export function deleteTodo(id: number): Promise<void> {
  return invoke<void>("delete_todo", { id: id });
}
```

`index.html` 與 `src/styles.css` 沒變，照樣貼一次：

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

`main.ts` 的 `render` 要多畫勾選框與刪除按鈕。勾選框是 `<input>`，用 JavaScript 建出來時，`type` 設成 `"checkbox"`；它的 `checked` 是 `boolean`，記著有沒有勾，可以讀也可以寫，就像輸入框的 `value`。使用者勾選或取消時，勾選框會發出 `change` 事件。輸入框的 `input` 事件是每打一個字就觸發，勾選框的 `change` 是勾選的狀態改變時觸發一次。

新增、勾選、刪除這三個動作的處理方式一樣：呼叫 `todos.ts` 的函式、失敗就把原因放進狀態列、最後重畫清單。把這段抽成 `attempt`。它收兩個參數：動作的名稱（用在錯誤訊息裡），以及一個「不收參數、回傳 `Promise<void>` 的函式」，型別寫成 `() => Promise<void>`（第 2 章第 5 集的函式型別）：

`src/main.ts`：

```ts
import { Todo, addTodo, deleteTodo, listTodos, setDone } from "./todos";

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

    const labelEl = document.createElement("label");
    const checkEl = document.createElement("input");
    checkEl.type = "checkbox";
    checkEl.checked = todo.done;
    checkEl.addEventListener("change", () => {
      attempt("更新", () => setDone(todo.id, checkEl.checked));
    });
    const textEl = document.createElement("span");
    textEl.textContent = todo.title;
    if (todo.done) {
      textEl.className = "done";
    }
    labelEl.append(checkEl, textEl);

    const deleteEl = document.createElement("button");
    deleteEl.type = "button";
    deleteEl.textContent = "刪除";
    deleteEl.addEventListener("click", () => {
      attempt("刪除", () => deleteTodo(todo.id));
    });

    itemEl.append(labelEl, deleteEl);
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

const attempt = async (action: string, work: () => Promise<void>) => {
  try {
    await work();
    show("");
  } catch (error) {
    show(`${action}失敗：${error}`);
  }
  await refresh();
};

formEl.addEventListener("submit", (event) => {
  event.preventDefault();
  attempt("新增", async () => {
    await addTodo(titleEl.value);
    titleEl.value = "";
  });
});

refresh();
```

- 每一列是 `li`，裡面一個 `label`（勾選框加文字）和一個刪除按鈕。`labelEl.append(checkEl, textEl)` 一次放兩個子元素，`append` 可以接好幾個。
- 勾選框的 `change` 函式裡，讀 `checkEl.checked` 的當下已經是改過的新狀態，把它送給 Rust 存起來。完成的待辦，文字加上 `done` 這個 class，套用第 1 集寫的灰色刪除線。
- `attempt` 先 `await work()`，不論成功或失敗，最後都重新讀一次清單。成功時，畫面要換成新的資料；失敗時，畫面可能本來就跟資料庫對不上，重新讀一次才會對齊（下面會看到一個例子）。
- 新增的處理函式只剩 `preventDefault()` 與一個 `attempt`：輸入框是在 `addTodo` 成功之後才清空，失敗時字還留著。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.94s
     Running `target/debug/tauri-app`
```

沿用上一集的資料：清單裡有「買牛奶」（`id` 是 1）和「Tom's list」（`id` 是 2）。勾選「買牛奶」，文字變成灰色加刪除線，再勾一次就恢復；按「Tom's list」旁邊的「刪除」，它就從清單消失。到終端機確認（macOS 的路徑）：

```bash
sqlite3 -header -column "$HOME/Library/Application Support/tw.example.tauri-app/todos.db" "SELECT * FROM todos"
```

```text
id  title   done
--  ------  ----
 1  買牛奶     1
```

資料庫裡只剩「買牛奶」，`done` 是 `1`。關掉 App 再開，它還是勾著的。

### 找不到那一列的時候

`NotFound` 平常不容易碰到，但可以刻意做出來。App 開著、清單上還有「買牛奶」，另開一個終端機，從資料庫背後把它刪掉：

```bash
sqlite3 "$HOME/Library/Application Support/tw.example.tauri-app/todos.db" "DELETE FROM todos WHERE id = 1"
```

回到 App，畫面上還是舊的清單，「買牛奶」還在。點它的勾選框，`UPDATE` 找不到 `id` 是 1 的那一列，改了 0 列，Rust 回傳 `NotFound`，狀態列出現「更新失敗：找不到編號 1 的待辦」。`attempt` 接著重新讀清單，「買牛奶」消失，畫面跟資料庫對上了。

## 重點整理

- 修改用 `UPDATE ... WHERE id = ?`，刪除用 `DELETE ... WHERE id = ?`；多個參數寫成 `params![a, b]`，一個參數寫成 `[a]`。
- `execute` 回傳受影響的列數。`WHERE` 沒找到任何一列時，SQLite 不會報錯，所以要自己檢查，回傳 `NotFound`。
- 勾選框用 `type = "checkbox"` 建立，`checked` 讀寫有沒有勾，勾選狀態改變時發出 `change` 事件。
- 新增、勾選、刪除的前端處理方式一樣，抽成 `attempt`：執行動作、失敗時顯示原因、最後重新讀一次清單。
