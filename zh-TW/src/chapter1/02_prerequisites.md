# 安裝前置需求

## 本集目標

依照你的作業系統，裝好 Tauri 需要的系統工具，並確認 Rust 與 Node.js 都能用。

## 正文

### 要準備三樣東西

1. **系統工具**：Tauri 要連結系統的 WebView，Rust 編譯時也需要 C/C++ 的編譯與連結工具。每個作業系統要裝的東西不一樣，下面分開講。
2. **Rust**：讀《從零開始學 Rust》時已經裝好了，這集只確認版本。
3. **Node.js**：前端的工具（npm、Vite、TypeScript）都跑在它上面。

Tauri 本身不用另外裝。下一集建好的專案會把 Tauri 的命令列工具放在專案裡，用 `npm run tauri` 執行。

先照你的作業系統裝好系統工具，最後再一起處理 Node.js。各平台的步驟與下載連結，都以 Tauri 官方的 Prerequisites 頁面（<https://v2.tauri.app/start/prerequisites/>）為準。

### macOS

在終端機執行：

```bash
xcode-select --install
```

這會安裝 Command Line Tools，只做桌面 App 的話這樣就夠了。裝好之後可以確認：

```bash
xcode-select -p
```

`xcode-select -p` 的輸出：

```text
/Library/Developer/CommandLineTools
```

如果你本來就裝了完整的 Xcode，路徑會指到 Xcode 裡面，也沒問題。完整的 Xcode 要到第 18 章做 iOS 才一定需要。

### Windows

Windows 要處理三件事：

1. **Microsoft C++ Build Tools**：到 Microsoft 的 C++ Build Tools 下載頁拿安裝程式，執行後勾選「使用 C++ 的桌面開發」（Desktop development with C++）。
2. **WebView2**：Windows 10（1803 以後）和 Windows 11 通常已經內建。如果沒有，到 Microsoft 的 WebView2 下載頁安裝「Evergreen Bootstrapper」。
3. **Rust 的 MSVC 工具鏈**：Windows 上的 Rust 有 MSVC 和 GNU 兩種，Tauri 要用 MSVC。不確定的話，執行：

```bash
rustup default stable-msvc
```

裝完之後，關掉終端機再重開。打包 MSI 安裝檔還需要啟用 Windows 的 VBSCRIPT 功能，那是第 15 章的事，現在不用管。

### Linux

Linux 要裝的套件隨發行版不同。Debian 與 Ubuntu：

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

其中 `libwebkit2gtk-4.1-dev` 就是第 1 集提到的 Linux 版 WebView（WebKitGTK）。Arch、Fedora、openSUSE 等其他發行版的套件名稱不同，請照官方的 Prerequisites 頁面安裝。

### Node.js

到 <https://nodejs.org> 安裝目前的 LTS 版本。本書用到的工具裡，要求最高的是第 9 章的測試工具：Node.js 22 系列 22.22.2 以上、24 系列 24.15.0 以上，或 26 以上，裝目前的 LTS 就符合。裝好確認：

```bash
node -v
npm -v
```

`node -v` 與 `npm -v` 的輸出：

```text
v26.5.0
11.17.0
```

你的版本號碼不一樣沒關係，只要 `node` 是目前的 LTS 版本就好。

### 確認 Rust

```bash
rustc --version
cargo --version
```

`rustc --version` 與 `cargo --version` 的輸出：

```text
rustc 1.99.1 (58cf8f9c3 2026-10-13)
cargo 1.99.1 (5f94df478 2026-08-27)
```

本書以 Rust 1.99 為準。範本使用 edition 2024，至少需要 Rust 1.85。版本太舊就用 `rustup update` 更新。

## 重點整理

- 要準備系統工具、Rust、Node.js 三樣，Tauri 本身不用另外安裝。
- macOS 執行 `xcode-select --install`；Windows 裝 C++ Build Tools，確認 WebView2 與 MSVC 版的 Rust；Linux 用套件管理工具裝 WebKitGTK 等函式庫。
- Node.js 用目前的 LTS 版本，用 `node -v` 和 `npm -v` 確認。
- Rust 用 `rustc --version` 確認，本書以 1.99 為準。
