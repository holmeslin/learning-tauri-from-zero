# 從 Rust 開新視窗：`WebviewWindowBuilder`

## 本集目標

學會在 command 裡用 `WebviewWindowBuilder` 開一個新視窗，指定它的 label、載入的頁面、標題與大小。

## 正文

### 用 Builder 建視窗

設定檔只能在 App 啟動時開固定的視窗。執行到一半才要開的視窗，像「關於」或「設定」，就用 Rust 建。`WebviewWindowBuilder::new` 要三樣東西：

- 第一個：誰來管這個視窗。拿 `AppHandle` 的參考就行（第 3 章第 9 集）。
- 第二個：視窗的 label，每個視窗都要有一個，下一集細講。
- 第三個：`WebviewUrl::App(路徑)`，視窗載入哪一頁。開發時是 `devUrl` 加上這個路徑，打包後是 `frontendDist` 裡的檔案。

接著用 `.title(...)`、`.inner_size(...)` 這類方法設定，最後 `.build()` 才真的開出來。方法的名字對應設定檔的欄位，只是改成 `snake_case`：`title`、`center`、`resizable`、`min_inner_size`。`inner_size(寬, 高)` 對應 `width`、`height`，單位同樣是邏輯像素。

### 範例：按鈕開「關於」視窗

command 是 `async fn`，原因寫在後面。新視窗載入跟主視窗同一頁 `index.html`，我們不另外寫 HTML：

`src-tauri/src/lib.rs`：

```rust
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
async fn open_about(app: AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(&app, "about", WebviewUrl::App("index.html".into()))
        .title("關於")
        .inner_size(360.0, 240.0)
        .center()
        .build()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_about])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`build()` 回傳 `tauri::Result<WebviewWindow>`，`?` 把失敗往外丟，成功的視窗物件這裡用不到。視窗屬於 App，不是屬於這個變數，函式結束了視窗也不會關。

兩個視窗載入同一頁，所以頁面要自己說「我是誰」。前端的 `getCurrentWindow().label` 就是這個視窗的 label，它只是一個欄位，不用 `await`，也不需要 permission：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>新視窗</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1 id="label-msg"></h1>
      <div class="row">
        <button id="open-btn" type="button">開新視窗</button>
      </div>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const labelMsgEl: HTMLElement | null = document.querySelector("#label-msg");

if (labelMsgEl) {
  labelMsgEl.textContent = `這是視窗：${getCurrentWindow().label}`;
}

document.querySelector("#open-btn")?.addEventListener("click", () => {
  invoke("open_about");
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.93s
     Running `target/debug/tauri-app`
```

主視窗顯示「這是視窗：main」。按「開新視窗」，螢幕中央多了一個 360×240 的視窗，標題是「關於」，畫面寫著「這是視窗：about」。兩個視窗各自獨立，關掉其中一個，另一個還在。再按一次「開新視窗」會失敗，下一集說明。

### 為什麼是 `async fn`

第 3 章第 8 集說過，同步 command 在主執行緒上跑。官方文件把這件事列為已知問題：在 Windows 上，於同步 command 或事件處理函式裡建視窗會卡死（deadlock），建議用 `async` command 或另開執行緒。我在 macOS 上試過同步版本沒有問題，但建視窗的 command 一律寫成 `async fn` 最保險。

### 為什麼不另外寫一個 HTML

`WebviewUrl::App("settings.html".into())` 也可以載入別的檔案。但 Vite 打包時預設只處理 `index.html`，其他 HTML 要在 Vite 的設定裡另外列為入口，這本書不展開。所以我們都讓新視窗載入 `index.html`，靠 `label` 決定要畫什麼。

## 重點整理

- `WebviewWindowBuilder::new(&app, label, WebviewUrl::App(路徑))` 開始建構，`.build()` 才會建出視窗，回傳 `tauri::Result<WebviewWindow>`。
- Builder 的方法對應設定檔的欄位：`title`、`inner_size`（寬高，邏輯像素）、`center`、`resizable`、`min_inner_size`。
- 建視窗的 command 寫成 `async fn`：官方文件指出 Windows 上在同步 command 裡建視窗會卡死。
- 新視窗載入同一個 `index.html` 時，用前端的 `getCurrentWindow().label` 分辨自己是哪個視窗。
