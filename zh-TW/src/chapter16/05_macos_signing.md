# macOS 簽章與公證

## 本集目標

分清楚簽章、公證與 Gatekeeper，用 `signingIdentity` 設成 `"-"` 實際做一次 ad-hoc 簽章並驗證。Developer ID 簽章與公證要付費的 Apple 帳號，這集只講流程與設定，沒有跑。

## 正文

### 三個詞

- **程式碼簽章**（code signing，下面簡稱簽章）：用 `codesign` 指令替 `.app` 簽名，通常搭配一張憑證，簽章裡記著 `.app` 內容的雜湊。
- **公證**（notarization）：把簽好的 `.app` 送給 Apple 檢查；通過之後 Apple 發一張公證票，把票貼回 `.app` 叫 staple。
- **Gatekeeper**：macOS 打開 App 之前把關的機制。tauri-bundler 的原始碼註解說，它會找貼在 App 上的票，判斷有沒有公證。

第 15 章第 4 集說，沒簽章的 App 給別人用，Gatekeeper 會擋，我們當時沒試。這集能試的會試，試不到的會講清楚。另外，第 2 集「產生簽章金鑰」那把金鑰簽的是更新檔，跟這集的 `codesign` 簽章是兩件事：金鑰不同，驗證的對象也不同。

### 不簽的樣子（實測）

從起始專案直接 `npm run tauri build -- --bundles app`，沒設任何簽章，輸出裡沒有 `Signing` 開頭的行。用 `codesign` 看打出來的 `.app`：

```bash
codesign -dvv src-tauri/target/release/bundle/macos/tauri-app.app
codesign --verify --deep --strict --verbose=2 src-tauri/target/release/bundle/macos/tauri-app.app
```

`codesign -dvv` 的輸出（節錄）：

```text
Identifier=tauri_app-3aad481a963c93af
CodeDirectory v=20400 size=34424 flags=0x20002(adhoc,linker-signed) hashes=1072+0 location=embedded
Signature=adhoc
Info.plist=not bound
TeamIdentifier=not set
Sealed Resources=none
```

`codesign --verify` 的輸出：

```text
src-tauri/target/release/bundle/macos/tauri-app.app: code has no resources but signature indicates they must be present
```

執行檔上有一個 ad-hoc 簽章，`flags` 寫著 `linker-signed`，是連結器附的（我們這台是 Apple Silicon）。但 `.app` 這個套件沒有封起來：`Info.plist` 沒綁進去、沒有 `Sealed Resources`，`--verify` 直接失敗。

### ad-hoc 簽章（實測）

在 `src-tauri/tauri.conf.json` 的 `bundle` 底下加一個 `macOS` 區塊，`signingIdentity` 填 `"-"`，其他欄位不動：

```json,ignore
{
  "bundle": {
    "macOS": {
      "signingIdentity": "-"
    }
  }
}
```

`codesign` 的規矩是身分填 `-` 就不用憑證，做 ad-hoc 簽章；bundler 把 `signingIdentity` 原封不動交給 `codesign -s`。重新打包：

`npm run tauri build -- --bundles app` 的輸出（節錄）：

```text
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
     Signing with identity "-"
Signing with identity "-"
Signing ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app/Contents/MacOS/tauri-app
~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app/Contents/MacOS/tauri-app: replacing existing signature
Signing with identity "-"
Signing ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app
~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app: replacing existing signature
        Warn skipping app notarization, no APPLE_ID & APPLE_PASSWORD & APPLE_TEAM_ID or APPLE_API_KEY & APPLE_API_ISSUER & APPLE_API_KEY_PATH environment variables found
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (4.29 MiB)
```

- 先簽裡面的執行檔，再簽整個 `.app`。原始碼註解引 Apple 的說法：簽章要由內而外。
- `replacing existing signature` 出現兩次：第一次換掉連結器附的簽章，第二次換掉前一步剛簽的。
- 最後的 `Warn`：簽了章但沒給公證的帳密，bundler 不會失敗，只警告並略過公證。

驗證用剛才的兩個 `codesign` 指令，再多一個 `spctl`：

```bash
spctl --assess --type execute -vv src-tauri/target/release/bundle/macos/tauri-app.app
```

`codesign -dvv` 的輸出（節錄）：

```text
Identifier=tw.example.tauri-app
CodeDirectory v=20500 size=8789 flags=0x10002(adhoc,runtime) hashes=268+3 location=embedded
Signature=adhoc
Info.plist entries=14
TeamIdentifier=not set
Sealed Resources version=2 rules=13 files=1
```

`codesign --verify` 的輸出：

```text
src-tauri/target/release/bundle/macos/tauri-app.app: valid on disk
src-tauri/target/release/bundle/macos/tauri-app.app: satisfies its Designated Requirement
```

