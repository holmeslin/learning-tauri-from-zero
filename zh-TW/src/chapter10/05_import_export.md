# 匯入、匯出（dialog）

## 本集目標

用 dialog plugin 讓使用者挑檔案，把一則筆記匯出成 `.md`，或從 `.md` 匯入成一則新筆記，成功時在狀態列說一聲。

## 正文

### 加上 dialog plugin

再裝一個 plugin：

```bash
npm run tauri add dialog
```

`lib.rs` 多註冊一個，`capabilities/default.json` 多一項 `dialog:default`：

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

### 匯出與匯入

匯出：`save()` 讓使用者決定檔名與位置，只回傳路徑，不寫檔案（第 7 章第 4 集），所以拿到路徑之後，自己用 `writeTextFile` 把內容寫進去。使用者按取消就是 `null`，什麼都不做。預設檔名用標題加 `.md`，內容只寫 `body`。

匯入：`open()` 挑檔案，拿到路徑就 `readTextFile`。標題用檔名去掉 `.md`。取檔名要用 `@tauri-apps/api/path` 的 `basename(路徑, ".md")`，它跟第 7 章第 6 集的 `join` 一樣是核心函式，會處理各平台的路徑分隔符號。

兩個函式都寫在 `notes.ts`。匯出回傳寫到哪裡，給畫面顯示；匯入回傳新的筆記，由 `main.ts` 決定放進清單：

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
  return JSON.parse(text);
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
    defaultPath: `${note.title}.md`,
    filters: markdown,
  });
  if (path === null) {
    return null;
  }
  await writeTextFile(path, note.body);
  return path;
}
```

這兩個函式碰的檔案是使用者自己挑的，不在 app 資料夾裡，卻沒有另外設定 scope。`writeTextFile` 這個函式已經被 `fs:allow-appdata-write-recursive` 放行了；至於路徑，經由 `open()`、`save()` 選出來的，plugin 會自動加進 fs 的 scope（第 7 章第 5 集）。`readTextFile` 的 permission 在 `fs:default` 裡。

### 接上按鈕與狀態列

畫面多兩個按鈕，和一行放狀態訊息的 `#status`：

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
body { display: flex; height: 100vh; margin: 0; font-family: sans-serif; }
.sidebar { width: 200px; padding: 8px; overflow-y: auto; background-color: #eee; }
main { flex: 1; display: flex; flex-direction: column; gap: 8px; padding: 8px; }
#note-list { padding: 0; list-style: none; }
#note-list li { padding: 6px 8px; cursor: pointer; }
#note-list .selected { background-color: #cde; }
#body-input { flex: 1; }
```

匯入的筆記跟「新增」一樣：放進陣列、選起來、存檔。`show` 把訊息寫進狀態列：

`src/main.ts`：

```ts
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

const edit = () => {
  current.title = titleEl.value;
  current.body = bodyEl.value;
  render();
  saveNotes(notes);
};

titleEl.addEventListener("input", edit);
bodyEl.addEventListener("input", edit);

document.querySelector("#new-btn")?.addEventListener("click", () => {
  const note: Note = { title: "新筆記", body: "" };
  notes.push(note);
  select(note);
  saveNotes(notes);
});

document.querySelector("#delete-btn")?.addEventListener("click", () => {
  notes = notes.filter((note) => note !== current);
  showFirst();
  saveNotes(notes);
});

document.querySelector("#import-btn")?.addEventListener("click", async () => {
  const note = await importNote();
  if (note) {
    notes.push(note);
    select(note);
    saveNotes(notes);
    show(`已匯入「${note.title}」`);
  }
});

document.querySelector("#export-btn")?.addEventListener("click", async () => {
  const path = await exportNote(current);
  if (path !== null) {
    show(`已匯出到 ${path}`);
  }
});

const start = async () => {
  notes = await loadNotes();
  showFirst();
};

start();
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-plugin-dialog v2.8.4
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.88s
     Running `target/debug/tauri-app`
```

先在標題打「購物清單」、內容打兩行字，按「匯出」。彈出系統的存檔對話框，預設檔名是 `購物清單.md`。選好位置按「Save」，狀態列顯示（macOS 上，`ada` 是範例的使用者名稱）：

```text
已匯出到 /Users/ada/Downloads/購物清單.md
```

用編輯器開啟匯出的檔案，裡面只有內容那兩行字，沒有標題。再按「匯入」，選剛剛存的檔案，清單最下面多了一則「購物清單」，內容跟檔案一樣。在對話框按「Cancel」，什麼都不會發生，這就是回傳 `null` 的情況。

## 重點整理

- `save()` 與 `open()` 都回傳路徑，取消是 `null`；`save()` 不寫檔案，要自己 `writeTextFile`。
- 使用者挑的檔案不在 app 資料夾，但 dialog 選出來的路徑會自動進 fs 的 scope；函式的 permission 還是要放行。
- `basename(路徑, ".md")` 取出去掉副檔名的檔名，各平台的路徑分隔符號都能處理。
- 匯入的筆記跟新增一樣：放進陣列、選起來、存檔；成功的結果用狀態列告訴使用者。
