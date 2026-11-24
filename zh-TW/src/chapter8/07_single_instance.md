# single-instance：只允許開一個

## 本集目標

學會用 single-instance plugin，讓使用者再啟動一次 App 的時候，新的行程自己結束，由已經在跑的那一個收到通知並把視窗叫到前面。

## 正文

### 為什麼要只開一個

什麼都不做的話，每啟動一次就多一個行程：兩個視窗、兩個系統匣圖示，還可能兩邊同時寫同一個檔案。上一集實測過，同一個 App 開兩個行程，兩邊都能註冊同一組全域快捷鍵。

single-instance plugin 用 `tauri.conf.json` 的 `identifier`（第 1 章第 6 集）判斷「已經有一個了」，所以 identifier 一樣的兩個 App 會被當成同一個。各平台的做法不同：macOS 在 `/tmp` 底下用一個以 identifier 命名的 Unix socket，Windows 用具名的 mutex，Linux 用 D-Bus。

### 安裝 plugin

```bash
npm run tauri add single-instance
```

`npm run tauri add single-instance` 的輸出：

```text
        Info Installing Cargo dependency "tauri-plugin-single-instance"...
        Info Adding plugin to ~/tauri-app/src-tauri/src/lib.rs
        Info Running `cargo fmt`...
```

這個 plugin 只有 Rust crate，沒有 npm 套件，也沒有 permission，因為前端不會呼叫它。跟上一集一樣，crate 放在 `Cargo.toml` 桌面專用的區塊。`lib.rs` 被加上 `.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))`，函式裡面是空的，這就是下面要填的地方。

### 範例：第二個行程來了，叫出視窗

`init` 收一個函式。這個函式在**第一個**行程裡執行，時機是第二個行程啟動的時候。它的三個參數：`app` 是 `AppHandle`，`args` 是第二個行程的命令列參數（第一個元素是執行檔的路徑），`cwd` 是第二個行程的工作目錄。第二個行程把 `args` 與 `cwd` 交給第一個之後，在 plugin 初始化的時候就自己結束了，不會開出視窗。

plugin 要**最先**註冊：plugin 照加入的順序啟動，第二個行程要在其他初始化開始之前就結束。為了讓它排第一個，又只在桌面加，範例先 `let mut builder`，再用 `builder = builder.plugin(...)` 接上去。「只在桌面加」靠的是 `#[cfg(desktop)]`，跟上一集一樣，第 18 章會正式介紹。

`src-tauri/src/lib.rs`：

```rust
use tauri::{AppHandle, Manager};

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            println!("第二個行程來過：參數 {args:?}，工作目錄 {cwd}");
            show_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

頁面維持範本的樣子，不過 `lib.rs` 沒有 `greet` command，Greet 按鈕這集不會有結果。`show_main_window` 跟第 4 集一樣：`show()`、`unminimize()`、`set_focus()` 三個都要。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.52s
     Running `target/debug/tauri-app`
```

App 開著的時候，另開一個終端機，在專案根目錄直接執行同一個執行檔，並隨便帶兩個參數：

```bash
./src-tauri/target/debug/tauri-app hello --flag
```

這個指令一下子就回來了，沒有開出新視窗。原本跑 `npm run tauri dev` 的終端機多了一行，視窗也回到最前面（最小化的話會還原）：

```text
第二個行程來過：參數 ["./src-tauri/target/debug/tauri-app", "hello", "--flag"]，工作目錄 ~/tauri-app
```

第二個行程的結束代碼是 0。

### 不同平台

- Linux 要有 D-Bus 才能用，D-Bus 的名稱預設也是 identifier。
- 這一集只在 macOS 實測，Windows 與 Linux 依 Tauri 文件與原始碼整理。

## 重點整理

- single-instance 用 `identifier` 判斷已經有行程在跑；第二個行程把參數與工作目錄交給第一個，自己結束。
- `init(|app, args, cwd| ...)` 的函式在第一個行程裡執行，適合拿來把視窗叫到前面；`args` 的第一個元素是執行檔路徑。
- 只有 Rust crate，沒有 npm 套件與 permission；要最先註冊，並用 `#[cfg(desktop)]` 只在桌面加。
- 想測試，直接在終端機再執行一次執行檔，並帶上參數。
