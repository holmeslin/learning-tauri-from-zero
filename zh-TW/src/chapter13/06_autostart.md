# 開機自動啟動（autostart）

## 本集目標

用 autostart plugin 在設定視窗放一個「開機時自動啟動」的勾選框，並知道這個狀態由作業系統記著，要去問它，不另外存一份。

## 正文

### 登入後啟動，各平台做法不同

autostart plugin 把「使用者登入之後，自動啟動這個 App」包成三個動作：`enable`、`disable`、`isEnabled`。底下各平台的做法不一樣：

- macOS：預設在 `~/Library/LaunchAgents/` 放一個 plist 檔，登入時系統照它的內容啟動 App。plugin 也能改用 AppleScript 加登入項目，這一集用預設的 LaunchAgent。
- Windows：在登錄檔的 `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` 加一項。
- Linux：在 `~/.config/autostart/` 放一個 `.desktop` 檔。

三個平台，「有沒有開」都是靠這些檔案或登錄檔項目判斷的，作業系統自己就是唯一的真相。所以勾選框的狀態不要另外存進 store：每次開啟設定視窗，用 `isEnabled()` 問一次；使用者勾選之後，也再問一次，畫面才會跟實際一致。要是自己存一份，使用者在系統設定裡把它關掉，兩邊就對不起來了。

### 裝 plugin

```bash
npm run tauri add autostart
```

`npm run tauri add autostart` 的輸出：

```text
        Info Installing Cargo dependency "tauri-plugin-autostart"...
        Info Installing NPM dependency "@tauri-apps/plugin-autostart@~2"...
        Info Capability matching platforms ["macOS", "windows", "linux"] not found, creating ~/tauri-app/src-tauri/capabilities/desktop.json
       Added permission `autostart:default` to `desktop-capability` at ~/tauri-app/src-tauri/capabilities/desktop.json
        Info Adding plugin to ~/tauri-app/src-tauri/src/lib.rs
        Info Running `cargo fmt`...
```

跟第 8 章第 6 集裝 global-shortcut 一樣，它做了三件事，每一件都要看一下：

- 手機沒有開機自動啟動，所以 crate 放在 `Cargo.toml` 桌面專用的區塊。
- `lib.rs` 的 `Builder` 鏈上多了 `.plugin(tauri_plugin_autostart::Builder::new().build())`。這個 plugin 只有桌面有，下面的範例改在 `setup` 裡用 `#[cfg(desktop)]` 註冊，所以直接用範例的整份 `lib.rs` 取代，不要兩邊都留。`Builder::new().build()` 在 macOS 預設用 LaunchAgent。
- 它另外建了 `capabilities/desktop.json`，放進 `autostart:default`，但裡面的 `windows` 只有 `"main"`。要用它的是設定視窗，所以我們把 `autostart:default` 寫進 `default.json`（`windows` 已經有 `settings`），再把 `desktop.json` 刪掉。`autostart:default` 放行三個動作：`allow-enable`、`allow-disable`、`allow-is-enabled`。

### 畫面與程式

設定視窗多一個勾選框。`main.ts` 的 `showSettings` 做三件事：

- 一開啟，就用 `isEnabled()` 問作業系統，把結果填進勾選框的 `checked`。
- 使用者勾選或取消，勾選框發出 `change` 事件（第 11 章第 4 集），依 `checked` 呼叫 `enable()` 或 `disable()`。
- 不管成功或失敗，最後都再問一次 `isEnabled()`，讓勾選框顯示的是作業系統真正的狀態。失敗的原因寫在 `#settings-msg`。

`isEnabled`、`enable`、`disable` 都是 `Promise`，要 `await`（第 2 章第 10 集）。

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the note and settings windows",
  "windows": ["main", "settings"],
  "permissions": [
    "core:default",
    "opener:default",
    "store:default",
    "autostart:default"
  ]
}
```

`src-tauri/src/lib.rs`：

```rust
use std::thread;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

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

fn open_settings_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        return window.set_focus();
    }
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("設定")
        .inner_size(320.0, 160.0)
        .center()
        .build()?;
    Ok(())
}

