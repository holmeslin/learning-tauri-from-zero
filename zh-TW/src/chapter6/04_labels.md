# 多個視窗與 label

## 本集目標

知道 label 是視窗的唯一名字：同名的視窗不能再建，可以用 label 找回已經開著的視窗，新視窗的 label 也要列進 capability。

## 正文

### label 不能重複

建視窗時，label 是你取的字串，只能用字母、數字與 `-`、`/`、`:`、`_`，違反的話 `build()` 回傳 `Err`。一個 App 裡每個 label 也只能有一個視窗：上一集的 `open_about` 再按一次，`build()` 回傳 `Err`，前端的 Promise 被拒絕，錯誤訊息是 ``a webview with label `about` already exists``。所以建視窗之前，先看看是不是已經開過了。

### 用 label 找視窗

`Manager` 的 `get_webview_window(label)` 回傳 `Option<WebviewWindow>`：有這個視窗就是 `Some`。找到的話，用 `set_focus()` 把它拉到最前面，不用再建一個：

`src-tauri/src/lib.rs`：

```rust
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
async fn open_window(app: AppHandle, label: String) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(&label) {
        return window.set_focus();
    }
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
        .title(&label)
        .inner_size(360.0, 240.0)
        .build()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`get_webview_window` 定義在 `Manager` 上，要先 `use`，跟第 4 章第 4 集一樣。`app.webview_windows()` 則回傳所有視窗，是 label 對上視窗的 `HashMap`。

前端借用範本的 Greet 表單，`index.html` 不用動：輸入框填 label，按 Greet 就呼叫 `open_window`。每個視窗載入時，也把自己的 label 與標題寫在 `#greet-msg`：

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();
const formEl: HTMLFormElement | null = document.querySelector("#greet-form");
const inputEl: HTMLInputElement | null = document.querySelector("#greet-input");
const msgEl: HTMLElement | null = document.querySelector("#greet-msg");

async function showTitle(): Promise<void> {
  if (!msgEl) {
    return;
  }
  try {
    msgEl.textContent = `視窗 ${appWindow.label} 的標題：${await appWindow.title()}`;
  } catch (error) {
    msgEl.textContent = `視窗 ${appWindow.label} 出錯了：${error}`;
  }
}

showTitle();

formEl?.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!inputEl || !msgEl) {
    return;
  }
  try {
    await invoke("open_window", { label: inputEl.value });
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.97s
     Running `target/debug/tauri-app`
```

主視窗顯示「視窗 main 的標題：tauri-app」。輸入 `note-1` 按 Greet，開出一個標題是 `note-1` 的視窗，可是它的段落是：

```text
視窗 note-1 出錯了：window.title not allowed on window "note-1", webview "note-1", URL: local allowed on: [windows: "main", URL: local] referenced by: capability: default, permission: allow-title
```

在主視窗再輸入一次 `note-1`，沒有錯誤，只是 `note-1` 跳到最前面。

### capability 只認得 `main`

範本的 `capabilities/default.json` 寫著 `"windows": ["main"]`：這份 permission 只給 label 是 `main` 的視窗。訊息裡的 `allowed on: [windows: "main"…]` 就是在說這件事。`note-1` 不在清單裡，連 `core:default` 的 `title()` 都不能用。自己寫的 command 不受影響：`note-1` 裡一樣能呼叫 `open_window`。第 3 章第 9 集說過，command 參數裡的 `WebviewWindow` 是呼叫它的視窗：同一個 command，從 `note-1` 呼叫，拿到的就是 `note-1` 這個視窗，不再是 `main`。

`windows` 可以寫 label，也可以用 `*` 當萬用字元。`note-*` 涵蓋所有 `note-` 開頭的視窗：

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main", "note-*"],
  "permissions": [
    "core:default",
    "opener:default"
  ]
}
```

`npm run tauri dev` 的輸出（改了 `capabilities/default.json` 之後）：

```text
        Info File src-tauri/capabilities/default.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.88s
     Running `target/debug/tauri-app`
```

重新開 `note-1`，段落變成「視窗 note-1 的標題：note-1」。之後開 `note-2`、`note-3` 也都有同樣的 permission。

### 跟 Electron 比一比

Electron 的 `BrowserWindow` 由系統編一個數字 `id`，你拿著視窗物件，或用 `BrowserWindow.getAllWindows()` 去找。Tauri 的視窗由你命名，名字就是身分：不能重複，找視窗、送事件（第 5 章第 4 集的 `emit_to`）、設定 permission，全都靠 label。

## 重點整理

- label 是視窗的唯一名字，只能用字母、數字與 `-`、`/`、`:`、`_`；同名的視窗再建會得到 `Err`。
- `app.get_webview_window(label)` 回傳 `Option<WebviewWindow>`，要先 `use tauri::Manager`；找到就 `set_focus()`，不要重複建。
- capability 的 `windows` 依 label 比對，可以用 `*` 萬用字元；新視窗的 label 不在清單裡，就連 `core:default` 都用不了。
- 自己寫的 command 不看 capability，從任何視窗都能呼叫。
