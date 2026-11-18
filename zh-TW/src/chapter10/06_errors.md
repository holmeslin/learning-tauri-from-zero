# 錯誤處理與提示

## 本集目標

讓讀檔、存檔、匯入、匯出出錯時，把原因顯示在狀態列；讀到壞掉的筆記檔時不再存檔，免得蓋掉它；刪除前先問一聲。

## 正文

### 哪些地方會出錯

目前所有呼叫 plugin 的地方，失敗時都只會在 Console 留下一行錯誤，畫面上完全看不出來：

- 讀檔：`notes.json` 被改壞了，`JSON.parse` 丟出錯誤，或內容是合法的 JSON，卻不是筆記陣列。
- 存檔：磁碟滿了、檔案被設成唯讀，`writeTextFile` 被拒絕。
- 匯入、匯出：檔案讀不到、挑的位置寫不進去。

使用者按取消不算出錯，那是 `open()`、`save()` 回傳 `null`，上一集已經處理了。

plugin 的函式失敗時，Promise 會被拒絕，`await` 會把它當成錯誤丟出來（第 2 章第 10 集），所以用 `try`/`catch` 接住，再把原因放進狀態列。plugin 的錯誤到前端是字串，所以直接用 `${error}` 放進訊息就能顯示，不用像第 2 章第 10 集那樣先檢查 `error` 的型別；`JSON.parse` 丟的是 `Error` 物件，放進 `${}` 會變成「名稱: 訊息」的文字，一樣看得懂。

### 壞掉的檔案不能被蓋掉

讀檔失敗之後，最危險的是存檔。讀不出來時，`notes` 還是空的；要是這時照常存檔，壞掉的 `notes.json` 就被一份空陣列蓋掉，使用者連手動搶救的機會都沒了。所以讀檔失敗時，記下「這次不要存檔」，並在狀態列說明；之後每次想存檔，狀態列都會再說明一次，不會默默什麼都沒發生。

`loadNotes` 順便多檢查一步：`JSON.parse` 成功不代表是筆記。`Array.isArray(值)` 回答值是不是陣列；`typeof 值 === "string"` 回答值是不是字串；陣列的 `every(函式)` 跟 `filter` 一樣收一個函式，所有元素都讓函式回答 `true`，結果才是 `true`。`note?.title` 的 `?.` 讓 `note` 是 `null` 時不會出錯。所以「是陣列，而且每一則都有字串的 `title` 與 `body`」才算筆記，否則丟錯。

`src/notes.ts`：

```ts
import { basename } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  BaseDirectory,
  exists,
  mkdir,
  readTextFile,
  writeTextFile,
} from "@tauri-apps/plugin-fs";

export interface Note {
  title: string;
  body: string;
}

const FILE = "notes.json";
const dir = BaseDirectory.AppData;
const markdown = [{ name: "Markdown", extensions: ["md"] }];

export async function loadNotes(): Promise<Note[]> {
  if (!(await exists(FILE, { baseDir: dir }))) {
    return [];
  }
  const text = await readTextFile(FILE, { baseDir: dir });
  const data = JSON.parse(text);
  if (
    !Array.isArray(data) ||
    !data.every(
      (note) =>
        typeof note?.title === "string" && typeof note?.body === "string",
    )
  ) {
    throw new Error(`${FILE} 的內容不是筆記陣列`);
  }
  return data;
}

export async function saveNotes(notes: Note[]): Promise<void> {
  await mkdir("", { baseDir: dir, recursive: true });
  const text = JSON.stringify(notes, null, 2);
  await writeTextFile(FILE, text, { baseDir: dir });
}

export async function importNote(): Promise<Note | null> {
  const path = await open({ filters: markdown });
  if (path === null) {
    return null;
  }
  const body = await readTextFile(path);
  return { title: await basename(path, ".md"), body: body };
}

export async function exportNote(note: Note): Promise<string | null> {
  const path = await save({
    defaultPath: `${note.title || "未命名"}.md`,
    filters: markdown,
  });
  if (path === null) {
    return null;
  }
  await writeTextFile(path, note.body);
  return path;
}
```

### 在畫面上接住

`main.ts` 的改動：

