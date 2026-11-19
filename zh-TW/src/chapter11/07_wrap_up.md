# 收尾

## 本集目標

替待辦清單換上自己的視窗標題與大小，替資料庫函式寫測試（包括舊資料庫升級），清掉範本留下的檔案，最後回頭看這一章做了什麼、還能怎麼改。

## 正文

### 視窗設定

視窗標題現在還是 `tauri-app`。第 6 章第 1 集說過，標題與大小在 `src-tauri/tauri.conf.json` 的 `app.windows`。單欄的畫面不需要太寬，改成 480×640，其餘設定不動：

```json,ignore
"windows": [
  {
    "title": "待辦清單",
    "width": 480,
    "height": 640
  }
]
```

`identifier` 不要動：它決定 app 資料夾的名字（第 7 章第 6 集），改了就找不到 `todos.db`，使用者會以為待辦全不見了。

### 測試資料庫函式

`db.rs` 的函式只收 `&Connection`，不碰 Tauri 的型別，所以不用開 App、不用開視窗就能測它們（第 9 章第 1 集）。測試要的是一個乾淨的資料庫：`Connection::open_in_memory()` 開一個只存在記憶體裡的資料庫，跟第 1 集的 `:memory:` 一樣，不留檔案，每個測試各拿一個全新的，互不影響。

測試放在 `db.rs` 最後的 `tests` 模組。`use super::*` 讓它看得到同一個檔案裡的所有東西，包括沒有 `pub` 的 `migrate`。其餘的檔案沒有改，為了讓這一集也能獨立開始，一併貼在下面：

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
        return Err(AppError::TooNew(version, LATEST));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        conn
    }

    #[test]
    fn add_trims_and_rejects_empty() {
        let conn = memory();
        add(&conn, "  買牛奶 ").unwrap();
        assert!(matches!(add(&conn, "   "), Err(AppError::EmptyTitle)));
        let todos = list(&conn).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "買牛奶");
        assert!(!todos[0].done);
        assert!(todos[0].created_on.is_some());
    }

    #[test]
    fn set_done_and_delete_report_missing_id() {
        let conn = memory();
        add(&conn, "買牛奶").unwrap();
        let id = list(&conn).unwrap()[0].id;
        set_done(&conn, id, true).unwrap();
        assert!(list(&conn).unwrap()[0].done);
        assert!(matches!(
            set_done(&conn, id + 1, true),
            Err(AppError::NotFound(_))
        ));
        delete(&conn, id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
        assert!(matches!(delete(&conn, id), Err(AppError::NotFound(_))));
    }

    #[test]
    fn migrate_upgrades_an_old_database() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE todos (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                done INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO todos (title) VALUES ('舊的待辦');",
        )
        .unwrap();
        migrate(&mut conn).unwrap();
        let todos = list(&conn).unwrap();
        assert_eq!(todos[0].title, "舊的待辦");
        assert_eq!(todos[0].created_on, None);
        migrate(&mut conn).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 1);
    }

    #[test]
    fn migrate_refuses_a_newer_database() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        assert!(matches!(migrate(&mut conn), Err(AppError::TooNew(99, _))));
    }
}
```

四個測試各守一件事：

- `add_trims_and_rejects_empty`：標題前後的空白會去掉，只有空白的標題被拒絕，新增的待辦預設沒完成、有建立日期。
- `set_done_and_delete_report_missing_id`：勾選與刪除都改得到資料；對不存在的 `id` 回傳 `NotFound`，不是默默成功。
- `migrate_upgrades_an_old_database`：先手動建出第 5 集那個沒有 `created_on` 的舊資料表、放一列資料，再升級：舊資料還在，`created_on` 是 `NULL`，再升級一次什麼都不會變。這就是上一集在 App 裡手動做的事，現在每次 `cargo test` 都會自動做一遍。
- `migrate_refuses_a_newer_database`：把版本設成 99，`migrate` 要回傳 `TooNew`。

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

`cargo test --manifest-path src-tauri/Cargo.toml` 的輸出（節錄）：

```text
running 4 tests
test db::tests::migrate_refuses_a_newer_database ... ok
test db::tests::migrate_upgrades_an_old_database ... ok
test db::tests::add_trims_and_rejects_empty ... ok
test db::tests::set_done_and_delete_report_missing_id ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

這些測試只管 SQL 與 migration。command 的接線與事件沒有測：第 9 章第 2 集的 mock runtime 不能直接套用，因為 command 參數裡的 `AppHandle` 預設是真實視窗系統的那一版，放不進 mock 的 `Builder`，得把 command 改寫成泛型才行，這一章不做。

