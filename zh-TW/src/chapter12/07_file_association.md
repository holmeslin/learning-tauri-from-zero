# 檔案關聯（雙擊 `.md` 開啟）

## 本集目標

在 `tauri.conf.json` 宣告 App 能開 `.md` 檔，並在 Rust 端接住作業系統交來的檔案路徑，交給前端開啟；也說清楚在 dev 模式下怎麼測、雙擊要等打包之後。

## 正文

### 檔案關聯是什麼

作業系統記得「這種副檔名，用哪個 App 開」。App 要上這張名單，得在打包時把自己「能開哪些檔案」寫進安裝檔，設定在 `tauri.conf.json` 的 `bundle.fileAssociations`：

- `ext`：副檔名的陣列，這裡是 `md` 與 `markdown`。
- `name`：這種檔案的名稱。
- `description`：說明文字，只有 Windows 會顯示（檔案總管「類型」那一欄）。
- `role`：App 對這種檔案的角色，`Editor` 是會修改它，`Viewer` 是只看。只有 macOS 會用，寫進 `Info.plist`。
- `mimeType`：MIME 類型，Markdown 是 `text/markdown`。Linux 會把它寫進 `.desktop` 檔的 `MimeType=`，macOS 寫進 `Info.plist`，Windows 用不到。

同一份設定，各平台只讀自己用得到的欄位，其餘的會被忽略（`tauri.conf.json` 的 schema 在每個欄位的說明裡都標了適用的平台）。這份設定是打包工具讀的，所以 `tauri dev` 跑的執行檔不會被登記進系統，雙擊 `.md` 檔不會找到它。打包是第 15 章的事，所以這一集分兩半：設定與程式現在就寫好，dev 模式用命令列測；等第 15 章打包出安裝檔，才能真的用 Finder 或檔案總管雙擊。

### 檔案怎麼交到 App 手上

作業系統把檔案交給 App 的方式不只一種：

- **命令列參數**：Windows 與 Linux 雙擊 `.md` 檔，是用 `App 路徑 檔案路徑` 的方式啟動 App，檔案路徑在 `std::env::args()` 裡。這是我們查原始碼整理的：Windows 的 NSIS 與 MSI 安裝檔登記的指令是 `"App 路徑" "%1"`，`%1` 是被雙擊的檔案。Linux 靠安裝檔裡的 `.desktop` 檔，打包工具（Tauri CLI 2.12 內建的 tauri-bundler 2.10）產生的 `Exec=` 那一行只有執行檔的名稱，沒有 `%F`、`%U` 這種「把檔案交給程式」的欄位碼。遇到這種沒有欄位碼的 `.desktop`，GNOME 底層的 GLib 會自己把檔案接在後面，當成 `%f` 處理（它的原始碼註解寫著 KDE 也是這樣）。傳進來的是一般路徑，不是 `file://` 網址，所以下面的 `file_arg` 不用改。別的桌面環境的啟動器怎麼做，我們沒有查；Linux 這一段也沒有實測。如果已經有一個 App 在跑，又被啟動了第二個行程，就是第 8 章第 7 集的 single-instance：第二個行程把參數交給第一個，自己結束，參數會出現在 `init` 函式的 `args`。
- **`RunEvent::Opened`**：macOS 不靠命令列參數，而是向 App 送一個「請開啟這些檔案」的事件，Tauri 把它變成 `RunEvent::Opened { urls }`，`urls` 是 `file://` 開頭的網址陣列。

三個入口（啟動參數、第二個行程、`Opened`），要的都是同一件事：把路徑交給前端去開。

### 一個出口：`PendingFile` 加一個事件

最直覺的做法是 Rust 收到路徑就 `emit` 一個事件，前端 `listen`。但冷啟動（cold start，App 原本沒在跑，是被這個檔案叫起來的）的時候，路徑在 App 一啟動就到了，那時網頁還在載入，前端還沒開始 `listen`，事件送出去就沒人收，路徑也跟著弄丟。

