# Linux：AppImage、deb、rpm

## 本集目標

在 Linux 上跑 `npm run tauri build`，看它做出 deb、rpm、AppImage 三個檔案，並分清楚三者最大的差別：App 要用的系統函式庫，是交給系統裝，還是跟著檔案一起帶。

## 正文

### 這一集的環境

這一集的指令是實際跑過的：OrbStack 2.2.5 裡新建的 Ubuntu 26.04.1 LTS（arm64），裝好第 1 章第 2 集那組套件（`libwebkit2gtk-4.1-dev` 是 2.52.6），Rust 1.99.1、Node.js v26.5.0、Tauri CLI 2.12.5。專案是起始專案，複製到 Linux 自己的檔案系統，在那裡 `npm install`，一個檔案都沒改。打包出來的 App，我們只試過它能不能啟動，沒有看過視窗的畫面。

三種格式只有在 Linux 上打得出來。範本的 `bundle.targets` 是 `"all"`，在 Linux 上展開成 `deb`、`rpm`、`appimage`；同一份設定在 macOS 上是 `app` 與 `dmg`。

`npm run tauri build` 的輸出（節錄，中間的編譯訊息省略）：

```text
     Running beforeBuildCommand `npm run build`
   Compiling tauri v2.12.6
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `release` profile [optimized] target(s) in 1m 19s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling tauri-app_0.1.0_arm64.deb (~/tauri-app/src-tauri/target/release/bundle/deb/tauri-app_0.1.0_arm64.deb)
    Bundling tauri-app-0.1.0-1.aarch64.rpm (~/tauri-app/src-tauri/target/release/bundle/rpm/tauri-app-0.1.0-1.aarch64.rpm)
    Bundling tauri-app_0.1.0_aarch64.AppImage (~/tauri-app/src-tauri/target/release/bundle/appimage/tauri-app_0.1.0_aarch64.AppImage)
    Finished 3 bundles at:
        ~/tauri-app/src-tauri/target/release/bundle/deb/tauri-app_0.1.0_arm64.deb (2.21 MiB)
        ~/tauri-app/src-tauri/target/release/bundle/rpm/tauri-app-0.1.0-1.aarch64.rpm (2.22 MiB)
        ~/tauri-app/src-tauri/target/release/bundle/appimage/tauri-app_0.1.0_aarch64.AppImage (79.60 MiB)
```

第一次打 AppImage 時，輸出裡會多幾行 `Downloading`：打包工具要從網路下載 `linuxdeploy` 一類的工具，存在 `~/.cache/tauri`，之後就不用再下載。

### 檔名裡的架構

這台機器是 arm64，檔名就是 arm64 的拼法，而且三種格式拼得不一樣。沒有指定 `--target` 的時候，檔名的架構就是你打包那台機器的架構：

| 格式 | 這次打出來的（arm64） | x86_64 機器會得到的 |
| --- | --- | --- |
| deb | `tauri-app_0.1.0_arm64.deb` | `tauri-app_0.1.0_amd64.deb` |
| rpm | `tauri-app-0.1.0-1.aarch64.rpm` | `tauri-app-0.1.0-1.x86_64.rpm` |
| AppImage | `tauri-app_0.1.0_aarch64.AppImage` | `tauri-app_0.1.0_amd64.AppImage` |

最後一欄是照 tauri-bundler 的原始碼整理的，這裡沒有實際打過；第 17 章在 GitHub Actions 的 x86_64 機器上打包時，會看到這三個檔名。要給別種 CPU 的使用者，最直接的做法是在那種 CPU 的機器上打，一次打好各平台就是第 17 章要做的事。

### deb 與 rpm：函式庫交給系統

deb 與 rpm 都只放你的程式、桌面項目和圖示，App 要用的函式庫不放進去，改成寫在「相依套件」清單裡，由安裝時的套件管理員去裝。`dpkg-deb -I tauri-app_0.1.0_arm64.deb` 的輸出（節錄）：

```text
 Package: tauri-app
 Version: 0.1.0
 Architecture: arm64
 Maintainer: you
 Depends: libwebkit2gtk-4.1-0, libgtk-3-0
```

`libwebkit2gtk-4.1-0` 是第 1 章第 1 集說的 Linux 版 WebView（WebKitGTK），`libgtk-3-0` 是視窗用的 GTK，這兩個每個 Tauri App 都會有。專案若像第 8 章那樣替 `tauri` 開了 `tray-icon`，原始碼會再把系統匣用的 `libayatana-appindicator3-1` 加進清單，這一點我們沒有再用那種專案打包確認。`Maintainer: you` 來自 `Cargo.toml` 的 `authors`，想改就改那裡。

rpm 一樣，只是清單寫成函式庫檔名。`rpm -qpR tauri-app-0.1.0-1.aarch64.rpm` 的輸出（節錄）：

```text
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
/usr/share/icons/hicolor/32x32/apps/tauri-app.png
/usr/share/icons/hicolor/128x128/apps/tauri-app.png
/usr/share/icons/hicolor/256x256@2/apps/tauri-app.png
/usr/share/applications/tauri-app.desktop
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

AppImage 是另一個做法：它是單一個檔案，`file` 看它是 ELF 執行檔，不用安裝就能執行。它也不寫相依套件清單，而是把函式庫整包放進檔案裡。打包前的檔案樹留在 `bundle/appimage/tauri-app.AppDir/`，其中 `usr/lib` 沒壓縮的大小是 265 MiB，`libwebkit2gtk-4.1.so.0`（WebKitGTK）一個就有 94 MiB，`libgtk-3.so.0` 也在裡面。AppImage 本身是壓縮過的，79.60 MiB；同樣是壓縮檔的 deb 只有 2.21 MiB，差的主要就是這些函式庫。

每種格式旁邊都會留一個打包前的資料夾（`tauri-app.AppDir`、`tauri-app_0.1.0_arm64`），要上傳或發佈時，只拿 `.AppImage`、`.deb`、`.rpm` 這三個檔案就好。

### 能不能啟動

VM 沒有螢幕，我們另外裝了虛擬畫面工具 `xvfb`，用 `xvfb-run` 分別啟動裝好的 `tauri-app` 與 AppImage，兩個在 12 到 15 秒的限時結束前都還在執行，沒有當掉。這台 VM 只有 `fuse3`、沒有 `libfuse2`，AppImage 一樣能啟動，沒有出現 FUSE 的錯誤；別的系統怎樣，我們沒有試。

### 該挑哪一種

- Debian、Ubuntu 系列的使用者：給 `.deb`，檔案小，裝了就進選單。
- Fedora、openSUSE 系列的使用者：給 `.rpm`。我們沒有在這些發行版上試過安裝。
- 不確定使用者用哪個發行版，或不想管他們的系統裝了什麼：給 `.AppImage`，代價是檔案大了三十幾倍。

Linux 安裝檔的簽章，本書不介紹。

## 重點整理

- 在 Linux 上，`bundle.targets: "all"` 做出 `deb`、`rpm`、`appimage`，都放在 `src-tauri/target/release/bundle/` 底下各自的資料夾。
- 檔名的架構跟著打包的機器走，三種格式拼法不同：arm64 是 `arm64`／`aarch64`／`aarch64`，x86_64 是 `amd64`／`x86_64`／`amd64`。
- deb 與 rpm 只有 2 MiB 出頭，WebKitGTK、GTK 寫成相依套件由系統裝；開了 `tray-icon` 的專案，系統匣的函式庫也會加進清單。
- AppImage 把函式庫整包帶著，不用安裝，但壓縮後也有 79.60 MiB，是 deb 的三十幾倍。
