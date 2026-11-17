# `import`/`export`、npm 與 Vite

## 本集目標

學會把程式拆成多個檔案，用 `export` 和 `import` 串起來，也知道 npm 套件與 Vite 在其中各做什麼。

## 正文

### 一個檔案就是一個模組（module）

這一集把程式拆成兩個檔案。新增一個 `src/greeting.ts`，裡面的東西用 `export` 開放出去：

`src/greeting.ts`：

```ts
export const appName = "我的第一個 Tauri App";

export function makeGreeting(who: string): string {
  return `你好，${who}！歡迎使用 ${appName}`;
}
```

`src/main.ts` 用 `import` 把要的東西拿進來：

`src/main.ts`：

```ts
import { makeGreeting } from "./greeting";

const msgEl: HTMLElement | null = document.querySelector("#msg");
if (msgEl) {
  msgEl.textContent = makeGreeting("Tauri");
}
```

頁面只要一個放訊息的段落：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>模組</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>模組</h1>
      <p id="msg"></p>
    </main>
  </body>
</html>
```

存檔後，頁面顯示「你好，Tauri！歡迎使用 我的第一個 Tauri App」。

### `export` 與 `import`

- `export`：放在宣告前面，這個名字就能被別的檔案拿去用；沒加 `export` 的只有這個檔案看得到。`greeting.ts` 開放了 `appName` 和 `makeGreeting`。
- `import { 名字 } from "路徑"`：把別的檔案開放的東西拿進來，大括號裡可以列好幾個，用不到的不必列。`main.ts` 只拿了 `makeGreeting`。
- 路徑開頭的 `./` 表示「同一個資料夾」，不用寫副檔名。

對 Rust 讀者來說，`export` 像 `pub`，`import` 像 `use`。差別是不用寫 `mod`：每個檔案自動就是一個模組。

### npm 套件

`import` 的路徑如果不是以 `./` 開頭，像範本的這一行：

```ts,ignore
import { invoke } from "@tauri-apps/api/core";
```

意思就變成「去 `node_modules/` 找這個套件」。`@tauri-apps/api` 是 Tauri 發佈在 npm 上的套件，`core` 是它的一個部分，`invoke` 是 `core` 開放出來的函式。`invoke` 怎麼用，第 3 章會正式介紹，這集只要看懂這行的結構。

這跟 Rust 的 Cargo 是同一個思路：

| Rust | 前端 |
| --- | --- |
| `Cargo.toml` 的 `[dependencies]` | `package.json` 的 `dependencies` |
| `Cargo.lock` | `package-lock.json` |
| `cargo add` | `npm install 套件名稱` |
| 下載的 crate | `node_modules/` 裡的套件 |

第 1 章第 4 集看過 `package.json`。這一集不用裝任何新套件。

### Vite 在做什麼

第 1 章用過的 Vite，工作有兩件：

- **開發時**（`tauri dev`）：它是一個伺服器，視窗要哪個檔案就送哪個。送出 `.ts` 之前先轉成 JavaScript（第 4 集說過），也把 `@tauri-apps/api/core` 這種套件路徑換成瀏覽器找得到的位置。這就是 `main.ts` 能直接 `import` 的原因。
- **打包時**（`npm run build`）：先用 `tsc` 檢查型別，再用 `vite build` 把所有模組合併、壓縮，放進 `dist/`。第 1 章第 6 集說過，正式版的 App 載入的就是 `dist/` 裡的檔案。

在專案資料夾執行：

```bash
npm run build
```

`npm run build` 的輸出：

```text
> tauri-app@0.1.0 build
> tsc && vite build

vite v8.3.8 building client environment for production...
transforming...
✓ 6 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                 0.45 kB │ gzip: 0.30 kB
dist/assets/index-0Y_Rzos9.css  1.40 kB │ gzip: 0.65 kB
dist/assets/index-BODUox0R.js   0.82 kB │ gzip: 0.51 kB

✓ built in 177ms
```

`main.ts` 和 `greeting.ts` 被合併成 `dist/assets/` 裡一個 `.js` 檔。檔名中間那串雜湊（`0Y_Rzos9` 這種）是依檔案內容算出來的：程式碼和 Vite 版本沒變，重新打包出來的檔名和大小就一樣，只有最後一行的 `built in` 時間每次不同；內容一改，雜湊就跟著變。你的數字如果跟這裡不一樣，通常是程式碼或 Vite 版本不同。

## 重點整理

- 一個檔案就是一個模組；加上 `export` 才能被別的檔案 `import`，用法像 Rust 的 `pub` 與 `use`。
- `import { x } from "./file"` 引入自己的檔案，`from "套件名稱"` 引入 npm 套件。
- npm 對應 Cargo：`package.json`、`package-lock.json`、`node_modules/` 分別像 `Cargo.toml`、`Cargo.lock` 與下載的 crate。
- Vite 開發時負責轉換並送出檔案，`npm run build` 先用 `tsc` 檢查，再用 Vite 打包進 `dist/`。