`spctl --assess` 的輸出：

```text
src-tauri/target/release/bundle/macos/tauri-app.app: rejected
```

跟不簽的時候比：`Identifier` 變成 `tauri.conf.json` 的 `identifier`；`Info.plist` 綁進簽章，`Sealed Resources` 有東西（`Contents/_CodeSignature/CodeResources` 記著資源檔的雜湊），`--verify` 通過；`flags` 多了 `runtime`，那是 hardened runtime，`hardenedRuntime` 預設 `true`，bundler 對可執行檔加 `--options runtime`。

不過 `Signature=adhoc`、`TeamIdentifier=not set`：沒有憑證，沒有人替這個 App 背書。`spctl --assess` 是在命令列問 macOS 的系統安全政策這個 App 放不放行，輸出只有 `rejected`，沒寫原因。這是我們唯一看到的結果：把 `.app` 傳給別人、對方雙擊會看到什麼畫面，我們沒有試。

### Developer ID 簽章與公證（沒有實測）

這一段沒有跑。Developer ID 憑證與公證都要加入 Apple Developer Program，要付費，我們沒有帳號。這台 Mac 上 `security find-identity -v -p codesigning` 的輸出：

```text
     0 valid identities found
```

下面是讀 tauri-bundler 2.10.5 與它用的 tauri-macos-sign 2.4.3 原始碼整理的。

**簽章用的憑證。** `signingIdentity` 填鑰匙圈（keychain）裡那張憑證的名稱，或設環境變數 `APPLE_SIGNING_IDENTITY`；兩邊都有，環境變數優先。沒有鑰匙圈的環境（像 CI）改設 `APPLE_CERTIFICATE`（`.p12` 憑證的 base64）與 `APPLE_CERTIFICATE_PASSWORD`，bundler 會建一個暫時的鑰匙圈匯入，打包完刪掉。怎麼放進 CI，第 17 章會正式介紹。

**公證用的帳密。** 兩組擇一，每組要設齊：

| 做法 | 環境變數 |
| --- | --- |
| Apple ID | `APPLE_ID`、`APPLE_PASSWORD`（App 專用密碼）、`APPLE_TEAM_ID` |
| API 金鑰 | `APPLE_API_KEY`、`APPLE_API_ISSUER`、`APPLE_API_KEY_PATH`（`.p8` 檔，可省略） |

有 `APPLE_ID` 與 `APPLE_PASSWORD` 卻沒有 `APPLE_TEAM_ID`，bundler 直接報錯，不是警告。沒設 `APPLE_API_KEY_PATH` 的話，bundler 會到 `./private_keys`、`~/private_keys`、`~/.private_keys` 與 `~/.appstoreconnect/private_keys` 找 `AuthKey_<APPLE_API_KEY>.p8`。

**bundler 的順序。**

- 簽章：framework 與 sidecar 先，`.app` 最後；簽之前先跑 `xattr -crs` 清掉擴充屬性。`hardenedRuntime` 與 `entitlements`（plist 檔路徑）會傳給 `codesign`，這集不展開。
- 公證：`ditto` 把 `.app` 壓成 zip，`xcrun notarytool submit --wait` 送出；狀態是 `Accepted` 才用 `xcrun stapler staple` 貼票，不是的話，錯誤訊息會附上 `notarytool log`。
- `tauri build --skip-stapling` 送出就結束，不等結果也不貼票。原始碼註解說第一次公證可能要好幾個小時，之後建議關掉。
- 公證只送 `.app`；`.dmg` 會簽章（`signingIdentity` 是 `"-"` 時例外，`dmg/mod.rs` 會跳過），但 bundler 沒有另外替它送公證。這次只打 `app`，`.dmg` 的部分沒有跑。

## 重點整理

- 簽章是 `codesign` 用憑證替 `.app` 簽名，公證是把簽好的 `.app` 送 Apple 檢查再貼票；更新檔的簽章是另一件事。
- 沒設 `signingIdentity`，`tauri build` 不簽 `.app`：只有連結器附在執行檔上的簽章，`codesign --verify` 失敗。
- `bundle.macOS.signingIdentity` 設成 `"-"` 是 ad-hoc 簽章：`.app` 封得起來、`--verify` 通過，但沒有憑證，`spctl --assess` 回 `rejected`。
- Developer ID 簽章要付費帳號：憑證用 `signingIdentity` 或 `APPLE_SIGNING_IDENTITY`，公證用 `APPLE_ID` 那組或 `APPLE_API_KEY` 那組環境變數；沒有帳密只會警告略過公證。
- 我們實測了 ad-hoc 簽章與驗證；Developer ID 簽章、公證與 Gatekeeper 對下載來的 App 的實際反應，都沒有實測。
