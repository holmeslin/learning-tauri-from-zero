# Linux：AppImage、deb、rpm

## 本集目標

在 Linux 上跑 `npm run tauri build`，看它做出 deb、rpm、AppImage 三個檔案，並分清楚三者最大的差別：App 要用的系統函式庫，是交給系統裝，還是跟著檔案一起帶。

## 正文

### 這一集的環境

這一集的指令是實際跑過的：OrbStack 2.2.5 裡的 Ubuntu 26.04.1 LTS（arm64），裝好第 1 章第 2 集那組套件（`libwebkit2gtk-4.1-dev` 是 2.52.6），Rust 1.99.1、Node.js v26.5.0、Tauri CLI 2.12.5。專案是起始專案，一個檔案都沒改，也沒有開啟打包出來的 App 視窗。

三種格式只有在 Linux 上打得出來。範本的 `bundle.targets` 是 `"all"`，在 Linux 上展開成 `deb`、`rpm`、`appimage`；同一份設定在 macOS 上是 `app` 與 `dmg`。

`npm run tauri build` 的輸出（節錄，中間的編譯訊息省略）：

```text
     Running beforeBuildCommand `npm run build`
   Compiling tauri v2.12.6
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `release` profile [optimized] target(s) in 1m 45s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling tauri-app_0.1.0_arm64.deb (~/tauri-app/src-tauri/target/release/bundle/deb/tauri-app_0.1.0_arm64.deb)
    Bundling tauri-app-0.1.0-1.aarch64.rpm (~/tauri-app/src-tauri/target/release/bundle/rpm/tauri-app-0.1.0-1.aarch64.rpm)
    Bundling tauri-app_0.1.0_aarch64.AppImage (~/tauri-app/src-tauri/target/release/bundle/appimage/tauri-app_0.1.0_aarch64.AppImage)
    Finished 3 bundles at:
        ~/tauri-app/src-tauri/target/release/bundle/deb/tauri-app_0.1.0_arm64.deb (2.37 MiB)
        ~/tauri-app/src-tauri/target/release/bundle/rpm/tauri-app-0.1.0-1.aarch64.rpm (2.38 MiB)
        ~/tauri-app/src-tauri/target/release/bundle/appimage/tauri-app_0.1.0_aarch64.AppImage (79.89 MiB)
```

第一次打 AppImage 時，輸出裡會多幾行 `Downloading`：打包工具要從網路下載 `linuxdeploy` 一類的工具，存在 `~/.cache/tauri`，之後就不用再下載。

### 檔名裡的架構

這台機器是 arm64，檔名就是 arm64 的拼法，而且三種格式拼得不一樣。沒有指定 `--target` 的時候，檔名的架構就是你打包那台機器的架構：

| 格式 | 這次打出來的（arm64） | x86_64 機器會得到的 |
| --- | --- | --- |
| deb | `tauri-app_0.1.0_arm64.deb` | `tauri-app_0.1.0_amd64.deb` |
| rpm | `tauri-app-0.1.0-1.aarch64.rpm` | `tauri-app-0.1.0-1.x86_64.rpm` |
| AppImage | `tauri-app_0.1.0_aarch64.AppImage` | `tauri-app_0.1.0_amd64.AppImage` |

最後一欄是照 tauri-bundler 的原始碼整理的，我們沒有在 x86_64 的機器上跑過。要給別種 CPU 的使用者，就得在那種 CPU 的機器上打；一次打好各平台，是第 17 章 GitHub Actions 的事。

### deb 與 rpm：函式庫交給系統

deb 與 rpm 都只放你的程式、桌面項目和圖示，App 要用的函式庫不放進去，改成寫在「相依套件」清單裡，由安裝時的套件管理員去裝。`dpkg-deb -I tauri-app_0.1.0_arm64.deb` 的輸出（節錄）：

```text
 Package: tauri-app
 Version: 0.1.0
 Architecture: arm64
 Maintainer: you
 Depends: libayatana-appindicator3-1, libwebkit2gtk-4.1-0, libgtk-3-0
```

`libwebkit2gtk-4.1-0` 與 `libgtk-3-0` 是每個 Tauri App 都會有的，也就是第 1 章第 1 集說的 Linux 版 WebView（WebKitGTK）。`libayatana-appindicator3-1` 是這個範本才有的：第 8 章替 `tauri` 開了 `tray-icon`，CLI 看到就把系統匣用的函式庫也加進清單。用沒有 `tray-icon` 的專案打包，清單會少這一個。`Maintainer: you` 來自 `Cargo.toml` 的 `authors`，想改就改那裡。

