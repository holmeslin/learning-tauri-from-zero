# 收尾

## 本集目標

替筆記 App 換上自己的視窗標題，並限制視窗最小能縮到多小；清掉範本留下的檔案，確認能打包前端，最後回頭看做了什麼、還能怎麼改。

## 正文

### 視窗設定

視窗標題現在還是 `tauri-app`。第 6 章第 1 集說過，標題與大小寫在 `app.windows`。預設的 800×600 剛好夠用，所以大小不動；筆記 App 的版面是左右兩欄，視窗再小就擠了，所以加上最小尺寸。`identifier` 不要動：它決定 app 資料夾的名字（第 7 章第 6 集），改了就找不到已經存好的筆記。

`src-tauri/tauri.conf.json`：

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "tauri-app",
  "version": "0.1.0",
  "identifier": "tw.example.tauri-app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "筆記",
        "width": 800,
        "height": 600,
        "minWidth": 480,
        "minHeight": 320
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

其餘檔案跟上一集一模一樣。為了讓這一集也能獨立開始，完整貼在下面：

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
body { display: flex; height: 100vh; margin: 0; font-family: sans-serif; }
.sidebar { width: 200px; padding: 8px; overflow-y: auto; background-color: #eee; }
main { flex: 1; display: flex; flex-direction: column; gap: 8px; padding: 8px; }
#note-list { padding: 0; list-style: none; }
#note-list li { padding: 6px 8px; cursor: pointer; }
#note-list .selected { background-color: #cde; }
#body-input { flex: 1; }
```

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
  if (!Array.isArray(data)) {
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
    return;
  }
  try {
    await saveNotes(notes);
    show("已儲存");
  } catch (error) {
    show(`儲存失敗：${error}`);
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
  const yes = await ask(`要刪除「${current.title}」嗎？`, { kind: "warning" });
  if (yes) {
    notes = notes.filter((note) => note !== current);
    showFirst();
    persist();
  }
});

document.querySelector("#import-btn")?.addEventListener("click", async () => {
  try {
    const note = await importNote();
    if (note) {
      notes.push(note);
      select(note);
      persist();
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
    show(`讀取失敗，這次不會存檔：${error}`);
  }
  showFirst();
};

start();
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.75s
     Running `target/debug/tauri-app`
```

視窗標題變成「筆記」，拖拉邊緣縮小時，縮到 480×320 就停住。

### 清掉範本留下的檔案

範本的 `src/assets/` 放著三個 logo（Vite、Tauri、TypeScript），這個專案從頭到尾沒用到。刪掉它們，再用 `npm run build` 確認沒有任何地方還需要它：

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
✓ 11 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                 0.90 kB │ gzip: 0.45 kB
dist/assets/index-21Z9uvcG.css  0.34 kB │ gzip: 0.23 kB
dist/assets/index-BXqH5t-R.js   5.35 kB │ gzip: 2.23 kB

✓ built in 176ms
```

`npm run build` 先用 `tsc` 做型別檢查，再用 Vite 打包進 `dist/`（第 2 章第 11 集）。通過了，表示前端沒有留下型別錯誤，也沒有找不到的檔案。

### 回頭看

筆記 App 的每一塊，都是前面學過的東西接起來的：

- 畫面：HTML、CSS 的 flex 與 DOM（第 2 章）。
- 存檔：fs plugin、`BaseDirectory.AppData` 與 permission（第 7 章）。
- 匯入、匯出：dialog plugin，以及「dialog 選出的路徑自動進 scope」（第 7 章）。
- 出錯：`try`/`catch` 加狀態列，以及讀檔失敗就停止存檔。

整個專案沒有寫任何一個自己的 command。

### 還能怎麼改

- 每打一個字就寫一次整個檔案。連續輸入時，上一次的寫入還沒結束，下一次就開始了，兩次寫入可能互相重疊。比較穩的做法是停手一小段時間才存。
- 筆記一多，整份讀進來、整份寫回去就太慢了，也沒辦法搜尋。第 11 章會正式介紹用 SQLite 存資料。
- `notes.ts` 可以像第 9 章第 3 集那樣，用 mock 假裝 plugin 的回答來測試。
- 匯出的 `.md` 只是文字。第 12 章會正式介紹怎麼預覽 Markdown。
- `fs:allow-appdata-write-recursive` 對這個 App 來說放得比較寬，更細的 permission 與 scope 設定，第 14 章會正式介紹。
- 做成安裝檔給別人用，第 15 章會正式介紹。

## 重點整理

- 視窗的標題與最小尺寸寫在 `tauri.conf.json` 的 `app.windows`；`identifier` 決定資料夾，不要隨便改。
- 把範本沒用到的檔案刪掉，再跑 `npm run build`，確認沒有東西還需要它。
- 整個筆記 App 只用了前面學過的 DOM、fs 與 dialog，沒有自己的 command。
- 做完先想想哪裡還能更穩：連續存檔、檔案變大、更窄的 permission。
