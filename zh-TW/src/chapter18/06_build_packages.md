# 打包 APK、AAB 與 IPA

## 本集目標

認識行動版的三種發佈檔：APK、AAB 與 IPA，知道各用哪個指令打、檔案放在哪裡，再看上架前的簽章與上傳流程。需要付費帳號的部分只說明流程，沒有跑。

## 正文

這一集沿用前幾集的專案：`src-tauri/gen/` 是第 2 集產生的，版本還是範本的 `0.1.0`。沒有要改的 Rust 與 TypeScript，所以沒有程式碼檢查點。簽章是什麼，第 16 章第 5 集講過；憑證與密碼只放 secret，第 17 章第 4 集講過。這集不重講，只看行動版的做法。

### 三種檔案

| 檔案 | 平台 | 用途 |
| --- | --- | --- |
| APK | Android | 直接裝在手機上，用來測試或在商店之外發佈 |
| AAB | Android | 上傳 Google Play 的格式，官方文件推薦用它 |
| IPA | iOS | iOS 的安裝檔，上傳 App Store Connect 用 |

### 打包 Android

```bash
npm run tauri android build
```

沒加旗標，APK 與 AAB 都會打。預設是 release 模式，四種 CPU 架構（`aarch64`、`armv7`、`i686`、`x86_64`）合成一個 universal 檔。跟第 15 章一樣，旗標前面要加 `--`：

```bash
npm run tauri android build -- --aab
npm run tauri android build -- --apk --split-per-abi
npm run tauri android build -- --aab --target aarch64 --target armv7
```

- `--aab`、`--apk`：只打指定的格式。
- `--target`：只編指定的架構，可以重複寫。
- `--split-per-abi`：每種架構各一個檔，不合成 universal（ABI 先當成 CPU 架構）。官方文件說它只適合測試或商店外發佈，Google Play 會自己處理架構。
- `--debug`：改打 debug 版。

CLI 先跑 `beforeBuildCommand`（`npm run build`），再叫 `gen/android` 裡的 Gradle 專案打包。Gradle 是 Android 專案用的建置工具。

### 檔案在哪

三個檔案的位置，都從 `src-tauri/` 算起：

```text
gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab
gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
gen/apple/build/arm64/<App 名稱>.ipa
```

CLI 打完會印出路徑。APK 的檔名，CLI 的原始碼同時找 `app-universal-release.apk` 與帶 `unsigned` 的版本，取修改時間較新的，所以沒簽章時檔名可能不一樣。

### 版本號

Android 要一個整數的 `versionCode`，預設由 `version` 算出：`主 * 1000000 + 次 * 1000 + 修`，`0.1.0` 就是 `1000`。想自己管，用 `bundle.android.versionCode` 直接指定，或把 `bundle.android.autoIncrementVersionCode` 設成 `true`，每次非 dev 的建置加 1。計數存在 `gen/android/app/tauri.properties`，要讓它進 Git，得先從 `gen/android/app/.gitignore` 拿掉 `/tauri.properties`。

iOS 的 `CFBundleVersion` 預設也取自 `version`，用 `bundle.iOS.bundleVersion` 改。最低系統版本是 `bundle.android.minSdkVersion`（預設 24，Android 7.0）與 `bundle.iOS.minimumSystemVersion`（預設 `15.0`）。

### Android 簽章

APK 與 AAB 上傳發佈之前都要簽章。Android 的簽章檔是你自己用 `keytool` 產生的 keystore，不用向誰申請憑證。流程三步。

第一步，產生 keystore。`keytool` 是 JDK 附的指令，找不到的話，Android Studio 附的 JDK 裡有，macOS 在 `/Applications/Android\ Studio.app/Contents/jbr/Contents/Home/bin/keytool`：

```bash
keytool -genkey -v -keystore ~/upload-keystore.jks -keyalg RSA -keysize 2048 -validity 10000 -alias upload
```

`.jks` 檔與你設的密碼自己保管，別放進公開的版本控制。

第二步，新增 `keystore.properties`，告訴 Gradle 去哪找 keystore：

`src-tauri/gen/android/keystore.properties`：

```properties
password=<執行 keytool 時設的密碼>
keyAlias=upload
storeFile=/Users/<你的名字>/upload-keystore.jks
```

init 產生的 `gen/android/.gitignore` 已經忽略它，就算你把 `gen/` 提交進 Git，它也不會跟著進去。

第三步，改 `src-tauri/gen/android/app/build.gradle.kts`，只動三處：檔案開頭加 `import java.io.FileInputStream`（已經有 `import java.util.Properties`）；`buildTypes` 前面加下面的 `signingConfigs`；`buildTypes` 裡既有的 `getByName("release")` 大括號內加一行 `signingConfig = signingConfigs.getByName("release")`。

