# 右鍵選單

## 本集目標

學會在使用者按右鍵時，先擋下 WebView 內建的右鍵選單，再呼叫 Rust 的 command，用 `popup_menu` 在滑鼠旁彈出自己做的選單。

## 正文

### 擋下預設的，彈出自己的

在頁面上按右鍵，瀏覽器會發出 `contextmenu` 事件，接著 WebView 彈出它內建的選單。要換成自己的，分兩步：

1. 前端在 `contextmenu` 事件裡呼叫 `event.preventDefault()`（第 2 章第 9 集），擋下內建的選單，再用 `invoke` 叫 Rust。
2. Rust 建一份 `Menu`，對視窗呼叫 `popup_menu`，選單就出現在滑鼠的位置。

使用者選了項目，事件會進到上一集的 `on_menu_event`。

### 範例：按右鍵新增筆記

這個 command 的參數是 `WebviewWindow`（第 3 章第 9 集），也就是被按右鍵的那個視窗。它同時是建選單的 `manager`，所以 `MenuItem::with_id`、`MenuBuilder::new` 都傳 `&window`。

`src-tauri/src/lib.rs`：

```rust
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::{Emitter, WebviewWindow};

#[tauri::command]
fn show_context_menu(window: WebviewWindow) -> tauri::Result<()> {
    let new_note = MenuItem::with_id(&window, "new_note", "新增筆記", true, None::<&str>)?;
    let menu = MenuBuilder::new(&window)
        .item(&new_note)
        .separator()
        .copy_with_text("拷貝")
        .paste_with_text("貼上")
        .build()?;
    window.popup_menu(&menu)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![show_context_menu])
        .on_menu_event(|app, event| {
            if event.id() == "new_note" {
                app.emit("new-note", ()).unwrap();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- 選單是每次按右鍵現做的，做完交給 `window.popup_menu(&menu)`。它回傳 `tauri::Result<()>`，可以直接當 command 的回傳值。
- 「新增筆記」是自訂項目，選了之後的處理跟上一集一樣：`emit` 通知前端。「拷貝」「貼上」是預設項目，不用處理。
- 這裡沒有用 `.menu(...)`，所以 macOS 還是用預設的應用程式選單。

前端：`new-note` 的部分跟上一集一樣，多了一個 `contextmenu` 的監聽。事件註冊在 `document` 上（第 2 章第 8 集），整個頁面按右鍵都算。

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>右鍵選單</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>右鍵選單</h1>
      <input id="note-input" placeholder="在頁面上按右鍵" />
      <ul id="note-list"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const listEl: HTMLElement | null = document.querySelector("#note-list");
if (!listEl) {
  throw new Error("找不到 #note-list");
}
let count = 0;

listen("new-note", () => {
  count += 1;
  const itemEl = document.createElement("li");
  itemEl.textContent = `筆記 ${count}`;
  listEl.append(itemEl);
});

document.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  invoke("show_context_menu");
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.11s
     Running `target/debug/tauri-app`
```

在頁面上按右鍵，滑鼠旁彈出「新增筆記」「拷貝」「貼上」。選「新增筆記」，清單多一列「筆記 1」。

前端只呼叫自己的 command，沒有碰任何 plugin 或核心的選單函式，所以 `capabilities/default.json` 不用動。

### 幾個細節

- 拿掉 `event.preventDefault()`，WebView 內建的右鍵選單照樣會出現，不是我們做的那份。
- `popup_menu` 在選單關掉之前不會返回，所以 `invoke` 回傳的 Promise 要等到使用者選完或點到別處才完成。範例沒有 `await`，不受影響。
- 選單事件只有 id，不會告訴你右鍵點在頁面的哪個地方。要針對某個元素，就由前端把它的資訊傳給 command，編進 id（例如 `delete-3`），在 `on_menu_event` 拆開來用。
- 這一集只在 macOS 實測。

## 重點整理

- 右鍵選單分兩步：前端的 `contextmenu` 事件裡 `preventDefault()` 並 `invoke`，Rust 的 command 建 `Menu` 後呼叫 `window.popup_menu(&menu)`。
- command 用 `WebviewWindow` 參數拿到被按右鍵的視窗，也拿它當 `manager` 建選單。
- 選了項目進同一個 `on_menu_event`，用 id 辨識；預設項目（拷貝、貼上）不用處理。
- `popup_menu` 等選單關閉才返回；選單事件不帶點擊位置，要的話把資訊編進 id。
