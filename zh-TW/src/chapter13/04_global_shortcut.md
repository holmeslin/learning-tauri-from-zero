# 全域快捷鍵叫出視窗

## 本集目標

用全域快捷鍵叫出或收起便條視窗：視窗在最前面就收起來，否則叫出來；叫出來之後，游標直接在輸入區裡。

## 正文

### 「切換」不只是看有沒有顯示

便條的用法是同一個快捷鍵按來按去：按一下叫出來，寫幾個字，再按一下收起來。最直覺的寫法是「視窗有顯示就藏起來，沒顯示就叫出來」，但這會有一個問題：視窗顯示著，卻被別的視窗蓋住了。這時候使用者按快捷鍵，是想把它拉到前面，不是想收起來。

所以判斷要用兩個條件：視窗**有顯示**（`is_visible()`）而且**有焦點**（`is_focused()`）才收起來，其他情況都是叫出來。這兩個方法是 Rust 端視窗的方法，回傳 `tauri::Result<bool>`。判斷本身寫成一個沒有碰 Tauri 的小函式 `toggle_action`，要收還是要叫，用兩個 `bool` 就能決定，第 7 集會替它寫測試。

### 加上 plugin

照第 8 章第 6 集，全域快捷鍵由 global-shortcut plugin 提供。那一集用 `npm run tauri add` 裝，連前端的 npm 套件和 `capabilities/desktop.json` 一起加了。這個 App 從 Rust 註冊快捷鍵，用不到那兩樣，所以只加 Rust 這一側的 crate。手機沒有全域快捷鍵，用 `--target` 把它放進 `Cargo.toml` 桌面專用的區塊：

```bash
cargo add --manifest-path src-tauri/Cargo.toml --target 'cfg(not(any(target_os = "android", target_os = "ios")))' tauri-plugin-global-shortcut@2
```

`cargo add` 的輸出：

```text
    Updating crates.io index
      Adding tauri-plugin-global-shortcut v2 to dependencies for target `cfg(not(any(target_os = "android", target_os = "ios")))`
```

### 註冊快捷鍵

註冊的程式寫成 `register_shortcut`，整個函式都包在 `#[cfg(desktop)]` 裡（第 8 章第 6 集；`cfg(desktop)` 第 18 章會正式介紹），在 `setup` 裡呼叫。內容跟第 8 章第 6 集一樣：先把 plugin 註冊進來，再 `on_shortcut(字串, 函式)`，函式只處理 `Pressed`，不然按一次會做兩次。

快捷鍵選 `CommandOrControl+Alt+Shift+N`。全域快捷鍵一旦註冊，按下這組按鍵，最前面的 App 就收不到了，所以要挑別的 App 不太會用的組合：`Cmd+Shift+N` 這種兩個鍵的組合，Finder、瀏覽器都用了；三個修飾鍵再加一個字母，撞到的機會小很多。

註冊可能失敗：字串寫錯，或這組按鍵被別的程式佔走（第 8 章第 6 集）。第 8 章第 6 集用 `?` 把錯誤丟出去，註冊失敗 App 就打不開。這一次不一樣：快捷鍵只是入口之一，用不了還有系統匣，所以只用 `eprintln!` 印出訊息，App 照常啟動。

### 叫出來之後直接打字

視窗叫出來，使用者接著就要打字，不該還要再點一下輸入區。頁面上用 `window` 的 `focus` 事件：視窗取得焦點的時候觸發，用法跟第 2 章第 8 集的 `click` 一樣，用 `addEventListener` 接。事件發生的時候，呼叫輸入區的 `focus()`，把游標放進去。

`src/main.ts`：

```ts
const noteEl: HTMLTextAreaElement | null = document.querySelector("#note");
if (!noteEl) {
  throw new Error("找不到 #note");
}

window.addEventListener("focus", () => {
  noteEl.focus();
});
```

### 完整的 `lib.rs`

`src-tauri/src/lib.rs`：

