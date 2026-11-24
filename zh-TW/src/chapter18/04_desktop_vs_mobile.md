# 桌面與行動的差異

## 本集目標

看懂範本裡的 `#[cfg_attr(mobile, ...)]` 與 `#[cfg(desktop)]`：同一份 Rust 程式碼怎麼同時編成桌面與手機的 App，以及手機上少了哪些東西。

## 正文

這一集從起始專案出發，只改 `src-tauri/src/lib.rs`，不用跑任何指令。

### 手機的 App 是一個函式庫

桌面的 App 從 `main` 開始，手機不是。第 2 集產生的 Android 與 iOS 專案，是把 Rust 編成的函式庫放進去（Android 用 `cdylib`，iOS 用 `staticlib`，就是 `Cargo.toml` 的 `crate-type`），啟動時呼叫函式庫裡的入口。

所以範本把 App 的內容都放在 `lib.rs` 的 `run()`，`main.rs` 只剩一行 `tauri_app_lib::run()`。手機要呼叫哪個函式，靠 `run()` 上面那行指定：

```rust,ignore
#[cfg_attr(mobile, tauri::mobile_entry_point)]
```

`mobile_entry_point` 替 `run()` 補上手機端需要的東西：Android 的 JNI 綁定（Java 呼叫原生程式碼的介面）、iOS 呼叫用的 C 符號 `start_app`，還有把 panic 攔下來，直接中止程式，不讓它穿過 FFI 邊界。

這個巨集只在手機的編譯目標才存在，桌面編譯找不到它，所以寫成 `cfg_attr`：條件成立，才加上這個屬性。桌面上這行等於沒寫，`main.rs` 照樣呼叫同一個 `run()`。

忘了這行會怎樣？依 tauri-cli 的原始碼，Android 與 iOS 的建置腳本都會檢查函式庫裡有沒有入口，沒有就回報 `does not include required runtime symbols`，說你可能漏了 `tauri::mobile_entry_point`。

### `cfg(mobile)` 與 `cfg(desktop)` 從哪來

`mobile` 與 `desktop` 不是 Rust 內建的 cfg（內建的是 `target_os = "android"` 這一類）。它們是 `build.rs` 裡的 `tauri_build::build()` 設的別名：目標系統是 Android 或 iOS 就是 `mobile`，其他全部是 `desktop`。所以 `build.rs` 要有這一行（範本有），你的程式碼才能用這兩個 cfg。

`Cargo.toml` 看不到建置腳本設的別名。第 8 章那個桌面專用的相依套件區塊，只能寫成原始的 `cfg(not(any(target_os = "android", target_os = "ios")))`。

Rust 程式碼裡有三種用法：

| 寫法 | 意思 |
| --- | --- |
| `#[cfg(desktop)]` | 條件不成立，這段程式碼直接移除，不編譯 |
| `#[cfg_attr(mobile, X)]` | 條件成立，才加上屬性 `X` |
| `cfg!(mobile)` | 運算式，值是 `true` 或 `false`，兩邊的程式碼都要編得過 |

只想指定其中一個系統，改用 `target_os = "android"` 或 `"ios"`。

### 範例：隔開桌面專用的程式碼

下面的 `lib.rs` 是範本加兩處。`greet` 用 `cfg!(mobile)` 回報自己跑在哪一邊。`center_window` 用到只有桌面才有的 `center()`（把視窗置中，這裡只拿它當例子），所以寫成兩個版本。

`src-tauri/src/lib.rs`：

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    let kind = if cfg!(mobile) { "手機" } else { "桌面" };
    format!("Hello, {name}! 這裡是{kind}上的 Rust。")
}

#[cfg(desktop)]
fn center_window(app: &tauri::App) -> tauri::Result<()> {
    use tauri::Manager;

    if let Some(window) = app.get_webview_window("main") {
        window.center()?;
    }
    Ok(())
}

