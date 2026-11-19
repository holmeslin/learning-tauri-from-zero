# 設定視窗（store）

## 本集目標

加一個設定視窗，用 store 記住便條的字型大小；設定視窗一改，便條視窗馬上跟著變。

## 正文

### 兩個視窗，一份設定

第 7 章第 7 集說過：同一個檔名 `load` 兩次，拿到的是同一張表。所以設定視窗寫進去的值，便條視窗再 `load` 一次就讀得到，資料不用傳來傳去。剩下的問題是：便條視窗怎麼知道設定變了？

答案是事件（第 5 章）。設定視窗存好之後，`emit("settings-changed")`；便條視窗 `listen` 到，就重新讀一次設定。這個事件不帶資料，只是按一下門鈴，真正的值還是放在 store 裡。第 11 章第 5 集的 `todos-changed` 是另一種做法：payload 直接帶整份清單。這裡不用帶，是因為收到事件的便條視窗本來就會自己去 store 讀，帶了也只是多一份。

### 裝 store

第 7 章第 7 集教過，用 `tauri add` 裝：

```bash
npm run tauri add store
```

`npm run tauri add store` 的輸出：

```text
        Info Installing Cargo dependency "tauri-plugin-store"...
        Info Installing NPM dependency "@tauri-apps/plugin-store@~2"...
       Added permission `store:default` to `default` at ~/tauri-app/src-tauri/capabilities/default.json
        Info Adding plugin to ~/tauri-app/src-tauri/src/lib.rs
        Info Running `cargo fmt`...
```

它在 `lib.rs` 的 `Builder` 鏈上加了 `.plugin(tauri_plugin_store::Builder::new().build())`，並把 `store:default` 寫進 `capabilities/default.json`。

### capability 是給「視窗」的

