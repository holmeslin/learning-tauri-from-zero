# bundle 設定與版本號

## 本集目標

看懂 `tauri.conf.json` 裡 `bundle`、`productName`、`version` 各管什麼，換成自己的名稱與版本號，並在打包出來的 `Info.plist` 看到結果。

## 正文

### `bundle` 的三個欄位

範本的 `bundle`（第 1 章第 6 集）有三個欄位：

- `active`：要不要打包。沒寫的預設值是 `false`，範本寫了 `true`。設成 `false`，`tauri build` 就只編譯，效果跟 `--no-bundle` 一樣，我們試過，`bundle/` 資料夾不會出現。
- `targets`：要做哪些格式。`"all"` 是這個平台做得出來的全部：macOS 是 `app` 與 `dmg`，Windows 是 `msi` 與 `nsis`，Linux 是 `deb`、`rpm` 與 `appimage`。也可以寫成陣列，例如 `["app"]`。指定了別的平台才有的格式，會被略過：我們在 macOS 試過 `["msi"]`，不報錯，也沒有東西被打包。命令列的 `--bundles` 優先，它會蓋掉這個欄位（第 1 集介紹過）。
- `icon`：上一集的圖示清單。

### `productName` 與 `version`

- `productName` 是給使用者看的名字，也是安裝檔的檔名。`.app` 叫 `<productName>.app`，`.dmg` 叫 `<productName>_<version>_<架構>.dmg`（下一集會看到 `tauri-app_0.1.0_aarch64.dmg`）。名字裡不能有檔名不能用的字元，像 `/` 與 `:`。範本的 `tauri-app` 只是預設值，所以第 1 集的輸出才會有 `Warn`。
- `version` 要是 `主.次.修` 的 semver 字串，寫成 `1.0` 會被擋下來：`tauri.conf.json > version` must be a semver string。

專案裡有三個 `0.1.0`：`tauri.conf.json`、`Cargo.toml` 與 `package.json`。它們各自獨立，改一個不會動另外兩個。打進安裝檔的版本號看 `tauri.conf.json` 的 `version`；那一欄如果沒寫，才用 `Cargo.toml` 的 `package.version`。不想同步兩個地方，`version` 可以改寫成 `package.json` 的路徑，路徑從 `tauri.conf.json` 所在的資料夾算起：

```json,ignore
"version": "../package.json"
```

我們把 `package.json` 的版本改成 `2.0.0` 試過，打包出來的 `.dmg` 叫 `..._2.0.0_aarch64.dmg`。各平台對版本號格式的要求不同，Windows 的 MSI 最嚴格，細節在第 5 集。

### 替安裝檔加上說明

`bundle` 還有幾個說明用的欄位。在 macOS 看得到的只有兩個：

- `copyright`：版權文字，寫進 `.app` 的 `Info.plist`。
- `category`：App 的分類，例如 `Productivity`，macOS 與 Linux 用它。

`publisher`（發行者）、`shortDescription`（簡短說明）這類欄位，是 Windows 的安裝檔與 Linux 的套件才會用到，macOS 的 `.app` 裡找不到它們。

### 改成自己的

換個名字、版本號與說明：

`src-tauri/tauri.conf.json`（改了 `productName`、`version` 與 `bundle`）：

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Hello Tauri",
  "version": "1.2.0",
  "identifier": "tw.example.tauri-app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "tauri-app",
        "width": 800,
        "height": 600
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "publisher": "Example Studio",
    "copyright": "Copyright © 2026 Example Studio",
    "category": "Productivity",
    "shortDescription": "我的第一個 Tauri App",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

```bash
npm run tauri build -- --bundles app
```

`npm run tauri build -- --bundles app` 的輸出（節錄）：

```text
     Running beforeBuildCommand `npm run build`
    Finished `release` profile [optimized] target(s) in 41.09s
       Built application at: ~/tauri-app/src-tauri/target/release/tauri-app
    Bundling Hello Tauri.app (~/tauri-app/src-tauri/target/release/bundle/macos/Hello Tauri.app)
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/Hello Tauri.app (3.95 MiB)
```

`Warn` 不見了，`.app` 也改叫 `Hello Tauri.app`。`Built application at` 的執行檔還是 `tauri-app`，那是 `Cargo.toml` 的 package 名稱；`productName` 管的是安裝檔與顯示給使用者的名字。看 `.app` 裡的 `Info.plist`：

```bash
plutil -p "src-tauri/target/release/bundle/macos/Hello Tauri.app/Contents/Info.plist"
```

`plutil -p` 的輸出（節錄）：

```text
  "CFBundleDisplayName" => "Hello Tauri"
  "CFBundleExecutable" => "tauri-app"
  "CFBundleIdentifier" => "tw.example.tauri-app"
  "CFBundleName" => "Hello Tauri"
  "CFBundleShortVersionString" => "1.2.0"
  "CFBundleVersion" => "1.2.0"
  "LSApplicationCategoryType" => "public.app-category.productivity"
  "NSHumanReadableCopyright" => "Copyright © 2026 Example Studio"
```

名稱、版本號、分類與版權都進來了，`publisher` 與 `shortDescription` 在裡面找不到，跟上面說的一樣。

## 重點整理

- `bundle.active` 決定要不要打包，`targets` 決定做哪些格式（`"all"` 是這個平台做得出來的全部），`--bundles` 會蓋掉 `targets`。
- `productName` 與 `version` 決定安裝檔的檔名與版本號；版本號要是 semver，只看 `tauri.conf.json`，也可以寫成 `"../package.json"`。
- `copyright`、`category` 在 macOS 的 `Info.plist` 看得到；`publisher`、`shortDescription` 等到 Windows 與 Linux 才會用上。