所以路徑不放在事件裡，而是先存進 `State`，前端準備好了再用 command 去拿。事件只負責「按門鈴」：它是通知，不帶路徑。這跟第 11 章第 5 集的 `todos-changed` 不一樣，那個事件的 payload 是整份最新的清單，前端收到就能重畫。那邊可以這樣做，是因為清單隨時能重讀，漏掉一次事件也不會弄丟資料；這裡的路徑只送來這一次，漏了就沒了。整個做法是：

- `PendingFile` 放一個 `Option<PathBuf>`，是「等著被開啟的路徑」，用 `Mutex` 包住（第 4 章第 2 集）。
- `request_open(app, 路徑)` 把路徑存進去，再 `emit("open-requested", ())` 按門鈴。`emit` 失敗只印出來，不影響路徑已經存好了這件事（第 5 章第 1 集）。
- `take_pending_file` 是前端呼叫的 command：用 `take()` 拿走路徑並清空，沒有就是 `None`，到前端是 `null`。
- 前端啟動時先 `listen`，再呼叫 `take_pending_file` 一次，之後每次聽到 `open-requested`，也呼叫它。冷啟動的路徑是啟動時那一次拿到的；App 已在跑時的路徑，是門鈴響了才去拿的。順序是「先 `listen`，再拿」，兩個動作之間不會漏掉任何一個。

### 加上 single-instance

第二個行程這個入口，靠的是第 8 章第 7 集的 single-instance plugin。專案裡還沒有的話，先加進來。這個 plugin 只有 Rust 的 crate，沒有 npm 套件，也沒有 permission（前端不會呼叫它），所以只要 `cargo add`。它只有桌面能用，用 `--target` 放進 `Cargo.toml` 桌面專用的區塊：

```bash
cargo add --manifest-path src-tauri/Cargo.toml --target 'cfg(not(any(target_os = "android", target_os = "ios")))' tauri-plugin-single-instance@2
```

`cargo add` 的輸出（節錄）：

```text
    Updating crates.io index
      Adding tauri-plugin-single-instance v2 to dependencies for target `cfg(not(any(target_os = "android", target_os = "ios")))`
             Features as of v2.0.0:
             - deep-link
             - semver
```

確認一下：`src-tauri/Cargo.toml` 桌面專用的那個區塊（`[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]`）裡，多了一行 `tauri-plugin-single-instance = "2"`。註冊 plugin 的程式在下面的 `lib.rs`。

### Rust 端：三個入口

- 啟動參數：`run()` 一開始，從 `std::env::args()` 與目前的工作目錄算出路徑，放進 `PendingFile` 的初始值。`file_arg(args, cwd)` 跳過第一個參數（程式本身）和 `-` 開頭的選項，取第一個剩下的，接到 `cwd` 後面：絕對路徑接上去還是原本的路徑，相對路徑就變成從 `cwd` 算起的完整路徑。
- 第二個行程：single-instance 的函式收到的 `cwd`，是第二個行程的工作目錄，不是第一個行程的，所以相對路徑要用它來算。算出路徑就 `request_open`，再用第 8 章第 7 集的 `show_main_window` 把視窗叫到前面。plugin 要最先註冊，寫法跟那一集一樣（`let mut builder`、`#[cfg(desktop)]`）。
- macOS 的 `Opened`：之前都是 `.run(generate_context!())` 一行跑起來，要接 `RunEvent` 得拆成兩步。`Builder::run(context)` 其實就是 `build(context)` 之後再 `run(|_, _| {})`，現在改成 `.build(context)` 拿到 `App`，再呼叫 `App::run(函式)`，函式每收到一個 `RunEvent` 就被呼叫一次，參數是 `&AppHandle` 與 `RunEvent`。

