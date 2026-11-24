# 資料庫結構升級（migration）

## 本集目標

替待辦多加一個「建立日期」欄位，並學會用 `PRAGMA user_version` 與一串 migration，讓已經存了資料的舊資料庫也能升級，不丟資料。

## 正文

### 資料表改版：舊資料庫怎麼辦

我們想在每則待辦旁邊顯示是哪一天建立的，所以 `todos` 要多一個欄位 `created_on`。最直覺的做法，是在 `CREATE TABLE IF NOT EXISTS` 那段 SQL 裡加上 `created_on TEXT`，`list` 的 `SELECT` 也跟著加上這個欄位。

全新的資料庫沒有問題。但你的 `todos.db` 在前幾集就已經建好、也存了資料。對它來說，`todos` 資料表早就存在，`IF NOT EXISTS` 讓整句 SQL 什麼都不做，欄位根本沒有被加上去。照這個做法改完再啟動，狀態列會出現：

```text
讀取失敗：資料庫錯誤：no such column: created_on in SELECT id, title, done, created_on FROM todos ORDER BY id at offset 24
```

App 發出去之後，使用者的電腦上各有一份舊資料庫，沒辦法請他們刪掉重來。所以資料表的結構要「升級」：用 SQL 改掉舊的結構，資料原封不動。這件事叫 migration（結構遷移）。

### 資料庫自己記著版本

升級要知道「現在是第幾版」。SQLite 在資料庫檔案裡替你預留了一個整數，叫 `user_version`，不屬於任何資料表，預設是 `0`，讀寫用 `PRAGMA`（SQLite 的設定指令）。可以先在第 5 集的版本裡新增「買牛奶」和「寫第 11 章」兩則待辦，關掉 App，用 `sqlite3` 看看版本（macOS 的路徑）：

```bash
sqlite3 "$HOME/Library/Application Support/tw.example.tauri-app/todos.db" "PRAGMA user_version"
```

```text
0
```

版本是 `0`。我們的做法是：把「每一次結構改動」寫成一句 SQL，依序放進一個陣列。第 `n` 句就是「從版本 `n - 1` 升到版本 `n`」。啟動時讀出資料庫目前的版本，把還沒跑過的幾句依序跑完，每跑完一句就把版本更新。

- 第 1 句：`CREATE TABLE IF NOT EXISTS todos ...`，就是前幾集建立資料表的那句。舊資料庫（版本 `0`，但資料表已經在了）跑它不會有任何影響，只是把版本記成 `1`；全新的資料庫則在這裡建出資料表。
- 第 2 句：`ALTER TABLE todos ADD COLUMN created_on TEXT`，替資料表加一個欄位。

已經發出去的 migration 不能改。改了，跑過舊版的資料庫就不會再跑它，結構會跟新資料庫不一樣。要改結構，一律在陣列後面再加一句。

### `migrate`

`db.rs` 的 `open` 不再自己建資料表，改成呼叫 `migrate`。`Todo` 也多了 `created_on`：舊的待辦是這個欄位出現之前建立的，升級之後它們的 `created_on` 是 `NULL`，所以型別是 `Option<String>`，SQL 的 `NULL` 對應 Rust 的 `None`。`rename_all` 讓它在前端是 `createdOn`（第 3 章第 5 集）。

`src-tauri/src/db.rs`：

