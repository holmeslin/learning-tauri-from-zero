# 前置需求（Android Studio、Xcode）

## 本集目標

裝好把專案做成 Android 與 iOS App 需要的開發工具，並設好環境變數與 Rust 的 target。這集只準備環境，專案要到第 2 集才動手。

## 正文

### 桌面之外多裝什麼

第 1 章第 2 集裝的是桌面的前置需求。做手機要另外準備，Android 與 iOS 各一份：

| | Android | iOS |
| --- | --- | --- |
| 作業系統 | macOS、Windows、Linux | 只有 macOS |
| 要裝的工具 | Android Studio，再用它裝 SDK 與 NDK | 完整的 Xcode、Homebrew、CocoaPods |
| Rust target | 4 個 | 3 個 |

官方文件把兩邊分成兩段寫。只想做其中一邊，就照那一邊準備。

### Android：Android Studio 與 SDK

先到 Android 開發者網站裝 Android Studio。裝好之後，在它的 SDK Manager 裝這五個元件：

| 元件 | 做什麼 |
| --- | --- |
| Android SDK Platform | Android 系統的函式庫與介面 |
| Android SDK Platform-Tools | 裡面有 `adb`，電腦靠它跟模擬器（emulator）、實機（physical device）溝通，第 3 集才用到 |
| NDK (Side by side) | C/C++ 的編譯工具，Rust 編成 Android 的函式庫時，用它的 `clang` 當連結器 |
| Android SDK Build-Tools | 建置 App 用的工具 |
| Android SDK Command-line Tools | 命令列的管理工具，裡面有 `sdkmanager` |

SDK Manager 在選單裡的位置會隨 Android Studio 的版本變，這裡不寫。

### Android 的三個環境變數

官方文件要你設三個環境變數。`JAVA_HOME` 指向 Android Studio 內附的 Java，`ANDROID_HOME` 指向 SDK，`NDK_HOME` 指向某一版 NDK。macOS 的寫法：

```bash
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk)"
```

把這幾行放進 shell 的設定檔（zsh 是 `~/.zshrc`），每次開新的終端機才都有。Windows 與 Linux 的路徑不一樣，照官方 Prerequisites 頁面（<https://v2.tauri.app/start/prerequisites/>）的「Configure for Mobile Targets」設。

最後一行用 `ls -1` 取 NDK 的資料夾名稱。`ndk` 裡裝了不只一版的話，會抓到好幾個名字，這時改成直接寫出你要的那一版。

### Android 的 Rust target

Rust 要有 Android 的 target，才編得出 Android 能用的函式庫：

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

四個 target 對應四種 CPU 架構，Android 稱為 ABI。官方文件的順序是先裝好；沒先裝也可以，第 2 集的 `tauri android init` 會補裝缺的。

### iOS：完整的 Xcode

iOS 只能在 macOS 上做。第 1 章第 2 集裝的 Command Line Tools 只夠做桌面，iOS 要裝完整的 Xcode。官方文件特別提醒，不能拿 Command Line Tools 頂替。

從 Mac App Store 或 Apple Developer 網站安裝。裝好後先開一次 Xcode，讓它完成設定。

第 1 章第 2 集裝過 Command Line Tools 的話，再跑一次 `xcode-select -p`，看現在指的是不是 Xcode。如果還是 `/Library/Developer/CommandLineTools`，用 `sudo xcode-select -s /Applications/Xcode.app/Contents/Developer` 切過去，不然需要完整 Xcode 的工具可能找不到它。

官方文件沒寫這一步，依據是 cargo-mobile2（CLI 用的函式庫）的原始碼：它自己的檢查把前者當成不對的路徑，建議改成後者。

### iOS 的 Rust target、Homebrew 與 CocoaPods

```bash
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
```

`aarch64-apple-ios` 給實機。另外兩個給 iOS 的模擬器（simulator）：`x86_64-apple-ios` 是 Intel Mac 用的，`aarch64-apple-ios-sim` 是 Apple Silicon 用的。

接著官方文件要你裝 Homebrew（<https://brew.sh>），再用它裝 CocoaPods，一個管理 iOS 專案相依套件的工具：

```bash
brew install cocoapods
```

### CLI 自己會補的部分

上面是官方文件要你做的。tauri-cli 2.12.5 的原始碼還多做了一些事，文件沒有寫，所以只當備案：

- `JAVA_HOME` 沒設，找 Android Studio 的預設位置；找不到再看 PATH 上有沒有 `java`。
- `ANDROID_HOME` 沒設，改讀 `ANDROID_SDK_ROOT`，再找預設的 SDK 資料夾（macOS 是 `~/Library/Android/sdk`）。
- `NDK_HOME` 沒設，在 `$ANDROID_HOME/ndk` 裡挑排在最後的一版。
- 找不到 SDK 時，終端機可以互動的話，會問你要不要下載 command-line tools，再用 `sdkmanager` 裝；不能互動的環境（像 CI）直接報錯。
- `NDK_HOME` 有設就必須是存在的資料夾，否則直接報錯，不會改去找預設的。

iOS 這邊，`tauri ios init` 會檢查 XcodeGen（產生 Xcode 專案）與 libimobiledevice（跟 iPhone 實機溝通），缺了就用 Homebrew 裝，所以 Homebrew 一定要有。CocoaPods 沒裝的話，它不是走 Homebrew，而是用 `sudo gem install`，所以照上面先用 `brew install` 裝好。

沒有 iOS 的模擬器可用時，CLI 會問你要不要下載最新的 iOS runtime，執行的是 `xcodebuild -downloadPlatform iOS`。第 3 集的 `ios dev` 可能碰到這個。

### 這集實測了什麼

沒有實測。寫這一章的電腦沒有 Android 與 iOS 的開發工具，所以這集的指令與設定都沒有跑過，也沒有貼輸出。

- 依據：Tauri 官方 Prerequisites 頁面的「Configure for Mobile Targets」，以及 tauri-cli 2.12.5 與 cargo-mobile2 0.22.5 的原始碼。
- 原始碼那一節（CLI 自己會補的部分），只讀了程式，沒有驗證它實際跑起來的行為。
- `adb`、NDK 與 `sdkmanager` 的說明查過原始碼。SDK Platform 與 Build-Tools 的說明是 Android 的一般知識，沒有查證。
- `xcode-select` 那一步官方文件沒寫，只讀了 cargo-mobile2 的原始碼，沒有跑過；Command Line Tools 還在用時，Xcode 的工具實際會出什麼錯，沒有查證。
- Android Studio 與 Xcode 的版本、選單位置與畫面，沒有查證。

## 重點整理

- 做手機要在桌面的前置需求之外，另外準備 Android 與 iOS 兩份；iOS 只能在 macOS 上做。
- Android 要裝 Android Studio，再用 SDK Manager 裝五個元件，設 `JAVA_HOME`、`ANDROID_HOME`、`NDK_HOME`，並加四個 Rust target。
- iOS 要完整的 Xcode（不是 Command Line Tools，裝好後確認 `xcode-select -p` 指向它）、三個 Rust target、Homebrew 與 CocoaPods。
- CLI 對沒設的環境變數與缺少的工具有備案，但官方文件的步驟照做最保險。
