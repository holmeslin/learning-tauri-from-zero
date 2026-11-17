# 專案結構導覽

## 本集目標

知道起始專案裡每個資料夾和重要檔案在做什麼，特別是 `src/` 與 `src-tauri/` 怎麼分工。

## 正文

### 一個資料夾，兩個專案

打開 `tauri-app/`，只看書上會用到的檔案，長這樣：

```text
tauri-app/
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
├── src/
│   ├── main.ts
│   ├── styles.css
│   └── assets/
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json
    ├── icons/
    └── src/
        ├── main.rs
        └── lib.rs
```

最外層是前端專案，由 npm 管理；`src-tauri/` 是 Rust 專案，由 cargo 管理。Tauri 的命令列工具靠 `tauri.conf.json` 找到這個 Rust 專案。

有兩個 `src` 要分清楚：`src/` 放前端的 TypeScript，`src-tauri/src/` 放 Rust。

### 前端這一側

- `index.html`：視窗載入的頁面，介面的骨架。
- `src/main.ts`：TypeScript 程式，決定頁面怎麼動。
- `src/styles.css`：外觀。
- `src/assets/`：圖片。
- `package.json`：npm 專案的設定檔，角色像 `Cargo.toml`。`dependencies` 是相依套件，`scripts` 是 `npm run <名稱>` 能執行的指令。
- `vite.config.ts`：Vite 的設定。Vite 是前端的開發伺服器與打包工具，第 2 章第 11 集會介紹。
- `tsconfig.json`：TypeScript 的設定，範本開了 strict 模式。

### Rust 這一側

- `Cargo.toml`：你熟悉的那個。要注意 `[lib]` 這一段，函式庫名稱是 `tauri_app_lib`，`crate-type` 列了三種，是為了讓同一份程式碼也能編成行動裝置的函式庫（第 18 章）。
- `build.rs`：Cargo 編譯主程式前會先跑的建置腳本，裡面只有一行 `tauri_build::build()`，Tauri 在編譯前需要的前置作業交給它。
- `tauri.conf.json`：Tauri 的設定檔，第 6 集專門看它。
- `capabilities/`：App 的 permission 設定，第 7 章與第 14 章會用到。
- `icons/`：App 圖示，第 15 章打包時才會動它。

最後兩個檔案是 Rust 程式碼。先看 `src-tauri/src/main.rs`，這就是範本原本的內容：

`src-tauri/src/main.rs`（範本原本的內容）：

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri_app_lib::run()
}
```

第一行屬性只對 Windows 有意義：release 版本不要多開一個黑色的終端機視窗。`main` 本身只剩一行，呼叫函式庫裡的 `run`。真正的程式在 `lib.rs`，之後我們幾乎只會改它，`main.rs` 不用動。

### 自動產生的東西

`npm install` 會產生 `node_modules/`，第一次編譯會產生 `src-tauri/target/`，還有 `src-tauri/gen/schemas/`（給編輯器替 `capabilities/` 裡的 JSON 自動補全用）。這些都是機器產生的，範本附的 `.gitignore` 已經排除了：根目錄的排除 `node_modules`，`src-tauri/.gitignore` 排除 `/target/` 和 `/gen/schemas`。不用管，也不要手動改。

## 重點整理

- 專案根目錄是前端專案（npm），`src-tauri/` 是 Rust 專案（cargo）。
- `src/` 放前端 TypeScript，`src-tauri/src/` 放 Rust，兩個 `src` 不一樣。
- `index.html`、`src/main.ts`、`src/styles.css` 組成前端畫面。
- `tauri.conf.json` 是 Tauri 的設定檔；`capabilities/` 管 permission；`icons/` 放圖示。
- `main.rs` 只負責啟動，程式寫在 `lib.rs`。
