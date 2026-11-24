# tauri-action 簡介

## 本集目標

把專案放上 GitHub，寫出第一個 workflow，讓 GitHub 的 macOS 機器用 tauri-action 替你打包，再把成品下載回來。

## 正文

這一集從起始專案出發。第 1～3 集接著做同一個專案：每集的 `.github/workflows/release.yml` 都是上一集那份改出來的。第 4、5 集還要用到第 16 章的 updater 設定，到時會列出從前面帶過來的檔案。

### 要先有的東西

GitHub Actions 只在 GitHub 上跑，專案得先放進 GitHub 的 repo。這一章用 GitHub CLI（`gh`）操作 GitHub，安裝與 `gh auth login` 登入照 <https://cli.github.com> 的說明做。在專案資料夾裡：

```bash
git init -b main
git add .
git commit -m "first commit"
gh repo create tauri-app --public --source=. --push
```

最後一行建一個叫 `tauri-app` 的公開 repo，把這個資料夾推上去。`package-lock.json` 與 `src-tauri/Cargo.lock` 一定要一起進 Git，CI 靠它們裝出跟你本機相同的版本；範本的 `.gitignore` 沒有排除它們。

用公開 repo，是因為官方計費頁寫標準 runner 在公開 repo 免費；私有 repo 有每月額度，超過要付費，頁面上 macOS 每分鐘的單價大約是 Linux 的十倍。我們沒有用私有 repo 試過。

### workflow、job、step、runner

- **workflow**：`.github/workflows/` 底下的一個 YAML 檔，描述「什麼時候、做什麼事」。
- **job**：workflow 裡的一組工作，整個跑在同一台機器上。
- **runner**：跑 job 的那台機器。`runs-on` 選種類，我們用 GitHub 提供的 `macos-latest`。
- **step**：job 裡照順序做的步驟。不是用 `run:` 跑一行指令，就是用 `uses:` 套一個別人寫好的 action。

### 第一個 workflow

`.github/workflows/release.yml`：

```yaml
name: release

on: workflow_dispatch

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: lts/*
      - run: npm ci
      - uses: tauri-apps/tauri-action@v1
        with:
          uploadWorkflowArtifacts: true
```

- `on: workflow_dispatch`：只在你手動叫它的時候跑。之後會長成發佈用的，所以檔名先叫 `release.yml`。
- `actions/checkout`：把 repo 的內容抓到 runner 上，不然機器上什麼都沒有。
- `actions/setup-node`：裝 Node.js。`lts/*` 取目前的 LTS，跟第 1 章第 2 集在本機裝的是同一條線。
- `npm ci`：照 `package-lock.json` 裝前端相依套件。tauri-action 不會替你做這件事。
- `tauri-apps/tauri-action@v1`：本集的主角，下面說明。
- `@v7`、`@v1` 是 tag，跟著那個版本系列走；log 會記下它實際抓到哪個 commit。

我們沒有寫安裝 Rust 的 step，打包照樣成功，因為 GitHub 的 runner 映像檔已經裝好 Rust。

### 叫它跑

把檔案推上去，再手動觸發：

```bash
git add .github
git commit -m "add release workflow"
git push
gh workflow run release.yml
```

`gh workflow run release.yml` 的輸出：

```text
https://github.com/<你的帳號>/tauri-app/actions/runs/45129496303
```

這個網址是這一次執行（run）的頁面，最後的數字是這次執行的編號。`gh run watch` 即時顯示進度，跑完用 `gh run view 45129496303` 看結果，輸出（節錄）：

```text
✓ main release · 45129496303
JOBS
✓ build in 8m6s (ID 137541501055)
ARTIFACTS
darwin-aarch64-dmg
darwin-aarch64-app
```

### tauri-action 做了什麼

它做的事不多：找到專案用的 Tauri CLI，跑 `tauri build`，再找出打包的成果。tauri-action 那個 step 的 log（節錄，路徑裡的帳號與 repo 名稱換成了 `tauri-app`，其餘沒改）：

