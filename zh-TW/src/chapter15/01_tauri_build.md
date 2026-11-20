# `npm run tauri build`

## 本集目標

跑一次 `npm run tauri build`，知道它依序做了什麼、做出來的檔案放在哪；認識 `--debug`、`--no-bundle`、`--bundles` 三個旗標，並替 release 版打開開發者工具。

## 正文

### 打包做三件事

`npm run tauri build` 依序做三件事：

1. 執行 `tauri.conf.json` 的 `build.beforeBuildCommand`。範本是 `npm run build`：檢查 TypeScript，再用 Vite 把前端打包進 `dist/`（第 1 章第 6 集）。
2. 用 release 模式編譯 Rust，並把 `dist/` 的檔案放進執行檔。
3. 照 `bundle` 的設定，把執行檔做成安裝檔。

直接下 `npm run tauri build`，macOS 會做出 `.app` 與 `.dmg` 兩種。`.dmg` 的做法比較特別，留到第 4 集，這一集先用 `--bundles` 只做 `.app`。`npm run` 後面要先放 `--`，後面的旗標才會交給 `tauri`（第 1 章第 8 集的 `--debug` 也是這樣）：

```bash
npm run tauri build -- --bundles app
```

`npm run tauri build -- --bundles app` 的輸出（節錄，第一次要把所有相依套件編譯一遍，所以比較久）：

```text
        Warn The `productName` is still set to the default value `tauri-app`, it must be unique across applications since it is written into install paths and platform metadata that are expected to be unique to your application, like the Windows installer upgrade code.
     Running beforeBuildCommand `npm run build`
   Compiling tauri v2.12.6
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `release` profile [optimized] target(s) in 58.58s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (3.98 MiB)
```

照三個步驟對一對：

- `Warn` 那行是說 `productName` 還是範本的 `tauri-app`。這個名字會寫進安裝檔的檔名與系統的資料，要換成自己的，第 3 集處理。
- `Running beforeBuildCommand` 是第 1 步。前端有 TypeScript 錯誤的話，會在這裡停下來，Rust 不會開始編譯。
- `Finished `release` profile` 是第 2 步，編的是最佳化過的版本，比 `tauri dev` 久得多。`Built application at` 是執行檔，在 `src-tauri/target/release/`（Windows 是 `tauri-app.exe`）。
- `Finished 1 bundle at` 是第 3 步。安裝檔放在 `src-tauri/target/release/bundle/` 底下，每種格式一個資料夾，`.app` 在 `macos/`。

### 三個常用旗標

| 旗標 | 做什麼 |
| --- | --- |
| `--debug` | 用 debug 模式編譯，輸出放在 `target/debug/` |
| `--no-bundle` | 只編譯，不做安裝檔（第 14 章第 5 集用過） |
| `--bundles app` | 只做指定的格式，多種用逗號隔開 |

`--debug` 值得多看一眼：

```bash
npm run tauri build -- --debug --bundles app
```

`npm run tauri build -- --debug --bundles app` 的輸出（節錄）：

```text
     Running beforeBuildCommand `npm run build`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.46s
       Built application at: ~/tauri-app/src-tauri/target/debug/tauri-app
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/debug/bundle/macos/tauri-app.app)
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/debug/bundle/macos/tauri-app.app (24.56 MiB)
```

這次用的是 `dev` profile，編譯比較快（我們的機器上 20 秒，release 要 58 秒），檔案卻大了很多：`target/debug/tauri-app` 約 24 MiB，release 的不到 4 MiB。範本的 `[profile.release]` 打開了 `lto` 與 `strip`，就是為了縮小檔案。debug 版自帶開發者工具，遇到「只有打包後才會發生」的問題，先拿它來查。

注意 `--debug` 載入的還是打包進去的前端，不是 `devUrl`。第 14 章第 5 集用它驗證 CSP，靠的就是這一點。

### release 版的開發者工具

release 版預設沒有開發者工具。要有的話，得在 `src-tauri/Cargo.toml` 替 `tauri` 打開 `devtools` 這個 feature，也就是第 1 章第 8 集預告的那一步。一個剛建好的專案，改完是這樣（如果你的專案還有別的 feature，像第 8 章的 `tray-icon`，會列在同一個清單裡）：

`src-tauri/Cargo.toml`（打開 `devtools` 之後）：

```toml
[package]
name = "tauri-app"
version = "0.1.0"
description = "A Tauri App"
authors = ["you"]
edition = "2024"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
# The `_lib` suffix may seem redundant but it is necessary
# to make the lib name unique and wouldn't conflict with the bin name.
# This seems to be only an issue on Windows, see https://github.com/rust-lang/cargo/issues/8519
name = "tauri_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"


# Read the optimization guideline for more details: https://tauri.app/concept/size/#cargo-configuration
[profile.release]
codegen-units = 1
lto = true
opt-level = 3
panic = "abort"
strip = true
```

重新打包（只有 `tauri` 與依賴它的幾個 crate 要重編）：

```bash
npm run tauri build -- --bundles app
```

`npm run tauri build -- --bundles app` 的輸出（節錄）：

```text
   Compiling tauri v2.12.6
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `release` profile [optimized] target(s) in 48.62s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (4.00 MiB)
```

新的 `.app` 打開後，右鍵選單會出現 Inspect Element，`Command + Option + I` 也能用。這一點我們沒有開啟打包後的視窗確認，是依 `wry`（Tauri 用來開 WebView 的函式庫）的說明整理的。要留意兩件事：

- 這個 feature 在 macOS 會呼叫私有 API，而 App Store 不收這樣的 App，要上架的版本別打開它。
- 發佈給使用者的版本最好也拿掉，不然每個使用者都能打開你的前端來翻。

### 跟 Electron 比一比

Electron 的打包要另外裝 electron-builder 或 electron-forge 這類工具，做的主要是把你的 JavaScript 和一份現成的 Electron 執行檔包在一起，沒有東西要編譯。Tauri 的打包器內建在 CLI 裡，打包前還要把整個 Rust 專案用 release 模式編一遍，第一次會等一陣子，專案大了更久。

## 重點整理

- `npm run tauri build` 依序執行 `beforeBuildCommand`、用 release 模式編譯 Rust、照 `bundle` 設定做安裝檔，結果在 `src-tauri/target/release/bundle/`。
- `--debug` 編譯快、檔案大、有開發者工具；`--no-bundle` 只編譯；`--bundles` 指定要做哪些格式，`npm run` 後面要先放 `--`。
- release 版沒有開發者工具，要在 `src-tauri/Cargo.toml` 替 `tauri` 打開 `devtools` feature；上架 App Store 或發佈給使用者的版本別留著。
- 輸出的 `Warn` 在提醒我們換掉 `productName`，第 3 集處理。
