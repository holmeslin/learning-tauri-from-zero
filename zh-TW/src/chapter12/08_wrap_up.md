# 收尾

## 本集目標

替編輯器換上自己的視窗標題與大小，替 `lib.rs` 裡不碰 Tauri 的函式寫測試，清掉範本留下的檔案，最後回頭看這一章做了什麼、還能怎麼改。

## 正文

### 視窗設定

視窗標題與大小在 `app.windows`（第 6 章第 1 集）。標題在程式啟動之後，會用 `setTitle` 換成「檔名 — Markdown 編輯器」，設定檔裡的只是網頁還沒載入時看到的樣子，所以只寫「Markdown 編輯器」。左右兩欄各佔一半，寬一點才寫得舒服，大小改成 1000×700，再加上最小尺寸，視窗再小，兩欄就擠得不能用了：

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
        "title": "Markdown 編輯器",
        "width": 1000,
        "height": 700,
        "minWidth": 600,
        "minHeight": 400
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "fileAssociations": [
      {
        "ext": ["md", "markdown"],
        "name": "Markdown",
        "description": "Markdown 文件",
        "role": "Editor",
        "mimeType": "text/markdown"
      }
    ],
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

`identifier` 不要動：single-instance 靠它判斷「已經有一個在跑」（第 8 章第 7 集）。

### 測試 `lib.rs` 裡不碰 Tauri 的函式

`markdown.rs` 的測試在第 2 集就有了。`lib.rs` 裡有三個東西不需要 Tauri 就能測：`file_arg` 是純函式，`read_file` 與 `write_file` 雖然掛了 `#[tauri::command]`，參數卻只有普通的值，沒有 `State` 或 `AppHandle`，所以不用像第 9 章第 1 集那樣再抽出來，直接呼叫就能測。測試放在 `lib.rs` 最後的 `tests` 模組，四個測試各守一件事：

- `file_arg_skips_program_and_flags`：跳過程式本身和 `-` 開頭的選項，相對路徑接在 `cwd` 後面；只有程式本身時沒有路徑。
- `file_arg_keeps_absolute_paths`：絕對路徑接上 `cwd` 之後還是原本的路徑。測試用 `std::env::temp_dir()` 組出一個絕對路徑，Windows 與 macOS 都能跑。
- `write_then_read_round_trips`：寫進暫存資料夾的檔案，讀回來一模一樣，測完刪掉。
- `read_error_mentions_the_path`：讀不存在的檔案，錯誤訊息要以路徑開頭，這是第 4 集 `map_err` 的目的。

`src-tauri/src/lib.rs`：

```rust
mod markdown;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

struct PendingFile(Mutex<Option<PathBuf>>);

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

#[tauri::command]
fn take_pending_file(state: State<PendingFile>) -> Option<String> {
    let path = state.0.lock().unwrap().take()?;
    Some(path.to_string_lossy().into_owned())
}

fn file_arg(args: &[String], cwd: &Path) -> Option<PathBuf> {
    let arg = args.iter().skip(1).find(|arg| !arg.starts_with('-'))?;
    Some(cwd.join(arg))
}

fn request_open(app: &AppHandle, path: PathBuf) {
    *app.state::<PendingFile>().0.lock().unwrap() = Some(path);
    if let Err(error) = app.emit("open-requested", ()) {
        eprintln!("送出 open-requested 失敗：{error}");
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir().unwrap_or_default();
    let pending = PendingFile(Mutex::new(file_arg(&args, &cwd)));

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Some(path) = file_arg(&args, Path::new(&cwd)) {
                request_open(app, path);
            }
            show_main_window(app);
        }));
    }

    builder
        .manage(pending)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            render_markdown,
            read_file,
            write_file,
            take_pending_file
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = event
                && let Some(path) = urls.iter().find_map(|url| url.to_file_path().ok())
            {
                request_open(app, path);
                show_main_window(app);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn file_arg_skips_program_and_flags() {
        let cwd = Path::new("/work");
        assert_eq!(
            file_arg(&args(&["app", "--flag", "note.md"]), cwd),
            Some(cwd.join("note.md"))
        );
        assert_eq!(file_arg(&args(&["app"]), cwd), None);
    }

    #[test]
    fn file_arg_keeps_absolute_paths() {
        let abs = std::env::temp_dir().join("note.md");
        let given = abs.to_string_lossy().into_owned();
        assert_eq!(
            file_arg(&args(&["app", &given]), Path::new("/work")),
            Some(abs)
        );
    }

    #[test]
    fn write_then_read_round_trips() {
        let path = std::env::temp_dir().join("tauri-md-editor-test.md");
        let path = path.to_string_lossy().into_owned();
        write_file(&path, "# 你好\n").unwrap();
        assert_eq!(read_file(&path).unwrap(), "# 你好\n");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn read_error_mentions_the_path() {
        let missing = std::env::temp_dir().join("tauri-md-editor-missing.md");
        let missing = missing.to_string_lossy().into_owned();
        assert!(read_file(&missing).unwrap_err().starts_with(&missing));
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

`cargo test --manifest-path src-tauri/Cargo.toml` 的輸出（節錄）：

```text
running 7 tests
test tests::file_arg_keeps_absolute_paths ... ok
test tests::file_arg_skips_program_and_flags ... ok
test tests::read_error_mentions_the_path ... ok
test markdown::tests::converts_headings_and_emphasis ... ok
test markdown::tests::raw_html_is_shown_as_text ... ok
test markdown::tests::supports_tables_and_task_lists ... ok
test tests::write_then_read_round_trips ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

