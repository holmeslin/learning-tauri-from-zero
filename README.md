[![CC BY-NC-ND 4.0](https://img.shields.io/badge/content-CC%20BY--NC--ND%204.0-blue)](https://creativecommons.org/licenses/by-nc-nd/4.0/)
[![CC0 1.0](https://img.shields.io/badge/code-CC0%201.0-blue)](https://creativecommons.org/publicdomain/zero/1.0/)

# 從零開始學 Tauri

寫給讀完《[從零開始學 Rust](https://github.com/holmeslin/learning-rust-from-zero)》的人的 Tauri 入門教學（Tauri 2.12、Rust 1.99、前端 vanilla TypeScript），請閱讀[線上版](https://holmeslin.github.io/learning-tauri-from-zero/zh-TW/)。

## 本機建置

需要 [rustup](https://rustup.rs/)（會依 `rust-toolchain.toml` 自動使用 Rust 1.99）、Node.js LTS、[mdBook](https://github.com/rust-lang/mdBook) 0.5.4，以及 [Tauri 的系統前置需求](https://v2.tauri.app/start/prerequisites/)：

```bash
npm ci --prefix tools/checkcode/template                            # 安裝範本專案的前端套件（第一次）
cargo run --manifest-path tools/checkcode/Cargo.toml -- zh-TW/src   # 驗證書中所有範例
cd zh-TW && mdbook serve                                             # 本機預覽
```

### 範例程式碼怎麼驗證

書中的範例都放進同一個起始專案驗證：`tools/checkcode/template/` 是 `create-tauri-app` vanilla-ts 範本產生的專案。一集（一個 .md 檔）由上往下重播：

- 「`src-tauri/src/lib.rs`：」這種路徑標籤加程式區塊，是該檔案的完整內容；沒貼出來的檔案沿用範本。
- 「`npm run tauri dev` 的輸出：」加 `text` 區塊是檢查點：執行 `cargo fmt --check`、`cargo clippy -D warnings`、`tsc --noEmit`，成功或失敗要跟書上貼的相符（錯誤碼要對，文字不逐字比對）。
- 最後一個檢查點之後還改過檔案，檔案結尾必須能編譯成功。
- 沒有路徑標籤的 `rust`、`ts`、`json`、`toml` 片段要加 `ignore`。

第一次驗證要編譯整個 Tauri，會花幾分鐘，之後使用 `tools/checkcode/template/src-tauri/target` 的快取，CI 也會快取這個目錄。

詳細規則見 `docs/style-guide.md`。

## 授權

除非另有標示，本教學的文字、圖片與非程式碼內容以 [CC BY-NC-ND 4.0](https://creativecommons.org/licenses/by-nc-nd/4.0/) 授權。

本教學中的程式碼範例以 [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) 釋出，可自由複製、修改與使用。
