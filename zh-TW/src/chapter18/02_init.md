# `tauri android init` 與 `tauri ios init`

## 本集目標

用 `tauri android init` 與 `tauri ios init` 產生行動版的專案，看它在 `src-tauri/gen/` 放了什麼、沒動你的什麼，以及這個資料夾要不要進 Git。

## 正文

### 兩個指令

第 1 集的工具裝好之後，在專案根目錄各跑一次。這兩行就是第 1 章第 3 集 create-tauri-app 印出來的提示：

```bash
npm run tauri android init
npm run tauri ios init
```

iOS 的指令只有 macOS 上才有，CLI 的原始碼把整個 iOS 子命令包在 `cfg(target_os = "macos")` 裡。

一個專案做一次就好。之後 `dev` 和 `build` 找不到產生出來的資料夾，會報錯叫你先 init。

三個旗標，各一句：

- `--ci`：略過互動詢問，例如 Android 找不到 SDK 時，不問要不要下載，直接報錯。環境變數 `CI` 有設，就等於加了它。
- `--skip-targets-install`：不替你補裝缺的 Rust target。
- `--reinstall-deps`：只有 iOS 有，連已經裝好的相依工具也重裝。

旗標跟前面章節一樣要放在 `--` 後面，例如 `npm run tauri android init -- --ci`。

### Android 專案

init 先確認 JDK、SDK、NDK 找得到，補裝缺的 Rust target，再把範本寫進 `src-tauri/gen/android/`。

這是一個完整的 Android Studio 專案，用 Gradle（Android 的建置工具）建置。幾個以後會碰到的檔案：

- `app/src/main/AndroidManifest.xml`：App 的宣告檔，權限寫在這裡。
- `app/src/main/java/tw/example/tauri_app/MainActivity.kt`：Android 的入口，範本寫好了。
- `app/build.gradle.kts`：App 這個模組的 Gradle 設定。
- `buildSrc/`：Gradle 建置時，這裡的程式會回頭呼叫 tauri CLI 去編 Rust。

### iOS 專案

iOS 同樣先補 Rust target，再檢查 XcodeGen、libimobiledevice、CocoaPods 這些工具，缺的會自動裝（怎麼裝，第 1 集說過）。

接著把範本寫進 `src-tauri/gen/apple/`，最後呼叫 XcodeGen 依 `project.yml` 產生 `.xcodeproj`。XcodeGen 是用一份文字檔描述 Xcode 專案、再產生它的工具。

資料夾叫 `apple`，不是 `ios`。官方的環境變數頁還寫 `gen/ios`，依原始碼以 `gen/apple` 為準。

### init 沒動你的檔案

`Cargo.toml`、`tauri.conf.json`、`src/`、`package.json`、`capabilities/` 都不會被改，init 的專案檔只寫在 `gen/` 底下。

不用改，是因為範本的 `[lib]` 一開始就是行動版要的樣子（第 1 章第 4 集提過）：

```toml,ignore
[lib]
name = "tauri_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

三種 crate-type 各有用處：

- `cdylib`：Android 載入的 `libtauri_app_lib.so`。
- `staticlib`：iOS 連結的 `libtauri_app_lib.a`。少了它，iOS 的建置會報錯，提醒要有 `staticlib`。
- `rlib`：桌面版 `main.rs` 呼叫 `tauri_app_lib::run()` 用的。

名字加 `_lib`，是為了不跟執行檔同名，範本的註解說 Windows 上會衝突。`lib.rs` 裡的 `#[cfg_attr(mobile, tauri::mobile_entry_point)]` 也是範本本來就有的，第 4 集解釋。

### 一個 identifier，兩個 id

範本的 `identifier` 是 `tw.example.tauri-app`。CLI 依平台改寫它：

| 平台 | 用的名稱 | 值 |
| --- | --- | --- |
| Android | applicationId，也是 Kotlin 的套件名 | `tw.example.tauri_app` |
| iOS | bundle id | `tw.example.tauri-app` |

