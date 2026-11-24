# 規劃資料表

## 本集目標

決定待辦清單要存哪些資料、程式怎麼分工，認識這一章會用到的四個 SQL 語句，並先用 HTML 與 CSS 排出畫面。

## 正文

### 要做什麼

這個 App 要能：

- 輸入一句話，按「新增」，加進清單。
- 勾選表示完成，再勾一次就取消。
- 刪除一則待辦。
- 資料存在資料庫裡，關掉再開還在。
- 之後想替資料多加一個欄位時，舊資料不能丟（第 6 集）。

畫面是單欄的：最上面是標題，下面是輸入列，再下面是清單，最底下一行是狀態列，出錯時在這裡說明。這個待辦清單之後還要搬到手機上（第 18 章會正式介紹），所以版面不分左右兩欄，也不靠滑鼠移上去才出現的按鈕。

### 為什麼用 SQLite

第 10 章的筆記存成一個 JSON 檔，每次都整份讀進來、整份寫回去。資料少的時候沒問題；資料一多，改一筆就得重寫整個檔案，想「只找出還沒完成的」也得自己寫迴圈。

資料庫（database）就是為這種事設計的。SQLite 是一個放在單一檔案裡的資料庫：不用另外安裝，也不用啟動伺服器，App 開啟那個檔案就能用。新增、查詢、修改都用 SQL，一種專門跟資料庫說話的語言。很多 App 都拿它當本機的儲存空間。

### 資料表

SQLite 把資料放在資料表（table）裡。資料表像一張試算表：每個欄位（column）有名字和型別，每一列（row）是一筆資料。待辦清單只需要一張資料表 `todos`：

```sql
CREATE TABLE todos (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  done INTEGER NOT NULL DEFAULT 0
);
```

三個欄位：

- `id`：每一列的編號，是這張資料表的主鍵（primary key）。`INTEGER PRIMARY KEY` 會自動編號：新增時不給值，SQLite 就用「目前最大的編號加一」。之後要改或刪某一則，都靠 `id` 指定。
- `title`：待辦的文字。`TEXT` 是文字型別，`NOT NULL` 表示一定要有值。SQL 的 `NULL` 是「沒有值」，角色跟 Rust 的 `None` 差不多。
- `done`：有沒有完成。SQLite 沒有獨立的布林型別，慣例用整數：`0` 是還沒完成，`1` 是完成。`DEFAULT 0` 讓新增時可以不寫它。

### 四個 SQL 語句

這一章用到的 SQL 只有四種，對應新增、讀取、修改、刪除：

```sql
INSERT INTO todos (title) VALUES ('買牛奶');
SELECT id, title, done FROM todos ORDER BY id;
UPDATE todos SET done = 1 WHERE id = 2;
DELETE FROM todos WHERE id = 1;
```

- `INSERT INTO 表 (欄位) VALUES (值)`：新增一列。沒列出的欄位（`id`、`done`）由 SQLite 補上。
- `SELECT 欄位 FROM 表 ORDER BY 欄位`：讀出列，這裡依 `id` 由小到大排。
- `UPDATE 表 SET 欄位 = 值 WHERE 條件`：修改符合條件的列。
- `DELETE FROM 表 WHERE 條件`：刪除符合條件的列。

`UPDATE` 與 `DELETE` 後面的 `WHERE` 不能忘：沒寫條件，整張資料表的每一列都會被改掉、刪掉。

macOS 內建 `sqlite3` 指令，可以拿來試試看；沒有這個指令的話，跳過這一段也沒關係，下一集會在 Rust 裡跑同樣的 SQL。`:memory:` 是特殊的資料庫名稱，資料只放在記憶體裡，指令結束就消失，不會留下任何檔案：

```bash
sqlite3 -header -column :memory: "
CREATE TABLE todos (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  done INTEGER NOT NULL DEFAULT 0
);
INSERT INTO todos (title) VALUES ('買牛奶');
INSERT INTO todos (title) VALUES ('寫第 11 章');
SELECT * FROM todos;
UPDATE todos SET done = 1 WHERE id = 2;
SELECT * FROM todos;
DELETE FROM todos WHERE id = 1;
SELECT * FROM todos;
"
```

`sqlite3` 的輸出：

```text
id    title     done
--  ----------  ----
 1  買牛奶         0
 2  寫第 11 章     0
id    title     done
--  ----------  ----
 1  買牛奶         0
 2  寫第 11 章     1
id    title     done
--  ----------  ----
 2  寫第 11 章     1
```

三次 `SELECT` 各印一張資料表：新增兩列之後有兩列；`UPDATE` 之後第二列的 `done` 變成 `1`；`DELETE` 之後第一列不見了，第二列的 `id` 還是 `2`，不會重新編號。