七個測試都過了：`markdown.rs` 的三個加上 `lib.rs` 的四個。測試是同時跑的，所以每次印出來的順序可能不一樣。這些測試沒有碰 `request_open`、single-instance 與 `RunEvent`：它們需要真的 `AppHandle`，跟第 11 章一樣，這一章不做。

### 其餘的檔案

其餘的檔案跟第 7 集一模一樣，為了讓這一集也能獨立開始，完整貼在下面：

`src/main.ts`：

```ts
import { basename } from "@tauri-apps/api/path";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  onOpenRequested,
  readFile,
  renderMarkdown,
  takePendingFile,
  writeFile,
} from "./commands";

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

const openPending = async () => {
  const target = await takePendingFile();
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

const start = async () => {
  await onOpenRequested(() => attempt("開啟", openPending));
  await attempt("開啟", openPending);
  await refreshTitle();
};

start();
```

`src/commands.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { UnlistenFn, listen } from "@tauri-apps/api/event";

export function renderMarkdown(text: string): Promise<string> {
  return invoke<string>("render_markdown", { text: text });
}

export function readFile(path: string): Promise<string> {
  return invoke<string>("read_file", { path: path });
}

export function writeFile(path: string, text: string): Promise<void> {
  return invoke<void>("write_file", { path: path, text: text });
}

export function takePendingFile(): Promise<string | null> {
  return invoke<string | null>("take_pending_file");
}

export function onOpenRequested(callback: () => void): Promise<UnlistenFn> {
  return listen("open-requested", () => callback());
}
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
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.46s
     Running `target/debug/tauri-app`
```

視窗出現的時候，標題是「未命名 — Markdown 編輯器」，大小是 1000×700。試著把視窗縮小，縮到 600×400 就縮不下去了。

### 清掉範本留下的檔案

範本的 `src/assets/` 放著三個 logo，這個專案沒用到。刪掉它們（下面的 `rm -r` 是 macOS、Linux 的指令；Windows 直接在檔案總管刪掉 `src/assets` 資料夾），再用 `npm run build` 確認沒有任何地方還需要它：

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
✓ 15 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                  0.91 kB │ gzip: 0.50 kB
dist/assets/index-CY40rICE.css   0.62 kB │ gzip: 0.33 kB
dist/assets/index-0PkZjrjg.js   20.16 kB │ gzip: 5.89 kB

✓ built in 146ms
```

`tsc` 型別檢查通過，Vite 也打包成功，表示前端沒有型別錯誤，也沒有找不到的檔案。

### 回頭看

編輯器的每一塊，主要靠這些東西做成：

- 轉換：pulldown-cmark 的 `Parser` 與 `push_html`，原始 HTML 在 Rust 端換成文字，預覽才敢用 `innerHTML`（第 2、3 集）。
- 預覽：`input` 事件加 command，替每次呼叫編號，舊的回應丟掉；連結交給 `openUrl`（第 3 集）。
- 檔案：dialog 只問路徑，讀寫是自己的兩個 command，前端不碰 fs plugin（第 4 集）。
- 狀態：`path` 與 `savedText`，有沒存的修改是比較出來的，不是旗標（第 1、5 集）。
- 自動儲存：`setTimeout` 加 `clearTimeout`，停手一秒才存，沒有路徑的文件不存（第 5 集）。
- 提醒：同一個 `confirmDiscard()`，用在關視窗、新增與開啟（第 6 集）。
- 檔案關聯：`bundle.fileAssociations`，三個入口一個出口，路徑放 `State`、事件只按門鈴（第 7 集）。

這個 App 刻意用了只有桌面才有的東西（single-instance、檔案關聯），所以沒有設計成能搬到手機上。

### 還能怎麼改

- 關視窗時的提醒只有「OK」與「Cancel」。想要「儲存、不儲存、取消」三個按鈕，要查 dialog plugin 的 `message` 對話框有哪些按鈕選項，這一章沒有教。
- macOS 的 Cmd+Q 擋不住（第 6 集）。要擋，得在 Rust 端用 `RunEvent` 處理。
- 沒有 Cmd+S、Cmd+O 這類快捷鍵。可以用第 2 章第 8 集的 `keydown`，或是用第 8 章第 1 集的選單加上快捷鍵。
- `write_file` 直接蓋掉原檔，寫到一半當機，檔案就壞了。比較穩的做法是先寫到旁邊的暫存檔，寫完再改名蓋過去。
- 兩個檔案 command 會讀寫前端給的任何路徑，只在自己電腦上用沒關係，要發佈給別人，就該限制範圍，第 14 章會正式介紹。
- `<textarea>` 會把換行統一成 `\n`，所以 Windows 的 `\r\n` 檔案存回去之後，換行會變成 `\n`。
- 「自動儲存」的勾選狀態關掉 App 就忘了。要記住，可以用第 7 章第 7 集的 store。
- 做成安裝檔給別人用，第 15 章會正式介紹。

## 重點整理

- 視窗標題與大小寫在 `app.windows`，`minWidth`、`minHeight` 限制視窗最小能縮到多小。
- 參數只有普通值的 command（沒有 `State`、`AppHandle`），掛了 `#[tauri::command]` 之後仍然可以直接呼叫、直接測；要用到 `AppHandle` 的部分這一章沒有測。
- 測試檔案讀寫，用 `std::env::temp_dir()` 組路徑，測完刪掉，Windows 與 macOS 都能跑。
- 刪掉範本沒用到的檔案之後跑 `npm run build`，確認沒有東西還需要它。