`RunEvent::Opened` 只在 macOS、iOS 與 Android 才有（Tauri 2.12 的原始碼是這樣標的）。桌面上只有 macOS，所以這裡寫 `#[cfg(target_os = "macos")]`；iOS 與 Android 不在這個 App 的範圍。其他平台這個分支不存在，函式的兩個參數就沒人用，編譯器會警告「沒用到的變數」。所以另一邊用 `#[cfg(not(target_os = "macos"))]` 接一行 `let _ = (app, event);`，把它們用掉。

macOS 那一支用了 `if let ... && let ...`：兩個 `if let` 用 `&&` 接在同一個 `if` 裡，這是 Rust 2024 edition 的語法。巢狀的兩層 `if let`，clippy 會要求你寫成這樣。`Url` 的 `to_file_path()` 把 `file:///Users/ada/a.md` 轉成路徑，不是檔案網址就回傳 `Err`，`ok()` 把它變成 `None`，`find_map` 取第一個轉得成功的。這個編輯器一次只開一份文件，多選的檔案只取第一個。

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
```

`tauri.conf.json` 加上 `bundle.fileAssociations`，其餘照舊：

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
        "title": "tauri-app",
        "width": 800,
        "height": 600
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

### 前端：先聽，再拿

`commands.ts` 多兩個函式：`takePendingFile` 包住 command，`onOpenRequested` 包住 `listen`，收到事件就呼叫 `callback`，不看 payload。`listen` 呼叫函式時會傳進一個事件物件，`callback` 沒有宣告參數，那個物件就被忽略了，所以直接把 `callback` 交給 `listen` 就行：

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
  return listen("open-requested", callback);
}
```

`main.ts` 多一個 `openPending`：拿到路徑，過了第 6 集的 `confirmDiscard()`，才用 `openPath` 開啟。所以有沒存的內容時，從作業系統送來的檔案，也一樣會先問。`start` 照上面說的順序：先 `listen`，再拿一次，最後更新標題（沒有路徑被送來的時候，標題仍然要設成「未命名」）。

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
const UNTITLED = "未命名";

let path: string | null = null;
let fileName = UNTITLED;
let savedText = "";
let shownTitle = "";
let renderCount = 0;
let docId = 0;
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
  await attempt("更新視窗標題", async () => {
    await appWindow.setTitle(title);
    shownTitle = title;
  });
};

const refreshPreview = async () => {
  renderCount += 1;
  const mine = renderCount;
  await attempt("轉換", async () => {
    const html = await renderMarkdown(editorEl.value);
    if (mine === renderCount) {
      previewEl.innerHTML = html;
    }
  });
};

const load = async (newPath: string | null, text: string) => {
  const name = newPath === null ? UNTITLED : await basename(newPath);
  docId += 1;
  path = newPath;
  fileName = name;
  editorEl.value = text;
  savedText = editorEl.value;
  await refreshPreview();
  await refreshTitle();
};

const saveTo = async (target: string) => {
  const text = editorEl.value;
  const mine = docId;
  const name = await basename(target);
  await writeFile(target, text);
  if (mine !== docId) {
    return;
  }
  path = target;
  fileName = name;
  savedText = text;
  show(`已儲存 ${target}`);
  await refreshTitle();
};

