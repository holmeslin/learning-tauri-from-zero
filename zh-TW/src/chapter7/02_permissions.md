# permission 初探

## 本集目標

知道 `capabilities/default.json` 的 `permissions` 決定前端能呼叫什麼，看得懂「not allowed」錯誤訊息裡的 permission 名稱，並會補上缺的那一項。

## 正文

### 前端要先被放行

第 6 章已經遇過：前端想改視窗標題、關視窗，沒被放行就會被擋下。plugin 也一樣。plugin 裝好了，前端呼叫它的功能時，Tauri 還要檢查 `src-tauri/capabilities/default.json` 的 `permissions` 清單，沒列在裡面就拒絕。

### 範例：漏掉 permission

上一集的 `tauri add dialog` 會替你把 permission 一起加好。這一集假設是手動安裝，只做了 `lib.rs` 那一步，漏掉 permission：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
    <title>permission</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>permission</h1>
      <div class="row">
        <button id="hello-btn" type="button">打開對話框</button>
      </div>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { message } from "@tauri-apps/plugin-dialog";

const msgEl: HTMLElement | null = document.querySelector("#msg");
if (!msgEl) {
  throw new Error("找不到 #msg");
}

document.querySelector("#hello-btn")?.addEventListener("click", async () => {
  try {
    await message("permission 放行了", { title: "permission" });
    msgEl.textContent = "對話框關掉了";
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

`capabilities/default.json` 沒動，還是範本的樣子，裡面只有 `core:default` 和 `opener:default`。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s
     Running `target/debug/tauri-app`
```

按下按鈕，沒有對話框，段落出現：

```text
出錯了：dialog.message not allowed. Permissions associated with this command: dialog:allow-ask, dialog:allow-confirm, dialog:allow-message, dialog:default
```

### 讀懂錯誤訊息

- `dialog.message not allowed`：dialog plugin 的 `message` command 被拒絕。這發生在 command 真正執行之前。
- `Permissions associated with this command`：冒號後面列的 permission，列在 `permissions` 裡的任何一項都能放行它。

permission 的名稱是 `<plugin>:<名稱>`，常見的有兩種：

- `dialog:allow-message`：只放行一個 command。`allow-` 後面是 command 的名稱，底線換成連字號，例如 `open_url` 變成 `opener:allow-open-url`。
- `dialog:default`：plugin 作者挑好的一組常用 permission，一次放行好幾個。

清單裡還有 `allow-ask` 和 `allow-confirm`，是因為 `ask()`、`confirm()` 這兩個函式在背後也是呼叫 `message` command。第 6 章看到的 `core:window:allow-set-title` 是同樣的格式，只是 plugin 名稱換成 `core`。

### 補上 permission

把 `dialog:default` 加進 `permissions`：

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default"
  ]
}
```

`npm run tauri dev` 的輸出（改了 `capabilities/default.json` 之後）：

```text
        Info File src-tauri/capabilities/default.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.71s
     Running `target/debug/tauri-app`
```

再按一次按鈕，對話框彈出來，按「OK」之後畫面出現「對話框關掉了」。

### 只放行用得到的

這個範例只用了 `message`，其實只放行 `dialog:allow-message` 就夠了，對話框一樣能彈。`dialog:default` 還多放行了選檔、存檔的對話框（第 4 集會用）。原則是用到什麼放行什麼：放行得越少，前端出狀況時能做的事越少。

### 怎麼知道該加哪一項

- 看錯誤訊息，最後一段就是答案。
- `default.json` 裡的 `$schema` 指向的 `src-tauri/gen/schemas/desktop-schema.json` 是編譯時自動產生的，裡面有所有 permission 的名稱和說明。支援 JSON Schema 的編輯器會在你輸入時列出來。

## 重點整理

- 前端呼叫 plugin 的功能，要先在 `capabilities/default.json` 的 `permissions` 放行，沒放行就會被擋下，加了就能用。
- 錯誤訊息 `<plugin>.<command> not allowed` 的最後一段，列出能放行它的 permission。
- 名稱是 `<plugin>:<名稱>`：`allow-<command>` 放行單一個 command，`default` 是 plugin 挑好的一組。
- 用到什麼放行什麼；`tauri add` 會替你加 `<plugin>:default`。
