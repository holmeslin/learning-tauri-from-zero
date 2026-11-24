# 在 CI 裡簽章

## 本集目標

把第 16 章的更新檔簽章搬進 CI：私鑰存成 GitHub secret，交給 tauri-action，三個平台各簽出 `.sig`。也說明程式碼簽章的憑證怎麼接進 CI，那部分我們沒有憑證，沒有實測。

## 正文

這一集接著第 3 集，也接著第 16 章。從前面帶過來的檔案：

- `.github/workflows/release.yml`：第 3 集那份，下面只改一個 step。
- `src-tauri/src/lib.rs`：第 16 章第 4 集那一整份（啟動時檢查並安裝更新），一個字都不改。
- `src-tauri/tauri.conf.json`：第 16 章設好的 `createUpdaterArtifacts`、`pubkey`、`requireSignedVersion`，下面改三處。
- `src-tauri/Cargo.toml`：第 16 章第 1 集 `tauri add updater` 加的 `tauri-plugin-updater`，不用動。
- 第 16 章第 2 集的私鑰 `~/.tauri/tauri-app.key` 與你設的密碼。

### 設定檔改三處

下面只列 `src-tauri/tauri.conf.json` 裡跟這集有關的欄位，其他不動，`pubkey` 是你自己那把公鑰的內容：

```json,ignore
"version": "1.0.0",
"plugins": {
  "updater": {
    "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6...",
    "requireSignedVersion": true,
    "endpoints": [
      "https://github.com/<你的帳號>/tauri-app/releases/latest/download/latest.json"
    ]
  }
}
```

`version` 設回 `1.0.0`，當成第一個公開的版本。`endpoints` 換成 GitHub Release 的網址（第 5 集說它怎麼來），也拿掉第 16 章的 `dangerousInsecureTransportProtocol`，GitHub 是 `https`。`endpoints` 現在就要設：App 問哪裡更新，是編譯時就寫進去的，已經發出去的版本改不了。

### 金鑰要放進 secret

CI 的 runner 每次都是新的，你電腦上的私鑰檔不在上面，而私鑰不能進 Git（第 16 章第 2 集）。GitHub 的 secret 是存在 repo 設定裡的加密值，設好之後看不到內容，workflow 用 `${{ secrets.名字 }}` 取用，印在 log 裡會被遮成 `***`。

要把 secret 交給 Tauri CLI，用第 16 章說過的兩個環境變數。在第 3 集的 `release.yml`，tauri-action 那個 step 的 `env` 裡，`GITHUB_TOKEN` 那行下面加這兩行（縮排跟它一樣），其他都不動：

```yaml
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```

### 沒設 secret 會怎樣

先不設 secret，推上去直接跑。沒設的 secret 是空字串，三個平台都編譯、打包完，到簽章那步失敗。tauri-action step 的 log（節錄）：

```text
    Finished 2 bundles at:
        /Users/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (5.88 MiB)
        /Users/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_1.0.0_aarch64.dmg (2.83 MiB)
        /Users/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz (updater) (2.83 MiB)

failed to decode secret key: incorrect updater private key password: Missing comment in secret key
       Error failed to decode secret key: incorrect updater private key password: Missing comment in secret key
##[error]Command "npm ["run","tauri","build"]" failed with exit code 1
```

這跟第 16 章第 2 集的 `A public key has been found, but no private key` 不一樣：那是環境變數根本沒設；這裡變數設了，值是空的，Tauri 當它是一把壞掉的金鑰。訊息裡的「密碼不對」會誤導人，其實是私鑰沒有內容。

### 設定 secret

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/tauri-app.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

第一行從檔案讀進整把私鑰；第二行 `gh` 會問你要貼上的值，輸入時不顯示。我們實際跑的是兩行都從檔案讀入（密碼檔用 `printf %s` 寫，避免多一個換行）；互動輸入那個形式，我們沒有逐字跑過。設好之後 `gh secret list` 的輸出：

```text
TAURI_SIGNING_PRIVATE_KEY	2026-11-23T08:17:30Z
TAURI_SIGNING_PRIVATE_KEY_PASSWORD	2026-11-23T08:17:31Z
```

只列名字與時間，看不到值。也可以在 repo 的設定頁面加（我們沒有試過網頁介面）。

