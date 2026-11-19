# 未儲存就關閉的提醒

## 本集目標

有沒存的內容時，在關視窗、按「新增」、開啟另一個檔案之前先問一聲；並知道這個提醒擋不住 macOS 的 Cmd+Q。

## 正文

### 同一個提醒，用在三個地方

會丟掉編輯區內容的動作有三個：關視窗、按「新增」、開啟別的檔案。三個地方問的是同一件事，所以寫成一個函式 `confirmDiscard()`：沒有沒存的修改，直接回傳 `true`（放行）；有的話，用 dialog plugin 的 `confirm` 彈出原生對話框（第 7 章第 4 集），使用者按「OK」回傳 `true`，按「Cancel」回傳 `false`。呼叫的人只要看它回傳什麼。

- 「新增」：`await confirmDiscard()` 通過，才清掉編輯區。
- 「開啟」：先選檔案，再問。如果使用者在選檔案的對話框就取消了，什麼都沒丟，不需要被問。選好了檔案，才 `confirmDiscard()`，通過才讀檔、載入。讀檔的部分抽成 `openPath(路徑)`，下一集還會用到它。

### 關視窗

關視窗用第 6 章第 6 集的 `onCloseRequested`。函式可以是 `async`（第 7 章第 4 集），Tauri 會等它跑完：`confirmDiscard()` 回傳 `false` 就 `event.preventDefault()` 擋下關閉，回傳 `true` 就什麼都不做，視窗照常關閉。

有了 `onCloseRequested`，關視窗的最後一步是 `@tauri-apps/api` 替你呼叫 `destroy()`，所以 `capabilities/default.json` 要放行 `core:window:allow-destroy`。第 6 章第 6 集說過，少了它，使用者按了「OK」，視窗卻關不掉。

`src/main.ts`：

```ts
import { basename } from "@tauri-apps/api/path";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
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

const confirmDiscard = async () => {
  if (!isDirty()) {
    return true;
  }
  return await confirm("目前的內容還沒儲存，確定要放棄嗎？", {
    title: "尚未儲存",
    kind: "warning",
  });
};

const openPath = async (target: string) => {
  const text = await readFile(target);
  await load(target, text);
  show(`已開啟 ${target}`);
};

const openWithDialog = async () => {
  const target = await open({ filters: markdownFilter });
  if (target !== null && (await confirmDiscard())) {
    await openPath(target);
  }
};

const startNew = async () => {
  if (await confirmDiscard()) {
    await load(null, "");
    show("");
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
  attempt("新增", startNew);
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

appWindow.onCloseRequested(async (event) => {
  if (!(await confirmDiscard())) {
    event.preventDefault();
  }
});

refreshTitle();
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
    "core:window:allow-set-title",
    "core:window:allow-destroy"
  ]
}
```

`index.html`、`commands.ts`、`lib.rs`、`markdown.rs` 與 `src/styles.css` 都沒有改：

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

`src-tauri/src/markdown.rs`：

