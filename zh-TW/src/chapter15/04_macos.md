# macOS：`.app` 與 `.dmg`

## 本集目標

看懂 macOS 打包出來的 `.app` 與 `.dmg` 各是什麼，知道做 `.dmg` 為什麼會用到 Finder、沒有 Finder 的環境怎麼辦，以及前面幾章留下的「要打包之後才能測」的事，哪些現在查得到、哪些還得自己試。

## 正文

### `.app` 是一個資料夾

第 1 集做出來的 `tauri-app.app`，在 Finder 裡看起來是一個圖示，其實是一個資料夾：

```bash
find src-tauri/target/release/bundle/macos/tauri-app.app
```

`find` 的輸出：

```text
src-tauri/target/release/bundle/macos/tauri-app.app
src-tauri/target/release/bundle/macos/tauri-app.app/Contents
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/MacOS
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/MacOS/tauri-app
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/icon.icns
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Info.plist
```

- `Contents/MacOS/tauri-app` 是執行檔。
- `Contents/Info.plist` 是 App 的資料：名稱、版本號、identifier，第 3 集看過。裡面的 `LSMinimumSystemVersion` 是 `10.13`，是 Tauri 預設的 macOS 版本下限。
- `Contents/Resources/` 放圖示，第 7 集的資源檔也會放進這裡。

在 Finder 對 `.app` 按右鍵，選「顯示套件內容」就能看到這些。macOS 的 App 沒有安裝這個步驟：把 `.app` 拖進「應用程式」資料夾就是裝好了，丟進垃圾桶就是移除。

### 做 `.dmg`

`.app` 是資料夾，要散佈得先包成一個檔案，常見的做法是 `.dmg`，一個磁碟映像檔（disk image）。`bundle.targets` 預設的 `"all"`，在 macOS 就是 `app` 加 `dmg`，所以直接跑 `npm run tauri build` 兩個都會做。

`.dmg` 打開時的視窗大小，以及 App 與「應用程式」捷徑擺的位置，是 Tauri 用 AppleScript 遙控 Finder 排出來的。所以做到這一步，macOS 會詢問終端機能不能控制 Finder，要按允許。沒人回應或不允許，這一步就會失敗，輸出像這樣（節錄）：

```text
    Bundling tauri-app_0.1.0_aarch64.dmg (~/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg)
     Running bundle_dmg.sh
/var/folders/…/createdmg.tmp.XXXXXXXXXX.4eQtCyum8m:394:406: execution error: Finder發生錯誤：AppleEvent逾時。 (-1712)
Failed running AppleScript
       Error failed to bundle project: error running bundle_dmg.sh: ...
```

我們遇到的是 `-1712`，Finder 一直沒回應，等到逾時。使用者明確按了不允許的話，錯誤碼是 `-1743`，依原始碼，Tauri 會多給一行提示，要你到「系統設定」的「隱私權與安全性」，在「自動化」裡放行。

沒有 Finder 可以用的環境，像沒人在螢幕前的 CI 機器，可以設環境變數 `CI=true`。Tauri 看到它就略過 AppleScript，`.dmg` 照樣做得出來，只是視窗沒有排版：沒有背景圖，圖示的位置也是系統預設的。GitHub Actions 預設就有這個變數，第 17 章會碰到。

`CI=true npm run tauri build -- --bundles app,dmg` 的輸出（節錄）：

```text
    Finished `release` profile [optimized] target(s) in 45.25s
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
    Bundling tauri-app_0.1.0_aarch64.dmg (~/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg)
     Running bundle_dmg.sh
    Finished 2 bundles at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (3.95 MiB)
        ~/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg (1.81 MiB)
```

有個細節：只寫 `--bundles dmg` 的話，輸出會多一行 `Cleaning ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app`：`.app` 做了、包進 `.dmg` 之後又刪掉，`macos/` 資料夾是空的，所以兩個都要就寫 `app,dmg`。

### `.dmg` 裡面有什麼

把做好的 `.dmg` 掛載起來看（`-nobrowse` 是不讓它出現在 Finder 的側邊欄與桌面，`-readonly` 是唯讀）：

```bash
hdiutil attach -nobrowse -readonly src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg
ls -a /Volumes/tauri-app
hdiutil detach /Volumes/tauri-app
```

`ls -a /Volumes/tauri-app` 的輸出：

```text
.
..
.VolumeIcon.icns
Applications
tauri-app.app
```

