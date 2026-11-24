# 處理選單事件

## 本集目標

學會用 `Builder::on_menu_event` 接住使用者選的項目，用 `event.id()` 分辨是哪一項，並知道處理的方式有兩種：在 Rust 直接做，或用 `emit` 通知前端。

## 正文

### 事件只帶一個 id

使用者選了一個自訂項目，Tauri 就發出一個選單事件。事件裡只有項目的 id（`MenuId`），所以上一集 `MenuItem::with_id` 的 id 要取得好認，而且不能重複。

處理函式用 `tauri::Builder::on_menu_event` 註冊，它收 `&AppHandle` 和事件。`event.id().as_ref()` 把 id 轉成 `&str`，就能拿去 `match`。只處理一個 id 的時候，也可以寫 `if event.id() == "new-note"`。

這個處理函式是全域的：不管選的是 App 的選單、視窗的選單，還是後面會學的右鍵選單與系統匣選單，都會進到同一個地方。預設項目（拷貝、貼上這類）的動作由作業系統或 Tauri 完成，不用在這裡寫。

### 範例：新增筆記與結束

兩個自訂項目，示範兩種處理方式。id 是 `new-note` 的交給前端：Rust 用 `emit`（第 5 章第 1 集）送出 `new-note` 事件，頁面收到就加一則筆記。事件剛好跟 id 同名，兩者是不同的東西。`quit` 在 Rust 直接做完：`app.exit(0)` 結束整個 App，`0` 是代表正常結束的結束代碼。上一集說過 `quit` 預設項目在 Linux 不支援，自訂的項目每個平台都能用，所以這裡自己做結束。

`src-tauri/src/lib.rs`：

```rust
use tauri::Emitter;
use tauri::menu::{MenuBuilder, MenuItem, SubmenuBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .menu(|app| {
            let quit = MenuItem::with_id(app, "quit", "結束", true, Some("CmdOrCtrl+Q"))?;
            let app_menu = SubmenuBuilder::new(app, "tauri-app").item(&quit).build()?;

            let new_note =
                MenuItem::with_id(app, "new-note", "新增筆記", true, Some("CmdOrCtrl+N"))?;
            let file_menu = SubmenuBuilder::new(app, "檔案").item(&new_note).build()?;

            MenuBuilder::new(app)
                .items(&[&app_menu, &file_menu])
                .build()
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "new-note" => {
                app.emit("new-note", ()).unwrap();
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`app.emit("new-note", ())` 沒有資料要帶，payload 就放 `()`。`_ => {}` 接住其他所有 id，什麼都不做。

前端只需要一個清單，加筆記的邏輯跟第 2 章第 7 集一樣：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>處理選單事件</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>處理選單事件</h1>
      <ul id="note-list"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
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
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.41s
     Running `target/debug/tauri-app`
```

選「檔案 > 新增筆記」，清單多一列「筆記 1」；按 Cmd+N（其他平台是 Ctrl+N），也多一列。快捷鍵跟用滑鼠選是同一個事件。選 App 選單裡的「結束」，或按 Cmd+Q，視窗關閉，`npm run tauri dev` 也跟著結束。

### id 對不上會怎樣

`match` 的 id 是字串，寫錯了（例如把 `"new-note"` 寫成 `"newnote"`）編譯器不會報錯，選了就是沒有反應，因為落進 `_ => {}`。選單沒反應的時候，先檢查兩邊的 id 是不是同一個字。

## 重點整理

- 選單事件只帶一個 id；用 `tauri::Builder::on_menu_event(|app, event| ...)` 註冊處理函式，`event.id().as_ref()` 拿去 `match`。
- 處理函式是全域的，所有選單（App、視窗、右鍵、系統匣）的事件都進同一個地方，所以 id 不要重複。
- 處理有兩種：在 Rust 直接做（像 `app.exit(0)`），或用 `emit` 通知前端。
- 預設項目不用處理；id 寫錯不會有編譯錯誤，只是沒反應。
