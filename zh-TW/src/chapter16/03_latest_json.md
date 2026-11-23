# 更新資訊檔 `latest.json`

## 本集目標

看懂更新資訊檔的每個欄位，替 1.1.0 做出更新檔，寫好 `latest.json`，並用本機伺服器放出來，確認拿得到。

## 正文

這一集接著上一集的專案。帶過來的檔案是 `src-tauri/tauri.conf.json`（有 `pubkey`、`createUpdaterArtifacts`，`version` 是 `1.0.0`）、第 1 集的 `src-tauri/src/lib.rs`，加上金鑰與環境變數。上一集停在 `1.0.0` 打包完成，現在要準備「下一個版本」。

### 更新資訊檔長什麼樣

App 去問的網址，要回一份 JSON。檔名叫什麼 plugin 不在乎，慣例叫 `latest.json`。內容：

| 欄位 | 意思 |
| --- | --- |
| `version` | 最新版的版本號。要比 App 目前的版本新（照語意化版本比較）才算有更新；可以有 `v` 開頭。 |
| `notes` | 更新說明，選填。 |
| `pub_date` | 發佈時間，選填，要是 RFC 3339 格式，像 `2026-11-23T03:00:00Z`。 |
| `platforms` | 各平台的下載資訊，key 是 `作業系統-架構`。 |
| `url`（在平台底下） | 更新檔的下載網址。 |
| `signature`（在平台底下） | `.sig` 檔的**內容**，不是檔案路徑。 |

macOS 的作業系統叫 `darwin`，不是 `macos`：Apple Silicon 是 `darwin-aarch64`，Intel 是 `darwin-x86_64`；Windows 與 Linux 是 `windows-x86_64`、`linux-x86_64`。key 後面還可以加安裝類型，像 `darwin-aarch64-app`，plugin 會先找有加的，找不到再找沒加的。這些是依原始碼整理的，我們只用過 `darwin-aarch64`。

### 做出 1.1.0 的更新檔

把 `tauri.conf.json` 的 `version` 改成 `1.1.0`，用上一集的環境變數（同一個終端機還在的話不用再設）打包：

```bash
npm run tauri build -- --bundles app
```

打包出的檔案跟上一集一樣：`tauri-app.app.tar.gz` 與 `tauri-app.app.tar.gz.sig`。`.sig` 是 base64，解開看裡面有什麼：

```bash
base64 -d < src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz.sig
```

`base64 -d` 的輸出（兩行簽章本體只列開頭）：

```text
untrusted comment: signature from tauri secret key
RUT8daYbSRRi6sAhqSofawm3n1F3yWWOOpzRc4Zivv5B5bAKfNAn3kMUaqLpsJf7zWopeH...
trusted comment: timestamp:1795401872	file:tauri-app.app.tar.gz	version:1.1.0
fuSK2UupwK+zyp+pVqLagzEYc6O8cd3TKdpSbaA8L9m16MLEwh4cPU6gT35wPGgRUF8RuN...
```

第三行 `trusted comment` 記了時間、檔名，以及**這個更新檔是替哪個版本簽的**，這一行也在簽章的保護範圍內，改了就驗不過。plugin 驗簽章時，會拿這個版本跟 `latest.json` 的 `version` 比，不一樣就拒絕。所以 `latest.json` 的 `version` 不能隨便寫：要跟打包時 `tauri.conf.json` 的版本一致，下一集會看到不一致的樣子。

### 寫 `latest.json`

在家目錄建一個資料夾 `update-server`，把更新檔複製進去：

```bash
mkdir ~/update-server
cp src-tauri/target/release/bundle/macos/tauri-app.app.tar.gz ~/update-server/
```

再在裡面建 `latest.json`（`signature` 只列開頭，你要貼 `.sig` 整個檔案的內容，一整行、中間不能斷行）：

```json,ignore
{
  "version": "1.1.0",
  "notes": "第一次自動更新",
  "pub_date": "2026-11-23T03:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkK...",
      "url": "http://localhost:8000/tauri-app.app.tar.gz"
    }
  }
}
```

`url` 先寫本機的 `http://localhost:8000/...`，等一下就在這個位置放伺服器。真正發佈的時候，`latest.json` 與更新檔要放在你自己的伺服器或物件儲存上，網址是 `https`，下一集會講為什麼。

### 用本機伺服器放出來

Python 內建一個簡單的靜態檔案伺服器，開一個終端機，讓它一直跑著：

```bash
python3 -m http.server 8000 --directory ~/update-server
```

在另一個終端機確認兩個網址都拿得到，`-w` 讓 `curl` 只印狀態碼與檔案類型：

```bash
curl -s -o /dev/null -w "%{http_code} %{content_type}\n" http://localhost:8000/latest.json
curl -s -o /dev/null -w "%{http_code} %{content_type} %{size_download}\n" http://localhost:8000/tauri-app.app.tar.gz
```

兩個 `curl` 的輸出：

```text
200 application/json
200 application/gzip 2976579
```

伺服器那個終端機也會印出每一次請求。拿得到之後，跑伺服器的那個終端機先別關，下一集要用。

## 重點整理

- 更新資訊檔是一份 JSON：`version`、選填的 `notes` 與 `pub_date`，加上 `platforms`，每個平台有 `url` 與 `signature`。macOS 的 key 是 `darwin-aarch64` 或 `darwin-x86_64`。
- `signature` 要貼 `.sig` 檔的內容；簽章裡記著它替哪個版本簽的，`latest.json` 的 `version` 要跟它一致。
- 發佈新版的步驟：提高版本號、用私鑰打包、把更新檔與 `latest.json` 放到伺服器。
- 本機測試用 `python3 -m http.server`，用 `curl` 確認拿得到；真正發佈要用 `https`。
