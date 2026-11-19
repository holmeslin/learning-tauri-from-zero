# Rust 端轉 HTML（pulldown-cmark）

## 本集目標

加上 pulldown-cmark，在 Rust 端把 Markdown 轉成 HTML，用 command 讓前端呼叫；並擋掉 Markdown 裡夾帶的原始 HTML，為下一集把結果放進畫面做準備。

## 正文

### 加上 pulldown-cmark

pulldown-cmark 是 Rust 的 Markdown 解析函式庫，照 CommonMark 規格解析。它是「拉式」（pull）解析器：你把文字交給 `Parser`，`Parser` 是一個迭代器，每次 `next()` 吐出一個 `Event`，像 `Event::Start`（一個標題或段落開始）、`Event::Text`（一段文字）、`Event::End`（結束）。再把整串事件交給 `html::push_html`，它就把事件轉成 HTML 字串。《從零開始學 Rust》沒教過它，範本也沒有，先加進專案：

```bash
cargo add --manifest-path src-tauri/Cargo.toml pulldown-cmark@0.13
```

`cargo add` 的輸出（節錄）：

```text
      Adding pulldown-cmark v0.13 to dependencies
             Features as of v0.13.0:
             + getopts
             + html
             + pulldown-cmark-escape
             - gen-tests
             - serde
             - simd
```

預設的 feature 就夠用：`html` 提供 `html::push_html`，`getopts` 是它附的命令列工具用的，這個專案用不到，也不用特地關掉。

### 第一版：`to_html`

轉換的函式放進新檔案 `markdown.rs`，不碰任何 Tauri 的型別。CommonMark 本身沒有表格、刪除線與待辦勾選框，這些是 GitHub 那一派的擴充，要用 `Options` 一項一項打開，用 `|` 合在一起：

`src-tauri/src/markdown.rs`：

```rust
use pulldown_cmark::{Options, Parser, html};

pub fn to_html(text: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(text, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}
```

`Parser::new_ext(文字, 選項)` 建出事件串流，`html::push_html(&mut out, parser)` 把 HTML 接在 `out` 後面。`lib.rs` 加上 `mod markdown;`（第 3 章第 10 集），再寫一個 command，函式直接交給 `markdown::to_html`。範本的 `greet` 用不到，一併拿掉：

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

這一集的畫面先做最簡單的：左邊輸入，按下按鈕，右邊用 `<pre>` 把 Rust 傳回來的 HTML 原文印出來。`<pre>` 保留空白與換行，適合看程式碼。用 `textContent` 放進去，字串裡的 `<` 不會被當成 HTML：

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
      <button id="convert-btn" type="button">轉成 HTML</button>
    </header>
    <main>
      <textarea id="editor" placeholder="在這裡寫 Markdown" spellcheck="false"></textarea>
      <pre id="preview"></pre>
    </main>
    <p id="status"></p>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const editorEl: HTMLTextAreaElement | null = document.querySelector("#editor");
const previewEl: HTMLElement | null = document.querySelector("#preview");
const statusEl: HTMLElement | null = document.querySelector("#status");
if (!editorEl || !previewEl || !statusEl) {
  throw new Error("找不到必要的元素");
}

document.querySelector("#convert-btn")?.addEventListener("click", async () => {
  try {
    previewEl.textContent = await invoke<string>("render_markdown", {
      text: editorEl.value,
    });
    statusEl.textContent = "";
  } catch (error) {
    statusEl.textContent = `轉換失敗：${error}`;
  }
});
```

`src/styles.css` 跟第 1 集一樣：

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
   Compiling unicode-width v0.2.3
   Compiling pulldown-cmark v0.13.6
   Compiling pulldown-cmark-escape v0.11.0
   Compiling unicase v2.10.0
   Compiling getopts v0.2.24
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.97s
     Running `target/debug/tauri-app`
```

在左邊打 `# 標題`、空一行，再打 `**粗體**與[連結](https://tauri.app)`，按「轉成 HTML」，右邊是：

```text
<h1>標題</h1>
<p><strong>粗體</strong>與<a href="https://tauri.app">連結</a></p>
```

Markdown 的每個語法都變成對應的 HTML 元素：`#` 是 `<h1>`，空行隔開的是 `<p>`，`**` 是 `<strong>`，`[文字](網址)` 是 `<a>`。

### 檔案裡夾的 HTML 會原封不動

Markdown 允許直接夾 HTML。把左邊換成兩行：一行普通的文字加 `<b>` 元素，空一行，再一行 `<img>`。這個 `<img>` 的圖片網址是壞的，壞掉的時候會觸發 `onerror` 屬性裡的 JavaScript：