```rust
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, WindowEvent};

#[derive(Debug, PartialEq)]
enum ToggleAction {
    Show,
    Hide,
}

fn toggle_action(visible: bool, focused: bool) -> ToggleAction {
    if visible && focused {
        ToggleAction::Hide
    } else {
        ToggleAction::Show
    }
}

fn log_error(result: tauri::Result<()>) {
    if let Err(error) = result {
        eprintln!("{error}");
    }
}

fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }
    Ok(())
}

fn hide_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }
    #[cfg(target_os = "macos")]
    app.hide()?;
    Ok(())
}

fn toggle_main_window(app: &AppHandle) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    match toggle_action(window.is_visible()?, window.is_focused()?) {
        ToggleAction::Hide => hide_main_window(app),
        ToggleAction::Show => show_main_window(app),
    }
}

fn build_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "顯示便條", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "結束", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("快速便條")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => log_error(show_main_window(app)),
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
                log_error(show_main_window(tray.app_handle()));
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg(desktop)]
fn register_shortcut(app: &App) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    const SHORTCUT: &str = "CommandOrControl+Alt+Shift+N";

    app.handle()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
    let registered = app
        .global_shortcut()
        .on_shortcut(SHORTCUT, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                log_error(toggle_main_window(app));
            }
        });
    if let Err(error) = registered {
        eprintln!("註冊快捷鍵 {SHORTCUT} 失敗：{error}");
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                log_error(hide_main_window(window.app_handle()));
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            build_tray(app)?;
            #[cfg(desktop)]
            register_shortcut(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `toggle_main_window` 取得視窗之後，把 `is_visible()` 與 `is_focused()` 交給 `toggle_action`。要收起來就走 `hide_main_window`（macOS 會連 App 一起藏），要叫出來就走 `show_main_window`。
- `register_shortcut` 的 `SHORTCUT` 放在函式裡，只有這裡用得到。
- 快捷鍵的處理函式，第一個參數 `app` 就是 `&AppHandle`，直接傳給 `toggle_main_window`。

其餘三個檔案跟第 3 集一模一樣，為了讓這一集也能獨立開始，一併貼在下面：

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
        "title": "快速便條",
        "width": 360,
        "height": 240,
        "center": true,
        "visible": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
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

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>快速便條</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main id="note-view">
      <textarea id="note" placeholder="想到什麼，寫下來" spellcheck="false"></textarea>
    </main>
  </body>
</html>
```

`src/styles.css`：

```css
body {
  margin: 0;
  font-family: sans-serif;
}

#note-view {
  height: 100vh;
}

#note {
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 12px;
  border: none;
  outline: none;
  resize: none;
  font-family: inherit;
  font-size: 16px;
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.21s
     Running `target/debug/tauri-app`
```

App 啟動之後沒有視窗。切到別的 App，按 `Cmd+Option+Shift+N`（Windows 與 Linux 是 `Ctrl+Alt+Shift+N`），便條視窗出現在螢幕中央，輸入區裡已經有游標，直接打字就會寫進去。再按一次，視窗收起來，焦點回到剛才的 App。

把 `src/main.ts` 的 `focus` 那段拿掉，視窗一樣會出現，但輸入區沒有焦點，打的字不會進去，得先用滑鼠點一下輸入區。

### 註冊失敗的時候

把 `SHORTCUT` 改成 `"CommandOrControl-Alt-Shift-N"`（用 `-` 取代 `+`）。App 照常啟動，系統匣圖示還在，只是快捷鍵沒作用，終端機印出：

```text
註冊快捷鍵 CommandOrControl-Alt-Shift-N 失敗：Couldn't recognize "CommandOrControl-Alt-Shift-N" as a valid key for hotkey, if you feel like it should be, please report this to https://github.com/tauri-apps/muda
```

### 不同平台

- Linux 只支援 X11，不支援 Wayland（第 8 章第 6 集）。
- `is_focused()` 在各平台的行為，這一集只在 macOS 實測。Windows 與 Linux 依 Tauri 文件整理。

## 重點整理

- 切換視窗要看兩個條件：有顯示而且有焦點才收起來，其他都是叫出來，這樣被蓋住的視窗按快捷鍵會被拉到前面。判斷寫成不碰 Tauri 的 `toggle_action(visible, focused)`。
- 全域快捷鍵用 global-shortcut plugin 在 Rust 註冊；從 Rust 註冊不用 permission，只需要 `cargo add` 加桌面專用的 crate，不需要 npm 套件。
- 挑別的 App 不太會用的組合，像三個修飾鍵加一個字母；註冊失敗時印出訊息就好，不要讓整個 App 打不開，因為系統匣還是入口。
- 頁面用 `window` 的 `focus` 事件呼叫 `focus()`，叫出視窗之後游標直接在輸入區。
