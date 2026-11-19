# 開檔與存檔

## 本集目標

用 dialog plugin 讓使用者選檔案，讀、寫檔案交給兩個自己寫的 command；開啟或存檔之後，視窗標題顯示檔名。

## 正文

### 兩個 command：讀檔、寫檔

第 1 集說過，檔案讀寫放在 Rust 端。兩個 command 都是標準函式庫的一行：`std::fs::read_to_string` 讀出整個檔案成字串，`std::fs::write` 把字串寫進去，檔案不存在就建立，存在就整個蓋掉。

它們回傳 `Result<_, String>`（第 3 章第 6 集）。`std::io::Error` 的訊息只說「No such file or directory」，不說是哪個檔案，所以用 `map_err` 在前面接上路徑再回傳，前端顯示在狀態列時，使用者才知道是誰出了問題。`read_to_string` 只讀得了合法的 UTF-8，讀到別種編碼或二進位檔，會得到一個錯誤，不會讀出亂碼。

另外用 `npm run tauri add dialog` 裝 dialog plugin（第 7 章第 4 集），它會註冊 plugin，並在 `capabilities/default.json` 加上 `dialog:default`。這個專案要用 `setTitle` 改視窗標題（第 6 章第 2 集），所以自己再加一項 `core:window:allow-set-title`：

```bash
npm run tauri add dialog
```

`src-tauri/src/lib.rs`：

```rust
mod markdown;

#[tauri::command]
fn render_markdown(text: &str) -> String {
    markdown::to_html(text)
}

#[tauri::command]
fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{path}：{error}"))
}

#[tauri::command]
fn write_file(path: &str, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|error| format!("{path}：{error}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            render_markdown,
            read_file,
            write_file
        ])
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
    "dialog:default",
    "core:window:allow-set-title"
  ]
}
```

### 前端：選路徑，再交給 command

`commands.ts` 多兩個函式，一個包住 `read_file`，一個包住 `write_file`：

`src/commands.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

export function renderMarkdown(text: string): Promise<string> {
  return invoke<string>("render_markdown", { text: text });
}

export function readFile(path: string): Promise<string> {
  return invoke<string>("read_file", { path: path });
}

export function writeFile(path: string, text: string): Promise<void> {
  return invoke<void>("write_file", { path: path, text: text });
}
```

`main.ts` 長大了，一塊一塊來看：

- `path` 是目前文件的路徑，沒存過是 `null`；`fileName` 是標題要顯示的檔名。
- `load(新的路徑, 文字)` 把一份文件放進編輯區：換掉 `path`、`fileName` 與文字，再更新預覽與標題。程式設定 `editorEl.value` 不會發出 `input` 事件（那是使用者打字才有的），所以預覽要自己呼叫 `refreshPreview()` 更新。
- `saveTo(路徑)` 把編輯區的文字寫進路徑；`saveAs()` 用 `save()` 對話框問路徑（第 7 章第 4 集），取消會回傳 `null`，就什麼都不做；`saveCurrent()` 在有 `path` 時直接寫，沒有就改走 `saveAs()`。
- `openWithDialog()` 用 `open()` 選一個檔案，讀進來再 `load`。兩個對話框都用 `filters` 限制在 `.md`、`.markdown`。
- 標題用第 10 章第 5 集的 `basename(路徑)` 取檔名，沒有路徑就叫「未命名」。

這裡有一個順序要小心：`saveTo` 先算好檔名，再寫檔。寫檔成功之後，就不該再有可能丟出錯誤的步驟，否則已經寫進去的檔案，畫面上卻會顯示「儲存失敗」。所以 `await basename(...)` 放在 `writeFile` 前面，寫完只剩改變數、顯示訊息，以及自己會處理錯誤的 `refreshTitle`。`load` 也一樣，先算 `name`，才動 `path` 與編輯區。

四個按鈕的動作都用 `attempt` 包起來：`attempt(動作名稱, 函式)` 呼叫函式，出錯就把「動作失敗：原因」顯示在狀態列，第 11 章第 4 集的 `attempt` 就是這個做法。「新增」現在會直接清掉編輯區，不問；第 6 集再補上確認。