Android 的 `-` 換成 `_`，因為 Java 與 Kotlin 的套件名不能有連字號；iOS 不變。同一個 App，在兩邊的 id 不一樣。

不要自己把 `identifier` 改成底線。`tauri build`、`tauri android build`、`tauri ios build` 都會檢查它，只准英數字、連字號與句點，底線直接報錯。改寫是 CLI 的事，保留連字號就好。

### 改了名字要重新 init

- Android：資料夾依 `identifier` 建。改了之後 CLI 找不到對應的資料夾，會報錯叫你刪掉 `gen/android` 重跑 init。
- iOS：Xcode 專案用 `Cargo.toml` 的 `[package] name` 命名。改了它，CLI 會問要不要幫你改專案名，不要就報錯，一樣叫你刪掉 `gen/apple` 重跑。
- iOS 的 `identifier` 與 `productName` 不用重跑，`dev` 和 `build` 會把它們同步進 Xcode 專案。

其他名字（像 `[lib] name`）改了會怎樣，我們沒有查證。

### 圖示：先 init 再 `tauri icon`

第 15 章第 2 集說過，`tauri icon` 會產生 `ios/` 與 `android/`。去處看 `gen/` 在不在：

- `gen/android/app/src/main/res/` 或 `gen/apple/Assets.xcassets/AppIcon.appiconset/` 已經存在，直接寫進去。
- 不存在，才寫進 `src-tauri/icons/android/` 與 `src-tauri/icons/ios/`。

init 不會把後面這兩個資料夾複製進 `gen/`。所以第 15 章跑過的 `tauri icon`，行動版圖示不會自動用上；init 之後再跑一次 `npm run tauri icon <來源圖>`。

### `gen/` 要不要進 Git

範本的 `src-tauri/.gitignore` 只排除 `/target/` 與 `/gen/schemas`（第 1 章第 4 集），所以 `gen/android` 與 `gen/apple` 預設會被 Git 追蹤。

兩個資料夾裡各有自己的 `.gitignore`，排除建置輸出、`local.properties`（本機的 SDK 路徑）與 `keystore.properties`（第 6 集的簽章設定）。

之後要手改 `gen/` 裡的檔案，例如第 6 集改 `app/build.gradle.kts`。改動只存在這裡，換一台電腦或重跑 init 就沒了，所以別把 `gen/` 加進 `.gitignore`，連同手改的檔案一起 commit。官方文件沒有哪一句明說要 commit `gen/`，這是從 `.gitignore` 與文件要你手改檔案推論的。

### 這集實測了什麼

- 沒有實測。這台電腦沒有 Android 與 iOS 的開發工具，兩個 init 指令都沒跑過，也就不知道實際會印什麼，書中不貼輸出。
- 內容依 v2.tauri.app 的 CLI 參考，以及 tauri-cli 2.12.5 與它用的 cargo-mobile2 0.22.5 的原始碼整理：產生的資料夾、init 做的事、`identifier` 的改寫與檢查、`tauri icon` 的去處。
- 沒查證：`gen/` 裡每個檔案的細節、`[lib] name` 改名的影響，以及官方有沒有建議 commit `gen/`。

## 重點整理

- `npm run tauri android init` 與 `npm run tauri ios init` 把專案產生在 `src-tauri/gen/android` 與 `gen/apple`；iOS 只有 macOS 才有。
- init 的專案檔都在 `gen/`，不改 `Cargo.toml`、`tauri.conf.json` 與 `src/`；範本的 `[lib]` 三種 crate-type 早就為行動版備好。
- 同一個 `identifier`，Android 的 applicationId 把 `-` 換成 `_`，iOS 的 bundle id 不變；別自己改成底線，build 會擋。
- 改了 `identifier`（Android）或 `[package] name`（iOS），要刪掉 `gen/` 裡對應的資料夾重跑 init；`gen/` 預設不被 Git 排除，手改的檔案要 commit。
- 先 init，再跑 `tauri icon`，圖示才會寫進 `gen/`。
