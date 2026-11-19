# 系統匣選單

## 本集目標

替系統匣圖示掛上「顯示便條」與「結束」，左鍵直接叫出視窗、右鍵才開選單，並把建系統匣的程式拆成一個函式。

## 正文

### 沒有「結束」，App 就關不掉

上一集的 App，關掉視窗只是把它藏起來，唯一的結束方式是在終端機按 `Ctrl+C`。使用者手上沒有那個終端機，所以系統匣的選單必須有一個「結束」。第 8 章第 5 集說過，系統匣選單最常放兩項：把視窗叫回來，以及結束 App。這個 App 照做。

做法跟第 8 章第 5 集一樣：

- `Menu::with_items(app, &[&項目, ...])` 把項目放進選單，`TrayIconBuilder::menu(&menu)` 掛上去。
- `on_menu_event` 收的是**所有**選單的事件，要用 `event.id()` 分辨。id 是建項目時給的字串：這裡用 `"show"` 與 `"quit"`。
- 預設左鍵也會開選單。`show_menu_on_left_click(false)` 把它關掉，左鍵留給 `on_tray_icon_event`：像上一集一樣，左鍵放開就叫出視窗，右鍵才彈出選單。
- 「結束」用 `app.exit(0)`。視窗關掉只是藏起來，所以這是唯一真正結束 App 的地方。

### 拆出 `build_tray`

選單加進去之後，`setup` 裡建系統匣的程式變長了，不如拆成自己的函式 `build_tray`。它收 `&App`：`MenuItem::with_id`、`TrayIconBuilder::build` 要的是任何能當 `Manager` 的東西，`&App` 可以。`setup` 的 `app` 是 `&mut App`，直接傳給它就行，Rust 會自己把 `&mut` 降成 `&`。`build_tray` 回傳 `tauri::Result<()>`，`setup` 用 `?` 接。

`src-tauri/src/lib.rs`：

```rust
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, WindowEvent};

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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

這一集只動了 `lib.rs`。其餘三個檔案跟第 2 集一模一樣，為了讓這一集也能獨立開始，一併貼在下面：

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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.79s
     Running `target/debug/tauri-app`
```

App 啟動之後，跟第 2 集一樣沒有視窗，也沒有 Dock 圖示。左鍵點系統匣圖示，便條視窗出現；右鍵點，彈出兩個選項：「顯示便條」與「結束」。先用關閉鈕把視窗藏起來，再從選單選「顯示便條」，視窗回來。選「結束」，App 結束，`npm run tauri dev` 也跟著結束，終端機回到提示符號。

### 不同平台

- Linux 不支援 `show_menu_on_left_click`；Linux 的圖示有時要先設選單才會出現；一旦設了選單，就不能移除或換一份，只能改裡面的項目（第 8 章第 5 集）。
- 這一集只在 macOS 實測，Windows 與 Linux 依 Tauri 文件整理。

## 重點整理

- 關視窗只是藏起來，所以結束 App 的入口要放在系統匣選單：`MenuItem` 加 `app.exit(0)`。
- `on_menu_event` 收所有選單的事件，用 `event.id()` 分辨；`show_menu_on_left_click(false)` 讓左鍵留給 `on_tray_icon_event`，右鍵才開選單。
- 建系統匣的程式拆成 `build_tray(app: &App)`，回傳 `tauri::Result<()>`，`setup` 用 `?` 接。
