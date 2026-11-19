# 即時預覽

## 本集目標

每次輸入就呼叫 `render_markdown`，把結果放進預覽欄；處理好非同步回應的順序，並攔下預覽裡連結的點擊，改交給系統瀏覽器開啟。

## 正文

### 每改一次就轉一次

上一集要按按鈕才轉換，這一集改成輸入框的內容每變一次就轉一次。輸入框每次改變都會發出 `input` 事件（第 6 章第 6 集用過），收到事件就呼叫 command，拿到 HTML 放進右邊的 `<article id="preview">`。

呼叫 command 的 `invoke` 先包進 `commands.ts`，做法跟第 11 章的 `todos.ts` 一樣，之後各集會一直往這個檔案加函式：

`src/commands.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

export function renderMarkdown(text: string): Promise<string> {
  return invoke<string>("render_markdown", { text: text });
}
```

### 預覽欄：`innerHTML`

把 HTML 字串變成畫面上真的元素，要用 `innerHTML`。第 2 章第 7 集說過本書不用它，因為字串裡混進別人寫的內容有安全風險。預覽是唯一不得不用的地方，而風險已經在上一集堵住了：Markdown 裡夾的原始 HTML，在 Rust 端就被轉成跳脫過的文字，`innerHTML` 收到的只剩 pulldown-cmark 自己產生的元素。

### 回應的順序

`invoke` 是非同步的（第 3 章第 2 集）。連續打字會連續呼叫好幾次，但沒有任何規定說回應一定照呼叫的順序回來。如果比較舊的那一次後到，它就會把新的結果蓋掉，預覽停在舊的內容。

解法是替每次呼叫編號：`renderCount` 每呼叫一次加一，呼叫的當下把自己的號碼記在 `mine`。回應回來時，只有號碼還是最新的才放進畫面，舊的直接丟掉。出錯時，錯誤顯示在狀態列。

`src/main.ts`：

```ts
import { renderMarkdown } from "./commands";

const editorEl: HTMLTextAreaElement | null = document.querySelector("#editor");
const previewEl: HTMLElement | null = document.querySelector("#preview");
const statusEl: HTMLElement | null = document.querySelector("#status");
if (!editorEl || !previewEl || !statusEl) {
  throw new Error("找不到必要的元素");
}

let renderCount = 0;

const refreshPreview = async () => {
  renderCount += 1;
  const mine = renderCount;
  try {
    const html = await renderMarkdown(editorEl.value);
    if (mine === renderCount) {
      previewEl.innerHTML = html;
      statusEl.textContent = "";
    }
  } catch (error) {
    statusEl.textContent = `轉換失敗：${error}`;
  }
};

editorEl.addEventListener("input", refreshPreview);
```

畫面只剩輸入框和預覽欄，所以 `index.html` 拿掉上一集的按鈕：

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
    <main>
      <textarea id="editor" placeholder="在這裡寫 Markdown" spellcheck="false"></textarea>
      <article id="preview"></article>
    </main>
    <p id="status"></p>
  </body>
</html>
```

`lib.rs`、`markdown.rs` 與 `src/styles.css` 都沒有改，為了讓這一集也能獨立開始，一併貼在下面：

`src-tauri/src/lib.rs`：

```rust
mod markdown;

#[tauri::command]
fn render_markdown(text: &str) -> String {
    markdown::to_html(text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![render_markdown])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running `target/debug/tauri-app`
```

在左邊打 `# 預覽`、空一行，再打 `這是[範例連結](https://example.com)，這是**粗體**。`，右邊跟著打字同步出現大標題「預覽」，後面那一行也立刻排成有連結、有粗體的段落。每打一個字，右邊就更新一次。

### 點連結，編輯器就不見了

試著點預覽裡的「範例連結」。整個視窗換成了 `example.com` 的頁面，編輯器不見了。預覽裡的 `<a>` 是真的連結，WebView 照瀏覽器的規矩，直接前往那個網址。

要攔下這個點擊。事件冒泡（第 2 章第 8 集）的特性在這裡派上用場：點擊會從被點到的元素一路往外傳，所以不用替每個連結各掛一個監聽，只要在 `#preview` 這個外層元素上聽一次，每次預覽重新產生元素都不用重掛。

- `event.target` 是被點到的那個元素，可能是 `<a>`，也可能是 `<a>` 裡面的 `<strong>`。它的型別是 `EventTarget | null`，沒有 `closest` 方法，所以先用 `instanceof Element` 確認，TypeScript 才會放行。
- `closest("a")` 從這個元素開始往外找最近的 `<a>`，找不到回傳 `null`。
- 是連結就先 `event.preventDefault()`，擋下瀏覽器預設的前往動作。
- 只有網址以 `http://` 或 `https://` 開頭，才交給 opener plugin 的 `openUrl`（第 7 章第 3 集），用系統預設的瀏覽器開啟，`opener:default` 範本本來就放行了。其他的，像 `javascript:` 開頭的網址、相對路徑，一律不處理。

`src/main.ts`：

```ts
import { openUrl } from "@tauri-apps/plugin-opener";
import { renderMarkdown } from "./commands";

const editorEl: HTMLTextAreaElement | null = document.querySelector("#editor");
const previewEl: HTMLElement | null = document.querySelector("#preview");
const statusEl: HTMLElement | null = document.querySelector("#status");
if (!editorEl || !previewEl || !statusEl) {
  throw new Error("找不到必要的元素");
}

let renderCount = 0;

const refreshPreview = async () => {
  renderCount += 1;
  const mine = renderCount;
  try {
    const html = await renderMarkdown(editorEl.value);
    if (mine === renderCount) {
      previewEl.innerHTML = html;
      statusEl.textContent = "";
    }
  } catch (error) {
    statusEl.textContent = `轉換失敗：${error}`;
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
      statusEl.textContent = `開啟連結失敗：${error}`;
    }
  }
});
```

存檔後 Vite 自動重新整理頁面（第 1 章第 5 集）。再試一次：打字、點連結，視窗還是編輯器，沒有被換掉，狀態列也沒有出現錯誤；網址由 `openUrl` 交給系統預設的瀏覽器。

## 重點整理

- 輸入框的 `input` 事件每次改變就發出，收到就呼叫 `render_markdown`，結果用 `innerHTML` 放進預覽欄；原始 HTML 已在 Rust 端擋掉，這是本書唯一用 `innerHTML` 的地方。
- 連續呼叫 command 時，回應不保證照順序回來；替每次呼叫編號，只有最新的那次才更新畫面。
- 預覽裡的連結不攔的話，WebView 會直接前往那個網址，把編輯器換掉。
- 在外層的 `#preview` 聽 `click`，用 `closest("a")` 找到連結，`preventDefault()` 之後，只有 `http://`、`https://` 才交給 `openUrl`。