fn build_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "顯示便條", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "設定…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "結束", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("快速便條")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => log_error(show_main_window(app)),
            "settings" => {
                let app = app.clone();
                thread::spawn(move || log_error(open_settings_window(&app)));
            }
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
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                log_error(hide_main_window(window.app_handle()));
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            build_tray(app)?;
            #[cfg(desktop)]
            {
                register_shortcut(app)?;
                app.handle()
                    .plugin(tauri_plugin_autostart::Builder::new().build())?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
    <main id="note-view" hidden>
      <textarea id="note" placeholder="想到什麼，寫下來" spellcheck="false"></textarea>
    </main>
    <main id="settings-view" hidden>
      <label>
        字型大小
        <input id="font-size" type="number" min="12" max="32" step="1" />
      </label>
      <label>
        <input id="autostart" type="checkbox" />
        開機時自動啟動
      </label>
      <p id="settings-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { loadFontSize, saveFontSize } from "./settings";

const noteView: HTMLElement | null = document.querySelector("#note-view");
const noteEl: HTMLTextAreaElement | null = document.querySelector("#note");
const settingsView: HTMLElement | null =
  document.querySelector("#settings-view");
const fontSizeEl: HTMLInputElement | null =
  document.querySelector("#font-size");
const autostartEl: HTMLInputElement | null =
  document.querySelector("#autostart");
const msgEl: HTMLElement | null = document.querySelector("#settings-msg");
if (
  !noteView ||
  !noteEl ||
  !settingsView ||
  !fontSizeEl ||
  !autostartEl ||
  !msgEl
) {
  throw new Error("找不到必要的元素");
}

const applyFontSize = async () => {
  try {
    noteEl.style.fontSize = `${await loadFontSize()}px`;
  } catch (error) {
    console.error("讀取字型大小失敗", error);
  }
};

const syncAutostart = async () => {
  try {
    autostartEl.checked = await isEnabled();
  } catch (error) {
    msgEl.textContent = `讀取開機設定失敗：${error}`;
  }
};

const showNote = () => {
  noteView.hidden = false;
  applyFontSize();
  listen("settings-changed", applyFontSize);
  window.addEventListener("focus", () => {
    noteEl.focus();
  });
};

const showSettings = async () => {
  settingsView.hidden = false;
  try {
    fontSizeEl.value = String(await loadFontSize());
  } catch (error) {
    msgEl.textContent = `讀取設定失敗：${error}`;
  }
  await syncAutostart();

  fontSizeEl.addEventListener("change", async () => {
    try {
      fontSizeEl.value = String(await saveFontSize(fontSizeEl.valueAsNumber));
      await emit("settings-changed");
      msgEl.textContent = "";
    } catch (error) {
      msgEl.textContent = `儲存失敗：${error}`;
    }
  });

  autostartEl.addEventListener("change", async () => {
    try {
      if (autostartEl.checked) {
        await enable();
      } else {
        await disable();
      }
      msgEl.textContent = "";
    } catch (error) {
      msgEl.textContent = `設定開機啟動失敗：${error}`;
    }
    await syncAutostart();
  });
};

if (getCurrentWindow().label === "settings") {
  showSettings();
} else {
  showNote();
}
```

其餘三個檔案跟第 5 集一模一樣，為了讓這一集也能獨立開始，一併貼在下面：

`src/settings.ts`：

```ts
import { load } from "@tauri-apps/plugin-store";

const DEFAULT_FONT_SIZE = 16;
const MIN_FONT_SIZE = 12;
const MAX_FONT_SIZE = 32;

export function normalizeFontSize(value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return DEFAULT_FONT_SIZE;
  }
  return Math.min(MAX_FONT_SIZE, Math.max(MIN_FONT_SIZE, Math.round(value)));
}

export async function loadFontSize(): Promise<number> {
  const store = await load("settings.json");
  return normalizeFontSize(await store.get("fontSize"));
}

export async function saveFontSize(size: number): Promise<number> {
  const normalized = normalizeFontSize(size);
  const store = await load("settings.json");
  await store.set("fontSize", normalized);
  return normalized;
}
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

#settings-view {
  padding: 16px;
}

#settings-view label {
  display: block;
  margin-bottom: 12px;
}

#font-size {
  width: 4em;
  margin-left: 8px;
}

#settings-msg {
  min-height: 1.5em;
  margin: 0;
  color: #c00;
  font-size: 13px;
}
```

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

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.74s
     Running `target/debug/tauri-app`
```

從系統匣開設定視窗，除了「字型大小」，多了一個「開機時自動啟動」的勾選框，一開始沒有勾：作業系統那邊還沒有登記。

把它勾起來，勾選框維持勾著（`enable()` 成功之後，`isEnabled()` 回答 `true`）。這時 `~/Library/LaunchAgents/` 裡多了一個以 `productName` 命名的 plist 檔：

```bash
cat ~/Library/LaunchAgents/tauri-app.plist
```

輸出：

```text
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
  <key>Label</key>
  <string>tauri-app</string>
  <key>AssociatedBundleIdentifiers</key>
  <array></array>
  <key>ProgramArguments</key>
  <array><string>~/tauri-app/src-tauri/target/debug/tauri-app</string></array>
  <key>RunAtLoad</key>
  <true/>
  </dict>
</plist>
```

`ProgramArguments` 是登入時要執行的程式，這裡的 `~` 只是示意，實際檔案裡是完整的絕對路徑，指向 dev 版的執行檔。`RunAtLoad` 是 `true`，登入時就執行。

再按一次，取消勾選，勾選框回到沒勾，那個 plist 檔不見了（`cat` 會回報 `No such file or directory`）。這一集的測試就是勾選一次、再取消一次，不要讓這個登記留在你的電腦上。

### 先知道這些再試

- `dev` 模式下勾選，登記進去的是 `src-tauri/target/debug/tauri-app` 這個開發版執行檔的路徑，不是打包好的 App。試完請把勾選框取消，不然下次登入，系統會去找這個路徑。第 15 章打包之後，登記的才會是安裝好的 App。
- 登記的名稱用 `tauri.conf.json` 的 `productName`（plugin 的 `Builder` 有 `app_name` 可以改）。
- 在 macOS，LaunchAgent 的「有沒有開」就是那個 plist 檔在不在。使用者手動把它刪掉，`isEnabled()` 就變成 `false`，所以每次都要問，不要相信自己記的狀態。

### 不同平台

- Windows 與 Linux 的位置見上面「登入後啟動，各平台做法不同」，指令輸出與畫面依 plugin 的原始碼與文件整理，沒有實測。
- 這一集只在 macOS 實測。

## 重點整理

- autostart plugin 提供 `enable`、`disable`、`isEnabled`；macOS 預設在 `~/Library/LaunchAgents/` 放 plist，Windows 寫登錄檔，Linux 放 `.desktop` 檔。
- 「有沒有開機自動啟動」只有作業系統知道：開啟設定視窗時用 `isEnabled()` 填勾選框，改了之後再問一次，不另外存進 store。
- `tauri add autostart` 會把 plugin 放進 `Builder` 鏈、建一個只含 `main` 的 `desktop.json`；桌面專用的 plugin 要改在 `setup` 用 `#[cfg(desktop)]` 註冊，permission 寫進涵蓋設定視窗的 `default.json`。
- dev 模式登記的是開發版執行檔的路徑，試完要取消勾選。
