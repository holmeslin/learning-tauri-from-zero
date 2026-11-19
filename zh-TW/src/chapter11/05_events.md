# 用事件通知畫面更新

## 本集目標

讓 Rust 在資料庫有變動時送出 `todos-changed` 事件，前端收到事件才重畫清單，新增、勾選、刪除不用再各自重新讀一次。

## 正文

### 誰該說「資料變了」

上一集的前端靠 `attempt` 記得「動作做完要重新讀一次清單」。單一視窗、只有按鈕會改資料的 App，這樣完全夠用。但資料是 Rust 端在管：哪個 command、哪個視窗、甚至 Rust 自己的程式改了資料，畫面都該跟著變。這件事交給前端各自記得，早晚會漏掉一個地方。

換個做法：每次資料庫改完，由 Rust 送出一個 `todos-changed` 事件（第 5 章第 1 集），payload 就是最新的整份清單。前端只要 `listen` 這個事件，收到就重畫。畫面更新只剩一個入口，各個動作做完不用再管畫面。

清單很大的話，每次送整份太浪費，那時可以只送一個沒有內容的事件，讓前端收到後自己重新讀。待辦清單的資料量不大，直接送整份最簡單。

### Rust 端：改完就 `emit`

`db.rs` 只有一個改動：`Todo` 多了 `Clone`。`emit` 的 payload 要同時是 `Serialize` 與 `Clone`（第 5 章第 1 集），`Vec<Todo>` 要能 `Clone`，`Todo` 就得能 `Clone`：

`src-tauri/src/db.rs`：

```rust
use crate::AppError;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Serialize)]
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

`lib.rs` 加一個小函式 `emit_todos_changed`：讀出最新的清單，送出 `todos-changed`。三個會改資料的 command 都在成功之後呼叫它。`emit` 定義在 `Emitter` 上，要 `use tauri::Emitter`。`AppError` 沒有變，`emit_todos_changed` 也不回傳錯誤，原因在程式後面說明：

`src-tauri/src/lib.rs`：

```rust
mod db;

use rusqlite::Connection;
use serde::{Serialize, Serializer};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

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

fn emit_todos_changed(app: &AppHandle, conn: &Connection) {
    match db::list(conn) {
        Ok(todos) => {
            if let Err(error) = app.emit("todos-changed", todos) {
                eprintln!("送出 todos-changed 失敗：{error}");
            }
        }
        Err(error) => eprintln!("讀取清單失敗，沒有送出 todos-changed：{error}"),
    }
}

#[tauri::command]
fn list_todos(state: State<Mutex<Connection>>) -> Result<Vec<db::Todo>, AppError> {
    db::list(&state.lock().unwrap())
}

#[tauri::command]
fn add_todo(title: &str, app: AppHandle, state: State<Mutex<Connection>>) -> Result<(), AppError> {
    let conn = state.lock().unwrap();
    db::add(&conn, title)?;
    emit_todos_changed(&app, &conn);
    Ok(())
}

#[tauri::command]
fn set_done(
    id: i64,
    done: bool,
    app: AppHandle,
    state: State<Mutex<Connection>>,
) -> Result<(), AppError> {
    let conn = state.lock().unwrap();
    db::set_done(&conn, id, done)?;
    emit_todos_changed(&app, &conn);
    Ok(())
}