```kotlin
signingConfigs {
    create("release") {
        val keystorePropertiesFile = rootProject.file("keystore.properties")
        val keystoreProperties = Properties()
        if (keystorePropertiesFile.exists()) {
            keystoreProperties.load(FileInputStream(keystorePropertiesFile))
        }

        keyAlias = keystoreProperties["keyAlias"] as String
        keyPassword = keystoreProperties["password"] as String
        storeFile = file(keystoreProperties["storeFile"] as String)
        storePassword = keystoreProperties["password"] as String
    }
}
```

之後的 release 建置會自動簽章。Google Play 另外還有一層 Play App Signing，這集不展開。

### 上傳 Google Play

要有 Google Play Console 的開發者帳號，先在網站上登錄 App。第一次上傳得在網站上手動做，Google 會藉此檢查簽章與套件名稱，Tauri 目前沒有自動建立 release 的功能。第 17 章的 CI 只打桌面版，行動版的 CI 這本書不展開。

### 打包 iOS

打 IPA 只能在 macOS 上做：

```bash
npm run tauri ios build -- --export-method app-store-connect
```

`--export-method` 有三個值：`app-store-connect` 給 App Store，`release-testing` 給 TestFlight 或 ad hoc 發佈，`debugging` 給開發用。TestFlight 是 Apple 給測試人員試用的管道。

沒加 `--export-method` 的話，CLI 以 `gen/apple/ExportOptions.plist` 為底，範本裡它的 `method` 是 `debugging`。所以沒加旗標打出來的是開發簽章的 IPA，不是上架用的。這是依原始碼推論，沒有實測。另一個旗標 `--build-number <N>`，說明寫它是把 N 接在 app 版本號後面。

### iOS 簽章與上傳（沒有實測）

這一段沒有跑。跟第 16 章第 5 集的 Developer ID 一樣，要付費加入 Apple Developer Program，我們沒有帳號。官方文件寫撰寫時是每年 99 美元，簽章也要在 Apple 的裝置上做。流程大致如下：

1. 在 App Store Connect（Apple 管理上架的網站）登錄 App，Bundle ID 必須跟 `tauri.conf.json` 的 `identifier` 一樣。
2. 準備憑證與描述檔（provisioning profile，把 Bundle ID 與憑證綁在一起）。憑證類型要對得上 export method：`debugging` 用 Apple Development，`app-store-connect` 與 `release-testing` 用 Apple Distribution。
3. 用 `--export-method` 打包，簽章交給 Xcode。
4. 用 `xcrun altool` 上傳 IPA，Apple 驗證通過後會出現在 TestFlight。

第 2 步有兩種做法。自動簽章是預設：在 Xcode 的 Settings > Accounts 分頁加入帳號，團隊還是第 3 集的 `APPLE_DEVELOPMENT_TEAM`。CI 上沒有 Xcode 帳號，就建一把 App Store Connect API 金鑰，設 `APPLE_API_ISSUER`、`APPLE_API_KEY` 與 `APPLE_API_KEY_PATH`（`.p8` 檔的路徑）。手動簽章則自己設 `IOS_CERTIFICATE`（`.p12` 憑證轉 base64）、`IOS_CERTIFICATE_PASSWORD` 與 `IOS_MOBILE_PROVISION`（描述檔轉 base64）。

上傳指令如下，`$APPNAME` 換成 App 名稱：

```bash
xcrun altool --upload-app --type ios --file "src-tauri/gen/apple/build/arm64/$APPNAME.ipa" --apiKey $APPLE_API_KEY_ID --apiIssuer $APPLE_API_ISSUER
```

`$APPLE_API_KEY_ID` 與上面的 `APPLE_API_KEY` 都是金鑰列表裡的 Key ID，官方文件兩個頁面拼法不同。

### 這集實測了什麼

- 沒有跑任何行動裝置的指令：`tauri android build`、`tauri ios build`、`keytool`、`altool`，也沒有改過 Gradle 檔。這台電腦沒有 Android 與 iOS 的開發工具，也沒有 Play Console 與 Apple Developer 帳號。
- 內容依據 v2.tauri.app 的 Google Play、App Store、Android 簽章與 iOS 簽章四頁，加上 tauri-cli 2.12.5 與 cargo-mobile2 0.22.5 的原始碼。文件是撰寫時的版本，可能比 2.12 新。
- 沒有查證：沒簽章的 APK 實際檔名、Google Play 開發者帳號的費用、免費的 Apple 帳號能做到哪一步。

## 重點整理

- APK 用來測試或商店外發佈，AAB 是上傳 Google Play 的格式，IPA 是 iOS 的安裝檔；`tauri android build` 不加旗標就兩種都打。
- 成品在 `gen/android/app/build/outputs/` 與 `gen/apple/build/arm64/`；`versionCode` 預設由 `version` 算出，`0.1.0` 是 `1000`。
- Android 簽章：用 `keytool` 產生 keystore，寫 `keystore.properties`，再在 `app/build.gradle.kts` 加 `signingConfigs`。
- iOS 上架要付費的 Apple Developer Program，用 `--export-method app-store-connect` 打包、`altool` 上傳，這些都沒有實測。
