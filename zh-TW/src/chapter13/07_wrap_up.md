# 收尾

## 本集目標

替兩個不碰 Tauri 的判斷寫測試，清掉範本留下的檔案，最後回頭看這一章做了什麼、還能怎麼改。

## 正文

### 哪些東西可以測

這個 App 幾乎都是「把視窗、系統匣、plugin 接起來」的程式，這些要有真的 App 才能試。但裡面有兩處是純粹的判斷，不需要 Tauri 就能測：

- Rust 的 `toggle_action(visible, focused)`：給兩個 `bool`，回傳要叫出來還是收起來（第 4 集）。
- 前端的 `normalizeFontSize(value)`：給任何值，回傳 12 到 32 之間的整數（第 5 集）。

這兩個函式出錯，使用者馬上感覺得到：快捷鍵按了視窗不動，或字型大小變成 `NaN`。測試只要幾行。

Rust 這邊，測試放在 `lib.rs` 最後的 `tests` 模組，三個斷言守三種情況：有顯示又有焦點才收起來；沒顯示，或有顯示卻沒焦點，都是叫出來。「沒顯示卻有焦點」實際上不會發生，但函式不該因此出錯，一併測了。

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_when_visible_and_focused() {
        assert_eq!(toggle_action(true, true), ToggleAction::Hide);
    }

    #[test]
    fn shows_when_hidden_or_not_focused() {
        assert_eq!(toggle_action(false, false), ToggleAction::Show);
        assert_eq!(toggle_action(true, false), ToggleAction::Show);
    }

    #[test]
    fn shows_even_for_an_impossible_state() {
        assert_eq!(toggle_action(false, true), ToggleAction::Show);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

`cargo test --manifest-path src-tauri/Cargo.toml` 的輸出（節錄）：

```text
running 3 tests
test tests::hides_when_visible_and_focused ... ok
test tests::shows_even_for_an_impossible_state ... ok
test tests::shows_when_hidden_or_not_focused ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

前端這邊，`normalizeFontSize` 不碰 `window`，只要 vitest 就能測，不需要 jsdom（第 9 章第 3 集）。預期值都直接寫字面的數字，不去引用 `settings.ts` 裡的常數：測試和程式用同一個常數，常數寫錯了，測試也跟著錯。四個測試，守「照用」「四捨五入」「夾回上下限」「不是數字就用預設值」。最後一個故意放了字串 `"20"`：store 檔案裡的字串不會被當成數字。

`src/settings.test.ts`：

```ts
import { expect, test } from "vitest";
import { normalizeFontSize } from "./settings";

test("範圍內的整數原樣保留", () => {
  expect(normalizeFontSize(20)).toBe(20);
});

test("小數四捨五入成整數", () => {
  expect(normalizeFontSize(18.4)).toBe(18);
  expect(normalizeFontSize(18.6)).toBe(19);
});

test("超出範圍就夾回上下限", () => {
  expect(normalizeFontSize(99)).toBe(32);
  expect(normalizeFontSize(3)).toBe(12);
});

test("不是有限的數字就用預設值", () => {
  expect(normalizeFontSize(NaN)).toBe(16);
  expect(normalizeFontSize(Infinity)).toBe(16);
  expect(normalizeFontSize("20")).toBe(16);
  expect(normalizeFontSize(undefined)).toBe(16);
});
```

`npx vitest run` 的輸出（節錄）：

```text
 RUN  v5.1.0 ~/tauri-app

 Test Files  1 passed (1)
      Tests  4 passed (4)
```

### 其餘的檔案

其餘的檔案跟第 6 集一模一樣，為了讓這一集也能獨立開始，完整貼在下面：

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

`src/settings.ts`：

```ts
import { load } from "@tauri-apps/plugin-store";

export const DEFAULT_FONT_SIZE = 16;
export const MIN_FONT_SIZE = 12;
export const MAX_FONT_SIZE = 32;

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

`identifier` 不要隨便改：`settings.json` 放在以 `identifier` 命名的 app 資料夾裡（第 7 章第 6 集），改了就找不到原本的設定。

### 清掉範本留下的檔案

範本的 `src/assets/` 放著三個 logo，這個專案沒用到。刪掉它們（下面的 `rm -r` 是 macOS、Linux 的指令；Windows 直接在檔案總管刪掉 `src/assets` 資料夾），再用 `npm run build` 確認沒有任何地方還需要它。`npm run build` 會跑 `tsc`，連 `src/settings.test.ts` 一起做型別檢查：

```bash
rm -r src/assets
npm run build
```

`npm run build` 的輸出：

```text
> tauri-app@0.1.0 build
> tsc && vite build

vite v8.3.8 building client environment for production...
transforming...
✓ 14 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                  0.82 kB │ gzip: 0.50 kB
dist/assets/index-0PKtdcm5.css   0.39 kB │ gzip: 0.23 kB
dist/assets/index-DOMfVWOK.js   18.97 kB │ gzip: 5.10 kB

✓ built in 153ms
```

`tsc` 型別檢查通過，Vite 也打包成功，表示前端沒有型別錯誤，也沒有找不到的檔案。測試檔沒有人 `import`，所以不會被打進去。

### 回頭看

快速便條的每一塊，主要靠這些東西做成：

- 沒有主視窗：設定檔 `visible: false`，`on_window_event` 擋下關閉再 `hide()`，macOS 用 `Accessory` 不佔 Dock，也要連 `app.hide()` 把焦點還給上一個 App（第 2 集）。
- 系統匣：`TrayIconBuilder` 加選單，左鍵叫出視窗、右鍵開選單，「結束」是唯一真正結束 App 的地方（第 3 集）。
- 全域快捷鍵：`is_visible()` 加 `is_focused()` 決定切換，註冊失敗只印訊息；頁面在 `focus` 事件裡把游標放進輸入區（第 4 集）。
- 設定：兩個視窗共用同一份 store，事件只按門鈴，讀出來的值要先檢查；capability 的 `windows` 要列出設定視窗（第 5 集）。
- 開機自動啟動：狀態由作業系統記著，不存進 store，每次都用 `isEnabled()` 問（第 6 集）。
- 測試：`toggle_action` 與 `normalizeFontSize` 是純粹的判斷，各用一個測試守住（第 7 集）。

這個 App 刻意用了只有桌面才有的東西（系統匣、全域快捷鍵、開機自動啟動），所以沒有設計成能搬到手機上。

### 還能怎麼改

- 便條的文字，App 一結束就沒了。想留下來，可以用第 7 章第 7 集的 store，每次輸入就存一份，叫出視窗時再讀回來；或像第 10 章那樣寫成檔案。
- 快捷鍵寫死在程式裡。想讓使用者自己改，設定視窗要能收按鍵，存進 store，再換掉已經註冊的那一組；怎麼取消註冊，這一章沒有教。
- 使用者重複啟動 App，會多出一個系統匣圖示，兩個行程搶同一組快捷鍵。開機自動啟動之後，這更容易發生。第 8 章第 7 集的 single-instance 正好適合：第二個行程來的時候，把便條叫出來。
- 範本留下的 opener plugin，這個 App 用不到。第 14 章會談為什麼權限越少越好，到時候可以連同 `opener:default` 一起拿掉。
- 開機自動啟動，在 dev 版登記的是開發版執行檔。第 15 章打包之後，再用安裝好的 App 試一次。
- 這一章只在 macOS 實測。Windows 與 Linux 的系統匣、快捷鍵與開機啟動，都是依文件整理的。

## 重點整理

- 不碰 Tauri 的判斷（`toggle_action`、`normalizeFontSize`）可以直接測：Rust 用 `#[cfg(test)]`，前端用 vitest；需要真的視窗與系統匣的部分，只能開 App 試。
- 測試的預期值寫字面的數字，不引用程式裡的常數。
- `identifier` 決定 app 資料夾，改了會找不到舊的設定。
- 刪掉範本沒用到的檔案之後跑 `npm run build`，確認沒有東西還需要它。
