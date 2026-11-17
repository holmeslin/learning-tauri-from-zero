# 寫作風格指南

撰寫或修改 `zh-TW/src/` 內容前請先讀完。名詞定義與譯名見根目錄 `GLOSSARY.md`，全書目錄見 `docs/outline.md`。

## 讀者與語氣

- 讀者**讀完了《從零開始學 Rust》**：會 Rust、也會 Go，但**沒寫過前端、沒用過 Tauri**。Rust 語法不用再教；要教的是前端基礎（第 2 章）與 Tauri 本身。
- 繁體中文原生撰寫（台灣用語：程式、函式、變數、物件、預設、檔案、介面、視窗、選單、陣列、記憶體），不是翻譯腔。
- 口語、友善、短句，用「我們」「你」。不要說「這很簡單」「顯然」。
- 先講直覺，再上程式碼。不寫成 API 文件。
- 中英文之間加半形空格；程式碼、識別字、指令用反引號。
- 原創撰寫。不參考、不改寫、不翻譯其他教學的文字，包括 Tauri 官方文件與其他 Tauri 教學。官方文件只拿來查證 API 與行為。

## 每一集的格式

檔名 `##_slug.md`，標題與 `SUMMARY.md` 的標題一致：

```markdown
# <與 SUMMARY 相同的標題>

## 本集目標

一兩句話：讀完這集你會什麼。

## 正文

用 `###` 小標分段。每段一個重點，短範例 + 白話解說。

### 跟 Electron 比一比（選用）

## 重點整理

- 3～5 條，條列本集真正教了什麼。
```

- 一集只教**一個**小概念，篇幅大約 40～150 行 markdown。第二部（實戰）每集是專案的一個零件，要貼好幾個檔案，上限放寬到約 200 行。
- 每章的 `README.md` 只有 `# <章名>` 加一兩段導言，說明這章要學什麼、為什麼重要，不列目錄。

### 跟 Electron 比一比

- 只有在 Tauri 跟 Electron **不一樣、而且照 Electron 的直覺會出錯**時才加，放在正文最後一個小節（行程模型、IPC、檔案大小、權限、Node.js API）。
- 讀者不一定用過 Electron：每次比較都先用一句話說 Electron 怎麼做，再說 Tauri 哪裡不同。
- 重點是「別的框架的直覺在這裡會錯在哪」，不是比較誰比較好。

## 進度邊界（最重要）

- 範例與說明**只能使用 `SUMMARY.md` 中排在本集之前已教過的前端語法與 Tauri 功能**。Rust 語法不受限制（讀者已經會了），但用到《從零開始學 Rust》沒教的 crate 時要先介紹。
- 不得不提到後面的內容時，用一句話帶過並標明「第 X 章會正式介紹」，不展開。
- 不要「順便」介紹本集主題以外的 API。
- `docs/outline.md` 各章下方的「邊界」是額外規定，寫該章前先看。

## 起始專案

- 每一集都從同一個起點出發：`npm create tauri-app@latest` 選 vanilla-ts 範本產生的專案，也就是 `tools/checkcode/template/`（專案名 `tauri-app`）。第 1 章第 3 集帶讀者建出這個專案。
- 一集裡沒貼出來的檔案，就是範本原本的內容。要沿用上一集改過的檔案，這一集要再貼一次（一集是一個獨立的專案）。

## 程式碼區塊

- Rust 用 `rustfmt`（edition 2024）格式，縮排 4 個空格；TypeScript、HTML、CSS、JSON 縮排 2 個空格。
- **路徑標籤**：程式區塊前一行寫「`src-tauri/src/lib.rs`：」這種整行只有路徑加全形冒號的標籤（可以在冒號前加「（說明）」），區塊內容就是這個檔案的**完整內容**。路徑從專案根目錄算起。
- **檢查點**：「`npm run tauri dev` 的輸出：」加 ```` ```text ```` 區塊。`tools/checkcode` 在這一刻把所有檔案放進範本，執行：
  - `cargo fmt --check`（一律要通過）；
  - `cargo clippy --all-targets -- -D warnings`（rustc 與 clippy 的警告都算失敗）；
  - `tsc --noEmit`（範本的 `tsconfig.json` 是 strict 模式，未使用的變數也算錯）。

  書上貼的輸出有 `error` 開頭的行（或 `error TS`），就必須失敗，而且 `error[E….]`、`error TS….` 的錯誤碼都要出現；否則必須成功。輸出文字不逐字比對。
- 會驗證的檢查點指令：`npm run tauri dev`、`npm run tauri build`、`npm run build`、`cargo check`、`cargo build`、`cargo clippy`、`npx tsc`。`npm install`、`npm create` 這類不編譯的指令可以貼輸出，但不驗證。
- 最後一個檢查點之後還有路徑標籤的檔案，檔案結尾視為一個**必須成功**的檢查點。所以一集的最後狀態一定要能編譯。
- **沒有路徑標籤**的 `rust`、`ts`、`js`、`json`、`toml` 區塊，一定要加 `ignore`（例如 ```` ```rust,ignore ````、```` ```json,ignore ````），代表刻意不驗證的片段。能寫成完整檔案就不要用 `ignore`。
- `html`、`css`、`bash`、`text` 區塊沒有標籤時不驗證；有路徑標籤的 `index.html`、`src/styles.css` 會被放進專案（但只有 TypeScript 與 Rust 會被檢查）。
- 終端機指令用 ```` ```bash ````。
- 不使用 mdBook 的隱藏行，程式碼全部給讀者看。
- 用到範本沒有的 crate 或 npm 套件時，先把它加進 `tools/checkcode/template`（`Cargo.toml`、`package.json`，並更新 `Cargo.lock`、`package-lock.json`），書中的版本要跟範本一致。
- 行動裝置（第 18 章）、打包（第 15～17 章）的指令輸出無法自動驗證，要自己實際跑過再貼，並在 PR 說明是在哪個平台跑的。

## 語言版本

- Rust 1.99、edition 2024（`rust-toolchain.toml`）。
- Tauri 2.12（`@tauri-apps/cli`、`@tauri-apps/api`、`tauri` crate），版本以 `tools/checkcode/template` 的 lock 檔為準。
- Node.js 用目前的 LTS，套件管理一律 npm。
