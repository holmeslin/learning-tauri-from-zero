# 沒有主視窗的 App

## 本集目標

讓 App 啟動時不顯示視窗、關掉視窗不結束 App、在 macOS 不佔 Dock，並用系統匣圖示把視窗叫回來。

## 正文

### 三件事

「沒有主視窗」不是真的沒有視窗，而是視窗平常藏著。要做到三件事，缺一不可：

1. 啟動時，視窗不顯示。
2. 使用者關視窗的時候，視窗只是藏起來，App 還活著。
3. 有個地方能把視窗叫回來：系統匣圖示（第 8 章第 4 集）。

### 啟動時不顯示：`visible`

設定檔的視窗欄位多一個 `"visible": false`，視窗照樣建好，網頁也照樣載入，只是不顯示。要讓它出現，之後用 Rust 呼叫 `show()`，第 8 章第 4 集已經用過，這裡正式拿來用。順便把標題與大小換成便條的樣子，`"center": true` 讓它第一次出現時在螢幕中央：

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

### 關視窗只是藏起來：`on_window_event`

第 6 章第 6 集用前端的 `onCloseRequested` 擋下關閉。這個 App 的程式都在 Rust 這一側，所以這一集也在 Rust 擋。從 Rust 呼叫視窗的方法不用 permission（第 6 章第 3 集），前端那邊就不必放行什麼。

`Builder::on_window_event` 收的是**每個視窗**的事件，函式拿到 `window` 和 `event`。其中 `WindowEvent::CloseRequested { api, .. }` 就是「使用者要關這個視窗」，跟前端的 `onCloseRequested` 是同一件事。`api.prevent_close()` 擋下它，等於前端的 `event.preventDefault()`。擋下之後視窗還在，我們自己把它藏起來。

沒有 `prevent_close()` 的話，視窗會被銷毀。這是唯一的視窗，App 就跟著結束，跟第 6 章第 6 集一樣。

藏視窗的程式寫成 `hide_main_window`，第 4 集的快捷鍵也要用。它不只呼叫 `window.hide()`，在 macOS 還多一行 `app.hide()`。我們在 macOS 實測：只藏視窗的話，視窗是不見了，但作業系統認定的最前面的 App 還是我們，剛才用的那個 App 沒有拿回焦點；連 `app.hide()` 一起呼叫，焦點才回到上一個 App。`app.hide()` 只有 macOS 有，所以用 `#[cfg(target_os = "macos")]` 隔開。

還有一件事要先記著：`app.hide()` 藏的是整個 App，所有視窗一起藏，跟按 Cmd+H 一樣（我們查了 Tauri 底層視窗函式庫 tao 的原始碼，它呼叫的是 macOS `NSApplication` 的 `hide:`）。現在 App 只有便條一個視窗，沒有差別；第 5 集加了設定視窗之後，`hide_main_window` 會多一個判斷。

### macOS：不佔 Dock

macOS 的 App 預設會在 Dock 放一個圖示，選單列左邊也有 App 自己的選單。系統匣小工具不需要這些。在 `setup` 裡把 activation policy 設成 `Accessory`：這種 App 不出現在 Dock，也沒有 App 選單，但可以有視窗，也能被叫出來。這是 macOS 才有的東西，一樣用 `cfg` 隔開（跟第 12 章第 7 集的 `#[cfg(target_os = "macos")]` 一樣）。

### 用系統匣圖示叫回視窗

系統匣要先在 `Cargo.toml` 開啟 `tauri` 的 `tray-icon` feature，沒開的話 `use tauri::tray::...` 編譯不過（第 8 章第 4 集）。專案還沒開的話，在專案根目錄執行：

```bash
cargo add --manifest-path src-tauri/Cargo.toml tauri@2 --features tray-icon
```

建圖示跟第 8 章第 4 集一樣：只處理左鍵放開（`Up`），因為一次點擊會有 `Down` 與 `Up` 兩個事件。左鍵點下去，叫出便條視窗。叫出視窗的 `show_main_window`，也是第 8 章第 4 集那三個一起用：`show()`、`unminimize()`、`set_focus()`。這些操作都可能失敗，但事件處理函式沒地方回傳 `Err`，所以它們回傳 `tauri::Result<()>`，再交給 `log_error` 印到終端機就好。

`src-tauri/src/lib.rs`：

```rust
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

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

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("快速便條")
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
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- 這個 App 沒有自己的 command，範本的 `greet` 用不到，一併拿掉。
- `window.app_handle()` 從視窗取回 `AppHandle`：視窗跟 `AppHandle` 一樣實作了 `Manager`（第 4 章第 5 集）。
頁面這一集沒有改，維持第 1 集的樣子，為了讓這一集也能獨立開始，一併貼在下面：

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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.95s
     Running `target/debug/tauri-app`
```

App 啟動之後，螢幕上沒有視窗，Dock 裡也沒有圖示，只有選單列右側多了系統匣圖示。點系統匣圖示的左鍵，便條視窗在螢幕中央出現，標題是「快速便條」，大小 360×240。在便條裡打幾個字，例如 `buy milk`，再按視窗左上角的關閉鈕：視窗不見了，`npm run tauri dev` 卻沒有結束。再點一次系統匣圖示，視窗回來，`buy milk` 還在：視窗只是藏起來，頁面沒有重新載入，所以畫面上的狀態都留著。

視窗藏起來的這段時間，網頁其實一直活著。我們另外試過：讓頁面一載入就把當下的時間寫進 `<textarea>`，等了半分鐘才叫出視窗，裡面寫的是 App 剛啟動時的時間，不是叫出視窗的時間。所以「啟動時不顯示」只是不顯示，頁面在啟動的時候就載入了。

### 不同平台

- Windows 與 Linux 沒有 Dock，也沒有 activation policy。藏起來的視窗不會出現在工作列；視窗顯示的時候，工作列照樣有它的按鈕，不想要的話，在設定檔的視窗欄位加 `"skipTaskbar": true`。
- 這一集只在 macOS 實測，Windows 與 Linux 依 Tauri 文件整理。

## 重點整理

- 沒有主視窗是三件事：設定檔 `"visible": false` 讓視窗啟動時不顯示、攔下關閉改成藏起來、用系統匣圖示把視窗叫回來。
- 在 Rust 用 `Builder::on_window_event` 接 `WindowEvent::CloseRequested { api, .. }`，`api.prevent_close()` 擋下關閉，再 `hide()` 藏起來；從 Rust 呼叫不用 permission。
- macOS 只藏視窗，焦點不會回到上一個 App；要連 `app.hide()` 一起呼叫。`app.hide()` 藏的是整個 App 的所有視窗，有第二個視窗的時候要小心（第 5 集）。
- macOS 用 `set_activation_policy(ActivationPolicy::Accessory)` 讓 App 不佔 Dock，兩者都要用 `#[cfg(target_os = "macos")]` 隔開。
- 目前還沒有選單，唯一的結束方式是在終端機按 `Ctrl+C`；下一集補上「結束」。
