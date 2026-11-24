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

- 一集只教**一個**小概念，篇幅大約 40～150 行 markdown。第二部（實戰）每集是專案的一個零件，要重貼完整的檔案，上限放寬到約 400 行。第三部（上線）的安全性、打包與發佈常要貼完整設定檔與實跑輸出，上限放寬到約 250 行。
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
- 例外：一集需要前面某一集的狀態才講得通（例如第 16 章第 1～4 集先做出 1.0.0，再更新到 1.1.0），可以接著上一集的專案做，也可以接著前面某一章某一集的專案（例如第 18 章第 5 集接著第 11 章第 7 集）。這一集的開頭要說明是接著哪一集，並列出從那一集帶過來的檔案。

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
- `yaml` 區塊（像 `.github/workflows/release.yml`）不用加 `ignore`。有路徑標籤的 yaml 是完整檔案，checkcode 會把它放進專案，但不檢查內容。貼 CI 的 log 時，標籤不要用上面會驗證的指令名稱（像「`npm run tauri build` 的輸出」），改寫成「某個 step 的 log」。
- 不使用 mdBook 的隱藏行，程式碼全部給讀者看。
- 在專案裡加 crate 或打開 crate 的 feature，一律在專案根目錄用 `cargo add --manifest-path src-tauri/Cargo.toml <crate>@<版本> [--features …]`，版本寫到跟範本一致的那一層（`thiserror@2`、`log@0.4`、`rusqlite@0.43`、`tauri@2`）；有桌面限定時加 `--target`。不手改 `Cargo.toml`，也不在 `src-tauri/` 底下執行不帶版本的 `cargo add`。（統一前的寫法：`cargo add --manifest-path` 帶版本 7 集共 8 條指令、手改 `Cargo.toml` 2 集、`cargo add log` 1 集。）第 14 章第 6 集與第 15 章第 1 集貼完整 `Cargo.toml`，是下面的例外。
- 用到範本沒有的 crate 或 npm 套件時，先把它加進 `tools/checkcode/template`（`Cargo.toml`、`package.json`，並更新 `Cargo.lock`、`package-lock.json`），書中的版本要跟範本一致。
- 例外：有些 feature 不能預先加進範本，例如 `tauri` 與 `tauri-build` 的 `isolation`（`tauri-build` 要求 `Cargo.toml` 的 feature 跟 `tauri.conf.json` 的設定一致，範本打開了，其他集就會失敗）。這種集要用路徑標籤貼出完整的 `src-tauri/Cargo.toml`，`tools/checkcode` 看到這個檔案，就不加 `--locked`。目前是第 14 章第 6 集（`isolation`）與第 15 章第 1 集（`devtools`）。這個例外有三件事：
  - **基準**：貼的是剛建好的專案那份 `Cargo.toml`（`tauri`、`tauri-build`、`tauri-plugin-opener`、`serde`、`serde_json`），只改那一集要講的 feature。
  - **跟起始專案的關係**：起始專案是 `tools/checkcode/template/`，它的 `Cargo.toml` 已經裝好全書用到的 crate，`Cargo.lock` 鎖住版本。路徑標籤貼的是完整檔案，會整份換掉範本的，所以這一集的依賴只有貼出來的這些，跟範本不同。
  - **代價**：依賴不同，`Cargo.lock` 就一定要改，連只剪掉用不到的 crate 也一樣（第 15 章第 1 集沒有新 crate，仍要剪掉範本多出來的），所以不能用 `--locked`，由 cargo 更新工作目錄裡的 `Cargo.lock`。lock 裡已有的 crate 維持原版本，只有 lock 沒有的 crate（`isolation` 多出來的加密相關 crate）會重新解析：版本會浮動，而且需要網路。這幾集也不再檢查「用到的 crate 有沒有先加進範本」。
- 打包與 CI（第 15～17 章）的指令輸出無法自動驗證，要自己實際跑過再貼（CI 的輸出要來自真的在 GitHub Actions 跑過的那次），並在 PR 說明是在哪個平台跑的。沒跑過的 yaml 片段，要在該集的「這集實測了什麼」寫明。
- 第 18 章（行動裝置）的指令都沒有跑過：不貼輸出，也不寫成檢查點的樣子。每集的「這集實測了什麼」要寫明行動裝置的指令都沒有跑過、內容依據什麼查證，PR 說明也要寫沒跑過（見 `docs/outline.md` 第 18 章的邊界）。

## 語言版本

- Rust 1.99、edition 2024（`rust-toolchain.toml`）。
- Tauri 2.12（`@tauri-apps/cli`、`@tauri-apps/api`、`tauri` crate），版本以 `tools/checkcode/template` 的 lock 檔為準。
- Node.js 用目前的 LTS，套件管理一律 npm。書中寫的最低版本，以 `tools/checkcode/template/package-lock.json` 裡最嚴的 `engines` 為準：目前是 jsdom 31 的 `^22.22.2 || ^24.15.0 || >=26.0.0`，所以寫「Node.js 22.22.2、24.15.0 或 26 以上」（第 1 章第 2 集、第 9 章第 3 集、第 17 章第 1 集）。範本的套件升級後要重查。