### 程式怎麼分工

- Rust 端用 rusqlite（Rust 的 SQLite 函式庫，第 2 集介紹）持有資料庫，所有 SQL 都寫在這裡。
- 前端完全不碰資料庫，對資料庫的操作只走 `invoke`，呼叫四個 command：`list_todos`、`add_todo`、`set_done`、`delete_todo`。
- 事件只負責通知：資料有變動時，Rust 送出 `todos-changed` 事件，前端收到就重畫清單（第 5 集）。

Tauri 官方也有 SQL plugin（`tauri-plugin-sql`），讓前端直接把 SQL 送給資料庫。這一章刻意不用它：SQL 留在 Rust 端，前端只看得到「新增待辦」「刪除待辦」這幾個有意義的操作，不能對資料庫下任意的 SQL；以後資料庫的存法有變，前端也不用跟著改。

檔案這樣分：

- `src-tauri/src/db.rs`：資料庫的函式，只管 SQL，不碰 Tauri 的型別。第 9 章第 1 集說過，邏輯跟 Tauri 分開，才能不開 App 就測它（第 7 集會測）。
- `src-tauri/src/lib.rs`：command 與組裝 App。
- `src/todos.ts`：`Todo` 型別，以及包住 `invoke` 的幾個函式。
- `src/main.ts`：畫面的程式。

### 先排畫面

這一集只做靜態的畫面，清單裡的兩則是寫死的，`src/main.ts` 維持範本原樣，第 2 集就會換掉。清單的每一列是「勾選框加文字」，再加一個刪除按鈕。

勾選框是 `<input type="checkbox">`，`checked` 屬性表示一開始就是勾著的，跟第 10 章的 `readonly` 一樣，是不用寫值的屬性。勾選框跟文字一起包進 `<label>`，點文字也會勾到勾選框，用手指點起來比較容易。

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
      <ul id="todo-list">
        <li>
          <label>
            <input type="checkbox" />
            <span>買牛奶</span>
          </label>
          <button type="button">刪除</button>
        </li>
        <li>
          <label>
            <input type="checkbox" checked />
            <span class="done">寫第 11 章</span>
          </label>
          <button type="button">刪除</button>
        </li>
      </ul>
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

排版用的是第 2 章第 3 集的 flex：

- `main` 最寬 480px，用 `margin: 0 auto` 置中：視窗很寬時置中，視窗窄（例如手機）時就用滿整個寬度。
- `#add-form` 是 flex，輸入框 `flex: 1` 吃掉按鈕以外的寬度。`min-width: 0` 讓輸入框在很窄的視窗裡也能縮小，不然它會守住自己的預設寬度，把按鈕擠出畫面。
- `#todo-list li` 也是 flex，`align-items: center` 讓勾選框、文字和按鈕垂直置中。`label` 的 `flex: 1` 讓文字吃掉中間的空間，刪除按鈕就被推到最右邊。
- `.done` 是完成後的樣子：灰色加刪除線，`text-decoration: line-through` 就是刪除線。

`Todo` 型別先寫下來，第 3 集才會用到。它描述 Rust 端會送來的一則待辦，欄位跟資料表一樣，只是 `done` 在這裡是 `boolean`：

`src/todos.ts`：

```ts
export interface Todo {
  id: number;
  title: string;
  done: boolean;
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.38s
     Running `target/debug/tauri-app`
```

視窗裡是單欄的畫面：標題「待辦清單」，下面是輸入框和「新增」按鈕，再下面是兩則待辦。「買牛奶」沒勾；「寫第 11 章」勾著，文字是灰色加刪除線。現在點什麼都沒有反應，因為還沒有任何程式。

## 重點整理

- 待辦清單只要一張資料表 `todos`：`id` 是主鍵，`title` 是文字，`done` 用 `0`、`1` 表示有沒有完成。
- 四個 SQL 語句對應新增、讀取、修改、刪除：`INSERT`、`SELECT`、`UPDATE`、`DELETE`；`UPDATE` 與 `DELETE` 一定要寫 `WHERE`。
- 資料庫只在 Rust 端用 rusqlite 存取；對資料庫的操作只走 `invoke` 呼叫的四個 command，`todos-changed` 事件只負責通知畫面資料變了；不使用 `tauri-plugin-sql`。
- 檔案分工：`db.rs` 管 SQL，`lib.rs` 管 command 與組裝，`todos.ts` 包住 `invoke`，`main.ts` 管畫面。
- 畫面是單欄、不靠滑鼠才能操作，之後搬到手機時不用重排。
