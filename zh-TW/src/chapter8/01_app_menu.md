# 應用程式選單

## 本集目標

學會用 `tauri::Builder::menu` 替 App 做一份選單，認識選單的三層結構，並知道 macOS 與 Windows、Linux 的差別，以及 macOS 上少了「編輯」選單會發生什麼事。

## 正文

### 選單的三層

選單由三層組成，都在 `tauri::menu` 裡：

- `Menu`：整份選單，裡面放子選單（macOS 的 `Menu` 只能放子選單）。
- `Submenu`：「檔案」「編輯」這種點開會展開的標題，裡面放項目。
- 項目：有自訂的 `MenuItem`（有 id、文字、能不能點、快捷鍵）、分隔線，以及「拷貝」「貼上」「結束」這類預設項目，它們的動作由作業系統或 Tauri 完成。

### 範例：一份三個子選單的選單

建選單的函式都要 `&AppHandle`（第 3 章第 9 集），所以選單寫在 `Builder::menu` 收的函式裡，函式的參數就是它。

`src-tauri/src/lib.rs`：

```rust
use tauri::menu::{MenuBuilder, MenuItem, SubmenuBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .menu(|app| {
            let app_menu = SubmenuBuilder::new(app, "tauri-app")
                .quit_with_text("結束 tauri-app")
                .build()?;

            let new_note =
                MenuItem::with_id(app, "new_note", "新增筆記", true, Some("CmdOrCtrl+N"))?;
            let open_note = MenuItem::with_id(app, "open_note", "開啟筆記", false, None::<&str>)?;
            let file_menu = SubmenuBuilder::new(app, "檔案")
                .item(&new_note)
                .item(&open_note)
                .separator()
                .close_window_with_text("關閉視窗")
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "編輯")
                .cut_with_text("剪下")
                .copy_with_text("拷貝")
                .paste_with_text("貼上")
                .select_all_with_text("全選")
                .build()?;

            MenuBuilder::new(app)
                .items(&[&app_menu, &file_menu, &edit_menu])
                .build()
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `Builder::menu` 收的函式在 App 建好時呼叫一次，回傳整份選單。`?` 的用法跟前面一樣。
- `SubmenuBuilder::new(app, 標題)` 開始一個子選單，`.item(&項目)` 放自訂項目，`.separator()` 畫分隔線，`.build()?` 完成。
- `MenuItem::with_id(app, id, 文字, 可不可用, 快捷鍵)`：id 是這個項目的名字，下一集用它辨識使用者選了誰。第四個參數是 `false` 的項目會變灰，不能選。快捷鍵 `"CmdOrCtrl+N"` 在 macOS 是 Cmd+N，在其他平台是 Ctrl+N。不要快捷鍵就寫 `None::<&str>`，要標型別是因為光寫 `None` 看不出它是哪種字串。
- `.cut()`、`.copy()`、`.quit()` 這類方法放進預設項目。預設的文字是英文，加上 `_with_text` 的版本才能換成中文。
- `MenuBuilder::new(app).items(&[...]).build()` 把子選單組成整份選單。

頁面只要一個輸入框，等一下拿來試「編輯」選單：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>應用程式選單</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>應用程式選單</h1>
      <div class="row">
        <input id="note-input" placeholder="在這裡打字，試試全選" />
      </div>
    </main>
  </body>
</html>
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.52s
     Running `target/debug/tauri-app`
```

在 macOS 上，螢幕最上方的選單列出現「tauri-app」「檔案」「編輯」三個選單。「新增筆記」旁邊顯示 ⌘N；它選了沒有反應，因為還沒有人處理，下一集才處理。「開啟筆記」是灰的。「結束 tauri-app」（Cmd+Q）會結束 App。macOS 還會自己在「編輯」底下多補幾個系統項目，像是「Emoji & Symbols」，那不是我們加的。

### macOS 與 Windows、Linux 不一樣

- macOS：選單屬於整個 App，顯示在螢幕最上方。第一個子選單固定是 app 選單，macOS 會把它的標題換成 App 的名稱（`productName`），不管 `SubmenuBuilder::new` 寫了什麼。範例寫成 `"tauri-app"`，讓其他平台看到的也一樣。
- Windows、Linux：選單屬於視窗，顯示在視窗上緣。`Builder::menu` 的選單會套用到每一個沒有自己選單的視窗。

沒呼叫 `Builder::menu` 的時候，macOS 有 Tauri 準備好的預設選單（app、File、Edit、View、Window、Help），Windows 與 Linux 沒有選單。一呼叫 `.menu(...)`，預設的就整份被取代。

預設項目也不是每個平台都有：`quit`、`close_window` 在 Linux 不支援，`undo`、`redo` 在 Windows 與 Linux 不支援。所以範例的「編輯」只放四個各平台都有的。Windows 與 Linux 的部分依 Tauri 文件整理，沒有實測。

### 少了「編輯」選單會怎樣

在 macOS，輸入框的 Cmd+A、Cmd+V 這類快捷鍵，要靠選單裡的「全選」「貼上」項目。把範例最後的 `.items(&[&app_menu, &file_menu, &edit_menu])` 拿掉 `&edit_menu` 再執行，在輸入框打字後按 Cmd+A 不會全選，Cmd+V 也貼不上；放回去，兩個都正常。「剪下」「拷貝」放在同一個選單裡，沒有逐一測試。自己做選單的時候，別忘了放「編輯」。這是在 macOS 上看到的，其他平台沒有驗證。

## 重點整理

- 選單由 `Menu`、`Submenu`、項目三層組成；用 `tauri::Builder::menu(|app| ...)` 在 App 建好時給出整份選單。
- 自訂項目用 `MenuItem::with_id(app, id, 文字, 可不可用, 快捷鍵)`，id 留給下一集辨識；預設項目（`.copy()`、`.quit()` 等）預設是英文，要中文用 `_with_text`。
- macOS 的選單屬於整個 App，第一個子選單是 app 選單；Windows、Linux 的選單屬於視窗。沒設選單時只有 macOS 有預設選單。
- `.menu(...)` 會整份取代預設選單；在 macOS，沒有「編輯」選單，輸入框就不能用 Cmd+A、Cmd+V 這類快捷鍵。
