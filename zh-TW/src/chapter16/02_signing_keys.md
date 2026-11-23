# 產生簽章金鑰

## 本集目標

用 `tauri signer generate` 產生一對金鑰，把公鑰寫進設定，並在打包時用私鑰替更新檔簽出 `.sig`。

## 正文

這一集接著上一集的專案，不重新開始。

### 私鑰簽，公鑰驗

更新檔從網路下載，路上可能被換掉，或是伺服器被人動過手腳。App 要能確認「這個檔案真的是我的開發者發的」。做法是一對金鑰：

- **私鑰**只有你有，打包時用它替更新檔簽章。
- **公鑰**寫進 App 的設定，跟著 App 發出去。App 下載更新檔之後，用公鑰驗簽章，驗不過就不裝。

所以公鑰可以公開，私鑰不行：不要放進 Git，不要貼進對話或 issue。

### 產生金鑰

```bash
npm run tauri signer generate -- -w ~/.tauri/tauri-app.key
```

`--` 後面的參數是給 `tauri signer generate` 的；`-w` 指定私鑰寫到哪個檔案。它會要你輸入密碼來保護私鑰（輸入時畫面不顯示），要輸入兩次。

`npm run tauri signer generate -- -w ~/.tauri/tauri-app.key` 的輸出（節錄）：

```text
Please enter a password to protect the secret key.
Password: 
Password (one more time): 
Deriving a key from the password in order to encrypt the secret key... done

Your keypair was generated successfully:
Private: ~/.tauri/tauri-app.key (Keep it secret!)
Public: ~/.tauri/tauri-app.key.pub
---------------------------

ATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not work
```

出現了兩個檔案。金鑰檔的內容很長，也是不能貼的，所以只看開頭 48 個字元：

```bash
cut -c1-48 ~/.tauri/tauri-app.key
cut -c1-48 ~/.tauri/tauri-app.key.pub
```

輸出：

```text
dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBz
dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBr
```

兩個檔案都是一行 base64。用 `base64 -d` 解開，開頭分別是 `untrusted comment: rsign encrypted secret key` 與 `untrusted comment: minisign public key: …`，所以是 minisign 格式，私鑰是用你的密碼加密過的。

最後那行警告是真的：私鑰或密碼弄丟，就再也簽不出能被已發出去的 App 接受的更新檔，因為那些 App 裡的公鑰是固定的。把私鑰和密碼備份在安全的地方。

### 把公鑰與開關寫進設定

在 `tauri.conf.json` 做三件事。`version` 改成 `1.0.0`，當作我們要發出去的第一版（第 15 章第 3 集說過版本號要是 semver）；`bundle` 多一個 `createUpdaterArtifacts`，要求打包時順便做更新檔；`plugins.updater.pubkey` 貼上整個 `.key.pub` 檔的內容，是一整行，不是檔案路徑：

```json,ignore
"version": "1.0.0",
"bundle": {
  "createUpdaterArtifacts": true
},
"plugins": {
  "updater": {
    "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6...",
    "requireSignedVersion": true
  }
}
```

`bundle` 裡本來就有 `active`、`targets`、`icon`，`createUpdaterArtifacts` 是加在同一層；`plugins` 在最外層，上一集 `tauri add` 已經建好，只差 `pubkey`。`pubkey` 這裡為了排版只列開頭，你貼的要是完整的一行。

### 用私鑰打包

先不給私鑰，看會怎樣：

```bash
npm run tauri build -- --bundles app
```

沒設私鑰就打包，最後會停在這裡（節錄）：

```text
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (5.90 MiB)
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz (updater) (2.84 MiB)

A public key has been found, but no private key. Make sure to set `TAURI_SIGNING_PRIVATE_KEY` environment variable.
       Error A public key has been found, but no private key. Make sure to set `TAURI_SIGNING_PRIVATE_KEY` environment variable.
```

`.app` 與 `.tar.gz` 都做了，但簽章那一步停下來，指令以失敗結束。所以只要設了 `createUpdaterArtifacts` 與公鑰，沒有私鑰就打包不完。

`TAURI_SIGNING_PRIVATE_KEY` 可以是私鑰檔的路徑，也可以是金鑰本身；密碼放在 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。在同一個終端機設好再打包：

```bash
export TAURI_SIGNING_PRIVATE_KEY="$HOME/.tauri/tauri-app.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="你設的密碼"
npm run tauri build -- --bundles app
```

`npm run tauri build -- --bundles app` 的輸出（節錄）：

```text
    Finished `release` profile [optimized] target(s) in 51.58s
    Bundling tauri-app.app (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app)
    Bundling ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz (~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz)
    Finished 1 bundle at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app (5.90 MiB)
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz (updater) (2.84 MiB)

    Finished 1 updater signature at:
        ~/tauri-app/src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz.sig
```

`ls src-tauri/target/release/bundle/macos` 的輸出：

```text
tauri-app.app
tauri-app.app.tar.gz
tauri-app.app.tar.gz.sig
```

`tauri-app.app.tar.gz` 是把 `.app` 壓起來的更新檔，`.sig` 是它的簽章。下一集用這兩個檔案。

我們只在 macOS（Apple Silicon）上跑過。Windows 與 Linux 沒有 `.tar.gz`，依原始碼整理，是直接替安裝檔（`.msi`、`.exe`、`.AppImage`、`.deb`、`.rpm`）簽出各自的 `.sig`，我們沒有試。

## 重點整理

- 更新簽章用一對金鑰：私鑰打包時簽章，公鑰寫進 `plugins.updater.pubkey` 跟著 App 發出去。
- `npm run tauri signer generate -- -w <路徑>` 產生金鑰，私鑰用密碼加密；私鑰與密碼不能進 Git、不能弄丟。
- 設定要三處：`bundle.createUpdaterArtifacts: true`、`plugins.updater.pubkey`，版本號是 semver。
- 打包時用環境變數 `TAURI_SIGNING_PRIVATE_KEY`（路徑或內容）與 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 提供私鑰，沒設就會在簽章那步失敗。
- macOS 打包出 `.app.tar.gz` 加 `.sig`。