const saveAs = async () => {
  const target = await save({
    defaultPath: path === null ? `${UNTITLED}.md` : path,
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
    await attempt("開啟連結", () => openUrl(href));
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

`index.html`、`markdown.rs`、`capabilities/default.json` 與 `src/styles.css` 都沒有改：

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

### 在 dev 模式測試

先把別的練習專案關掉：`identifier` 相同的 App 會被 single-instance 當成同一個（第 8 章第 7 集），別的專案還開著，這一集的第二個行程會把路徑送到那一個去。接著在專案根目錄準備兩個檔案：

```bash
printf '# 你好\n\n從命令列開啟的檔案。\n' > hello.md
printf '# 另一份\n\n第二個行程送來的檔案。\n' > other.md
```

測第一個入口：啟動時帶路徑。`npm run tauri dev` 後面的參數要分三段：第一個 `--` 給 npm，第二個 `--` 之後是給 Tauri CLI 轉交 `cargo run` 的，第三個 `--` 之後才是給 App 的。

```bash
npm run tauri dev -- -- -- "$PWD/hello.md"
```

`npm run tauri dev -- -- -- "$PWD/hello.md"` 的輸出（節錄）：

```text
     Running DevCommand (`cargo  run --no-default-features --color always -- ~/tauri-app/hello.md`)
        Info Watching ~/tauri-app/src-tauri for changes...
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.37s
     Running `target/debug/tauri-app ~/tauri-app/hello.md`
```

視窗出現的時候，標題已經是「hello.md — Markdown 編輯器」，左邊是檔案的內容，右邊是排好的大標題，狀態列顯示（`ada` 是範例的使用者名稱）：

```text
已開啟 /Users/ada/tauri-app/hello.md
```

測第二個入口：App 開著的時候，另開一個終端機，在專案根目錄直接執行同一個執行檔（第 8 章第 7 集的做法），帶上另一個檔案的相對路徑：

```bash
./src-tauri/target/debug/tauri-app other.md
```

這個指令一下子就回來了，沒有開出新視窗。原本的視窗換成「other.md — Markdown 編輯器」，內容是 `other.md` 的，狀態列顯示「已開啟 /Users/ada/tauri-app/other.md」。相對路徑 `other.md` 之所以找得到，是因為 Rust 用 single-instance 給的 `cwd` 把它接成了完整路徑。在檔名前面多放一個 `--flag` 也沒有影響，`file_arg` 會跳過 `-` 開頭的參數。

再試兩個情況。先在編輯區隨便打幾個字，再執行一次上面的指令：視窗跳出第 6 集的確認對話框，按「Cancel」，編輯區的內容不會被換掉。接著把檔名換成一個不存在的，像 `./src-tauri/target/debug/tauri-app nope.md`，編輯區不動，狀態列顯示：

```text
開啟失敗：/Users/ada/tauri-app/nope.md：No such file or directory (os error 2)
```

測不到的是第三個入口。dev 模式用命令列帶路徑啟動，把 `run` 收到的 `RunEvent` 印出來看，只有 `Ready` 和視窗取得、失去焦點，沒有 `Opened`。`Opened` 是系統要 App 開啟檔案的通知，這一集沒有實測，等第 15 章打包之後，再用 Finder 雙擊 `.md` 檔試。這一集也只在 macOS 實測，Windows 與 Linux 收檔案的方式，是依 tauri-bundler、GLib 的原始碼與 single-instance 的行為整理的。打包安裝之後，系統的「打開方式」就會出現這個 App，要不要設成預設，由使用者在系統裡決定，App 不能替他選。

## 重點整理

- `bundle.fileAssociations` 宣告 App 能開哪些副檔名，是打包工具讀的設定；dev 模式的執行檔沒被登記進系統，雙擊要等第 15 章打包之後。
- 檔案路徑有三個入口：啟動參數、第二個行程（single-instance 的 `args` 與 `cwd`）、macOS 的 `RunEvent::Opened`；接 `RunEvent` 要把 `.run(context)` 拆成 `.build(context)` 加 `App::run(函式)`。
- 路徑先存進 `State`，前端啟動時先 `listen` 再用 command 去拿；事件只負責通知，這樣冷啟動時事件早到，也不會弄丟路徑。
- 第二個行程的相對路徑要用它自己的 `cwd` 來算；`-` 開頭的參數要跳過；`RunEvent::Opened` 在桌面上只有 macOS 有（手機的 iOS、Android 也有），要用 `cfg` 隔開，其他桌面平台才不會有沒用到的變數。
- 在 dev 模式，用 `npm run tauri dev -- -- -- 路徑` 測啟動參數，直接再執行一次執行檔測第二個行程。