`src/main.ts`：

```ts
import { basename } from "@tauri-apps/api/path";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { readFile, renderMarkdown, writeFile } from "./commands";

const editorEl: HTMLTextAreaElement | null = document.querySelector("#editor");
const previewEl: HTMLElement | null = document.querySelector("#preview");
const statusEl: HTMLElement | null = document.querySelector("#status");
if (!editorEl || !previewEl || !statusEl) {
  throw new Error("找不到必要的元素");
}

const appWindow = getCurrentWindow();
const markdownFilter = [{ name: "Markdown", extensions: ["md", "markdown"] }];

let path: string | null = null;
let fileName = "未命名";
let renderCount = 0;

const show = (message: string) => {
  statusEl.textContent = message;
};

const attempt = async (action: string, work: () => Promise<void>) => {
  try {
    await work();
  } catch (error) {
    show(`${action}失敗：${error}`);
  }
};

const refreshPreview = async () => {
  renderCount += 1;
  const mine = renderCount;
  try {
    const html = await renderMarkdown(editorEl.value);
    if (mine === renderCount) {
      previewEl.innerHTML = html;
    }
  } catch (error) {
    show(`轉換失敗：${error}`);
  }
};

const refreshTitle = async () => {
  try {
    await appWindow.setTitle(`${fileName} — Markdown 編輯器`);
  } catch (error) {
    show(`更新視窗標題失敗：${error}`);
  }
};

const load = async (newPath: string | null, text: string) => {
  const name = newPath === null ? "未命名" : await basename(newPath);
  path = newPath;
  fileName = name;
  editorEl.value = text;
  await refreshPreview();
  await refreshTitle();
};

const saveTo = async (target: string) => {
  const name = await basename(target);
  await writeFile(target, editorEl.value);
  path = target;
  fileName = name;
  show(`已儲存 ${target}`);
  await refreshTitle();
};

const saveAs = async () => {
  const target = await save({
    defaultPath: path === null ? "未命名.md" : path,
    filters: markdownFilter,
  });
  if (target !== null) {
    await saveTo(target);
  }
};

const saveCurrent = async () => {
  if (path === null) {
    await saveAs();
  } else {
    await saveTo(path);
  }
};

const openWithDialog = async () => {
  const target = await open({ filters: markdownFilter });
  if (target !== null) {
    const text = await readFile(target);
    await load(target, text);
    show(`已開啟 ${target}`);
  }
};

editorEl.addEventListener("input", refreshPreview);

previewEl.addEventListener("click", async (event) => {
  const link =
    event.target instanceof Element ? event.target.closest("a") : null;
  if (link === null) {
    return;
  }
  event.preventDefault();
  const href = link.getAttribute("href");
  if (href === null) {
    return;
  }
  if (href.startsWith("http://") || href.startsWith("https://")) {
    try {
      await openUrl(href);
    } catch (error) {
      show(`開啟連結失敗：${error}`);
    }
  }
});

document.querySelector("#new-btn")?.addEventListener("click", () => {
  attempt("新增", async () => {
    await load(null, "");
    show("");
  });
});
document.querySelector("#open-btn")?.addEventListener("click", () => {
  attempt("開啟", openWithDialog);
});
document.querySelector("#save-btn")?.addEventListener("click", () => {
  attempt("儲存", saveCurrent);
});
document.querySelector("#save-as-btn")?.addEventListener("click", () => {
  attempt("另存新檔", saveAs);
});

refreshTitle();
```

`index.html` 把第 1 集的四個按鈕放回來，自動儲存的勾選框要到第 5 集才有：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Markdown 編輯器</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <header>
      <button id="new-btn" type="button">新增</button>
      <button id="open-btn" type="button">開啟</button>
      <button id="save-btn" type="button">儲存</button>
      <button id="save-as-btn" type="button">另存新檔</button>
    </header>
    <main>
      <textarea id="editor" placeholder="在這裡寫 Markdown" spellcheck="false"></textarea>
      <article id="preview"></article>
    </main>
    <p id="status"></p>
  </body>
