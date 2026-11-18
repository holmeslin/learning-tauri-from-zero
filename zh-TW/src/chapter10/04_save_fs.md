# 存到 app 資料目錄（fs）

## 本集目標

用 fs plugin 把筆記存成 app 資料夾裡的 `notes.json`，啟動時讀回來，關掉再開筆記還在。

## 正文

### 裝 fs plugin

照第 7 章第 1 集的方式：

```bash
npm run tauri add fs
```

`lib.rs` 多一行註冊，`capabilities/default.json` 多一項 `fs:default`。它只放行讀取與建立 app 資料夾，存檔要寫入，所以再加上第 7 章第 5 集挑的 `fs:allow-appdata-write-recursive`：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
    "fs:allow-appdata-write-recursive"
  ]
}
```

### 讀檔與寫檔

讀寫筆記的函式放進 `notes.ts`，不碰畫面：

`src/notes.ts`：

```ts
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
```

- `exists` 是 fs plugin 的另一個函式，回傳檔案在不在。第一次執行時 `notes.json` 還不存在，這時回傳空陣列，不是錯誤。`fs:default` 已經放行它。
- `JSON.parse(文字)` 把 JSON 文字轉回 JavaScript 的值，`JSON.stringify(值, null, 2)` 反過來，把值轉成 JSON 文字；後面兩個參數照抄，意思是用 2 個空格縮排，用編輯器開啟檔案時才好讀。`JSON.parse` 不檢查形狀，回傳的型別是 `any`，所以直接當 `Note[]` 回傳也能通過型別檢查。檔案內容對不對，第 6 集再檢查。
- 寫檔前的 `mkdir` 跟第 7 章第 5 集一樣不能省：app 資料夾第一次寫入時還不存在。

### 載入與存檔

`index.html` 與 `src/styles.css` 沒變：

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
      </div>
      <ul id="note-list"></ul>
    </div>
    <main>
      <input id="title-input" placeholder="標題" />
      <textarea id="body-input" placeholder="內容"></textarea>
      <div>
        <button id="delete-btn" type="button">刪除</button>
      </div>
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

`main.ts` 的改動有兩處。一是假資料不要了：`notes` 一開始是空的，`start` 讀完檔案才換成真的。讀檔完成之前，`current` 先放一則空白的，讀完再換掉。二是每次資料變了，就呼叫 `saveNotes(notes)`：打字、新增、刪除都一樣。呼叫 `saveNotes` 沒有 `await`，因為畫面不用等它寫完，跟範本呼叫 `greet()` 同一個寫法。

`src/main.ts`：

```ts
import { Note, loadNotes, saveNotes } from "./notes";

const listEl: HTMLUListElement | null = document.querySelector("#note-list");
const titleEl: HTMLInputElement | null = document.querySelector("#title-input");
const bodyEl: HTMLTextAreaElement | null = document.querySelector("#body-input");
if (!listEl || !titleEl || !bodyEl) {
  throw new Error("找不到必要的元素");
}

let notes: Note[] = [];
let current: Note = { title: "", body: "" };

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

const start = async () => {
  notes = await loadNotes();
  showFirst();
};

start();
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-plugin-fs v2.7.1
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.17s
     Running `target/debug/tauri-app`
```

第一次執行，清單裡只有一則空白的「新筆記」。把標題改成「購物清單」，內容打「牛奶」，然後關掉視窗，再執行一次 `npm run tauri dev`：清單裡還是「購物清單」。到終端機看檔案（macOS 的路徑，Windows 與 Linux 的位置用 `appDataDir()` 查，第 7 章第 6 集）：

```bash
cat "$HOME/Library/Application Support/tw.example.tauri-app/notes.json"
```

```text
[
  {
    "title": "購物清單",
    "body": "牛奶"
  }
]
```

### 先記著的限制

- 每打一個字就寫一次整個檔案。筆記少的時候感覺不到，收尾那集會再談怎麼改。
- 沒有任何錯誤處理。檔案讀不到、寫不進去、內容壞掉時，現在的程式不會告訴使用者，只會在 Console 留下錯誤，第 6 集處理。

## 重點整理

- 裝 fs plugin 並放行 `fs:default` 與 `fs:allow-appdata-write-recursive`，筆記才讀得到、寫得進 app 資料夾。
- 讀寫檔案的函式放在 `notes.ts`：`loadNotes` 先用 `exists` 判斷檔案在不在，不在就回傳空陣列；`saveNotes` 先 `mkdir`，再把 `JSON.stringify` 的結果寫進 `notes.json`。
- `JSON.parse` 把文字轉回值，但不檢查形狀。
- `main.ts` 啟動時 `await loadNotes()`，資料每次變動就呼叫 `saveNotes(notes)`。