rpm 一樣，只是清單寫成函式庫檔名。`rpm -qpR tauri-app-0.1.0-1.aarch64.rpm` 的輸出（節錄）：

```text
libayatana-appindicator3.so.1()(64bit)
libgtk-3.so.0()(64bit)
libwebkit2gtk-4.1.so.0()(64bit)
```

打包這兩種檔案，Tauri 都是自己組出來的，不需要 `rpmbuild`。我們的 Ubuntu 在打包時沒有裝 `rpm` 相關工具，照樣打出了 `.rpm`；上面那行指令，是之後才裝了 `rpm` 來查看內容。

在 Ubuntu 上用 `apt` 安裝，它會順便把清單裡缺的套件裝起來：

```bash
sudo apt install ./tauri-app_0.1.0_arm64.deb
```

`dpkg -L tauri-app` 列出的檔案（只留檔案，不列資料夾）：

```text
/usr/bin/tauri-app
/usr/share/applications/tauri-app.desktop
/usr/share/icons/hicolor/32x32/apps/tauri-app.png
/usr/share/icons/hicolor/128x128/apps/tauri-app.png
/usr/share/icons/hicolor/256x256@2/apps/tauri-app.png
```

執行檔在 `/usr/bin`，圖示放進 `hicolor` 圖示主題，`.desktop` 檔則讓桌面環境的選單找得到這個 App。`.desktop` 檔的內容：

```text
[Desktop Entry]
Categories=
Comment=A Tauri App
Exec=tauri-app
StartupWMClass=tauri-app
Icon=tauri-app
Name=tauri-app
Terminal=false
Type=Application
```

`Exec=` 這一行果然只有執行檔名稱，沒有 `%F`、`%U` 這種欄位碼，跟第 12 章第 7 集從原始碼讀到的一樣。`Categories=` 是空的，因為範本的 `bundle` 沒有設 `category`。

### AppImage：全部帶著走

AppImage 是另一個做法：它是單一個檔案，`file` 看它是 ELF 執行檔，設計上不用安裝就能執行（我們沒有開啟它）。它也不寫相依套件清單，而是把函式庫整包放進檔案裡。`bundle/appimage/tauri-app.AppDir/usr/lib` 是打包前的檔案樹，有 162 個項目，約 266 MB，其中 `libwebkit2gtk-4.1.so.0`（WebKitGTK）一個就有 94 MB，`libgtk-3.so.0` 也在裡面。同一個 App，AppImage 有 79.89 MiB，deb 只有 2.37 MiB，差的主要就是這些。

每種格式旁邊都會留一個打包前的資料夾（`tauri-app.AppDir`、`tauri-app_0.1.0_arm64`），要上傳或發佈時，只拿 `.AppImage`、`.deb`、`.rpm` 這三個檔案就好。

### 該挑哪一種

- Debian、Ubuntu 系列的使用者：給 `.deb`，檔案小，裝了就進選單。
- Fedora、openSUSE 系列的使用者：給 `.rpm`。我們沒有在這些發行版上試過安裝。
- 不確定使用者用哪個發行版，或不想管他們的系統裝了什麼：給 `.AppImage`，代價是檔案大了三十幾倍。

安裝檔的簽章，第 16 章會正式介紹。

## 重點整理

- 在 Linux 上，`bundle.targets: "all"` 做出 `deb`、`rpm`、`appimage`，都放在 `src-tauri/target/release/bundle/` 底下各自的資料夾。
- 檔名的架構跟著打包的機器走，三種格式拼法不同：arm64 是 `arm64`／`aarch64`／`aarch64`，x86_64 是 `amd64`／`x86_64`／`amd64`。
- deb 與 rpm 很小，WebKitGTK、GTK 寫成相依套件由系統裝；`tray-icon` 會讓系統匣的函式庫也加進清單。
- AppImage 把函式庫整包帶著，不用安裝，但大了三十幾倍。
- 這一集在 Ubuntu 26.04 arm64 實測，沒有開啟打包出來的視窗；x86_64 的檔名依原始碼整理，Fedora 與 openSUSE 沒有試過。