- 新增 `persist`：所有 `saveNotes(notes)` 都改呼叫它。`canSave` 是 `false` 時不存檔，改成把 `noSaveMessage`（不存檔的原因）再顯示一次；否則 `try` 存檔，成功顯示「已儲存」，失敗顯示原因。存好回傳 `true`，其他情況回傳 `false`。
- `canSave` 一開始是 `false`，`start` 讀檔成功才改成 `true`；讀檔失敗就維持 `false`，並把原因記進 `noSaveMessage`。這樣連讀檔還沒完成的那一小段時間，也不會存檔。
- 匯入、匯出各包一層 `try`/`catch`。匯入成功後，等 `persist` 回傳 `true` 才顯示「已匯入」，這樣不會被「已儲存」蓋掉；存檔失敗時，狀態列留著「儲存失敗」。
- 刪除前用 dialog 的 `ask()`（第 7 章第 4 集）確認，回傳 `false` 就什麼都不做，`ask()` 本身出錯也用 `try`/`catch` 接住。標題是空的，提示就寫「未命名」。`dialog:default` 已經放行 `ask`，permission 不用改。

`src-tauri/src/lib.rs`、`src-tauri/capabilities/default.json`、`index.html`、`src/styles.css` 都跟上一集一樣，再貼一次：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "fs:default",
    "fs:allow-appdata-write-recursive",
    "dialog:default"
  ]
}
```

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>筆記</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <div class="sidebar">
      <div>
        <button id="new-btn" type="button">新增</button>
        <button id="import-btn" type="button">匯入</button>
      </div>
      <ul id="note-list"></ul>
    </div>
    <main>
      <input id="title-input" placeholder="標題" />
      <textarea id="body-input" placeholder="內容"></textarea>
      <div>
        <button id="export-btn" type="button">匯出</button>
        <button id="delete-btn" type="button">刪除</button>
      </div>
      <p id="status"></p>
    </main>
  </body>
</html>
```

`src/styles.css`：

```css
body {
  display: flex;
  height: 100vh;
  margin: 0;
  font-family: sans-serif;
}

.sidebar {
  width: 200px;
  padding: 8px;
  overflow-y: auto;
  background-color: #eee;
}

main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px;
}

#note-list {
  padding: 0;
  list-style: none;
}

#note-list li {
  padding: 6px 8px;
  cursor: pointer;
}

#note-list .selected {
  background-color: #cde;
}

#body-input {
  flex: 1;
}
```

`src/main.ts`：

```ts
import { ask } from "@tauri-apps/plugin-dialog";
import { Note, exportNote, importNote, loadNotes, saveNotes } from "./notes";

const listEl: HTMLUListElement | null = document.querySelector("#note-list");
const titleEl: HTMLInputElement | null = document.querySelector("#title-input");
const bodyEl: HTMLTextAreaElement | null = document.querySelector("#body-input");
const statusEl: HTMLElement | null = document.querySelector("#status");
if (!listEl || !titleEl || !bodyEl || !statusEl) {
  throw new Error("找不到必要的元素");
}

let notes: Note[] = [];
let current: Note = { title: "", body: "" };
let canSave = false;
let noSaveMessage = "還在讀取筆記，先不存檔";

const show = (message: string) => {
  statusEl.textContent = message;
};

const render = () => {
  listEl.textContent = "";
  for (const note of notes) {
    const itemEl = document.createElement("li");
    itemEl.textContent = note.title;
    if (note.title === "") {
      itemEl.textContent = "（沒有標題）";
    }
    if (note === current) {
      itemEl.className = "selected";
    }
    itemEl.addEventListener("click", () => select(note));
    listEl.append(itemEl);
  }
};

const select = (note: Note) => {
  current = note;
  titleEl.value = note.title;
  bodyEl.value = note.body;
  render();
};

const showFirst = () => {
  if (notes.length === 0) {
    notes.push({ title: "新筆記", body: "" });
  }
  select(notes[0]);
};

const persist = async () => {
  if (!canSave) {
    show(noSaveMessage);
    return false;
  }
  try {
    await saveNotes(notes);
    show("已儲存");
    return true;
  } catch (error) {
    show(`儲存失敗：${error}`);
    return false;
  }
};

const edit = () => {
  current.title = titleEl.value;
  current.body = bodyEl.value;
  render();
  persist();
};

titleEl.addEventListener("input", edit);
bodyEl.addEventListener("input", edit);

document.querySelector("#new-btn")?.addEventListener("click", () => {
  const note: Note = { title: "新筆記", body: "" };
  notes.push(note);
  select(note);
  persist();
});

document.querySelector("#delete-btn")?.addEventListener("click", async () => {
  try {
    const yes = await ask(`要刪除「${current.title || "未命名"}」嗎？`, {
      kind: "warning",
    });
    if (yes) {
      notes = notes.filter((note) => note !== current);
      showFirst();
      persist();
    }
  } catch (error) {
    show(`刪除失敗：${error}`);
  }
});

document.querySelector("#import-btn")?.addEventListener("click", async () => {
  try {
    const note = await importNote();
    if (note) {
      notes.push(note);
      select(note);
      if (await persist()) {
        show(`已匯入「${note.title}」`);
      }
    }
  } catch (error) {
    show(`匯入失敗：${error}`);
  }
});

document.querySelector("#export-btn")?.addEventListener("click", async () => {
  try {
    const path = await exportNote(current);
    if (path !== null) {
      show(`已匯出到 ${path}`);
    }
  } catch (error) {
    show(`匯出失敗：${error}`);
  }
});

const start = async () => {
  try {
    notes = await loadNotes();
    canSave = true;
  } catch (error) {
    noSaveMessage = `讀取失敗，這次不會存檔：${error}`;
    show(noSaveMessage);
  }
  showFirst();
};

start();
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.37s
     Running `target/debug/tauri-app`
```

