# 在 app 裡檢查與安裝更新

## 本集目標

在 Rust 寫出「啟動時檢查、下載、驗簽章、安裝、重新啟動」，並實際把 1.0.0 的 App 更新成 1.1.0。

## 正文

這一集接著上一集的專案。帶過來的檔案是 `src-tauri/tauri.conf.json`（金鑰、`pubkey`、`createUpdaterArtifacts` 都設好了），`~/update-server` 也有伺服器在放；`src-tauri/src/lib.rs` 下面整份換掉。

### 設定 `endpoints`

在 `plugins.updater` 加上 App 要去問的網址，以及一個只給本機測試用的開關：

```json,ignore
"plugins": {
  "updater": {
    "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6...",
    "requireSignedVersion": true,
    "endpoints": ["http://localhost:8000/latest.json"],
    "dangerousInsecureTransportProtocol": true
  }
}
```

`endpoints` 是陣列，plugin 照順序問，第一個回得出資訊檔的就用它。release 版只收 `https`：我們試過拿掉最後那一行再打包 1.0.0，一執行 App 就 panic，原因是 `plugins.updater` 的 endpoint 必須用 `https`（原文 `must use a secure protocol`）。本機沒有 https，才加 `dangerousInsecureTransportProtocol` 明確同意用 http。名字裡有 `dangerous` 是有原因的：用 http，網路上的人看得到、也改得到 `latest.json`，只剩簽章擋住假的更新檔。**這只是測試用的，真正發佈要用 `https`，並把這一行拿掉。** 依原始碼，debug 版（`tauri dev`）遇到 http 只會警告，不會擋，我們沒有試。

### 檢查、下載、安裝、重新啟動

`src-tauri/src/lib.rs`：

