# 發佈成 GitHub Release 草稿

## 本集目標

讓 workflow 打完包，自動在 GitHub 建一個 Release 草稿，把三個平台的安裝檔都傳上去；看懂 `permissions` 與 `GITHUB_TOKEN`，以及草稿在你按下發佈之前，別人看不到。

## 正文

這一集接著第 2 集的專案，帶過來的檔案只有 `.github/workflows/release.yml`，下面整份換掉。

### Release 與草稿

GitHub 的 release 是一個版本的頁面：掛在一個 tag 上，有標題、說明，還有附件（asset），使用者從 repo 的 Releases 頁下載附件。草稿（draft）是還沒公開的 release，按下發佈之前，只有你這個有寫入權限的人登入後看得到。三個平台各自上傳，上傳到一半不該有人看到缺檔的版本，所以先放成草稿，到齊、確認過再發佈。

### 設定 release

`.github/workflows/release.yml`：

```yaml
name: release

on: workflow_dispatch

permissions:
  contents: write

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
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: app-v__VERSION__
          releaseName: App v__VERSION__
          releaseBody: 自動建置的草稿，確認各平台安裝檔都在再發佈。
          releaseDraft: true
```

跟第 2 集比，多了 `permissions`、`env` 和四個 `with` 設定，`uploadWorkflowArtifacts` 拿掉了，因為檔案直接放上 release。

- `tagName`：release 掛的 tag。`__VERSION__` 會換成 `tauri.conf.json` 的 `version`，我們是 `0.1.0`，tag 就是 `app-v0.1.0`。版本號跟 tag 都來自同一個地方，不會對不上。
- `releaseName`、`releaseBody`：release 的標題與說明，`__VERSION__` 在這裡也會換。
- `releaseDraft: true`：建成草稿。
- `GITHUB_TOKEN`：GitHub 在每次執行時自動產生的 token。`secrets.GITHUB_TOKEN` 取得它，tauri-action 用它呼叫 GitHub API 建 release、上傳附件；沒有它，tauri-action 會直接報 `GITHUB_TOKEN is required`（出自原始碼）。

### 沒有寫入權限會怎樣

`permissions` 決定這個 token 能做什麼。我們的 repo 預設只給讀取，第一次沒寫 `permissions`，三個平台都編譯、打包完才在最後失敗。tauri-action step 的 log（節錄），以及 `Set up job` 列的 token 權限：

```text
Looking for a draft release with tag app-v0.1.0...
Couldn't find release with tag app-v0.1.0. Creating one.
##[error]Resource not accessible by integration - https://docs.github.com/rest/releases/releases#create-a-release
```

```text
GITHUB_TOKEN Permissions
Contents: read
Metadata: read
```

建 release 要寫入 repo 的內容，所以加上 `permissions: contents: write`。寫在 workflow 檔裡，比到 repo 設定裡改預設值好：權限跟著檔案走，用什麼權限一看檔案就知道。

### 看結果

`gh release list` 的輸出：

```text
App v0.1.0	Draft	app-v0.1.0	2026-11-23T08:13:03Z
```

`gh release view app-v0.1.0 --json assets --jq '.assets[].name'` 的輸出：

```text
tauri-app-0.1.0-1.x86_64.rpm
tauri-app_0.1.0_aarch64.app.tar.gz
tauri-app_0.1.0_aarch64.dmg
tauri-app_0.1.0_amd64.AppImage
tauri-app_0.1.0_amd64.deb
tauri-app_0.1.0_x64-setup.exe
tauri-app_0.1.0_x64_en-US.msi
```

只有一個草稿，七個附件：三個平台的安裝檔都在。`.app.tar.gz` 是 tauri-action 自己把 `.app` 壓成的，名字有版本與架構；第 16 章在本機打出的是沒有版本的 `tauri-app.app.tar.gz`。

三個 job 各自去找這個 tag 的草稿，找不到就建。這次最先完成的 macOS 建了它（log：`Couldn't find release with tag app-v0.1.0. Creating one.`），Linux 與 Windows 的 log 是 `Found draft release with tag app-v0.1.0 on the release list.`。兩個 job 剛好同時完成會不會建出兩份，我們沒有遇到，也沒試。log 最後還有一行 `Signature not found for the updater JSON. Skipping upload...`：沒有更新檔簽章就不產生 `latest.json`，第 4、5 集處理。

不登入，這個草稿就看不到。我們不帶帳號對這幾個網址 `curl`，狀態碼是：

```text
404 https://github.com/<你的帳號>/tauri-app/releases/tag/app-v0.1.0
404 https://github.com/<你的帳號>/tauri-app/releases/download/app-v0.1.0/tauri-app_0.1.0_aarch64.dmg
404 https://api.github.com/repos/<你的帳號>/tauri-app/releases/tags/app-v0.1.0
```

Releases 頁也沒有任何 release。草稿的發佈留到第 5 集。

### 執行期間，別推改了 workflow 的 commit

我們補上 `permissions` 之後的第一次執行，還是失敗了，錯誤跟沒設 `permissions` 時一模一樣，但 `Set up job` 明明寫著 `Contents: write`。原因是等它跑的時候，我們又推了一個改了 `release.yml` 的 commit。

tauri-action 建 release 時，tag 指向這次執行的 commit。我們用 `curl` 直接打建 release 的 API，換不同的 commit 當 tag 的目標：它帶的 `.github/workflows/` 檔案跟分支現在的一樣就成功，不一樣就是這個 `403`。回應的 `x-accepted-github-permissions` 標頭寫著 `contents=write; contents=write,workflows=write`，我們推測是 GitHub 把這個 tag 當成把另一版的 workflow 檔推進來，而 `GITHUB_TOKEN` 沒有 workflow 的寫入權限。這是試出來的規則，沒有找到官方文件寫它。重新觸發、中間不推任何東西，這次就成功了。

### 這集實測了什麼

- runner 與 tauri-action 版本同第 2 集（macOS 26 arm64、`ubuntu-22.04`、`windows-2025-vs2026`；v1.0.0）。成功的那次，整個 workflow 8 分 41 秒；macOS 5 分 26 秒、Linux 6 分 59 秒、Windows 8 分 38 秒。
- 沒測：在網頁介面操作 Release（我們只用 `gh`）、兩個 job 同時建草稿。

## 重點整理

- release 是掛在 tag 上的版本頁面，附件是各平台的安裝檔；草稿在發佈之前只有有寫入權限的人看得到（不帶帳號的 `curl` 得到 `404`）。
- tauri-action 用 `tagName`、`releaseName`、`releaseBody`、`releaseDraft` 建草稿，`__VERSION__` 換成 `tauri.conf.json` 的版本，並需要 `env` 裡的 `GITHUB_TOKEN`。
- 要寫入 release，workflow 要宣告 `permissions: contents: write`；沒宣告，最後一步會是 `Resource not accessible by integration`。
- release 在三個 job 之間共用：最先完成的建草稿，其他的找到就上傳；跑的時候別推改了 workflow 檔的 commit。
