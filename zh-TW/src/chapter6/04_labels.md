# 多個視窗與 label

## 本集目標

知道 label 是視窗的唯一名字：同名的視窗不能再建，要先用 label 找找看有沒有開著的視窗。

## 正文

### label 不能重複

建視窗時，label 是你取的字串。字母、數字（非 ASCII 的字母也算，例如中文）與 `-`、`/`、`:`、`_` 都可以用，句點、空白這類字元不行，`build()` 會回傳 `Err`。

一個 App 裡，每個 label 也只能有一個視窗。上一集的 `open_about` 再按一次，`build()` 同樣回傳 `Err`，前端的 Promise 被拒絕，就是上一集段落裡的那則錯誤。所以建視窗之前，先看看是不是已經開過了。

### 用 label 找視窗

`Manager` 的 `get_webview_window(label)` 回傳 `Option<WebviewWindow>`：有這個視窗就是 `Some`。找到的話，把它叫回最前面，不用再建一個：

`src-tauri/src/lib.rs`：

```rust
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
async fn open_window(app: AppHandle, label: String) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(&label) {
        window.show()?;
        window.unminimize()?;
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

`get_webview_window` 定義在 `Manager` 上，要先 `use`，跟第 4 章第 4 集一樣。

叫回視窗要三個一起用：`show()` 讓藏起來的視窗現身（第 13 章會正式介紹），`unminimize()` 還原最小化的視窗，最後 `set_focus()` 拉到最前面。依 Tauri 底層視窗函式庫 tao 的原始碼，macOS 上 `set_focus()` 碰到隱藏或最小化的視窗什麼都不做，所以三個都要。

前端用一個輸入框填 label，按下按鈕就呼叫 `open_window`。每個視窗載入時，也把自己的 label 寫在畫面上：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>多個視窗</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1 id="label-msg"></h1>
      <form class="row" id="open-form">
        <input id="label-input" placeholder="新視窗的 label" />
        <button type="submit">開視窗</button>
      </form>
      <p id="open-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const labelMsgEl: HTMLElement | null = document.querySelector("#label-msg");
const openMsgEl: HTMLElement | null = document.querySelector("#open-msg");
const formEl: HTMLFormElement | null = document.querySelector("#open-form");
const inputEl: HTMLInputElement | null = document.querySelector("#label-input");

if (labelMsgEl) {
  labelMsgEl.textContent = `這是視窗：${getCurrentWindow().label}`;
}

formEl?.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!inputEl || !openMsgEl) {
    return;
  }
  try {
    await invoke("open_window", { label: inputEl.value });
    openMsgEl.textContent = "";
  } catch (error) {
    openMsgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.97s
     Running `target/debug/tauri-app`
```

主視窗顯示「這是視窗：main」。輸入 `note-1` 按下按鈕，開出一個標題是 `note-1` 的視窗，畫面寫著「這是視窗：note-1」。在主視窗再輸入一次 `note-1`，沒有錯誤，只是 `note-1` 跳到最前面。輸入 `note.1`（有句點），段落出現錯誤：

```text
出錯了：runtime error: Window labels must only include alphanumeric characters, `-`, `/`, `:` and `_`.
```

第 3 章第 9 集說過，command 參數裡的 `WebviewWindow` 是呼叫它的視窗：同一個 command 從 `note-1` 呼叫，拿到的就是 `note-1`，不再是 `main`。

範本的 capability 只把 permission 給 label 是 `main` 的視窗，所以 `note-1` 呼叫 `title()` 這類需要 permission 的方法會被擋。這一集的新視窗只讀 `label`，加上 `invoke` 自己寫的 command（我們還沒替它們定義過 permission，Tauri 不會檢查），所以沒事。capability 怎麼涵蓋更多視窗，第 14 章會正式介紹。

### 跟 Electron 比一比

在 Electron 裡，同一段 `new BrowserWindow()` 執行幾次，就開幾個視窗，系統給每個視窗一個數字 `id`，不會報錯。照這個直覺寫 Tauri，像上一集的 `open_about` 連按兩次，第二次就失敗了：Tauri 的視窗由你命名，label 就是身分，不能重複。所以要開視窗之前，先用 `get_webview_window` 找找看。

## 重點整理

- label 只能用字母、數字（非 ASCII 的字母也算）與 `-`、`/`、`:`、`_`，其他字元會讓 `build()` 回傳 `Err`。
- label 是視窗的唯一名字：跟已開的視窗同名，`build()` 一樣回傳 `Err`。
- `app.get_webview_window(label)` 回傳 `Option<WebviewWindow>`，要先 `use tauri::Manager`；找到就 `show()`、`unminimize()`、`set_focus()` 叫回來，不要重複建。