### 其餘的檔案

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
    #[error("資料庫的版本是 {0}，這個程式只認得到 {1}，請更新程式")]
    TooNew(i64, i64),
    #[error("送出事件失敗：{0}")]
    Emit(#[from] tauri::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn notify(app: &AppHandle, conn: &Connection) -> Result<(), AppError> {
    app.emit("todos-changed", db::list(conn)?)?;
    Ok(())
}

#[tauri::command]
fn list_todos(state: State<Mutex<Connection>>) -> Result<Vec<db::Todo>, AppError> {
    db::list(&state.lock().unwrap())
}

#[tauri::command]
fn add_todo(title: &str, app: AppHandle, state: State<Mutex<Connection>>) -> Result<(), AppError> {
    let conn = state.lock().unwrap();
    db::add(&conn, title)?;
    notify(&app, &conn)
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
    notify(&app, &conn)
}

#[tauri::command]
fn delete_todo(id: i64, app: AppHandle, state: State<Mutex<Connection>>) -> Result<(), AppError> {
    let conn = state.lock().unwrap();
    db::delete(&conn, id)?;
    notify(&app, &conn)
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

#todo-list small {
  color: #888;
}

.done {
  color: #888;
  text-decoration: line-through;
}
```

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

### 清掉範本留下的檔案

範本的 `src/assets/` 放著三個 logo，這個專案沒用到。刪掉它們（下面的 `rm -r` 是 macOS、Linux 的指令；Windows 直接在檔案總管刪掉 `src/assets` 資料夾），再用 `npm run build` 確認沒有任何地方還需要它：

```bash
rm -r src/assets
npm run build
```

`npm run build` 的輸出：

```text
> tauri-app@0.1.0 build
> tsc && vite build

vite v8.3.8 building client environment for production...
transforming...
✓ 9 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                 0.63 kB │ gzip: 0.40 kB
dist/assets/index-B3MOc-hy.css  0.38 kB │ gzip: 0.24 kB
dist/assets/index-TxbkJYEi.js   3.17 kB │ gzip: 1.46 kB

✓ built in 31ms
```

`tsc` 型別檢查通過、Vite 也打包成功，表示前端沒有留下型別錯誤，也沒有找不到的檔案。

### 回頭看

待辦清單的每一塊，主要靠這些東西做成：

- 資料庫：rusqlite 的 `bundled`，連線放在 `State<Mutex<Connection>>`，在 `setup` 裡開啟（第 4 章、這一章第 2 集）。
- 讀寫：`prepare`、`query_map`、`execute`，值一律走參數；SQL 都收在 `db.rs`（這一章第 3、4 集）。
- 前端與 Rust 的分工：前端只用 `invoke` 呼叫四個 command，資料變了由 Rust 送 `todos-changed` 事件通知畫面（第 3、5 章）。
- 結構升級：`user_version` 加上一串 migration，舊資料庫不丟資料（這一章第 6 集）。
- 測試：`db.rs` 不碰 Tauri，用記憶體資料庫就能測（第 9 章）。

跟第 10 章相反，這一章沒有用到 fs、dialog 這類 plugin，所有資料庫的工作都在自己寫的 command 裡。

這個 App 刻意沒有用只有桌面才有的功能：沒有選單、系統匣、全域快捷鍵，也不開第二個視窗，畫面是單欄、每個動作都能用手指點。這樣第 18 章要把它搬到手機時，不用先拆掉這些東西。

### 還能怎麼改

- 沒有 `async` 的 command 跑在主執行緒（第 3 章第 8 集）。待辦這麼少的資料，查詢快到感覺不出來；資料多了、查詢變慢，畫面就會卡住。那時要把 command 改成 `async`，用 `tauri::async_runtime::spawn_blocking` 把資料庫的工作交給另一條執行緒。
- 標題建立之後不能編輯，也不能排序或篩選。這些都是 SQL 的事：`UPDATE` 改標題，`ORDER BY`、`WHERE` 排序與篩選。
- 刪除沒有確認，也不能復原。可以多加一個欄位標記「已刪除」，不真的刪掉資料列。
- 範本附的 opener plugin 這個專案沒用到，可以從 `lib.rs` 與 `capabilities/default.json` 拿掉；permission 與 capability 第 14 章會正式介紹。
- 做成安裝檔給別人用，第 15 章會正式介紹。

## 重點整理

- 視窗標題與大小寫在 `tauri.conf.json` 的 `app.windows`；`identifier` 決定資料夾，不能隨便改，否則找不到 `todos.db`。
- 資料庫函式只收 `&Connection`，就能用 `Connection::open_in_memory()` 在記憶體裡測，不用開 App；migration 也能這樣測：先做出舊結構、放資料，再升級。
- 把範本沒用到的檔案刪掉，再跑 `npm run build`，確認沒有東西還需要它。
- 同步的 command 跑在主執行緒，資料庫的工作變慢時，要改成 `async` command 加 `spawn_blocking`。
