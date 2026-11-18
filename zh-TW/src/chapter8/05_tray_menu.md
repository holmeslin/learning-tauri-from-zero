# 系統匣選單

## 本集目標

學會替系統匣圖示掛上選單，用 `TrayIconBuilder::on_menu_event` 處理選項，並用 `show_menu_on_left_click` 決定左鍵要打開選單，還是留給自己處理。

## 正文

### 選單掛在圖示上

系統匣的選單跟第 1 集的一樣是 `Menu`。這集只放一層項目，沒有做子選單，要放 `Submenu` 也可以。

`Menu::with_items(app, &[&項目, ...])` 一次把項目放進選單（一個一個加，用 `MenuBuilder` 也行），再用 `TrayIconBuilder::menu(&menu)` 掛上去。

項目被選中的事件，可以交給 `TrayIconBuilder::on_menu_event`，函式的樣子跟第 2 集一樣。它收的是**所有**選單的事件，不只是系統匣選單的。Tauri 文件這樣寫，第 2 集的全域處理函式也一樣。所以系統匣選單的 id 不要跟別的選單重複。

系統匣選單最常放的兩項是「顯示視窗」跟「結束」：使用者要有個地方把視窗叫回來，也要有個地方結束 App。

### 範例：顯示視窗與結束

「顯示視窗」做的事，選單和左鍵都要用，所以抽成 `show_main_window`，內容跟第 4 集一樣：`show()`、`unminimize()`、`set_focus()` 三個都要。`app.exit(0)` 跟第 2 集一樣。

預設左鍵也會打開選單，常見的做法卻是左鍵直接叫出視窗、右鍵才開選單。範例用 `.show_menu_on_left_click(false)` 關掉左鍵開選單，左鍵的處理寫在 `on_tray_icon_event`，跟第 4 集一樣只處理左鍵放開（`Up`）。

`src-tauri/src/lib.rs`：

```rust
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "顯示視窗", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "結束", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

這集的頁面不用動，維持範本的樣子。不過 `lib.rs` 沒有 `greet` command，頁面上的 Greet 按鈕這集不會有結果。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.70s
     Running `target/debug/tauri-app`
```

macOS 的選單列右側出現圖示。用別的視窗蓋住 App，點左鍵，App 回到最前面，沒有跳出選單；點右鍵才彈出選單，選「顯示視窗」也能叫回來，視窗最小化時也一樣；選「結束」，App 結束，`npm run tauri dev` 也跟著結束。

把 `.show_menu_on_left_click(false)` 那一行拿掉，左鍵點下去就會打開選單，跟右鍵一樣。

### 不同平台

- Linux 不支援 `show_menu_on_left_click`；Linux 的圖示有時要先設選單才會出現；一旦設了選單，就不能移除或換一份，只能改裡面的項目。
- 這一集只在 macOS 實測，Windows 與 Linux 依 Tauri 文件整理。

## 重點整理

- 系統匣選單用 `Menu::with_items` 做，直接放項目，再用 `TrayIconBuilder::menu(&menu)` 掛上去。
- `TrayIconBuilder::on_menu_event` 收的是所有選單的事件，所以 id 不要跟別的選單重複。
- 常見的兩項：「顯示視窗」用 `get_webview_window` 加 `show`、`unminimize`、`set_focus`，「結束」用 `app.exit(0)`。
- 預設左鍵右鍵都開選單；`show_menu_on_left_click(false)` 讓左鍵留給 `on_tray_icon_event`。
