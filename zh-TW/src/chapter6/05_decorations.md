# 無邊框與自訂標題列

## 本集目標

學會用 `decorations: false` 拿掉系統的標題列，再用 HTML 自己畫一條，加上 `data-tauri-drag-region` 讓它能拖動視窗。

## 正文

### 拿掉系統的標題列

視窗設定有一項 `decorations`，預設是 `true`：由作業系統畫標題列、邊框與按鈕。設成 `false`，這些全部消失，只剩網頁內容。Builder 上對應的是 `.decorations(false)`。

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
        "title": "自訂標題列",
        "width": 480,
        "height": 320,
        "decorations": false
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

沒有標題列，視窗就拖不動，也關不掉。得自己做一條。

### 自己畫標題列

標題列是頁面最上面的一個 `<div>`：一段標題、兩個按鈕。要讓它能拖動視窗，加上 `data-tauri-drag-region` 這個屬性。外觀用 HTML 的 `style` 屬性直接寫 CSS 宣告（第 2 章第 3 集），就不用另外動 `styles.css`；標題 `<span>` 設 `flex: 1`，把按鈕擠到右邊：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>自訂標題列</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body style="margin: 0">
    <div
      data-tauri-drag-region="deep"
      style="display: flex; align-items: center; gap: 8px; padding: 4px 12px; color: white; background-color: #396cd8"
    >
      <span style="flex: 1">自訂標題列</span>
      <button id="min-btn" type="button">－</button>
      <button id="close-btn" type="button">✕</button>
    </div>
    <p>拖動上面的藍色列，可以移動視窗。</p>
  </body>
</html>
```

兩個按鈕呼叫第 2 集學過的視窗方法：

`src/main.ts`：

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

document.querySelector("#min-btn")?.addEventListener("click", () => appWindow.minimize());
document.querySelector("#close-btn")?.addEventListener("click", () => appWindow.close());
```

拖動、最小化、關閉都要先放行，一次加進去：

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "core:window:allow-minimize",
    "core:window:allow-close",
    "opener:default"
  ]
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.04s
     Running `target/debug/tauri-app`
```

視窗沒有系統標題列，最上面是藍色的自訂列。按住它拖動，視窗跟著移動；「－」把視窗縮小，「✕」關閉視窗。

### `data-tauri-drag-region` 的規則

- 屬性值只寫 `data-tauri-drag-region`（或 `"true"`）時，只有直接點在那個元素本身才會拖動，點到裡面的標題文字就拖不動。範例用 `"deep"`，元素裡面的東西都算拖動區。
- 按鈕、連結、輸入框這類可以點的元素不會觸發拖動，所以 `deep` 底下的兩個按鈕照常能按。
- 拖動要 `core:window:allow-start-dragging`，它不在 `core:default` 裡。漏掉的話，按住標題列沒有反應，Console 會出現缺少這項 permission 的錯誤。

### 跟 Electron 比一比

Electron 的無邊框視窗（`frame: false`）用 CSS 的 `-webkit-app-region: drag` 指定拖動區。Tauri 用 HTML 屬性，而且拖動要先經過 permission 放行。

## 重點整理

- `decorations: false`（或 Builder 的 `.decorations(false)`）拿掉系統標題列，拖動與按鈕都得自己做。
- 元素加上 `data-tauri-drag-region` 就能拖動視窗；整個元素內都要能拖，值寫 `"deep"`。
- 拖動、`minimize()`、`close()` 各需要 `core:window:allow-start-dragging`、`allow-minimize`、`allow-close`。
