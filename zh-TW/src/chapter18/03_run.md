# 在模擬器與手機上執行

## 本集目標

用 `tauri android dev` 與 `tauri ios dev` 把 App 跑在模擬器或實機上，並知道 CLI 怎麼選裝置、手機怎麼連到前端的開發伺服器、log 要去哪裡看。

## 正文

這一集接著第 2 集：`src-tauri/gen/android` 與 `src-tauri/gen/apple` 已經產生好了。專案裡沒有任何檔案要改。沒跑過 init 的話，CLI 會叫你先跑。

### 兩個指令

```bash
npm run tauri android dev
npm run tauri ios dev
```

用法跟桌面的 `npm run tauri dev` 一樣：CLI 先跑 `beforeDevCommand`（`npm run dev`，啟動 Vite），再用 `devUrl` 載入頁面。範本的 `tauri.conf.json` 不用改。

第一次要把所有相依套件編成手機的目標，文件說可能要好幾分鐘，之後有快取，快很多。要加選項的話，跟前面各章一樣放在 `--` 後面，例如 `npm run tauri android dev -- --open`。

### 跑在哪個裝置上

CLI 照這個順序找：

| | Android | iOS |
| --- | --- | --- |
| 先找 | `adb` 看得到的裝置：接上的實機，或已經在跑的模擬器 | 接上的 iPhone |
| 找不到 | 列出 AVD，啟動你選的那個 | 列出模擬器，啟動你選的那個 |

`adb`（Android Debug Bridge）是 Android SDK 的 `platform-tools` 裡的指令列工具，`adb devices` 會列出它看得到的裝置。AVD（Android Virtual Device）是你在 Android Studio 建好的 Android 模擬器。怎麼建，Tauri 文件沒有寫，這集不展開。

候選有好幾個，CLI 會問你選哪個；只有一個就直接用。也可以把名字接在 `dev` 後面，不用 `--`，換成你自己的裝置名：

```bash
npm run tauri ios dev 'iPhone 15'
```

名字是模糊比對，不必打完整。

### 改用 Android Studio 或 Xcode 跑

```bash
npm run tauri android dev -- --open
```

`--open` 不替你選裝置，改成打開 Android Studio（iOS 是 Xcode），由你在 IDE 裡按執行。文件說這能幫你排查一些開發問題。

文件特別提醒：CLI 這個行程一定要開著、不能關掉，直到你關掉 IDE 為止。

### Android 實機

實機要先打開 USB 偵錯。文件的步驟：到手機的「設定」，進「關於」，把「版本號碼」點 7 次，開啟開發人員模式；再進「開發人員選項」，打開 USB 偵錯。每家廠商的選單不一樣，找不到就查手冊。

接上電腦後，用 `adb devices` 確認看得到它；CLI 找裝置時，會先找到它。

### iPhone 實機

iOS 強制簽章，在實機上跑要告訴 CLI 用哪個開發團隊（team）。有兩種設法，環境變數優先：

```bash
APPLE_DEVELOPMENT_TEAM=<你的 Team ID> npm run tauri ios dev
```

或在 `tauri.conf.json` 設 `bundle.iOS.developmentTeam`。CLI 會找你的簽章憑證，找到零個或好幾個，它會警告你要自己設；`npm run tauri info` 會列出找到的憑證。免費的 Apple ID 能不能在實機上跑，Tauri 文件沒說，我們沒有查證。

iPhone 另外要在「設定 > Safari > 進階」打開 Web Inspector（英文介面的名稱），才能除錯前端。

### 手機怎麼連到開發伺服器

手機或模擬器裡的 `localhost` 指的是它自己，不是你的電腦，所以 CLI 用兩種辦法讓它連得到。