`capabilities/default.json` 裡的 `windows` 是一份視窗 label 的清單，permission 只發給清單上的視窗。範本裡只有 `"main"`，而設定視窗的 label 是 `settings`，沒列進去，就算 `permissions` 裡有 `store:default`，設定視窗也一樣用不了 store。把 `windows` 改成 `["main", "settings"]`。capability 與 permission 完整的規則，第 14 章會正式介紹，這裡只要記得「permission 是發給視窗的」：

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
    "store:default"
  ]
}
```

少了 `"settings"` 的話，設定視窗讀不到 store，畫面上的訊息會是：

```text
讀取設定失敗：store.load not allowed on window "settings", webview "settings", URL: local allowed on: [windows: "main", URL: local] referenced by: capability: default, permission: allow-load
```

### 讀寫字型大小：`settings.ts`

字型大小存在 `settings.json` 的 `fontSize`。這個檔案使用者也能用文字編輯器開啟，改成字串或一個荒謬的數字，所以從 store 讀出來的值不能直接相信。`store.get("fontSize")` 沒指定型別時，回傳的是 `unknown`（第 9 章第 3 集用過這個型別：什麼都可能是，用之前要自己檢查）。`normalizeFontSize` 負責把任何值變成可以用的數字：

- 不是數字（或是 `NaN`、無限大），用預設的 16。`Number.isFinite(x)` 檢查 `x` 是不是有限的數字。
- 是數字，就用 `Math.round` 四捨五入成整數，再用 `Math.max`、`Math.min`（各回傳兩個數字裡較大、較小的那個）夾在 12 到 32 之間。

存的時候也先過同一個函式，再把處理過的值寫進去，並回傳給畫面，讓輸入框顯示真正存下去的值。`set` 之後不用呼叫 `save()`，預設的 `autoSave` 會在 100 毫秒後寫進檔案（第 7 章第 7 集）。

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

### 開設定視窗：Rust 這一側

設定視窗跟第 6 章第 3 集的「關於」視窗一樣，用 `WebviewWindowBuilder` 開，載入同一個 `index.html`；跟第 6 章第 4 集一樣，先用 label 找找看有沒有開著，有就拉到前面，沒有才建。

選單加一項「設定…」，id 是 `"settings"`。選單事件的處理函式是事件處理函式，第 6 章第 3 集說過：Windows 上，在同步的 command 裡建視窗可能卡死，Tauri 文件說事件處理函式也一樣。所以不在處理函式裡直接建，丟到另一個執行緒去（第 4 章第 5 集用過 `thread::spawn`）：`AppHandle` 可以複製，`move` 進去就行。

另一處要改：`on_window_event` 現在會收到兩個視窗的事件。只有便條視窗要藏起來，設定視窗按關閉鈕，就讓它真的關掉，下次要用再建新的，這樣一開就讀到最新的設定。所以先判斷 `window.label() == "main"`。

`hide_main_window` 也要改。第 2 集提過，macOS 的 `app.hide()` 藏的是整個 App 的所有視窗。便條視窗與設定視窗同時開著的時候，收起便條，連設定視窗也會一起被藏起來。所以改成：設定視窗有顯示的時候，只藏便條視窗，不呼叫 `app.hide()`；代價是這時候，焦點不會自動回到上一個 App。判斷寫成 macOS 專用的 `settings_window_visible`：`get_webview_window("settings")` 找得到視窗，而且 `is_visible()` 是 `true`，才算開著。設定視窗關掉就是銷毀，下次要用再建（上面說過），所以關掉之後找不到視窗，回到第 2 集的行為。

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

#[cfg(target_os = "macos")]
fn settings_window_visible(app: &AppHandle) -> tauri::Result<bool> {
    match app.get_webview_window("settings") {
        Some(window) => window.is_visible(),
        None => Ok(false),
    }
}

fn hide_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }
    #[cfg(target_os = "macos")]
    if !settings_window_visible(app)? {
        app.hide()?;
    }
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
            register_shortcut(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 頁面：同一頁，兩種畫面

兩個視窗載入同一個 `index.html`，裡面放了兩塊畫面，先都加上 `hidden`，讓 `main.ts` 看 label 決定顯示哪一塊（第 6 章第 6 集用過 `hidden`）。設定視窗只有一個輸入框：`<input type="number">` 是只收數字的輸入框，`min`、`max`、`step` 限制範圍與每次增減的量。使用者直接打字還是能打超出範圍的數字，所以 `normalizeFontSize` 仍然要在。

有一個陷阱：`hidden` 屬性是靠瀏覽器預設的 `display: none` 運作的，自己的 CSS 只要替**同一個元素**設了 `display`，就會把它蓋掉，元素又顯示出來了。所以 CSS 裡沒有替 `#note-view`、`#settings-view` 設 `display`，排版交給裡面的元素。

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
      <p id="settings-msg"></p>
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

`main.ts` 的做法：

- 便條視窗（`main`）：顯示便條那一塊，讀字型大小套上去，再 `listen("settings-changed", ...)`，每次收到就重新讀一次。套用字型大小靠 `style`：每個元素都有 `style` 欄位，可以在程式裡直接改它的 CSS，CSS 的 `font-size` 在這裡寫成 `fontSize`，值是帶單位的字串，例如 `"20px"`。
- 設定視窗（`settings`）：顯示設定那一塊，把目前的字型大小填進輸入框。輸入框的 `value` 是字串，所以數字要先用 `String(數字)` 轉成字串才放得進去。輸入框發出 `change` 事件（輸入完畢、離開輸入框時觸發一次，第 11 章第 4 集的勾選框也是這個事件）就存起來，再 `emit("settings-changed")`。`valueAsNumber` 是數字輸入框的欄位，直接給 `number`，輸入框是空的時候是 `NaN`，交給 `normalizeFontSize` 變回預設值。
- 讀取、儲存失敗時，便條視窗只能印到開發者工具的 Console（第 2 章第 7 集），程式裡用的是 `console.error`：用法跟那一集的 `console.log` 一樣，只是印出來是紅色的錯誤訊息。設定視窗有地方顯示，就寫在 `#settings-msg`。