```rust
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

const SAFE_SCHEMES: [&str; 3] = ["http", "https", "mailto"];

fn is_safe_url(url: &str) -> bool {
    match url.find([':', '/', '?', '#']) {
        Some(end) if url.as_bytes()[end] == b':' => {
            SAFE_SCHEMES.contains(&url[..end].to_ascii_lowercase().as_str())
        }
        _ => true,
    }
}

fn safe_url(url: CowStr) -> CowStr {
    if is_safe_url(&url) {
        url
    } else {
        CowStr::Borrowed("")
    }
}

pub fn to_html(text: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(text, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: safe_url(dest_url),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: safe_url(dest_url),
            title,
            id,
        }),
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

    #[test]
    fn dangerous_urls_are_removed() {
        assert_eq!(
            to_html("[x](javascript:alert(1))"),
            "<p><a href=\"\">x</a></p>\n"
        );
        for text in [
            "[x](JavaScript:alert(1))",
            "[x](&#106;avascript:alert(1))",
            "[x](<  javascript:alert(1)>)",
            "<javascript:alert(1)>",
            "[x][r]\n\n[r]: javascript:alert(1)",
            "![x](javascript:alert(1))",
            "[x](data:text/html;base64,AAAA)",
            "![x](data:image/svg+xml;base64,AAAA)",
        ] {
            let html = to_html(text).to_lowercase();
            assert!(!html.contains("=\"javascript:"), "{text} -> {html}");
            assert!(!html.contains("=\"data:"), "{text} -> {html}");
        }
    }

    #[test]
    fn safe_urls_are_kept() {
        assert_eq!(
            to_html("[x](HTTPS://example.com/a?b=c#d)"),
            "<p><a href=\"HTTPS://example.com/a?b=c#d\">x</a></p>\n"
        );
        assert_eq!(
            to_html("[x](notes/a.md)"),
            "<p><a href=\"notes/a.md\">x</a></p>\n"
        );
        assert_eq!(
            to_html("![圖](https://example.com/a.png)"),
            "<p><img src=\"https://example.com/a.png\" alt=\"圖\" /></p>\n"
        );
        assert!(to_html("<me@example.com>").contains("href=\"mailto:me@example.com\""));
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

`npm run tauri dev` 的輸出（改了 `capabilities/default.json` 之後）：

```text
        Info File src-tauri/capabilities/default.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.45s
     Running `target/debug/tauri-app`
```

在還沒存過的文件裡打 `# 草稿`，標題出現「●」。按「新增」，彈出一個對話框，標題是「尚未儲存」，內容是「目前的內容還沒儲存，確定要放棄嗎？」，下面兩個按鈕「Cancel」與「OK」（macOS 英文介面上的按鈕文字，依作業系統與語言而不同）。按「Cancel」，編輯區的 `# 草稿` 還在，標題的「●」也還在；再按一次「新增」、這次按「OK」，編輯區清空，標題變成「未命名 — Markdown 編輯器」。

再打一點字，按視窗的關閉鈕，一樣彈出對話框，視窗還開著。按「Cancel」，視窗留著，內容沒有少；再按一次關閉鈕、按「OK」，視窗關掉，`npm run tauri dev` 也跟著結束，因為這是最後一個視窗。如果沒有沒存的修改，`confirmDiscard()` 直接放行，不會有對話框。

### Cmd+Q 不經過 `onCloseRequested`

在 macOS 上，文件有沒存的修改時按 Cmd+Q，沒有任何對話框，App 直接結束。把 Rust 端 `.run` 收到的 `RunEvent` 印出來看（第 7 集會介紹這個寫法），按下 Cmd+Q 之後，只收到最後的 `RunEvent::Exit`，視窗沒有收到「要求關閉」，所以 `onCloseRequested` 不會被呼叫。這一集做的提醒，只管得到視窗的關閉鈕這一條路。

`Exit` 是 App 已經決定要結束之後的通知，這時已經太晚，攔不住了。要讓 Cmd+Q 也先問，得換掉預設選單的「結束」項目（第 8 章第 1 集），這一章不處理。實際上的風險有限：有路徑的文件，停手一秒就自動存了，真正會丟的是還沒存過的文件。

## 重點整理

- 會丟掉編輯區內容的動作，都先過同一個 `confirmDiscard()`：沒有沒存的就放行，有的話用 dialog 的 `confirm` 問。
- 關視窗用 `onCloseRequested`，回傳 `false` 就 `event.preventDefault()`；放行時由 API 呼叫 `destroy()`，要放行 `core:window:allow-destroy`。
- 開檔案要先選檔、再問：使用者在選檔對話框取消，不該被問。
- macOS 的 Cmd+Q 直接結束 App，視窗不會收到「要求關閉」，`onCloseRequested` 不會被呼叫，這個提醒擋不住它。