### 再跑一次

secret 設好，重新觸發。三個平台的 tauri-action step 在 `Finished N bundles` 後面，多了簽章那一段（路徑同樣只換了 repo 名稱）：

```text
    Finished 1 updater signature at:
        /Users/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz.sig
```

```text
    Finished 3 updater signatures at:
        /home/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/deb/tauri-app_1.0.0_amd64.deb.sig
        ...
```

macOS 簽 `.app.tar.gz`，Windows 簽 MSI 與 NSIS 的安裝檔（2 個），Linux 簽 deb、rpm 與 AppImage（3 個），跟第 16 章第 2 集依原始碼整理的一致，現在在 Windows 與 Linux 上實際看到了。workflow 這次 5 分 57 秒跑完，沒有失敗。簽章檔跟安裝檔一起到了 release 草稿，第 5 集看。

### 程式碼簽章的憑證（沒有實測）

這一段沒有跑。我們沒有 Apple Developer 帳號，也沒有 Windows 憑證，下面是依 tauri-bundler 2.10.5 與 tauri-macos-sign 2.4.3 的原始碼整理的。

**macOS。** bundler 自己會把憑證匯進鑰匙圈，不需要在 workflow 裡手動用 `security` 指令。只要 `APPLE_CERTIFICATE`（`.p12` 檔的 base64）與 `APPLE_CERTIFICATE_PASSWORD` 兩個都有設，它就建一個臨時鑰匙圈、匯入憑證，用完刪掉。再加上第 16 章第 5 集的公證帳密，`env` 會長這樣：

```yaml
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
```

`APPLE_CERTIFICATE` 的值：原始碼註解的做法是 `openssl base64 -in MyCertificate.p12 -out MyCertificate-base64.txt`，再用 `gh secret set APPLE_CERTIFICATE < MyCertificate-base64.txt` 放進 secret。`APPLE_SIGNING_IDENTITY` 可以不設，bundler 會用匯進去的那張憑證；設了的話，它必須是那張憑證名稱的一部分，否則 bundler 報錯。tauri-action 的範例是先自己用 `security` 指令匯入再打包，我們沒有比較兩種做法。

**Windows。** bundler 與 tauri-action 的原始碼都沒有匯入憑證的步驟。第 16 章第 6 集說過，bundler 只把 `certificateThumbprint` 交給 `signtool`，憑證得事先放在那台 runner 上，`signtool` 才找得到；怎麼放，取決於憑證供應商給你什麼，我們沒有寫也沒有試。改用 `signCommand` 呼叫簽章服務的命令列工具，密碼同樣存 secret、用環境變數傳給它。

兩個平台都一樣：憑證與密碼只放 secret，不寫進 `tauri.conf.json`，也別印在 log 裡。

### 這集實測了什麼

- 實測：更新檔簽章在 macOS、Linux、Windows 三個 runner 上，沒設 secret 的失敗與設好之後的成功。runner、tauri-action 版本同第 2 集；各 job 3 分 46 秒（macOS）、5 分 53 秒（Linux）、5 分 46 秒（Windows）。
- 同一份程式碼編譯的時間差很多：macOS 編譯，第 1 集是 6 分 56 秒，這次是 2 分 52 秒。我們沒有加快取，不知道原因。
- 實測用的是另外替示範 repo 產生的一次性金鑰，不是第 16 章那把，做法相同。
- 沒測：網頁介面設 secret、互動輸入的 `gh secret set`、所有憑證相關的設定（上面那段 `APPLE_*` 的 `env` 沒有在 workflow 裡跑過）。

## 重點整理

- 私鑰與密碼存成 GitHub secret（`gh secret set`），在 tauri-action 的 `env` 設成 `TAURI_SIGNING_PRIVATE_KEY` 與 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。
- secret 沒設，值是空字串，簽章步驟報 `failed to decode secret key`，不是 `no private key`。
- 簽章成功後，macOS 有 1 個 `.sig`、Windows 2 個、Linux 3 個；`endpoints` 要在第一版就設好。
- macOS 的憑證用 `APPLE_CERTIFICATE` 與 `APPLE_CERTIFICATE_PASSWORD` 交給 bundler 匯入；Windows 的憑證得自己事先放在 runner 上。這兩段沒有實測。