```text
文字<b>粗</b>

<img src=x onerror=alert(1)>
```

按「轉成 HTML」，右邊是：

```text
<p>文字<b>粗</b></p>
<img src=x onerror=alert(1)>
```

兩段 HTML 都原封不動地出現在輸出裡。這一集用 `textContent` 顯示，所以只是看到文字。但下一集要把輸出放進頁面當真的 HTML 來顯示，到時候 `<img>` 一載入失敗，`onerror` 裡的 JavaScript 就會跑起來。它跑在我們的視窗裡，跟我們自己寫的程式碼有一樣的權限，能呼叫這個 App 所有的 command，包括第 4 集會寫的、能寫入任何路徑的 `write_file`。也就是說，從網路上載下來的一個 `.md` 檔，打開就可能在你的電腦上寫檔案。

第 2 章第 7 集提過，`innerHTML` 混進使用者輸入的內容有安全風險，所以本書不用。預覽是唯一的例外：它的工作就是把 HTML 放進畫面。所以風險要在 Rust 端堵住：把原始 HTML 的事件改成 `Event::Text`，`push_html` 會把文字裡的 `<`、`>` 跳脫成 `&lt;`、`&gt;`，瀏覽器就只會把它當成文字顯示。

原始 HTML 對應兩種事件：獨立成一個區塊的是 `Event::Html`，夾在一行文字中間的是 `Event::InlineHtml`。`Parser` 是迭代器，用 `.map` 換掉它們就好，其他事件原樣放行：

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
```

`Event::Html(raw) | Event::InlineHtml(raw)` 是一個 `|` 合起來的 pattern，兩種事件都把裡面的字串綁到 `raw`。`Event::Text` 與 `Event::Html` 裡裝的都是 `CowStr`，同一個型別，所以可以直接換。

存檔後 `tauri dev` 自動重新編譯：

`npm run tauri dev` 的輸出（改了 `markdown.rs` 之後）：

```text
        Info File src-tauri/src/markdown.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.60s
     Running `target/debug/tauri-app`
```

視窗重開，把剛才那兩行再打一遍，按「轉成 HTML」，右邊是：

```text
<p>文字&lt;b&gt;粗&lt;/b&gt;</p>
&lt;img src=x onerror=alert(1)&gt;
```

`<b>` 與 `<img>` 都變成了跳脫過的文字。這個行為以後不能默默壞掉，所以替它寫測試。

### 測試 `to_html`

`to_html` 只收字串、回傳字串，不碰 Tauri，直接用 `cargo test` 就能測（第 9 章第 1 集）。測試放在 `markdown.rs` 最後的 `tests` 模組，三個測試各守一件事。預期的字串都是手寫的固定值，不是用別的程式算出來的：

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

- `converts_headings_and_emphasis`：標題、粗體、斜體各轉成對應的元素，整串比對。輸出的最後有一個換行。
- `raw_html_is_shown_as_text`：獨立一行的 `<img>`、夾在文字中間的 `<script>`，輸出裡都只剩跳脫過的文字。`assert!` 的第二個參數是失敗時要印的訊息，這裡把實際的輸出印出來，失敗時一眼就看得出是什麼。
- `supports_tables_and_task_lists`：打開的 `Options` 真的有效，表格轉成 `<table>`，待辦項目轉成勾選框。

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

`cargo test --manifest-path src-tauri/Cargo.toml` 的輸出（節錄）：

```text
running 3 tests
test markdown::tests::converts_headings_and_emphasis ... ok
test markdown::tests::raw_html_is_shown_as_text ... ok
test markdown::tests::supports_tables_and_task_lists ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 重點整理

- pulldown-cmark 的 `Parser` 是事件的迭代器，`html::push_html` 把事件轉成 HTML；表格、刪除線、待辦勾選框要用 `Options` 打開。用 `cargo add --manifest-path src-tauri/Cargo.toml pulldown-cmark@0.13` 加進專案。
- 轉換函式 `to_html` 放在 `markdown.rs`，不碰 Tauri 的型別；`lib.rs` 的 command 只是把文字交給它。
- Markdown 裡夾的 HTML 預設會原封不動出現在輸出裡；放進頁面就等於讓檔案的作者在你的視窗裡執行 JavaScript，所以要把 `Event::Html` 與 `Event::InlineHtml` 換成 `Event::Text`，讓它被跳脫。
- 這件事要用測試守住：輸出裡不能出現 `<img`、`<script`。
