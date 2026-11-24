# 系統匣圖示

## 本集目標

學會用 `TrayIconBuilder` 在系統匣放一個圖示，處理它的點擊事件，並知道一次點擊會收到「按下」與「放開」兩個事件。

## 正文

### 系統匣與 `tray-icon`

系統匣是螢幕角落放小圖示的地方：macOS 在選單列右側，Windows 在工作列右下角，Linux 在面板上。有了圖示，使用者不用開著視窗，也能找到這個 App。它的 API 在 `tauri::tray` 底下，但這個模組是 `tauri` crate 的選用功能，預設沒開。在專案根目錄執行這行，把它打開：

```bash
cargo add --manifest-path src-tauri/Cargo.toml tauri@2 --features tray-icon
```

`src-tauri/Cargo.toml` 裡 `tauri` 那一行會變成 `tauri = { version = "2", features = ["tray-icon"] }`。沒開的話，`use tauri::tray::...` 編譯不過。我們另外拿掉這個 feature 實際編譯過，錯誤是 ``error[E0432]: unresolved import `tauri::tray` ``，編譯器還會指出 `tray` 模組被 `tray-icon` feature 擋住了。

### 範例：點左鍵叫出視窗

圖示在 `setup`（第 4 章第 4 集）裡建好。處理函式收到的事件是 `TrayIconEvent`，有 `Click`、`Enter`、`Move`、`Leave` 幾種，這裡只處理 `Click`，並把按的是哪個鍵（`button`）、是按下還是放開（`button_state`）取出來。

`src-tauri/src/lib.rs`：

```rust
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

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
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("tauri-app")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button,
                        button_state,
                        ..
                    } = event
                    {
                        println!("{button:?} {button_state:?}");
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let app = tray.app_handle();
                            app.emit("tray-click", ()).unwrap();
                            show_main_window(app);
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `.icon(...)` 要一張圖。`app.default_window_icon()` 是視窗的預設圖示，來自 `tauri.conf.json` 的 `bundle.icon`，所以不用另外準備。它回傳 `Option<&Image>`，`clone()` 拿一份自己的。
- `.tooltip(...)` 是滑鼠停在圖示上時顯示的提示文字。
- `tray.app_handle()` 從圖示取回 `AppHandle`，後面的 `emit` 跟前面章節一樣用。
- `show_main_window` 把視窗叫回來：`show()` 顯示被藏起來的視窗（第 13 章會正式介紹），`unminimize()` 還原最小化的視窗，最後 `set_focus()` 拉到最前面。三個都要，是因為 `set_focus()` 碰到隱藏或最小化的視窗什麼都不做。後面幾集都用這個函式。
- `build(app)?` 回傳的 `TrayIcon` 不用存起來。圖示屬於 App，不會因為變數被丟掉就消失。
- `println!` 是為了看事件，確認之後可以刪掉。

頁面只顯示點了幾次：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>系統匣圖示</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>系統匣圖示</h1>
      <p id="msg">還沒點過系統匣圖示</p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { listen } from "@tauri-apps/api/event";

const msgEl: HTMLElement | null = document.querySelector("#msg");
if (!msgEl) {
  throw new Error("找不到 #msg");
}
let count = 0;

listen("tray-click", () => {
  count += 1;
  msgEl.textContent = `左鍵點了 ${count} 次`;
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.58s
     Running `target/debug/tauri-app`
```

macOS 的選單列右側出現圖示，滑鼠停上去會顯示 `tauri-app`。用別的視窗蓋住 App，再點一下左鍵，App 回到最前面，頁面顯示「左鍵點了 1 次」。先把視窗最小化再點，視窗會還原。

### 一次點擊，兩個事件

按一次左鍵，終端機印出：

```text
Left Down
Left Up
```

`Click` 事件在按下去和放開時各來一次。範例只在 `Up` 做事；如果只判斷 `button == MouseButton::Left`，一次點擊就做兩次，頁面上的數字會一次加二。右鍵也一樣，會是 `Right Down`、`Right Up`。

### 不同平台

- `DoubleClick`（雙擊）事件只有 Windows 有。
- Linux 不會送出任何點擊事件，圖示照樣顯示。Linux 也不支援 `tooltip`，而且有時要設了選單（下一集）圖示才會出現。
- 這一集只在 macOS 實測，Windows 與 Linux 依 Tauri 文件整理。

## 重點整理

- 系統匣在 `tauri::tray`，要先在 `Cargo.toml` 開 `tauri` 的 `tray-icon` feature。
- 在 `setup` 裡用 `TrayIconBuilder::new().icon(...).tooltip(...).build(app)?` 建圖示；`app.default_window_icon()` 可以拿視窗圖示來用，建好的 `TrayIcon` 不用存。
- `on_tray_icon_event` 收 `TrayIconEvent`；`Click` 帶 `button` 與 `button_state`，一次點擊會有 `Down`、`Up` 兩次，只處理其中一個。
- 叫回視窗要 `show()`、`unminimize()`、`set_focus()` 三個一起用，因為 `set_focus()` 對隱藏或最小化的視窗什麼都不做。
- Linux 沒有點擊事件，雙擊事件只有 Windows 有。