#[cfg(mobile)]
fn center_window(_app: &tauri::App) -> tauri::Result<()> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            center_window(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- 兩個 `center_window` 同名，編譯時只留下其中一個。手機那個什麼都不做。
- `Window::center()` 在手機上不存在，不是呼叫了沒反應。`setup` 裡直接寫它，手機的編譯會失敗。
- 桌面上 `greet` 回傳「這裡是桌面上的 Rust」。照第 3 集在手機上跑起來再按 Greet，依 `cfg!(mobile)` 的定義會是「手機」，我們沒有實測。
- 檢查程式只編桌面，`#[cfg(mobile)]` 那個函式沒被編譯過，所以只讓它回 `Ok(())`。

### 手機上沒有的功能

下面這些 API 只在桌面編譯。手機上它們不存在，不是呼叫了沒反應：

- 選單與右鍵選單：`tauri::menu` 整個模組只在桌面。
- 系統匣：`tauri::tray` 同樣只在桌面。
- 視窗的細部控制：`center`、`maximize`、`set_decorations`、`set_always_on_top` 這一類。
- 全域快捷鍵、只允許開一個、開機自動啟動、自動更新：這四個 plugin 手機都不支援（下一節）。

桌面編得過的程式碼用到這些，要像上面那樣用 `#[cfg(desktop)]` 隔開，不然手機的編譯會失敗。

也有編得過、但手機不採用的：`tauri.conf.json` 視窗的 `width` 與 `height`，手機上會被忽略。

手機也不是不能開多個視窗。官方有一頁 Multi-Window on Mobile，要另外設定，Android 與 iOS 各有系統版本的要求。本書不用，也不展開。

另外有手機才有的：`WindowEvent::Suspended` 與 `WindowEvent::Resumed`，App 離開前景與回到前景時送出，桌面不會有。

### 範本的 plugin 在手機上能不能用

下面是各 plugin 在 Android 與 iOS 的支援程度（兩個系統相同）：

| plugin | 手機 | 限制 |
| --- | --- | --- |
| store、log | 完整 | |
| notification | 完整 | 要真的向使用者要權限（下一節） |
| opener | 部分 | 只能開網址 |
| dialog | 部分 | 不支援選資料夾 |
| fs | 部分 | 預設只能存取 App 自己的資料夾 |
| clipboard-manager | 部分 | 只支援純文字 |
| autostart、global-shortcut、single-instance、updater | 沒有 | |

沒有的那四個，前三個的 crate 檔頭是 `#![cfg(not(any(target_os = "android", target_os = "ios")))]`：在手機上整個 crate 是空的，連 `tauri_plugin_global_shortcut::Builder` 這個名字都找不到。所以註冊它們的程式碼一定要用 `#[cfg(desktop)]` 隔開，就是第 8 章第 6、7 集的寫法。updater 的 crate 在手機上編得過，但安裝更新會回傳 `UnsupportedOs`，手機的 App 靠商店更新（第 16 章第 1 集）。

capability 沒寫 `platforms` 就是所有平台都適用（第 14 章第 2 集）。範本的 `capabilities/default.json` 沒寫，所以手機也用同一份。

### notification：手機要真的問

第 7 章第 8 集說過，桌面版的 notification 不會問：Rust 端的 `request_permission()` 與 `permission_state()` 一律回傳已允許。手機版會真的問，依原始碼，Android 上是向系統要 `POST_NOTIFICATIONS` 這個權限。那一集的寫法本來就是先問、被拒就不送，所以兩邊通用，不用改。

### 這集實測了什麼

- 實測：上面的 `lib.rs` 通過 checkcode（`cargo fmt`、`clippy -D warnings`、`tsc`），那是桌面編譯。另外在桌面跑了一個單元測試，確認 `greet` 回傳「桌面」、`cfg!(mobile)` 是 `false`。
- 沒測：手機的一切。`#[cfg(mobile)]` 那個函式、`mobile_entry_point` 展開後的樣子、手機上 `greet` 回傳「手機」，檢查程式沒編譯過，我們也沒有在模擬器或實機跑過。
- 依據：tauri 2.12.6、tauri-build 2.7.5、tauri-macros 2.7.5、tauri-runtime-wry 2.12.4 與各 plugin 的原始碼（版本依範本的 `Cargo.lock`）。plugin 的支援程度出自各自 `Cargo.toml` 的 `[package.metadata.platforms.support.*]`，缺少入口的錯誤訊息出自 tauri-cli 2.12.5，多視窗依 v2.tauri.app。

## 重點整理

- 手機把 App 當函式庫載入，入口是 `lib.rs` 的 `run()`；`#[cfg_attr(mobile, tauri::mobile_entry_point)]` 只在手機上套用，桌面的 `main.rs` 照樣呼叫它。
- `mobile` 與 `desktop` 是 `tauri_build::build()` 設的 cfg 別名：`#[cfg]` 移除程式碼，`cfg_attr` 加屬性，`cfg!` 是布林運算式。
- 選單、系統匣、視窗的細部控制，以及 autostart、global-shortcut、single-instance、updater，手機上都沒有，要用 `#[cfg(desktop)]` 隔開；檢查程式只編桌面，看不到手機的編譯錯誤。
- opener、dialog、fs、clipboard-manager 在手機上功能有限；notification 在手機要真的向使用者要權限。
