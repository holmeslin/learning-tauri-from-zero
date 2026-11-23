# 一起上傳更新檔

## 本集目標

看 tauri-action 怎麼把簽章檔與 `latest.json` 跟安裝檔一起放上 release，發佈草稿，讓第 16 章的 App 從 GitHub 拿到更新，並真的從 1.0.0 更新到 1.1.0。

## 正文

這一集接著第 4 集，沒有新的檔案要改，用的是第 4 集的 `release.yml`、`tauri.conf.json` 與第 16 章第 4 集的 `lib.rs`。最後要發 1.1.0，只會把 `tauri.conf.json` 的 `version` 改成 `1.1.0`。

### release 裡多了什麼

第 4 集簽章成功的那次執行，`gh release view app-v1.0.0 --json assets --jq '.assets[].name'` 的輸出（已排序）：

```text
latest.json
tauri-app-1.0.0-1.x86_64.rpm
tauri-app-1.0.0-1.x86_64.rpm.sig
tauri-app_1.0.0_aarch64.app.tar.gz
tauri-app_1.0.0_aarch64.app.tar.gz.sig
tauri-app_1.0.0_aarch64.dmg
tauri-app_1.0.0_amd64.AppImage
tauri-app_1.0.0_amd64.AppImage.sig
tauri-app_1.0.0_amd64.deb
tauri-app_1.0.0_amd64.deb.sig
tauri-app_1.0.0_x64-setup.exe
tauri-app_1.0.0_x64-setup.exe.sig
tauri-app_1.0.0_x64_en-US.msi
tauri-app_1.0.0_x64_en-US.msi.sig
```

第 3 集的 release 有七個附件，現在十四個：每個能拿來更新的檔案旁邊多一個 `.sig`，另外多了 `latest.json`。macOS 更新用的是 `.app.tar.gz`，`.dmg` 只給人下載，所以沒有 `.sig`。這些都是 tauri-action 的預設：`uploadUpdaterSignatures` 與 `uploadUpdaterJson` 的預設值都是 `true`，只要 Tauri 做出了 `.sig`，它就一起上傳。

### `latest.json`

`latest.json` 是 tauri-action 做的，三個 job 輪流來：每個 job 讀 release 上現有的檔案，加上自己平台那幾筆，刪掉舊的、傳新的。內容（`signature` 只列開頭，`platforms` 省略其餘六個，帳號與 repo 名稱換成了佔位）：

```json,ignore
{
  "version": "1.0.0",
  "notes": "自動建置的草稿，確認各平台安裝檔都在再發佈。",
  "pub_date": "2026-11-23T08:23:23.077Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1l…",
      "url": "https://api.github.com/repos/<你的帳號>/tauri-app/releases/assets/782128264"
    },
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1l…",
      "url": "https://api.github.com/repos/<你的帳號>/tauri-app/releases/assets/782134171"
    },
    "linux-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1l…",
      "url": "https://api.github.com/repos/<你的帳號>/tauri-app/releases/assets/782134239"
    }
  }
}
```

跟第 16 章第 3 集手寫的是同一種格式，只是欄位自動填好：`notes` 是 `releaseBody`，`signature` 是 `.sig` 檔的內容（我們比對過 macOS 那筆，一字不差）。省略的六個是 `darwin-aarch64-app`、`windows-x86_64-msi`、`windows-x86_64-nsis`、`linux-x86_64-appimage`、`-deb`、`-rpm`，也就是第 16 章說的「平台加安裝類型」。不帶安裝類型的 `windows-x86_64` 預設指向 MSI，要改成 NSIS 設 `updaterJsonPreferNsis: true`（原始碼，我們沒試）；`linux-x86_64` 指向 AppImage。

`url` 是 `api.github.com` 的網址，不是 `github.com/…/releases/download/…`：tauri-action 1.0 的更新說明寫它改用 GitHub 的網址。updater plugin 下載更新檔時會帶 `Accept: application/octet-stream`（plugin 2.14.1 原始碼），API 就把它導向檔案。

