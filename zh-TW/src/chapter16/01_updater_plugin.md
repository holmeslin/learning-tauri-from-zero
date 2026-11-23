# updater plugin

## 本集目標

知道 Tauri 的自動更新由哪幾樣東西組成，裝上 updater plugin 並在 Rust 註冊，也看到它在設定不完整時 App 為什麼起不來。

## 正文

這一集從起始專案出發。第 1～4 集會接著做同一個專案，從第 2 集起，每集開頭都會列出從上一集帶過來的檔案。

### 自動更新是怎麼運作的

App 啟動後，updater plugin 做這幾件事：

```text
問網路上的更新資訊檔：有沒有新版？
  ├─ 沒有：照常使用
  └─ 有：下載更新檔 → 用公鑰驗簽章 → 換掉舊的 App → 重新啟動
```

所以要準備三樣東西：

- **更新檔**：新版 App 打包出來的檔案，放在網路上。
- **更新資訊檔**：一份 JSON，寫著最新版本號，以及各平台的更新檔網址與簽章。App 啟動後去讀它。
- **一對金鑰**：私鑰替更新檔簽章，公鑰跟著 App 發出去，用來驗簽章。簽章不對，plugin 不會安裝。

第 2 集做金鑰，第 3 集做更新資訊檔，第 4 集在 App 裡寫檢查與安裝的程式。

這裡的簽章叫更新簽章，是 Tauri 自己的一套，免費、一定要做。它跟本章最後兩集的程式碼簽章（macOS 簽章與公證、Windows 簽章）是兩回事：程式碼簽章讓作業系統信任你的 App，要付費憑證。本章前四集沒有用到任何憑證。

### 裝 plugin

照第 7 章第 1 集的方式裝：

```bash
npm run tauri add updater
```

`npm run tauri add updater` 的輸出：

```text
        Info Installing Cargo dependency "tauri-plugin-updater"...
        Info Installing NPM dependency "@tauri-apps/plugin-updater@~2"...
        Info Capability matching platforms ["macOS", "windows", "linux"] not found, creating ~/tauri-app/src-tauri/capabilities/desktop.json
       Added permission `updater:default` to `desktop-capability` at ~/tauri-app/src-tauri/capabilities/desktop.json
        Info Enabled `plugins > updater > requireSignedVersion` in ~/tauri-app/src-tauri/tauri.conf.json
        Info Remember to set `plugins > updater > pubkey` and `endpoints`.
        Info Adding plugin to ~/tauri-app/src-tauri/src/lib.rs
        Info Running `cargo fmt`...
```

它動了四個地方：

- 手機的 App 靠商店更新，updater 只有桌面版，所以 crate 放在 `Cargo.toml` 桌面專用的區塊。
- `lib.rs` 的 `Builder` 鏈上多了一行 `.plugin(tauri_plugin_updater::Builder::new().build())`。下面的範例改在 `setup` 裡用 `#[cfg(desktop)]` 註冊，所以直接用範例的整份 `lib.rs` 取代，不要兩邊都留。
- `tauri.conf.json` 多了 `plugins.updater.requireSignedVersion: true`。簽章裡會記著它是替哪個版本號簽的，plugin 驗證時拿它跟更新資訊檔宣告的版本號比對；這個開關再加一條：沒記版本號的簽章一律拒絕。目的是防止有人拿舊版的更新檔，配上新的版本號冒充新版。
- 它另外建了 `capabilities/desktop.json`，放進 `updater:default`，npm 也多裝了前端用的套件。permission 管的是前端經 IPC 能呼叫什麼（第 14 章），這一章整條更新流程都在 Rust 寫，不經過前端，所以這兩樣用不到。我們實際做的時候沒有刪，留著或刪掉都行。

### 在 `setup` 註冊

`src-tauri/src/lib.rs`：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`#[cfg(desktop)]` 的用意跟第 8 章第 6 集一樣，只在桌面編譯這一行；`cfg(desktop)` 第 18 章會正式介紹。

### 還沒設定就跑

plugin 啟動時，會讀 `tauri.conf.json` 的 `plugins.updater`，裡面一定要有 `pubkey`。我們還沒有，所以現在跑會怎樣？

`npm run tauri dev` 的輸出（節錄）：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.46s
     Running `target/debug/tauri-app`

thread 'main' (751665) panicked at ~/.cargo/registry/src/index.crates.io-…/tauri-2.12.6/src/app.rs:1444:11:
Failed to setup app: error encountered during setup hook: failed to initialize plugin `updater`: Error deserializing 'plugins.updater' within your Tauri configuration: missing field `pubkey`
```

編譯通過，但視窗沒有出現，`setup` 裡的 plugin 初始化失敗，整個 App 就停了。這樣是對的：沒有公鑰就沒辦法驗簽章，plugin 寧可不啟動，也不讓沒驗過的更新檔裝進去。公鑰從哪裡來，下一集解決。

## 重點整理

- 自動更新要三樣東西：網路上的更新檔、更新資訊檔，以及一對金鑰（私鑰簽、公鑰驗）。
- 更新簽章是 Tauri 的一套，免費、不能省；程式碼簽章是作業系統的一套，要付費憑證，是另一回事。
- `npm run tauri add updater` 會加 crate（桌面專用）、npm 套件、`desktop.json` 與 `requireSignedVersion`；從 Rust 呼叫用不到 permission 與 npm 套件。
- plugin 在 `setup` 用 `#[cfg(desktop)]` 註冊；`plugins.updater` 沒有 `pubkey`，App 一啟動就 panic。
