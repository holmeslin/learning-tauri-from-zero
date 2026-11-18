# notification：系統通知

## 本集目標

學會用 notification plugin 送出系統通知，知道桌面版不會詢問通知權限，以及開發模式在 macOS 上通知是以「終端機」的名義送出的。

## 正文

### 三個函式

- `isPermissionGranted()`：現在有沒有通知的權限，回傳 `Promise<boolean>`。
- `requestPermission()`：向使用者要權限，回傳 `"granted"`、`"denied"` 或 `"default"`。
- `sendNotification({ title, body })`：送出通知。它回傳 `void`，不是 Promise，所以不知道通知有沒有真的顯示。

這裡的「權限」是作業系統層級的通知授權，跟 Tauri 的 permission 是兩回事。標準寫法是先問、被拒就不送。桌面版的 plugin 不會跳出詢問視窗：`isPermissionGranted()` 一律是 `true`，`requestPermission()` 一律是 `"granted"`。手機版（第 18 章會正式介紹）才需要真的問，所以這套寫法兩邊通用。

### 範例：按一下，送一則通知

Rust 端用 `init()` 註冊，permission 是 `notification:default`：

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

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
    "notification:default"
  ]
}
```

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>notification</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>notification</h1>
      <div class="row">
        <button id="notify-btn" type="button">送出通知</button>
      </div>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

const msgEl: HTMLElement | null = document.querySelector("#msg");

document.querySelector("#notify-btn")?.addEventListener("click", async () => {
  if (!msgEl) {
    return;
  }
  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      granted = (await requestPermission()) === "granted";
    }
    if (!granted) {
      msgEl.textContent = "沒有通知權限";
      return;
    }
    sendNotification({ title: "tauri-app", body: "從 Tauri 送出的通知" });
    msgEl.textContent = "已送出";
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.39s
     Running `target/debug/tauri-app`
```

按下「送出通知」，段落顯示「已送出」。

### 開發模式在 macOS 上

macOS 的通知要掛在某個 App 的名下，由系統決定顯不顯示。打包後的 App 用自己的 identifier；但 `npm run tauri dev` 跑的不是 `.app`，plugin 改用「終端機」（`com.apple.Terminal`）的名義送出。所以在開發模式，通知有沒有出現，看的是系統對「終端機」的通知設定。

在 macOS 上實測：程式回報「已送出」，但螢幕上沒有橫幅。系統記錄顯示，`com.apple.Terminal` 的通知授權是 `Denied`，通知被丟棄，前端完全不會知道。所以開發模式看不到通知時，先別懷疑程式。系統允許之後的橫幅長什麼樣子、打包後的 App 的行為（第 15 章會正式介紹打包），這一集沒有驗證。

### 沒有 permission 的時候

少了 `notification:default`，第一步 `isPermissionGranted()` 就被擋，段落顯示：

```text
出錯了：notification.is_permission_granted not allowed. Permissions associated with this command: notification:allow-is-permission-granted, notification:default
```

## 重點整理

- 三個函式：`isPermissionGranted()`、`requestPermission()`、`sendNotification({ title, body })`；先問再送是通用的寫法。
- 桌面版不會詢問使用者，前兩個函式一律回報已授權。
- `sendNotification` 沒有回傳值，系統若把通知丟掉，前端不會知道。
- macOS 開發模式的通知以「終端機」的名義送出，要看到橫幅，終端機得先被允許送通知。
- permission 是 `notification:default`，少了它連 `isPermissionGranted()` 都會被擋。
