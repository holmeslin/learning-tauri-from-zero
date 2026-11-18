# global-shortcut：全域快捷鍵

## 本集目標

學會用 global-shortcut plugin 在 Rust 註冊全域快捷鍵，知道處理函式要自己分辨 `Pressed` 與 `Released`，並看得懂快捷鍵字串的寫法與註冊失敗的樣子。

## 正文

### 全域：App 不在最前面也收得到

選單上的快捷鍵（像第 1 集的 `CmdOrCtrl+N`）只有 App 在最前面才有效。全域快捷鍵不一樣，不管現在是哪個 App 在最前面，使用者按下去，你的 App 都收得到。最常見的用途是「按一下，把 App 叫出來」。

### 安裝 plugin

照第 7 章第 1 集的方式裝：

```bash
npm run tauri add global-shortcut
```

`npm run tauri add global-shortcut` 的輸出：

```text
        Info Installing Cargo dependency "tauri-plugin-global-shortcut"...
        Info Installing NPM dependency "@tauri-apps/plugin-global-shortcut@~2"...
        Info Capability matching platforms ["macOS", "windows", "linux"] not found, creating ~/tauri-app/src-tauri/capabilities/desktop.json
       Added permission `global-shortcut:default` to `desktop-capability` at ~/tauri-app/src-tauri/capabilities/desktop.json
        Info Adding plugin to ~/tauri-app/src-tauri/src/lib.rs
        Info Running `cargo fmt`...
```

跟一般 plugin 不一樣的地方：

- 手機沒有全域快捷鍵，所以 crate 被放在 `Cargo.toml` 桌面專用的區塊：

  ```toml,ignore
  [target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]
  tauri-plugin-global-shortcut = "2"
  ```

- 它多建了一個 `capabilities/desktop.json`，裡面只有 `global-shortcut:default`。這集全部在 Rust 這一側做，npm 套件（前端版）與這份 permission 都用不到，留著不影響。

### 範例：按 Cmd+Shift+K 叫出 App

`src-tauri/src/lib.rs`：

```rust
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

                app.handle()
                    .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
                app.global_shortcut().on_shortcut(
                    "CommandOrControl+Shift+K",
                    |app, _shortcut, event| {
                        println!("快捷鍵：{:?}", event.state);
                        if event.state != ShortcutState::Pressed {
                            return;
                        }
                        if let Some(window) = app.get_webview_window("main") {
                            window.set_focus().unwrap();
                        }
                    },
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `#[cfg(desktop)]` 讓這個區塊只在桌面編譯，因為手機上沒有這個 crate。`cfg(desktop)` 和範本裡的 `cfg_attr(mobile, ...)` 是同一類，第 18 章會正式介紹。
- plugin 在 `setup` 裡用 `app.handle().plugin(...)` 註冊，意思跟 `Builder::plugin` 一樣。
- `use ...GlobalShortcutExt` 之後，`app.global_shortcut()` 才拿得到 plugin。`on_shortcut(字串, 函式)` 註冊快捷鍵，並指定按下時呼叫的函式。
- 函式收到 `app`、`shortcut`、`event`；範例用不到 `shortcut`，所以加底線。
- 頁面沒有改，維持範本的樣子。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.58s
     Running `target/debug/tauri-app`
```

切到別的 App，按 Cmd+Shift+K（其他平台是 Ctrl+Shift+K），終端機多出兩行，App 回到最前面：

```text
快捷鍵：Pressed
快捷鍵：Released
```

### `Pressed` 與 `Released`

按一次快捷鍵，處理函式被呼叫兩次：按下去是 `Pressed`，放開是 `Released`。範例在函式一開頭，遇到不是 `Pressed` 就 `return`，否則同一次按鍵會做兩次事。這跟第 4 集系統匣的 `Down`、`Up` 是同一個道理。

### 快捷鍵字串

- 修飾鍵在前，最後一個是主鍵，用 `+` 接，大小寫都行。主鍵只能有一個。
- 修飾鍵：`CommandOrControl`（macOS 是 Cmd，其他平台是 Ctrl）、`Ctrl`、`Alt`（也能寫 `Option`）、`Shift`、`Super`（也能寫 `Cmd`、`Command`）。

### 註冊會失敗

字串不合法、這個 App 已經註冊過同一組、作業系統不讓註冊，`on_shortcut` 都會回傳 `Err`。把範例的字串改成 `"CommandOrControl-Shift-K"`（用 `-` 取代 `+`），App 一啟動就停下來，終端機印出：

```text
Failed to setup app: error encountered during setup hook: Couldn't recognize "CommandOrControl-Shift-K" as a valid key for hotkey, if you feel like it should be, please report this to https://github.com/tauri-apps/muda
```

範例用 `?` 把錯誤丟出去，所以註冊失敗 App 就打不開。正式的 App 常改成只印出訊息，快捷鍵用不了，不該讓整個 App 用不了。

別的程式佔用同一組快捷鍵時，各平台的反應不同。在 macOS 上，同一個 App 開兩個行程各註冊一次，兩邊都沒有報錯。

### 不同平台

- Linux 只支援 X11，不支援 Wayland。
- 這一集只在 macOS 實測，Windows 與 Linux 依文件整理。

## 重點整理

- 全域快捷鍵不管哪個 App 在最前面都收得到；用 global-shortcut plugin，這一集全部在 Rust 這一側做，不需要 permission。
- plugin 只有桌面版：`Cargo.toml` 放桌面專用區塊，程式用 `#[cfg(desktop)]` 包起來，在 `setup` 裡 `app.handle().plugin(...)` 註冊。
- `app.global_shortcut().on_shortcut("CommandOrControl+Shift+K", 函式)` 註冊並指定處理函式；函式要自己判斷 `event.state`，只處理 `Pressed`。
- 字串是「修飾鍵 `+` 主鍵」；寫錯、已註冊、系統不讓註冊都會回傳 `Err`。
