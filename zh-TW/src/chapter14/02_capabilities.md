# capabilities 深入

## 本集目標

讀懂 capability 的每個欄位，用 `windows` 的 glob 讓一組視窗共用 permission，知道 `capabilities/` 底下的檔案預設全部生效，並看得懂「not allowed on window」那串訊息。

## 正文

### capability 是什麼

前面幾章改過好幾次 `capabilities/default.json`，這集把整份看完整。一份 capability 回答一件事：**哪些視窗**拿到**哪些 permission**。它是 `src-tauri/capabilities/` 底下的一個 JSON（或 TOML）檔，常用的欄位有：

| 欄位 | 意思 |
| --- | --- |
| `identifier` | 這份 capability 的名字，不能重複。 |
| `description` | 給人看的說明，不影響行為。 |
| `windows` | 哪些視窗拿到 permission，寫 label，可以用 `*` 比對任意字元（glob）。 |
| `permissions` | 放行的 permission，字串或物件（物件寫法第 3 集再說）。 |
| `platforms` | 只在這些平台生效，可以寫 `linux`、`macOS`、`windows`、`android`、`iOS`；不寫就是每個平台。 |
| `remote` | 讓從外部網址載入的頁面也拿到 permission，用 `urls` 列出網址；不寫就只有本機頁面（打包進 App 的頁面，或開發時的 `devUrl`）拿得到。 |

本書的 App 只載入自己的頁面，用不到 `remote`。如果哪天你的 App 要載入外部網站，那個網站就等於前端，要先想清楚它值不值得拿到這些 permission（第 1 集的信任界線）。

### 範例：主視窗與彈出視窗

這個 App 一啟動就有兩個視窗：範本的主視窗（label 預設是 `main`），以及在 `setup`（第 4 章第 4 集）裡建的 `popup-1`（建視窗的寫法在第 6 章第 3 集）。兩個視窗載入同一個頁面。

這個 App 沒有用 opener plugin，也用不到範本的 `greet`，所以 `lib.rs` 把這兩樣都拿掉，capability 也不放 `opener:default`：沒用到的 plugin，連同它的 permission 一起拿掉，兩邊要成對。第 11 章、第 13 章的 App 都沒用 opener，也可以這樣清。放行的越少，前端出事時能做的越少。

`src-tauri/src/lib.rs`：

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            WebviewWindowBuilder::new(app, "popup-1", WebviewUrl::App("index.html".into()))
                .title("popup-1")
                .inner_size(360.0, 240.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

頁面沿用範本的表單：在輸入框填新標題，按下按鈕，把這個視窗的標題改掉。

`src/main.ts`：

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();
const titleEl: HTMLInputElement | null = document.querySelector("#greet-input");
const msgEl: HTMLElement | null = document.querySelector("#greet-msg");
if (!titleEl || !msgEl) {
  throw new Error("找不到 #greet-input 或 #greet-msg");
}

document.querySelector("#greet-form")?.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await appWindow.setTitle(titleEl.value);
    msgEl.textContent = `${appWindow.label} 的標題改好了`;
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

capability 只放行主視窗改標題：

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "主視窗",
  "windows": ["main"],
  "permissions": ["core:default", "core:window:allow-set-title"]
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.65s
     Running `target/debug/tauri-app`
```

兩個視窗都填一個標題、按下按鈕。`main` 的標題改好了；`popup-1` 被擋下來：

```text
出錯了：window.set_title not allowed on window "popup-1", webview "popup-1", URL: local

allowed on: [windows: "main", URL: local]

referenced by: capability: default, permission: allow-set-title
```

### 讀懂「not allowed on window」

這則訊息分三段：

- `window.set_title not allowed on window "popup-1", webview "popup-1", URL: local`：哪個 command 被擋，以及是哪個視窗、webview、什麼來源的頁面發出的呼叫。webview 是視窗裡實際顯示頁面的那一塊，本書的視窗都只有一塊，label 跟視窗一樣；`URL: local` 表示頁面是本機的。
- `allowed on: [windows: "main", URL: local]`：這個 command 目前在哪裡放行了。
- `referenced by: capability: default, permission: allow-set-title`：是哪份 capability 的哪一項放行的。

第 7 章第 2 集看到的 `Permissions associated with this command` 是另一種情況：沒有任何 capability 放行這個 command。這一則多了 `allowed on`，表示別的視窗放行了，而你這個視窗不在名單裡。

### 另開一份 capability 給彈出視窗

可以把 `popup-1` 加進 `default.json` 的 `windows`，但彈出視窗通常不該拿到跟主視窗一樣多的 permission，所以另開一份：

`src-tauri/capabilities/popup.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "popup",
  "description": "彈出視窗",
  "windows": ["popup-*"],
  "permissions": ["core:window:allow-set-title"]
}
```

`popup-*` 是 glob，`popup-1`、`popup-2` 都比對得到。一個視窗被幾份 capability 比對到，拿到的就是全部 permission 加起來。

`npm run tauri dev` 的輸出（新增 `capabilities/popup.json` 之後）：

```text
        Info File src-tauri/capabilities/popup.json changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.38s
     Running `target/debug/tauri-app`
```

這次兩個視窗都改得了標題。

### 只啟用想要的 capability

`capabilities/` 底下的每個檔案預設都生效，包括你忘了刪的、複製來試試看的。想自己掌控，就在 `tauri.conf.json` 的 `app.security` 加一行 `"capabilities": ["default", "popup"]`，列出要用的 `identifier`，沒列到的檔案不生效。行為不變，兩個視窗還是都能改標題；把 `"popup"` 拿掉再存檔，`popup.json` 就失效，`popup-1` 又會被擋，訊息跟上面一樣。

## 重點整理

- capability 是「哪些視窗（`windows`，可用 glob）拿到哪些 permission」。一個視窗拿到的是所有比對到它的 capability 的聯集。
- `not allowed on window` 的訊息告訴你：被擋的 command、發出呼叫的視窗、現在在哪裡放行、是哪份 capability 放行的。
- `capabilities/` 底下全部都生效，用 `app.security.capabilities` 列出 `identifier` 才能只啟用想要的。
- `platforms` 限定平台；`remote.urls` 讓外部網址的頁面也拿到 permission，預設只有本機頁面。
- 用不到的 plugin 連同它的 permission 一起拿掉。
