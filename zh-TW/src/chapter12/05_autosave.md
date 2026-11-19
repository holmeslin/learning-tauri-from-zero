# 自動儲存

## 本集目標

勾選「自動儲存」時，有路徑的文件在停手一秒後自動存檔；用 `savedText` 判斷有沒有沒存的修改，並在視窗標題用「●」標出來。

## 正文

### 什麼時候存

每打一個字就存一次太頻繁，要等使用者停下來。做法是每次輸入都重新倒數一秒：一秒內又打了字，就取消前一次的倒數、重新開始；一秒都沒有新的輸入，才真的去存。

倒數用 `setTimeout(函式, 毫秒)`（第 2 章第 10 集用過），它會回傳一個計時器的編號，交給 `clearTimeout` 就能取消。編號的型別寫成 `ReturnType<typeof setTimeout>`，意思是「`setTimeout` 回傳什麼型別，就是什麼型別」，這樣不管環境把它定義成數字還是別的型別都適用。還沒設過計時器的時候是 `undefined`，`clearTimeout(undefined)` 什麼都不會做，所以不用另外判斷。

### 有沒有沒存：`savedText`

第 1 集說好了：有沒有沒存的修改，是比較編輯區現在的文字與 `savedText`。這一集把它做出來：

- `isDirty()` 回傳 `editorEl.value !== savedText`。
- `load` 載入新內容時，`savedText` 設成**編輯區的值**，不是剛讀進來的 `text`。`<textarea>` 會把換行統一成 `\n`，如果檔案是 Windows 的 `\r\n`，拿讀進來的原文當基準，一打開就會被判成「有修改」。
- `saveTo` 寫檔前先把編輯區的文字抄到 `text`，寫成功之後 `savedText` 設成這個 `text`，不是設成當下的編輯區。`writeFile` 是非同步的，等待的時候使用者可能又打了字，那些字還沒寫進檔案，不能算成已儲存。

### 標題的「●」

`refreshTitle` 現在會看 `isDirty()`：沒存的時候標題前面加「● 」。它在每次輸入都會被呼叫，所以先把要顯示的標題組好，跟上一次成功設定的 `shownTitle` 一樣就直接結束，不重複呼叫 `setTitle`。`shownTitle` 要等 `setTitle` 成功之後才更新，失敗了，下次還會再試。

### 自動儲存的函式

`scheduleAutosave` 每次輸入都被呼叫：先 `clearTimeout` 取消舊的倒數，再設一個新的。一秒後，三個條件都成立才存：

- 「自動儲存」勾選框是勾著的；
- `path` 不是 `null`：沒有路徑的文件不自動存（第 1 集的規則）；
- `isDirty()` 為真：中間如果已經手動存過，或是打了字又刪回原樣，就沒必要寫。

函式裡先把 `path` 抄進 `target`，再把 `target` 交給 `saveTo`。`path` 是會被改動的變數，TypeScript 不會讓「剛剛檢查過不是 `null`」這件事穿過另一個函式（`() => saveTo(target)`）；抄成 `const` 就沒問題，存的也確實是倒數結束那一刻的檔案。

自動儲存失敗時，`attempt("自動儲存", ...)` 把原因顯示在狀態列，文件仍然是「沒存」，標題上的「●」還在。下次再打字會重試；如果使用者就此停手，不會再重試，但狀態列的訊息和「●」會一直提醒他。

`index.html` 把「自動儲存」的勾選框放回來。其餘的檔案要不要改，下面一併貼出來：

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
const autosaveEl: HTMLInputElement | null = document.querySelector("#autosave");
if (!editorEl || !previewEl || !statusEl || !autosaveEl) {
  throw new Error("找不到必要的元素");
}

const appWindow = getCurrentWindow();
const markdownFilter = [{ name: "Markdown", extensions: ["md", "markdown"] }];

let path: string | null = null;
let fileName = "未命名";
let savedText = "";
let shownTitle = "";
let renderCount = 0;
let autosaveTimer: ReturnType<typeof setTimeout> | undefined;

const show = (message: string) => {
  statusEl.textContent = message;
};

const isDirty = () => editorEl.value !== savedText;

const attempt = async (action: string, work: () => Promise<void>) => {
  try {
    await work();
  } catch (error) {
    show(`${action}失敗：${error}`);
  }
};

const refreshTitle = async () => {
  const title = `${isDirty() ? "● " : ""}${fileName} — Markdown 編輯器`;
  if (title === shownTitle) {
    return;
  }
  try {
    await appWindow.setTitle(title);
    shownTitle = title;
  } catch (error) {
    show(`更新視窗標題失敗：${error}`);
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

const load = async (newPath: string | null, text: string) => {
  const name = newPath === null ? "未命名" : await basename(newPath);
  path = newPath;
  fileName = name;
  editorEl.value = text;
  savedText = editorEl.value;
  await refreshPreview();
  await refreshTitle();
};

const saveTo = async (target: string) => {
  const text = editorEl.value;
  const name = await basename(target);
  await writeFile(target, text);
  path = target;
  fileName = name;
  savedText = text;
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

const scheduleAutosave = () => {
  clearTimeout(autosaveTimer);
  autosaveTimer = setTimeout(() => {
    const target = path;
    if (autosaveEl.checked && target !== null && isDirty()) {
      attempt("自動儲存", () => saveTo(target));
    }
  }, 1000);
};

editorEl.addEventListener("input", () => {
  refreshPreview();
  refreshTitle();
  scheduleAutosave();
});

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
      <label><input id="autosave" type="checkbox" checked /> 自動儲存</label>
    </header>
    <main>
      <textarea id="editor" placeholder="在這裡寫 Markdown" spellcheck="false"></textarea>
      <article id="preview"></article>
    </main>
    <p id="status"></p>
  </body>
</html>
```

`lib.rs`、`capabilities/default.json`、`commands.ts`、`markdown.rs` 與 `src/styles.css` 都沒有改：

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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/tauri-app`
```

Rust 端沒有改，所以不用重新編譯。先在還沒存過的文件裡打字：標題變成「● 未命名 — Markdown 編輯器」，等了幾秒也沒有變，因為沒有路徑，不會自動存。

按「另存新檔」，存成 `未命名.md`，標題變成「未命名.md — Markdown 編輯器」。再打一行字，標題先出現「●」，停手大約一秒後「●」不見了，狀態列顯示：

```text
已儲存 /Users/ada/Downloads/未命名.md
```

到終端機看，檔案裡已經有剛打的那一行。手動儲存與自動儲存都走 `saveTo`，所以兩邊顯示的訊息一樣。

把「自動儲存」取消勾選，再打字：標題的「●」一直在，檔案的內容也沒有變。按「儲存」，「●」才消失，檔案才有新的內容。

## 重點整理

- 「停手一秒才存」用 `setTimeout` 加 `clearTimeout`：每次輸入先取消舊的倒數，再重新倒數。
- 有沒有沒存的修改，是 `editorEl.value !== savedText`。`savedText` 在載入時取編輯區的值（`textarea` 會統一換行），在存檔時取寫檔前抄下來的文字，不是寫完之後的編輯區。
- 沒有 `path` 的文件不自動存；自動存失敗時，訊息與標題的「●」會留著提醒使用者。
- `refreshTitle` 只在標題真的變了才呼叫 `setTitle`，成功之後才記下 `shownTitle`。
