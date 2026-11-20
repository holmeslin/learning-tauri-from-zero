# macOS：`.app` 與 `.dmg`

## 本集目標

看懂 macOS 打包出來的 `.app` 與 `.dmg` 各是什麼，知道做 `.dmg` 為什麼會用到 Finder、沒有 Finder 的環境怎麼辦，以及前面幾章留下的「要打包之後才能測」的事該怎麼接。

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
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Info.plist
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/MacOS/tauri-app
src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/icon.icns
```

- `Contents/MacOS/tauri-app` 是執行檔。
- `Contents/Info.plist` 是 App 的資料：名稱、版本號、identifier，第 3 集看過。裡面的 `LSMinimumSystemVersion` 是 `10.13`，這是 Tauri 預設的 macOS 版本下限，想改用 `bundle.macOS.minimumSystemVersion`。
- `Contents/Resources/` 放圖示，第 7 集的資源檔也會放進這裡。

在 Finder 對 `.app` 按右鍵，選「顯示套件內容」就能看到這些。沒有安裝程式這回事：把 `.app` 拖進「應用程式」資料夾就是安裝，丟進垃圾桶就是移除。

### `.dmg` 是給別人下載的

`.app` 是資料夾，要散佈得先包成一個檔案，常見的做法是 `.dmg`，一個磁碟映像檔。使用者雙擊它，系統會掛載成一個磁碟，裡面有 App 和一個指向「應用程式」的捷徑，把前者拖到後者就裝好了。我們把做好的 `.dmg` 掛載起來看（`-nobrowse` 是不讓它出現在 Finder 的側邊欄與桌面，`-readonly` 是唯讀）：

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

`Applications` 是捷徑，`.VolumeIcon.icns` 是磁碟本身的圖示。

### 做 `.dmg` 會用到 Finder

`.dmg` 打開時的視窗大小，以及 App 與捷徑擺的位置，是 Tauri 用 AppleScript 遙控 Finder 排出來的，所以直接跑 `npm run tauri build`，做到 `.dmg` 那一步，macOS 會詢問終端機能不能控制 Finder，要按允許。沒人回應或不允許，這一步就會失敗，輸出像這樣（節錄）：

```text
    Bundling tauri-app_0.1.0_aarch64.dmg (~/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg)
     Running bundle_dmg.sh
/var/folders/…/createdmg.tmp.XXXXXXXXXX.4eQtCyum8m:394:406: execution error: Finder發生錯誤：AppleEvent逾時。 (-1712)
Failed running AppleScript
       Error failed to bundle project: error running bundle_dmg.sh: ...
```

我們遇到的是 `-1712`，Finder 一直沒回應，等到逾時。使用者明確按了不允許的話，錯誤碼是 `-1743`，依原始碼，Tauri 會多給一行提示，要你到「系統設定」的「隱私權與安全性」，在「自動化」裡放行。

沒有 Finder 可以用的環境，像沒人在螢幕前的 CI 機器，可以設環境變數 `CI=true`。Tauri 看到它就略過 AppleScript，`.dmg` 照樣做得出來，只是視窗沒有排版：沒有背景圖，圖示的位置也是系統預設的。GitHub Actions 預設就有這個變數，第 17 章會碰到。

`CI=true npm run tauri build -- --bundles dmg` 的輸出（節錄）：

```text
    Finished `release` profile [optimized] target(s) in 34.80s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
    Bundling tauri-app_0.1.0_aarch64.dmg (~/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg)
     Running bundle_dmg.sh
    Cleaning ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg (1.84 MiB)
```

兩個細節。`Cleaning` 那行：只要 `.dmg` 時，`.app` 是做了、包進 `.dmg` 之後又刪掉，所以 `macos/` 資料夾是空的；兩個都要，就別用 `--bundles dmg`，用預設的 `"all"` 或 `--bundles app,dmg`。另外，視窗的大小、背景圖與圖示位置，可以在 `bundle.macOS.dmg` 調整，第一次用預設的就夠了。

### 還沒有簽章

用 `codesign` 看剛才的 `.app`：

```bash
codesign -dv src-tauri/target/release/bundle/macos/tauri-app.app
```

`codesign -dv` 的輸出（節錄）：

```text
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20400 size=31736 flags=0x20002(adhoc,linker-signed) hashes=988+0 location=embedded
Signature=adhoc
TeamIdentifier=not set
```

`adhoc` 是編譯器自動加的臨時簽章，不是開發者簽章。這樣的 App 別人下載之後，macOS 的 Gatekeeper 會擋下來，簽章與公證，第 16 章會正式介紹。

### 做給哪種 Mac

`.dmg` 檔名裡的 `aarch64` 是 Apple Silicon，在 Intel 的 Mac 上打包會是 `x64`，在 Apple Silicon 上打包出來的是 arm64 的 App，Intel 的 Mac 跑不了。兩種都要支援的話，用 `--target universal-apple-darwin`，檔名會是 `universal`。要先用 `rustup target add` 裝好 `aarch64-apple-darwin` 與 `x86_64-apple-darwin` 兩個目標。這一段我們沒有實際跑，是依 Tauri CLI 的原始碼整理的。

### 裝好之後才算數

前面幾章留下三件事，要用安裝好的 `.app` 才測得到。這一章的專案是範本，沒有那些功能，所以我們沒有實測，這裡只交代怎麼接：

- 檔案關聯（第 12 章）：`bundle.fileAssociations` 打包時會寫進 `Info.plist`。回到第 12 章的專案打包，把 `.app` 拖進「應用程式」，就能在 Finder 雙擊 `.md` 檔，測 `RunEvent::Opened` 有沒有收到。
- 開機自動啟動（第 13 章）：autostart plugin 在 macOS 登記的是執行檔所在的 `.app`，我們查了它的原始碼。所以要用 `.app` 啟動再勾選，登記的才是安裝好的 App，不是 `target/debug/tauri-app`。
- 通知（第 7 章）：打包後的 App 用自己的 identifier，也就是 `Info.plist` 的 `CFBundleIdentifier`，送通知，所以系統的通知設定裡出現的是 App 的名字，不是「終端機」。

## 重點整理

- `.app` 是個資料夾：`Contents/MacOS` 放執行檔，`Info.plist` 放資料，`Resources` 放圖示與資源檔；拖進「應用程式」就是安裝。
- `.dmg` 是給人下載的磁碟映像檔，內容是 `.app` 加一個「應用程式」捷徑。
- 做 `.dmg` 要用 AppleScript 遙控 Finder，需要自動化權限；沒有 Finder 的環境設 `CI=true`，`.dmg` 一樣做得出來，只是沒有排版。
- `--bundles dmg` 做完會把 `.app` 刪掉；兩個都要用 `"all"` 或 `--bundles app,dmg`。
- 這個 `.app` 沒有開發者簽章，別人下載後會被 Gatekeeper 擋下；簽章與公證，第 16 章會正式介紹。
