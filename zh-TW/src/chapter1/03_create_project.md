# 用 `create-tauri-app` 建專案

## 本集目標

用 `create-tauri-app` 建出本書所有範例共用的起始專案，並裝好前端套件。

## 正文

### 建立專案

在你放程式的資料夾打開終端機，執行：

```bash
npm create tauri-app@latest
```

`npm create` 會下載並執行 `create-tauri-app` 這個工具。第一次執行時，npm 可能會先問 `Ok to proceed? (y)`，輸入 `y` 就好。接著工具會問六個問題，用方向鍵選、Enter 確定。請照下表回答，之後每一集才會跟你的專案一致：

| 問題 | 回答 |
| --- | --- |
| Project name | `tauri-app`（預設值，直接按 Enter） |
| Identifier | `tw.example.tauri-app` |
| Choose which language to use for your frontend | `TypeScript / JavaScript` |
| Choose your package manager | `npm` |
| Choose your UI template | `Vanilla` |
| Choose your UI flavor | `TypeScript` |

Identifier 是 App 的唯一代號，預設值是依你的使用者名稱產生的，請改成 `tw.example.tauri-app`。它會寫進設定檔，第 6 集會看到。`Vanilla` 的意思是不用 Vue、React 這類前端框架，整本書都是這樣。

全部回答完，畫面上會留下這些：

`npm create tauri-app@latest` 的輸出：

```text
✔ Project name · tauri-app
✔ Identifier · tw.example.tauri-app
✔ Choose which language to use for your frontend · TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)
✔ Choose your package manager · npm
✔ Choose your UI template · Vanilla
✔ Choose your UI flavor · TypeScript

Template created! To get started run:
  cd tauri-app
  npm install
  npm run tauri android init
  npm run tauri ios init

For Desktop development, run:
  npm run tauri dev

For Android development, run:
  npm run tauri android dev

For iOS development, run:
  npm run tauri ios dev
```

`android` 與 `ios` 那幾行是行動裝置用的，第 18 章才會用到。

### 安裝前端套件

進到專案資料夾，安裝前端的相依套件：

```bash
cd tauri-app
npm install
```

`npm install` 的輸出：

```text
added 20 packages, and audited 21 packages in 10s

10 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities
```

`npm install` 會照 `package.json` 把前端用到的套件裝進 `node_modules/`：Tauri 的命令列工具（`@tauri-apps/cli`）、前端呼叫 Tauri 用的 `@tauri-apps/api`、opener plugin 的前端套件，還有 Vite 和 TypeScript。Rust 這一側的 crate 不在這一步，等第一次執行 `tauri dev` 時 cargo 才會下載、編譯。套件數量和耗時依你的網路而不同。

專案就建好了。範本附了一份 VS Code 的建議（`.vscode/extensions.json`）：安裝 Tauri 和 rust-analyzer 兩個擴充套件。用別的編輯器也可以。

之後每一集貼出來的檔案都是完整內容，可以直接覆蓋專案裡的同名檔案。沒貼出來的檔案，就維持剛建好時的樣子。

## 重點整理

- 用 `npm create tauri-app@latest` 建專案，依序選 `Vanilla` 與 `TypeScript`，套件管理選 `npm`。
- 專案名稱用 `tauri-app`，Identifier 用 `tw.example.tauri-app`，跟本書一致。
- 建好後進到專案資料夾，執行 `npm install` 安裝前端套件。
- Rust 的 crate 要等第一次 `tauri dev` 才會下載編譯。