```rust
use crate::AppError;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    id: i64,
    title: String,
    done: bool,
    created_on: Option<String>,
}

const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS todos (
        id INTEGER PRIMARY KEY,
        title TEXT NOT NULL,
        done INTEGER NOT NULL DEFAULT 0
    )",
    "ALTER TABLE todos ADD COLUMN created_on TEXT",
];

const LATEST: i64 = MIGRATIONS.len() as i64;

fn migrate(conn: &mut Connection) -> Result<(), AppError> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > LATEST {
        return Err(AppError::TooNew {
            found: version,
            supported: LATEST,
        });
    }
    for (n, sql) in (1..).zip(MIGRATIONS) {
        if n <= version {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", n)?;
        tx.commit()?;
    }
    Ok(())
}

pub fn open(path: &Path) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;
    migrate(&mut conn)?;
    Ok(conn)
}

pub fn list(conn: &Connection) -> Result<Vec<Todo>, AppError> {
    let mut stmt = conn.prepare("SELECT id, title, done, created_on FROM todos ORDER BY id")?;
    let todos = stmt
        .query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                done: row.get(2)?,
                created_on: row.get(3)?,
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
    conn.execute(
        "INSERT INTO todos (title, created_on) VALUES (?1, date('now', 'localtime'))",
        [title],
    )?;
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

- `LATEST` 是程式認得的最新版本，也就是 migration 的句數。
- `pragma_query_value` 讀出 `user_version`。`PRAGMA user_version` 的結果只有一個值，所以取第 0 個欄位。
- `(1..).zip(MIGRATIONS)` 把每一句配上它的版本號 `n`（從 1 開始）。`n <= version` 的是跑過的，`continue` 跳過；剩下的才執行。
- 每一句各開一個交易（transaction）：`conn.transaction()` 開始，`tx.commit()` 才算數；沒有 `commit` 就離開（例如某個 `?` 提早回傳），`tx` 被丟掉時會自動取消。交易裡先 `execute_batch` 跑那句 SQL，再用 `pragma_update` 把 `user_version` 設成 `n`，兩件事要嘛一起生效、要嘛一起取消：半途失敗的話，資料庫維持在原來的版本，下次啟動再試一次。
- `conn.transaction()` 要 `&mut Connection`，所以 `open` 裡的 `conn` 改成 `mut`。
- 資料庫的版本號比 `LATEST` 還大（例如使用者裝回舊版的 App），就回傳 `TooNew`，不去碰它：舊程式不認得新結構，硬寫進去可能會把資料寫壞。
- `add` 的 SQL 多寫入 `created_on`，值用 SQLite 的 `date('now', 'localtime')`：今天的日期，本地時區，格式是 `2026-11-19`，存成文字。
- SQLite 的 `ALTER TABLE` 能做的事有限，例如不能直接改欄位的型別；比較大的改動要建一張新的資料表、把資料搬過去、再換名字。這一章只加一個欄位，用不到。

`lib.rs` 只多了 `TooNew` 這個錯誤變體。它有兩個版本號，所以用具名欄位 `found`（資料庫的版本）與 `supported`（程式認得的版本），不靠位置：訊息裡寫 `{found}`、`{supported}`，建立錯誤時也不會把兩個數字放反：

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
    #[error("資料庫的版本是 {found}，這個程式只認得到 {supported}，請更新程式")]
    TooNew { found: i64, supported: i64 },
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

### 前端：顯示建立日期

`Todo` 多一個 `createdOn`，型別是 `string | null`：舊的待辦沒有日期，Rust 的 `None` 到前端就是 `null`。畫面只在有日期的時候才多畫一個 `<small>`（小字的元素）：

`src/todos.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { UnlistenFn, listen } from "@tauri-apps/api/event";

export interface Todo {
  id: number;
  title: string;
  done: boolean;
  createdOn: string | null;
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

`index.html` 沒變：

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

`src/styles.css` 多一條規則，讓日期是灰色的：

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

#todo-list small {
  color: #888;
}

.done {
  color: #888;
  text-decoration: line-through;
}
```

`main.ts` 只有 `render` 變了，在勾選框那一塊和刪除按鈕之間，加上日期：

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
    itemEl.append(labelEl);

    if (todo.createdOn !== null) {
      const dateEl = document.createElement("small");
      dateEl.textContent = todo.createdOn;
      itemEl.append(dateEl);
    }

    const deleteEl = document.createElement("button");
    deleteEl.type = "button";
    deleteEl.textContent = "刪除";
    deleteEl.addEventListener("click", () => {
      attempt("刪除", () => deleteTodo(todo.id));
    });
    itemEl.append(deleteEl);

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

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.01s
     Running `target/debug/tauri-app`
```

用前面存了兩則待辦的舊資料庫啟動，清單沒有任何錯誤，兩則舊待辦都在，而且沒有日期。新增一則「新的一則」，它旁邊多了一個灰色的日期。關掉 App，用 `sqlite3` 看資料庫：

```bash
sqlite3 -header -column "$HOME/Library/Application Support/tw.example.tauri-app/todos.db" "PRAGMA user_version" ".schema" "SELECT * FROM todos"
```

```text
user_version
------------
           2
CREATE TABLE todos (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0
        , created_on TEXT);
id    title     done  created_on
--  ----------  ----  ----------
 1  買牛奶         0
 2  寫第 11 章     0
 3  新的一則       0  2026-11-19
```

版本變成 `2`；`.schema` 最後多了 `created_on`，它是接在原本那句 SQL 後面的；舊的兩列在 `created_on` 是空的（`NULL`），新的那一列有日期。再啟動一次，版本已經是 `2`，兩句 migration 都不會再跑。全新的資料庫（把 `todos.db` 刪掉再啟動）則是兩句一起跑完，直接是版本 `2`。

## 重點整理

- `CREATE TABLE IF NOT EXISTS` 只管「沒有資料表才建」，不會改已經存在的資料表。結構要改，就得升級舊資料庫，這叫 migration。
- 版本號存在資料庫自己的 `PRAGMA user_version`（預設 `0`）。把每次結構改動寫成一句 SQL 放進陣列，啟動時讀版本、依序跑還沒跑過的，每句各在一個交易裡跑完再更新版本。
- 已經發出去的 migration 不能改，只能在後面加；資料庫版本比程式新時要拒絕，不去寫它。
- `ALTER TABLE ... ADD COLUMN` 加的欄位，舊的列是 `NULL`：Rust 用 `Option<T>` 接，前端是 `T | null`。