`Applications` 是指向「應用程式」的捷徑，`.VolumeIcon.icns` 是磁碟本身的圖示。使用者雙擊 `.dmg`，系統掛載出這個磁碟，把 App 拖到捷徑上就裝好了。這個 `.app` 與 `.dmg` 都沒有簽章。沒簽章的 App 給別人用，macOS 的 Gatekeeper 會擋，這一點我們沒有試；簽章與公證，第 16 章會正式介紹。

### 做給哪種 Mac

`.dmg` 檔名裡的 `aarch64` 是 Apple Silicon，也就是我們這台機器。檔名的架構跟著打包的機器走，Intel 的 Mac 上會是 `x64`（這是依原始碼整理的，我們沒有 Intel 的機器可試）。Apple Silicon 打出來的是 arm64 的 App，Intel 的 Mac 跑不了。同時支援兩種的 universal 打包，這本書不展開。

### 裝好之後

前面幾章留下三件事，要用裝好的 `.app` 才測得到。起始專案沒有那些功能，所以我們只做得到兩件事：看打包的結果，以及讀原始碼。每一件都分開寫清楚查到什麼、沒測什麼。

**檔案關聯（第 12 章第 7 集）。** 把第 12 章第 7 集的 `bundle.fileAssociations` 原封不動加進起始專案的 `tauri.conf.json`（跟 `targets` 同一層），重新打包，看 `Info.plist`：

```bash
plutil -p src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Info.plist
```

`plutil -p` 的輸出（節錄）：

```text
  "CFBundleDocumentTypes" => [
    0 => {
      "CFBundleTypeExtensions" => [
        0 => "md"
        1 => "markdown"
      ]
      "CFBundleTypeName" => "Markdown"
      "CFBundleTypeRole" => "Editor"
      "LSHandlerRank" => "Default"
    }
  ]
```

副檔名、名稱與 `role` 都進了 `Info.plist`，macOS 就是看這一段知道 App 能開 `.md`。`mimeType` 與 `description` 沒有出現在這份 plist 裡。把 `.app` 拖進「應用程式」之後，在 Finder 雙擊 `.md` 會不會叫起 App、`RunEvent::Opened` 收不收得到，要在 Finder 操作才知道，我們沒有測。

**開機自動啟動（第 13 章第 6 集）。** `Builder::new().build()` 在 macOS 用預設的 `MacosLauncher::LaunchAgent`。我們讀 autostart plugin 2.7.0 的原始碼，它登記的是執行檔的路徑，也就是 `.app/Contents/MacOS/` 底下那個檔案，不是 `.app` 資料夾；只有改用 `AppleScript` 的時候，才會把路徑裁成 `.app`。所以用裝好的 App 勾選，plist 的 `ProgramArguments` 會是 `…/tauri-app.app/Contents/MacOS/tauri-app` 這種路徑，不再是第 13 章看到的 `target/debug/tauri-app`。登入之後 App 會不會真的自己跑起來，我們沒有測。

**通知（第 7 章第 8 集）。** 通知 plugin 的原始碼用 `tauri::is_dev()` 決定名義：是 `tauri dev` 就用「終端機」（`com.apple.Terminal`），否則用 `identifier`。`tauri build` 做出來的執行檔，`is_dev()` 是 `false`（`--debug` 也一樣，它載入的是打包進去的前端），所以打包後的通知會掛在 `CFBundleIdentifier`（這個專案是 `tw.example.tauri-app`）名下。這也只讀了原始碼，橫幅有沒有出現，要使用者允許通知才看得到，我們沒有測。

## 重點整理

- `.app` 是個資料夾：`Contents/MacOS` 放執行檔，`Info.plist` 放資料，`Resources` 放圖示與資源檔；拖進「應用程式」就是安裝。
- `.dmg` 是給人下載的磁碟映像檔，內容是 `.app` 加一個「應用程式」捷徑；預設的 `"all"` 兩個都會做，只寫 `--bundles dmg` 會把 `.app` 刪掉，兩個都要寫 `--bundles app,dmg`。
- 做 `.dmg` 要用 AppleScript 遙控 Finder，需要自動化權限；沒有 Finder 的環境設 `CI=true`，`.dmg` 一樣做得出來，只是沒有排版。
- 前面章節留下的三件事：`fileAssociations` 打包後會寫進 `Info.plist` 的 `CFBundleDocumentTypes`；autostart 預設登記的是 `.app/Contents/MacOS/` 裡的執行檔路徑；通知在 `tauri build` 之後掛在 App 自己的 `identifier` 名下。
