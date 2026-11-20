# 一起打包資源檔

## 本集目標

用 `bundle.resources` 把額外的檔案一起打包，在 `.app` 裡找到它們，並用 `app.path().resolve(…, BaseDirectory::Resource)` 在執行時找到、讀出來。

## 正文

### 為什麼要列出來

打包出來的 `.app` 裡，只有執行檔、`Info.plist` 和圖示（第 4 集的 `find` 看過），前端是編進執行檔的。App 如果還要讀別的檔案，像範本、預設資料、說明文字，那些檔案躺在你的專案資料夾，不會自動跟著走，裝到別人的電腦上就找不到了。要跟著走的檔案叫資源檔（resource），列在 `bundle.resources`，打包時就會複製進去。

### 設定 `bundle.resources`

先放一個資源檔：

`src-tauri/data/greeting.txt`：

```text
你好，我是跟著 App 一起打包的資源檔。
```

再到 `tauri.conf.json` 的 `bundle` 裡（跟 `targets` 同一層）加一行。路徑從 `tauri.conf.json` 所在的 `src-tauri/` 算起，所以寫 `data/greeting.txt`：

```json,ignore
"resources": ["data/greeting.txt"],
```

### 在 Rust 讀資源檔

資源檔裝好之後在哪裡，每個平台、`tauri dev` 與打包後都不一樣，所以跟第 7 章第 6 集的 app 資料夾一樣，要用函式問。`app.path()` 的 `resolve` 要兩個參數：相對路徑，以及從哪個資料夾算起的 `BaseDirectory`（第 7 章第 5 集在 JavaScript 見過 `BaseDirectory.AppData`，Rust 這邊是 `tauri::path::BaseDirectory`）。選 `Resource`，就是資源資料夾，也就是 `app.path().resource_dir()` 回傳的那個。我們在 `setup`（第 4 章第 4 集）裡讀 `greeting.txt` 並印出來，`greet` 照範本留著：

`src-tauri/src/lib.rs`（加上 `setup`）：

```rust
use tauri::Manager;
use tauri::path::BaseDirectory;

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
            let path = app
                .path()
                .resolve("data/greeting.txt", BaseDirectory::Resource)?;
            println!("資源檔位置：{}", path.display());
            println!("內容：{}", std::fs::read_to_string(&path)?.trim());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`resolve` 的第一個參數是資源檔在資源資料夾裡的相對路徑，陣列寫法就是 `resources` 裡寫的那一串；它回傳 `Result<PathBuf>`，用標準函式庫讀檔，不需要 permission（第 7 章第 6 集）。

### 打包，看它放在哪裡

```bash
npm run tauri build -- --bundles app
```

`npm run tauri build -- --bundles app` 的輸出（節錄）：

```text
     Running beforeBuildCommand `npm run build`
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `release` profile [optimized] target(s) in 40.31s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (3.95 MiB)
```

第 4 集說過，`Contents/Resources/` 除了圖示，也放資源檔：

```bash
find src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources
```

`find` 的輸出：

```text
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/icon.icns
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/data
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/data/greeting.txt
```

`data/greeting.txt` 保留了你寫的相對路徑。直接執行 `.app` 裡的執行檔（會開出視窗，看完輸出按 `Control + C` 結束）：

```bash
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/MacOS/tauri-app
```

終端機印出：

```text
資源檔位置：~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/data/greeting.txt
內容：你好，我是跟著 App 一起打包的資源檔。
```

不是打包出來的版本，位置就不同。`tauri dev` 啟動的是 `src-tauri/target/debug/tauri-app`（第 13 章第 6 集），它印出的位置是 `~/tauri-app/src-tauri/target/debug/data/greeting.txt`，資源檔在執行檔旁邊，是 `tauri-build` 編譯時複製過去的。（我們沒有開 `tauri dev`，是 `cargo build` 之後直接執行那個檔案。）兩個位置不同，`resolve` 都指對了。

### `resources` 的寫法

`resources` 不只能列檔案。我們另外放了 `src-tauri/data/sub/a.txt` 與專案根目錄的 `shared/x.txt`，逐一改設定、用 `--debug --bundles app` 打包，再看 `Contents/Resources/`：

| `resources` 的寫法 | 結果 |
| --- | --- |
| `["data/greeting.txt"]` | `data/greeting.txt` |
| `["data"]` | 整個資料夾：`data/greeting.txt`、`data/sub/a.txt` |
| `["data/*"]` | 只有 `data/greeting.txt`，子資料夾 `sub` 不會進去 |
| `["../shared/x.txt"]` | `_up_/shared/x.txt`，`..` 被換成 `_up_` |
| `{"data/greeting.txt": "hi/hello.txt"}` | `hi/hello.txt`，自己指定位置 |

陣列與物件二選一。走出 `src-tauri/` 的路徑，位置會變成 `_up_/shared/x.txt`，不是你寫的樣子；但 `resolve` 照同樣的規則換，`resolve("../shared/x.txt", BaseDirectory::Resource)` 就會得到 `_up_` 那個位置（我們用 `cargo build` 做出來的執行檔試過，檔案找得到），所以陣列寫法照 `resources` 的字串給路徑就好，不用自己拼 `_up_`；物件寫法則是表裡右邊你指定的位置。資源檔放在 `src-tauri/` 底下仍然最單純。

路徑一定要存在。刪掉 `data/greeting.txt` 再跑 `cargo check`，馬上失敗，不用等到打包，因為 `tauri-build` 編譯時就要複製資源檔；`tauri dev` 也是先用 cargo 編譯，一樣會停。輸出（節錄）：

```text
error: failed to run custom build command for `tauri-app v0.1.0 (~/tauri-app/src-tauri)`
...
  resource path `data/greeting.txt` doesn't exist
```

### 其他平台

這一段是照 Tauri 2.12 的原始碼（`tauri-utils` 與 tauri-bundler）整理的，沒有在 Windows 與 Linux 上實測。Windows 的 `resource_dir()` 是執行檔所在的資料夾，安裝檔把資源檔裝在安裝資料夾裡；Linux 的 deb 與 rpm 裝到 `/usr/lib/<productName>/`，AppImage 則在包裡的 `usr/lib/<productName>/`。這些位置都由 `resolve` 替你處理。

### 資源檔只能讀

資源檔是安裝時放好的，當成唯讀。App 執行時要寫的資料，放第 7 章第 6 集的 `app_data_dir()`。這個 `.app` 也還沒簽章，簽章與公證，第 16 章會正式介紹。

## 重點整理

- 要跟著 App 走的檔案，列在 `bundle.resources`，路徑從 `src-tauri/` 算起；可以是檔案、資料夾、`*`，或用物件指定位置。
- macOS 的資源檔放進 `.app` 的 `Contents/Resources/`，保留相對路徑；`..` 會變成 `_up_`。
- Rust 用 `app.path().resolve("相對路徑", BaseDirectory::Resource)` 找資源檔，陣列寫法照 `resources` 的字串給路徑，連 `..` 變 `_up_` 都會替你換；`tauri dev`、打包後與各平台的位置都不同，不要自己拼。
- `resources` 指到不存在的路徑，連 `cargo check` 都會失敗；資源檔唯讀，要寫的資料放 `app_data_dir()`。