```text
running npm [ 'run', 'tauri', 'build' ]
    Finished `release` profile [optimized] target(s) in 6m 56s
     Running bundle_dmg.sh
    Finished 2 bundles at:
        /Users/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (4.28 MiB)
        /Users/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg (1.93 MiB)
Found artifacts:
/Users/runner/work/tauri-app/tauri-app/src-tauri/target/release/bundle/dmg/tauri-app_0.1.0_aarch64.dmg
...
```

第一行是它替你跑的指令，跟你在本機打的 `npm run tauri build` 一樣；中間是第 15 章看過的輸出（我們另外省略了幾行）。`Found artifacts` 是它從輸出資料夾找到的檔案。編譯佔了將近 7 分鐘，因為 runner 是全新的，沒有任何編譯快取。

`uploadWorkflowArtifacts: true` 讓它把這些檔案上傳成 workflow artifact，也就是這一次執行附帶的檔案。名字是 `平台-架構-安裝檔類型`，平台寫的是 `darwin`，不是 `macos`。沒有它，runner 用完就回收，打好的檔案也跟著消失。

### 跟第 15 章的 `CI=true` 不一樣

第 15 章第 4 集說，沒有 Finder 的環境可以設 `CI=true` 略過 AppleScript，並說 GitHub Actions 預設有這個變數。這裡要補充：tauri-action 在 macOS 上會另外設 `TAURI_BUNDLER_DMG_IGNORE_CI=true`（出自 tauri-action 的原始碼；bundler 看到它是 `true` 就不理 `CI`），所以 runner 上做 `.dmg` 還是會用 AppleScript 排版。

`Running bundle_dmg.sh` 到 `Finished 2 bundles` 隔了 40 秒，沒有錯誤，runner 的 Finder 回應得了。掛載 CI 做出來的 `.dmg`，`ls -a` 比第 15 章那份多一個 `.DS_Store`，那是 Finder 存視窗排版的檔案。我們也在自己的 Mac 上設 `CI=true`、不經 tauri-action 打了一次，`ls -a` 沒有 `.DS_Store`。所以自己在 CI 跑 `tauri build` 的話 `.dmg` 沒有排版，透過 tauri-action 就有。

### 把成品抓回來

workflow artifact 要登入才能下載（不帶帳號呼叫 API 會得到 `401`），`gh run download` 用的就是你登入的帳號：

```bash
gh run download 45129496303
ls darwin-aarch64-dmg
```

`ls` 的輸出：

```text
tauri-app_0.1.0_aarch64.dmg
```

每個 workflow artifact 下載成一個同名的資料夾（`.app` 那個資料夾裡是 `tauri-app.app`）。這些檔案是給你自己檢查的，要讓使用者下載得放成 GitHub Release，第 3 集做。

### 這集實測了什麼

- runner 是 `macos-latest`，這次是 macOS 26 的 arm64 映像檔 `macos-26-arm64`（版本 20261117.0516.1）。setup-node 裝到 Node v26.13.1。
- tauri-action 是 v1.0.0，Tauri CLI 是 lock 檔鎖的 2.12.5。整個 job 8 分 6 秒。
- 沒測：私有 repo、GitHub 網頁介面上手動觸發（我們只用 `gh`）、`gh repo create` 那一行（我們沒有逐字跑過）。

## 重點整理

- GitHub Actions 的 workflow 是 `.github/workflows/` 裡的 YAML，裡面有 job，job 跑在一台 runner 上，由一個個 step 組成。
- tauri-action 替你跑 `tauri build` 並找出成品；`npm ci` 與 Node 要自己的 step 裝，Rust 映像檔已經有了。
- 打好的檔案要上傳成 workflow artifact，不然 runner 一回收就沒了；`gh run download` 抓回來。
- tauri-action 在 macOS 上設了 `TAURI_BUNDLER_DMG_IGNORE_CI=true`，所以 runner 上的 `.dmg` 照樣用 AppleScript 排版。