### 故意弄壞看看

下面的指令與路徑是 macOS 的（`ada` 是範例的使用者名稱），錯誤訊息的文字依平台與 WebView 而不同。

**寫不進去。** 把檔案改成唯讀，再打幾個字：

```bash
chmod 444 "$HOME/Library/Application Support/tw.example.tauri-app/notes.json"
```

狀態列顯示：

```text
儲存失敗：failed to open file at path: /Users/ada/Library/Application Support/tw.example.tauri-app/notes.json with error: Permission denied (os error 13)
```

改回來之後，下一次打字又會顯示「已儲存」：

```bash
chmod 644 "$HOME/Library/Application Support/tw.example.tauri-app/notes.json"
```

**內容壞掉。** 把檔案改成一段不完整的 JSON，重新啟動 App：

```bash
echo '[{"title":' > "$HOME/Library/Application Support/tw.example.tauri-app/notes.json"
```

狀態列顯示：

```text
讀取失敗，這次不會存檔：SyntaxError: JSON Parse error: Unexpected EOF
```

清單裡是一則空白的「新筆記」，可以打字，但不會存檔，每打一個字，狀態列就再顯示一次這則訊息。看看檔案：

```bash
cat "$HOME/Library/Application Support/tw.example.tauri-app/notes.json"
```

```text
[{"title":
```

壞掉的檔案原封不動，使用者可以用編輯器修好它，或自己搬走。上一集的程式碰到同樣的情況，一打字檔案就被 `[]` 蓋掉了。

**不是筆記陣列。** 內容是合法的 JSON，但不是陣列：

```bash
echo '{"title": "hi"}' > "$HOME/Library/Application Support/tw.example.tauri-app/notes.json"
```

重新啟動，狀態列顯示：

```text
讀取失敗，這次不會存檔：Error: notes.json 的內容不是筆記陣列
```

裡面的筆記缺欄位，例如 `[{"title": "hi"}]`（少了 `body`），也會被擋下來，訊息相同。

### 刪除前確認

按「刪除」，跳出系統的確認對話框，「Yes」才真的刪，「No」什麼都不會發生。`ask()` 是第 7 章第 4 集介紹的；那一集也說過，瀏覽器的 `window.confirm()` 在 macOS 上不會彈出對話框，所以改用 dialog plugin 的。

## 重點整理

- plugin 函式失敗時，`await` 會丟出錯誤；用 `try`/`catch` 接住，把 `${error}` 放進狀態列，使用者才看得到原因。
- 使用者取消對話框是 `null`，不是錯誤。
- 讀檔成功之前不要存檔（`canSave`），否則壞掉的檔案會被空資料蓋掉；不存檔的原因要留在狀態列，不要默默略過。
- `JSON.parse` 成功不代表形狀對，用 `Array.isArray` 確認是陣列，再用 `every` 與 `typeof` 確認每一則都有字串的 `title` 與 `body`。
- 刪除這種救不回來的操作，先用 `ask()` 確認。