`src/main.ts`：

```ts
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { loadFontSize, saveFontSize } from "./settings";

const noteView: HTMLElement | null = document.querySelector("#note-view");
const noteEl: HTMLTextAreaElement | null = document.querySelector("#note");
const settingsView: HTMLElement | null =
  document.querySelector("#settings-view");
const fontSizeEl: HTMLInputElement | null =
  document.querySelector("#font-size");
const msgEl: HTMLElement | null = document.querySelector("#settings-msg");
if (!noteView || !noteEl || !settingsView || !fontSizeEl || !msgEl) {
  throw new Error("找不到必要的元素");
}

const applyFontSize = async () => {
  try {
    noteEl.style.fontSize = `${await loadFontSize()}px`;
  } catch (error) {
    console.error("讀取字型大小失敗", error);
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
  fontSizeEl.addEventListener("change", async () => {
    try {
      fontSizeEl.value = String(await saveFontSize(fontSizeEl.valueAsNumber));
      await emit("settings-changed");
      msgEl.textContent = "";
    } catch (error) {
      msgEl.textContent = `儲存失敗：${error}`;
    }
  });
};

if (getCurrentWindow().label === "settings") {
  showSettings();
} else {
  showNote();
}
```

其餘的檔案只有 `tauri.conf.json` 沒有改，跟第 4 集一模一樣：

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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.10s
     Running `target/debug/tauri-app`
```

App 啟動之後一樣沒有視窗。在系統匣圖示上按右鍵，選「設定…」，螢幕中央出現一個標題是「設定」、大小 320×160 的視窗，裡面只有「字型大小」和一個輸入框，值是預設的 16。

把輸入框改成 `20`，按 `Tab` 離開輸入框（這時才會觸發 `change`）。叫出便條視窗，字變大了。設定檔是一個 JSON 檔：

```bash
cat "$HOME/Library/Application Support/tw.example.tauri-app/settings.json"
```

輸出：

```text
{
  "fontSize": 20
}
```

按設定視窗的關閉鈕，它是真的關掉，不是藏起來；再從選單開一次，輸入框裡是 `20`。結束 `npm run tauri dev` 再啟動一次，便條的字型大小還是 20。在輸入框打 `99` 再離開，輸入框會變成 `32`，檔案裡存的也是 32，因為 `normalizeFontSize` 把它夾回了上限。把輸入框清空再離開，會變回 `16`：空的輸入框，`valueAsNumber` 是 `NaN`，被換成預設值。

### 不同平台

- 設定檔的位置依平台而定：macOS 是 `~/Library/Application Support/<identifier>/`，其他平台第 7 章第 6 集說過各自的 app 資料夾。
- 這一集只在 macOS 實測，Windows 與 Linux 依 Tauri 文件整理。

## 重點整理

- 兩個視窗各自 `load("settings.json")`，拿到的是同一張表。設定視窗寫、便條視窗讀，用 `settings-changed` 事件按門鈴，值留在 store。
- capability 的 `windows` 是視窗 label 的清單，permission 只發給列在裡面的視窗；新增的設定視窗要加進去，不然它用不了 store。
- store 檔案使用者也能改，讀出來的值要先檢查：`normalizeFontSize` 把任何值變成 12 到 32 之間的整數，不合法就用預設值 16。
- 設定視窗在選單事件裡開，丟到另一個執行緒建，避開 Windows 上事件處理函式裡建視窗會卡死的問題；`on_window_event` 只替 `main` 擋關閉，設定視窗正常關閉；macOS 的 `app.hide()` 會藏起所有視窗，所以設定視窗有顯示的時候，`hide_main_window` 只藏便條視窗。
- 有 `hidden` 屬性的元素，不要用 CSS 替它設 `display`，否則 `hidden` 會失效。