### 草稿用不了，發佈之後才行

App 要問的網址是 release 的 `latest.json`，也就是第 4 集設的：

```text
https://github.com/<你的帳號>/tauri-app/releases/latest/download/latest.json
```

`latest` 指最新「已發佈」的 release，草稿不算。發佈前我們 `curl -L` 它得到 `404`。發佈：

```bash
gh release edit app-v1.0.0 --draft=false
```

`gh release list` 的輸出：

```text
App v1.0.0	Latest	app-v1.0.0	2026-11-23T08:24:25Z
App v0.1.0	Draft	app-v0.1.0	2026-11-23T08:13:03Z
```

發佈完馬上抓，我們第一次還是 `404`，隔了一下才變成 `200`，內容是 1.0.0 的 `latest.json`。後面還會看到，發佈新版之後，這個網址要等一兩分鐘才換成新的內容。

### 真的更新一次

把 CI 做出來的 1.0.0 抓下來，在 macOS 上直接跑執行檔：

```bash
gh release download app-v1.0.0 -p '*.app.tar.gz'
tar xzf tauri-app_1.0.0_aarch64.app.tar.gz
./tauri-app.app/Contents/MacOS/tauri-app
```

`./tauri-app.app/Contents/MacOS/tauri-app` 的輸出：

```text
目前版本 1.0.0
已經是最新版
```

App 去 GitHub 問過，`latest.json` 說最新是 1.0.0，沒有新版。接著發 1.1.0：`version` 改成 `1.1.0`，推上去（這時別再推別的），觸發 workflow，等它跑完、草稿 `app-v1.1.0` 有了十四個附件，再 `gh release edit app-v1.1.0 --draft=false` 發佈。我們量到，發佈後約兩分鐘，`releases/latest/download/latest.json` 才從 1.0.0 變成 1.1.0。然後再跑一次同一個舊版執行檔：

```text
目前版本 1.0.0
找到新版 1.1.0（目前 1.0.0）
下載完成
共下載 2963513 bytes，安裝完成，重新啟動
```

`plutil -extract CFBundleShortVersionString raw -o - tauri-app.app/Contents/Info.plist` 先前讀到 `1.0.0`，更新之後是 `1.1.0`，`.app` 被換掉了。一個從 CI 做出來、經 GitHub Release 發佈的版本，更新成下一個 CI 發佈的版本，簽章驗過。

### 這集實測了什麼

- runner 與 tauri-action 版本同第 2 集，updater plugin 2.14.1。1.0.0 的 workflow 5 分 57 秒（第 4 集），1.1.0 這次 8 分 34 秒（macOS 6 分 35 秒、Linux 8 分 24 秒、Windows 8 分 29 秒）。
- 實際更新只在 macOS（Apple Silicon）上做，`.app` 放在資料夾裡從終端機啟動，repo 是公開的。Windows 與 Linux 的 `latest.json` 內容我們看了，但沒有在那兩個系統上更新過。
- 沒測：私有 repo（`latest.json` 網址要登入，App 拿不到）、`updaterJsonPreferNsis`、在網頁介面發佈草稿、下載來的 `.app` 被 Gatekeeper 擋下的情況。

## 重點整理

- tauri-action 預設把每個 `.sig` 與 `latest.json` 一起放上 release；`latest.json` 的 `url` 是 GitHub API 的附件網址，三個 job 各補自己平台的那幾筆。
- App 的 `endpoints` 用 `https://github.com/<帳號>/<repo>/releases/latest/download/latest.json`；`latest` 是最新已發佈的 release，草稿要先發佈才拿得到。
- 發佈用 `gh release edit <tag> --draft=false`；新版發佈後，`latest.json` 要隔一兩分鐘才換成新的。
- 完整流程：改版本號、推上去、觸發 workflow、確認草稿、發佈；第 16 章第 4 集那種啟動時檢查更新的舊版 App，下次啟動就會更新。