```rust
#[cfg(desktop)]
use tauri::AppHandle;
#[cfg(desktop)]
use tauri_plugin_updater::UpdaterExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(desktop)]
async fn check_for_update(app: AppHandle) -> tauri_plugin_updater::Result<()> {
    let Some(update) = app.updater()?.check().await? else {
        println!("已經是最新版");
        return Ok(());
    };
    println!(
        "找到新版 {}（目前 {}）",
        update.version, update.current_version
    );
    let mut downloaded = 0;
    update
        .download_and_install(|chunk, _total| downloaded += chunk, || println!("下載完成"))
        .await?;
    println!("共下載 {downloaded} bytes，安裝完成，重新啟動");
    app.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            println!("目前版本 {}", app.package_info().version);
            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = check_for_update(handle).await {
                        eprintln!("更新失敗：{error}");
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `use tauri_plugin_updater::UpdaterExt` 之後才有 `app.updater()`，跟 `GlobalShortcutExt` 同一種寫法。
- `check().await?` 回傳 `Option<Update>`：`None` 是沒有新版（版本沒有比目前新），`Some` 就是有。`Update` 帶著 `version`、`current_version`，還有 `body`（`notes`）與 `date`（`pub_date`）。
- `download_and_install` 收兩個函式：每收到一塊呼叫一次的 `(這塊的位元組數, 總大小)`，跟下載完成時呼叫的。它先驗簽章，驗不過不安裝；macOS 是解開 `.tar.gz`、換掉原本的 `.app`。
- `AppHandle::restart()` 是 Tauri 內建的：結束目前的行程，再開一個新的。它的回傳型別是 `!`，不會回來，所以函式最後不用再寫 `Ok(())`。macOS 與 Linux 安裝完要自己呼叫；Windows 的安裝程式會先把 App 關掉，所以走不到這一行（Windows 這部分是讀原始碼，沒有試）。
- `tauri::async_runtime::spawn` 把一個 `async` 區塊丟到 Tauri 的 async runtime 在背景跑，不會卡住呼叫它的地方。`setup` 不是 `async`，網路的事丟進去，視窗就不用等。這個 runtime 跟第 4 章第 3 集的 `tauri::async_runtime::Mutex` 是同一個。
- `println!` 只是為了看流程。真的 App 通常會先問使用者要不要更新，可以用第 7 章第 4 集的 dialog。

### 打包兩個版本

新版也要含這段程式碼，不然更新之後的 App 就不會再檢查更新，所以 1.1.0 要重打：

1. `version` 是 `1.1.0`，設好私鑰環境變數，`npm run tauri build -- --bundles app`。
2. 新的 `tauri-app.app.tar.gz` 複製進 `~/update-server`，`latest.json` 的 `signature` 換成新的 `.sig` 內容：程式碼變了，更新檔跟簽章都變，舊簽章驗不過。
3. `version` 改回 `1.0.0`，再打包一次，把 `tauri-app.app` 複製到 `~/update-test/`。這是要被更新的舊版。

### 真的更新一次

伺服器的終端機還在跑。另開一個終端機，直接執行舊版 `.app` 裡的執行檔，輸出才看得到：

```bash
~/update-test/tauri-app.app/Contents/MacOS/tauri-app
```

`~/update-test/tauri-app.app/Contents/MacOS/tauri-app` 的輸出：

```text
目前版本 1.0.0
找到新版 1.1.0（目前 1.0.0）
下載完成
共下載 2985377 bytes，安裝完成，重新啟動
```

伺服器終端機印出的請求：

```text
::1 - - [23/Nov/2026 10:44:40] "GET /latest.json HTTP/1.1" 200 -
::1 - - [23/Nov/2026 10:44:40] "GET /tauri-app.app.tar.gz HTTP/1.1" 200 -
::1 - - [23/Nov/2026 10:44:41] "GET /latest.json HTTP/1.1" 200 -
```

前兩行是 1.0.0 問資訊檔、下載更新檔。第三行是一秒後重開的 App 又來問，後面沒有下載：它的版本已經跟 `latest.json` 一樣。`plutil -extract CFBundleShortVersionString raw -o - ~/update-test/tauri-app.app/Contents/Info.plist` 更新後讀到 `1.1.0`（打包時是 `1.0.0`），`.app` 確實被換掉了。

重開的 1.1.0 沒有輸出在終端機：依原始碼，macOS 的 `restart()` 用 `open -n -g` 重新打開 `.app`，新行程的輸出接到 `/dev/null`。所以「重開成 1.1.0」是從第三個請求、`Info.plist` 與 `ps` 看到新的行程在跑確認的，我們沒有去看視窗。

簽章對不上會怎樣？把 `latest.json` 的 `version` 改成 `1.2.0`（更新檔還是 1.1.0 簽的），用更新好的 1.1.0 再跑一次 `~/update-test/tauri-app.app/Contents/MacOS/tauri-app`，輸出：

```text
目前版本 1.1.0
找到新版 1.2.0（目前 1.1.0）
下載完成
更新失敗：The update was signed for version 1.1.0 but the update endpoint announced version 1.2.0. The endpoint response may have been tampered with to force installing a different release.
```

檔案下載了，但簽章記的是 1.1.0，跟宣告的不同，plugin 拒絕安裝，錯誤走到我們的 `eprintln!`。

這集測了：macOS（Apple Silicon）、Tauri 2.12、updater plugin 2.14.1，http 的本機伺服器，`.app` 放在家目錄，從終端機啟動。沒測：`https`、Windows、Linux、安裝到 `/Applications`（要有寫入權限）、從 Finder 開啟。

## 重點整理

- `plugins.updater.endpoints` 寫資訊檔網址；release 版只收 `https`，本機測試才加 `dangerousInsecureTransportProtocol`，正式發佈要拿掉。
- 在 Rust：`app.updater()?.check().await?` 得到 `Option<Update>`，有的話 `download_and_install(...)`，最後 `app.restart()`。
- 下載後先驗簽章，驗不過不安裝；簽章裡的版本跟 `latest.json` 的 `version` 不同也不安裝。
- 檢查用 `tauri::async_runtime::spawn` 在背景做；新版也要含更新程式碼，每次改程式碼都要重新簽章、更新 `latest.json`。