</html>
```

`markdown.rs` 與 `src/styles.css` 沒有改，一併貼在下面：

`src-tauri/src/markdown.rs`：

```rust
use pulldown_cmark::{Event, Options, Parser, html};

pub fn to_html(text: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(text, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_headings_and_emphasis() {
        assert_eq!(
            to_html("# 標題\n\n**粗體**與*斜體*"),
            "<h1>標題</h1>\n<p><strong>粗體</strong>與<em>斜體</em></p>\n"
        );
    }

    #[test]
    fn raw_html_is_shown_as_text() {
        let block = to_html("<img src=x onerror=alert(1)>");
        assert!(block.contains("&lt;img"), "{block}");
        assert!(!block.contains("<img"), "{block}");
        let inline = to_html("文字<script>alert(1)</script>");
        assert!(inline.contains("&lt;script&gt;"), "{inline}");
        assert!(!inline.contains("<script"), "{inline}");
    }

    #[test]
    fn supports_tables_and_task_lists() {
        let table = to_html("| a | b |\n|---|---|\n| 1 | 2 |");
        assert!(table.contains("<table>"), "{table}");
        let tasks = to_html("- [x] 完成\n- [ ] 未完成");
        assert!(tasks.contains("type=\"checkbox\""), "{tasks}");
    }
}
```

`src/styles.css`：

```css
body {
  display: flex;
  flex-direction: column;
  height: 100vh;
  margin: 0;
  font-family: sans-serif;
}

header,
#status {
  padding: 8px;
  background-color: #eee;
}

header {
  display: flex;
  align-items: center;
  gap: 8px;
}

main {
  flex: 1;
  display: flex;
  min-height: 0;
}

#editor,
#preview {
  flex: 1;
  min-width: 0;
  box-sizing: border-box;
  margin: 0;
  padding: 12px;
  overflow: auto;
}

#editor {
  border: none;
  border-right: 1px solid #ddd;
  resize: none;
  font-family: monospace;
  font-size: 14px;
}

#preview pre {
  padding: 8px;
  overflow: auto;
  background-color: #f4f4f4;
}

#preview table {
  border-collapse: collapse;
}

#preview th,
#preview td {
  padding: 4px 8px;
  border: 1px solid #ccc;
}

#status {
  margin: 0;
  min-height: 1.5em;
  font-size: 12px;
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-plugin-dialog v2.8.4
   Compiling rfd v0.17.2
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.23s
     Running `target/debug/tauri-app`
```

視窗標題是「未命名 — Markdown 編輯器」。左邊打幾行 Markdown，按「另存新檔」，彈出系統的存檔對話框，檔名欄預設是 `未命名.md`（這是 `save()` 的 `defaultPath`），資料夾預設是 Downloads（這是 macOS 的預設，其他平台不一樣）。按「Save」（macOS 英文介面上的按鈕文字，依作業系統與語言而不同），狀態列顯示（`ada` 是範例的使用者名稱）：

```text
已儲存 /Users/ada/Downloads/未命名.md
```

視窗標題變成「未命名.md — Markdown 編輯器」。再多打一行，按「儲存」，這次沒有對話框，狀態列再顯示一次「已儲存」，到終端機看，檔案的內容也跟著多了那一行。

按「開啟」，彈出系統的選檔對話框，選一個 `.md` 檔，左邊出現檔案的內容，右邊是排好的樣子，標題換成檔名，狀態列顯示：

```text
已開啟 /Users/ada/Downloads/未命名.md
```

在對話框按取消，什麼都不會發生，這就是回傳 `null` 的情況。

## 重點整理

- 檔案讀寫是兩個 Rust command，用 `std::fs::read_to_string` 與 `std::fs::write`；錯誤訊息用 `map_err` 接上路徑，前端才知道是哪個檔案出問題。
- dialog plugin 只負責問路徑：`open()`、`save()` 回傳路徑，取消是 `null`；改視窗標題要放行 `core:window:allow-set-title`。
- 用程式改 `value` 不會發出 `input` 事件，載入新內容之後要自己重畫預覽。
- 寫檔成功之後，不要再留有可能丟出錯誤的步驟，免得已成功的寫入顯示成失敗；檔名等會失敗的東西先算好。