#[tauri::command]
fn delete_todo(id: i64, app: AppHandle, state: State<Mutex<Connection>>) -> Result<(), AppError> {
    let conn = state.lock().unwrap();
    db::delete(&conn, id)?;
    emit_todos_changed(&app, &conn);
    Ok(())
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

- 三個 command 都多了一個 `AppHandle` 參數（第 3 章第 9 集），`emit` 要靠它。`AppHandle` 跟 `State` 都是 Tauri 幫 command 填的參數，前端呼叫時完全不用傳。
- command 先 `let conn = state.lock().unwrap()` 把鎖守衛存下來，改完資料庫，再用同一個 `conn` 呼叫 `emit_todos_changed`。
- `emit_todos_changed` 收的是 `&Connection`，不是 `State`，因為它不該自己再 `lock()`：標準函式庫的 `Mutex` 不能重複上鎖，同一條執行緒鎖已經握在手上的 `Mutex`，會卡死或 panic（標準函式庫沒有保證是哪一種）。需要連線的函式一律收 `&Connection`，由最外層的 command 上鎖，是這一章 `db.rs` 一貫的寫法。
- 改失敗（標題是空的、找不到那一列）時，`?` 已經提早回傳，不會走到 `emit_todos_changed`：資料沒變，就不送事件。
- `emit_todos_changed` 沒有回傳值，失敗只用 `eprintln!` 印出來（第 5 章第 3 集用過）。它被呼叫時，資料已經寫進資料庫了：要是這時讀清單或 `emit` 失敗就回傳 `Err`，前端會顯示「新增失敗」，使用者再按一次，同一則待辦就新增兩次。所以 command 照樣回傳成功；這一次畫面沒更新，但資料是對的，下一個事件或重新載入就會補上。

### 前端：收到事件就重畫

事件的名字與 payload 的型別，一樣集中在 `todos.ts`。`onTodosChanged` 包住 `listen`（第 5 章第 2 集）：收到事件，就把 payload（最新的 `Todo[]`）交給呼叫端傳進來的函式：

`src/todos.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { UnlistenFn, listen } from "@tauri-apps/api/event";

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

export function onTodosChanged(
  callback: (todos: Todo[]) => void,
): Promise<UnlistenFn> {
  return listen<Todo[]>("todos-changed", (event) => callback(event.payload));
}
```

`index.html` 與 `src/styles.css` 沒變：

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

`main.ts` 有三處改動：

- 啟動的 `start` 先 `await onTodosChanged(render)` 登記監聽，再 `refresh()` 讀第一次。順序不能反：事件不會補發（第 5 章第 2 集），要先登記好，之後送來的事件才不會漏掉。
- `attempt` 成功時不用再 `refresh`：事件會帶著新清單來。
- `attempt` 失敗時仍然 `refresh`：失敗代表資料沒變，Rust 沒有送事件，要是畫面是因為過期才失敗的（上一集刪掉 `id` 為 1 的那個例子），就沒人替它更新，所以自己讀一次。

`src/main.ts`：

```ts
import {
  Todo,
  addTodo,
  deleteTodo,
  listTodos,
  onTodosChanged,
  setDone,
} from "./todos";

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
    await refresh();
  }
};

formEl.addEventListener("submit", (event) => {
  event.preventDefault();
  attempt("新增", async () => {
    await addTodo(titleEl.value);
    titleEl.value = "";
  });
});

const start = async () => {
  await onTodosChanged(render);
  await refresh();
};

start();
```

`listen` 需要的 permission，`capabilities/default.json` 的 `core:default` 已經包含（第 5 章第 1 集），不用改。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.47s
     Running `target/debug/tauri-app`
```

畫面的行為跟上一集一樣：新增、勾選、刪除之後，清單立刻更新。不同的是現在更新是事件帶來的，不是按鈕的函式自己去讀的。想親眼確認的話，把 `start` 裡 `await onTodosChanged(render);` 那一行暫時註解掉：存檔重新載入後，清單還是會讀出來（`refresh` 還在），但新增一則，畫面沒有任何變化，因為沒有人在聽 `todos-changed`；重新載入頁面才會看到它。把那行加回去就恢復了。

## 重點整理

- 資料由 Rust 管，就由 Rust 在資料庫改完之後 `emit("todos-changed", 最新的清單)`；前端只 `listen`，收到就重畫。畫面更新只剩一個入口。
- payload 要 `Serialize + Clone`，所以 `Todo` 要加 `Clone`；送出事件時資料已經寫進資料庫，`emit` 失敗只印出來，不讓 command 回傳 `Err`，免得使用者重按而重複新增。
- `emit_todos_changed` 收 `&Connection`，不自己 `lock()`：同一條執行緒對握在手上的標準 `Mutex` 再上鎖，會卡死或 panic。
- 前端先 `listen` 再讀第一次清單，因為事件不會補發；動作失敗時沒有事件，要自己重新讀一次，畫面才不會停在過期的狀態。
