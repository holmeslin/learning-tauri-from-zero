# 圖示（`tauri icon`）

## 本集目標

準備一張方形的圖，用 `tauri icon` 一次產生各平台要的全部圖示，並確認打包出來的 App 用的是新圖示。

## 正文

### 為什麼一張圖要變一堆檔案

第 1 章第 4 集看過，`src-tauri/icons/` 放的是 App 圖示，範本給了一組預設的。打開來看會發現檔案很多，因為每個平台吃的格式不一樣：macOS 用 `.icns`，Windows 用 `.ico`，Linux 用不同尺寸的 PNG。自己一張一張轉很麻煩，`tauri icon` 就是做這件事的：給它一張圖，它轉出全部。

### 準備來源圖

來源圖要是**正方形**，背景透明的 PNG 或 SVG。我們用 SVG，因為它是文字，直接寫在專案根目錄就好：一個藍色圓角方塊，中間一個白色圓環，四周留一點透明的邊。

`app-icon.svg`：

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <rect x="64" y="64" width="896" height="896" rx="200" fill="#2b6cb0"/>
  <circle cx="512" cy="512" r="256" fill="#ffffff"/>
  <circle cx="512" cy="512" r="128" fill="#2b6cb0"/>
</svg>
```

自己的圖建議 1024×1024。程式不檢查最小尺寸（我們用 512 的 PNG 試過也能轉），但產生出來最大的一張是 1024，來源比它小就會被放大，邊緣會糊。

### 產生圖示

```bash
npm run tauri icon app-icon.svg
```

指令預設讀專案根目錄的 `app-icon.png`，檔名剛好是這個的話，後面的參數可以省略。

`npm run tauri icon app-icon.svg` 的輸出（節錄）：

```text
        Appx Creating StoreLogo.png
        ICNS Creating icon.icns
         ICO Creating icon.ico
         PNG Creating 32x32.png
         PNG Creating 64x64.png
         PNG Creating 128x128.png
         PNG Creating 128x128@2x.png
         PNG Creating icon.png
         iOS Creating AppIcon-20x20@2x-1.png
     Android Creating mipmap-hdpi/ic_launcher.png
```

檔案寫進 `src-tauri/icons/`，**同名的會直接蓋掉**，沒有確認。已經有自己改過的圖示，先備份。裡面的東西是：

- `icon.icns` 給 macOS，`icon.ico` 給 Windows，一個檔案裡收了好幾種尺寸。
- `32x32.png`、`64x64.png`、`128x128.png`、`128x128@2x.png`（256 像素）、`icon.png`（512 像素）是 PNG，Linux 用。
- `StoreLogo.png` 與 `Square*Logo.png` 是 Windows 商店（Microsoft Store）用的尺寸。
- `ios/` 與 `android/` 是手機版用的，第 18 章才會碰到。桌面 App 用不到，不想看到可以刪掉。

跟範本原本的比，多了 `64x64.png`、`ios/` 與 `android/`，其他檔名都一樣。

### `bundle.icon` 挑哪些檔案

打包用哪些圖示，是 `tauri.conf.json` 的 `bundle.icon` 決定的（第 1 章第 6 集看過）：

```json,ignore
"icon": [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico"
]
```

`tauri icon` 不會改 `tauri.conf.json`。產生出來的檔名跟範本一樣，所以這份清單不用動。macOS 的 `.app` 會用清單裡的 `.icns`，Windows 的安裝檔用 `.ico`，Linux 用 PNG。如果你換了檔名或另外放圖示，清單要跟著改。

來源圖不是正方形的話，指令會停下來，說 `Source image must be square`。加上 `--fit contain`（整張縮進正方形，不夠的地方補透明）或 `--fit cover`（填滿正方形，多出來的裁掉）就能轉，跟 CSS 的 `object-fit` 同一個意思。

### 重新打包，確認圖示換了

```bash
npm run tauri build -- --bundles app
```

`npm run tauri build -- --bundles app` 的輸出（節錄）：

```text
    Finished `release` profile [optimized] target(s) in 40.40s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (3.91 MiB)
```

在 Finder 看 `src-tauri/target/release/bundle/macos/`，`tauri-app.app` 的圖示就是剛剛畫的那個。想用指令確認的話，`.app` 裡的圖示在 `Contents/Resources/icon.icns`，跟 `icons/icon.icns` 比一下雜湊值（你的數字會不同，重點是兩行一樣）：

```bash
shasum src-tauri/icons/icon.icns src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/icon.icns
```

`shasum` 的輸出：

```text
04d3a882ece3feca11bcec20ba59db1ab2fe5c9c  src-tauri/icons/icon.icns
04d3a882ece3feca11bcec20ba59db1ab2fe5c9c  src-tauri/target/release/bundle/macos/tauri-app.app/Contents/Resources/icon.icns
```

## 重點整理

- 各平台吃的圖示格式不同，`npm run tauri icon <來源圖>` 從一張正方形的 PNG 或 SVG 產生全部，建議 1024×1024。
- 輸出寫進 `src-tauri/icons/`，同名檔案直接覆蓋；另外會多出手機版用的 `ios/` 與 `android/`。
- 打包用哪些圖示由 `bundle.icon` 決定，`tauri icon` 不會改它；檔名沒變，清單就不用動。
- 來源圖不是正方形，要加 `--fit contain` 或 `--fit cover`。