- `devUrl` 的主機是 `localhost` 時，CLI 對選到的 Android 裝置執行 `adb reverse tcp:1420 tcp:1420`，把裝置上的 1420 埠轉到你電腦的 1420 埠。
- CLI 選到的是實機、加了 `--host`，或 `devUrl` 的主機是 `0.0.0.0` 時，CLI 把 `devUrl` 的主機換成你電腦在區域網路的 IP，再設環境變數 `TAURI_DEV_HOST`，最後才跑 `beforeDevCommand`。Windows 上 `--host` 預設就是開的，CLI 原始碼的註解說是因為 adb 的埠轉發在 Windows 上不好用。

第二種要靠開發伺服器配合。範本的 `vite.config.ts` 已經寫好了，節錄兩段：

```ts,ignore
const host = process.env.TAURI_DEV_HOST;
// ...
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
```

沒有 `TAURI_DEV_HOST`（桌面、模擬器）時，`host` 是 `false`，Vite 只聽 `localhost`，`hmr` 用預設。有的話，Vite 改聽那個 IP，熱重載的 WebSocket 走 1421 埠。你不用改。

`--open` 的時候 CLI 沒選裝置，不會自動換。要在實機上跑，得自己加 `--host`：`npm run tauri ios dev -- --open --host`。iOS 打開 Xcode 卻沒加時，CLI 會警告。

iOS 第一次執行，系統會問要不要允許這個 App 找區域網路上的裝置。按允許，再重開 App。

### 看 log 與除錯

Rust 的 `println!` 與 `eprintln!` 照常能用，只是去的地方不同。

- Android：stdout 與 stderr 會接到 Android 的系統 log（logcat），tag 是 `RustStdoutStderr`。不加 `--open` 時，CLI 會把這個 App 的 logcat 接到你的終端機；用 `--open` 時在 Android Studio 看，怎麼過濾，我們沒有查證。
- iOS：debug 版會把 stdout 與 stderr 轉成 os_log（Apple 的系統 log）。`ios dev` 是 debug 版，所以看得到；release 版預設不會。實際顯示在哪、長什麼樣子，沒有查證。

前端的 `console.log` 要用 WebView 的檢查工具。

- iOS：Mac 的 Safari 設定 > 進階，打開 Show features for web developers，選單列會多一個 Develop 選單。從裡面選你的裝置或模擬器，點 `localhost`。
- Android：在電腦的 Chrome 開 `chrome://inspect`，App 在跑的話，裝置會出現在清單裡，點 inspect。模擬器預設就能檢查，實機要先開 USB 偵錯。

Xcode 專案有一個 build phase 會執行 Tauri CLI，它的 PATH 可能不是你終端機的那份。用 nvm 這類管理 Node 的工具時，可能找不到 Node，文件有提醒。

### 這集實測了什麼

沒有實測。作者的電腦沒有 Android 與 iOS 的開發工具，沒有模擬器，也沒有實機，所以這集沒有貼任何輸出。

內容依 v2.tauri.app 的開發文件，以及 tauri-cli 2.12.5 與它用的 cargo-mobile2 的原始碼查證。裝置的選法、`devUrl` 的改寫、`adb reverse`、log 的去向是讀原始碼得來的；USB 偵錯與檢查工具的步驟照文件寫。都沒有看過它們實際執行。

## 重點整理

- `npm run tauri android dev` 與 `npm run tauri ios dev` 跟桌面的 `dev` 一樣用 `beforeDevCommand` 與 `devUrl`；先找接上的裝置，沒有就啟動你選的模擬器。
- `--open` 改成打開 Android Studio 或 Xcode，CLI 要一直開著。
- 實機要連到開發伺服器，CLI 會把 `devUrl` 換成區域網路 IP 並設 `TAURI_DEV_HOST`，範本的 `vite.config.ts` 已經接好。
- Android 實機要開 USB 偵錯；iPhone 實機要設 `APPLE_DEVELOPMENT_TEAM`。
- Rust 的 `println!` 在 Android 看 logcat 的 `RustStdoutStderr`；前端用 Safari 或 `chrome://inspect` 除錯。
