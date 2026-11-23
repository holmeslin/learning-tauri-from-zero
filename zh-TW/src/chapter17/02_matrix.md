# 三平台 matrix

## 本集目標

把第 1 集的 workflow 擴成 macOS、Linux、Windows 三個 job 同時跑，補上 Linux 要的系統套件，並看三個平台各自做出哪些檔案。

## 正文

這一集接著第 1 集的專案，帶過來的檔案只有 `.github/workflows/release.yml`，下面整份換掉。

### matrix：同一個 job 跑三次

第 1 集的 job 寫死了 `runs-on: macos-latest`。要三個平台不用複製三份，改用 matrix：

```yaml
    strategy:
      fail-fast: false
      matrix:
        os: [macos-latest, ubuntu-22.04, windows-latest]
    runs-on: ${{ matrix.os }}
```

`matrix.os` 有三個值，GitHub 就把這個 job 展開成三個，各自在一台 runner 上同時跑，`${{ matrix.os }}` 換成那一個的值。`fail-fast` 預設是 `true`：一個 job 失敗，其他還沒跑完的會被取消（官方文件的說法）。我們要看每個平台自己的結果，所以關掉。

Linux 選 `ubuntu-22.04`，不選 `ubuntu-latest`，因為 `-latest` 會在 GitHub 換映像時跟著換，打包的環境就變了。我們沒有試過別的 Ubuntu 版本。

### 先不裝套件，看會怎樣

只改上面那段，其他不動，推上去跑。`gh run view` 的輸出（節錄）：

```text
X main release · 45130985979
JOBS
✓ build (macos-latest) in 7m24s (ID 137546178884)
✓ build (windows-latest) in 7m2s (ID 137546179275)
X build (ubuntu-22.04) in 42s (ID 137546179415)
```

macOS 與 Windows 都成功，不用額外設定。Linux 不到一分鐘就失敗，因為 `fail-fast: false`，另外兩個照樣跑完。失敗的 tauri-action step 的 log（節錄）：

```text
error: failed to run custom build command for `gdk-sys v0.18.2`
  pkg-config exited with status code 1
  > PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1 pkg-config --libs --cflags gdk-3.0 'gdk-3.0 >= 3.22'
    Package gdk-3.0 was not found in the pkg-config search path.
  The system library `gdk-3.0` required by crate `gdk-sys` was not found.
##[error]Command "npm ["run","tauri","build"]" failed with exit code 1
```

第 1 章第 2 集裝過的 GTK 與 WebKitGTK，runner 上沒有。Rust 的 `gdk-sys` crate 編譯時找不到系統的 `gdk-3.0`，整個編譯就停了。Windows 與 macOS 用系統內建的 WebView，所以不用裝。

### 補上 Linux 的套件

第 1 章第 2 集那份 `apt` 清單裡，`build-essential`、`curl`、`wget`、`file`、`libssl-dev` 不用裝：我們只裝下面四個，編譯與三種安裝檔都過了。`.github/workflows/release.yml`：

```yaml
name: release

on: workflow_dispatch

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        os: [macos-latest, ubuntu-22.04, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: lts/*
      - name: install Linux dependencies
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev
      - run: npm ci
      - uses: tauri-apps/tauri-action@v1
        with:
          uploadWorkflowArtifacts: true
```

`if: runner.os == 'Linux'` 讓這個 step 只在 Linux 的 runner 上做，其他兩個 job 跳過它。`runner.os` 是 GitHub 提供的值，`Linux` 是其中一個。這一步我們的 runner 花了約 28 秒。

### 三個平台做出什麼

這次三個 job 都成功了，整個 workflow 用了 8 分 57 秒（三個 job 同時跑，看最慢的）。各 job 的 tauri-action step 都有 `Finished N bundles`，整理起來：

| runner | 檔案 |
| --- | --- |
| `macos-latest` | `tauri-app_0.1.0_aarch64.dmg`（1.93 MiB）、`tauri-app.app` |
| `windows-latest` | `tauri-app_0.1.0_x64_en-US.msi`（2.11 MiB）、`tauri-app_0.1.0_x64-setup.exe`（1.42 MiB） |
| `ubuntu-22.04` | `tauri-app_0.1.0_amd64.deb`（2.46 MiB）、`tauri-app-0.1.0-1.x86_64.rpm`（2.47 MiB）、`tauri-app_0.1.0_amd64.AppImage`（77.51 MiB） |

這跟第 15 章第 5、6 集的內容一致：Windows 的兩個檔名、Linux 的 `amd64`、`x86_64`、`amd64` 拼法（那時只讀了原始碼，現在在 x86_64 的機器上對上了），Windows 的 log 也看得到下載 WiX 3.14 與 NSIS 3.13：

```text
 Downloading https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip
     Running light to produce D:\a\tauri-app\tauri-app\src-tauri\target\release\bundle\msi\tauri-app_0.1.0_x64_en-US.msi
 Downloading https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.13/nsis-3.13.zip
     Running makensis to produce D:\a\tauri-app\tauri-app\src-tauri\target\release\bundle\nsis\tauri-app_0.1.0_x64-setup.exe
```

（路徑裡的 repo 名稱換成了 `tauri-app`，其餘沒改。）如果你沒有 Windows 機器，這個 job 就是第 15 章第 5 集說的「第 17 章的 CI」。

workflow artifact 有七個：`darwin-aarch64-dmg`、`darwin-aarch64-app`、`windows-x64-msi`、`windows-x64-nsis`、`linux-amd64-deb`、`linux-x86_64-rpm`、`linux-amd64-appimage`。名字的架構照各檔案自己的拼法，所以 Linux 的 deb 與 rpm 不一樣。

### 這集實測了什麼

- 三個 runner 的映像檔：`macos-26-arm64`（20261117.0516.1）、`ubuntu-22.04`（20261116.365.1）、`windows-2025-vs2026`（20261116.315.1），tauri-action v1.0.0。
- 各 job 耗時：macOS 5 分 42 秒、Linux 7 分 50 秒、Windows 8 分 54 秒。同一份 workflow 在加套件之前跑的那次，macOS 7 分 24 秒、Windows 7 分 2 秒，同樣的程式碼，時間會差一兩分鐘。
- 沒測：Intel 的 Mac（`macos-latest` 是 arm64，要另外處理）、Linux 的 arm64、`ubuntu-latest`、安裝檔在各平台能不能裝與開。

## 重點整理

- `strategy.matrix` 把一個 job 展開成每個值一個，`runs-on: ${{ matrix.os }}` 取用；`fail-fast: false` 讓一個平台失敗時，其他平台照樣跑完。
- macOS 與 Windows 的 runner 不用額外設定；Linux 要先用 `apt` 裝 GTK 與 WebKitGTK 等系統套件，否則 `gdk-sys` 編譯失敗。
- `if: runner.os == 'Linux'` 讓 step 只在某個平台做。
- 三個平台做出的檔名，跟第 15 章整理的一致：`.dmg`、MSI 與 NSIS、deb、rpm、AppImage。
