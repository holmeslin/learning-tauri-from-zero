# Windows 簽章

## 本集目標

知道 Windows 上的 `npm run tauri build` 怎麼替安裝檔簽章：要填哪些欄位，打包時哪些檔案會被簽。也會講不用 `signtool` 時怎麼換成自己的簽章指令，以及付費憑證從申請到用上的大致流程。

## 正文

### 先說清楚：這一集沒有實測

跟第 15 章第 5 集一樣，我們沒有 Windows 機器，也沒有任何簽章憑證，所以這一集沒有貼任何打包的輸出。你要照著做，需要一台 Windows 和一張真的憑證。

下面的欄位、簽哪些檔案、簽的順序，都是對照 Tauri CLI 2.12 內建的 tauri-bundler 2.10 原始碼和 CLI 的設定 schema 查出來的。原始碼查不到的說法，我們會當場標出來。

### 簽章在做什麼

簽章是用憑證的私鑰替檔案蓋章，Windows 靠它判斷發行者是誰、檔案發行後有沒有被改過；沒簽章的安裝檔，Windows 的 SmartScreen 會對不明發行者提出警告。這是簽章機制的一般說法，不在 Tauri 的原始碼裡，我們也沒有試過。

Tauri 做的事單純得多：打包時，替每個該簽的檔案呼叫一次簽章工具。預設的工具是 Windows SDK 的 `signtool.exe`，bundler 先看環境變數 `TAURI_WINDOWS_SIGNTOOL_PATH`，沒設就去登錄檔查 Windows SDK 裝在哪（`Windows Kits`），從最新版往舊版找；找不到就中斷，錯誤是 `SignTool not found`。

### 設定：憑證指紋與時間戳記

簽章的設定都在 `bundle.windows` 底下。最小的一組長這樣：

```json,ignore
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8A9B0",
      "digestAlgorithm": "sha256",
      "timestampUrl": "https://timestamp.example.com"
    }
  }
}
```

指紋與網址都是示意用的假值。各欄位的意思：

- `certificateThumbprint`：憑證的 SHA1 雜湊，也就是指紋。bundler 把它交給 `signtool` 的 `/sha1` 參數，由 `signtool` 去找那張憑證。設定裡沒有憑證檔路徑或密碼的欄位，憑證怎麼放、私鑰怎麼取用，是 `signtool` 與 Windows 的事。
- `digestAlgorithm`：檔案的雜湊演算法，沒設就是 `sha256`。
- `timestampUrl`：時間戳記伺服器，選用，沒設就不蓋時間戳記。
- `tsp`：決定時間戳記的參數。預設 `false`，用 `/t <網址>`；設成 `true`，改用 `/tr <網址> /td <演算法>`，也就是 RFC 3161 的 Time-Stamp Protocol。你的憑證供應商用哪一種，要問供應商。

簽章**只有在** `certificateThumbprint` 或下一節的 `signCommand` 至少設了一個時才會做，兩個都沒設，打包照常完成，檔案不會被簽。

為什麼要時間戳記：憑證有效期有限，簽的當下蓋上時間，憑證過期之後，簽章還能證明它是在有效期內簽的。這同樣是一般說法，不是原始碼的內容。

### 打包時簽了哪些檔案

不是只有最後的安裝檔。從原始碼數得出來的有：

- **App 的執行檔**：每一種安裝檔各簽一次。原因是 bundler 會先改寫執行檔、寫進打包類型，改過的檔案要重簽，所以簽章排在改寫之後。專案裡其他的執行檔目標，則在打包前就簽好。
- **`bundle.resources` 裡的 `.exe` 與 `.dll`**（第 15 章第 7 集）：只有這兩種副檔名才簽。在 Windows 上，bundler 先用 `signtool verify /pa` 檢查，已經簽過的就跳過。
- **安裝檔本身**：`.msi` 與 `-setup.exe`。這就是第 15 章第 5 集預告的安裝檔簽章。
- **NSIS 的 uninstaller**：uninstaller 是 `makensis` 打包時才做出來的，所以 bundler 把整條簽章指令（路徑的位置先寫成 `%1`）交給 `makensis` 的 `!uninstfinalize`，由 NSIS 在做好之後執行。
- **bundler 自己帶的 DLL**：NSIS 的外掛 DLL、WiX 的擴充 DLL。bundler 先複製一份再簽，不動原本的工具資料夾。MSI 還會把輸出資料夾（`target/release`）裡、沒列在資源檔的 `.dll` 一併簽掉。

跟簽章有關的旗標是 `--no-sign`：`npm run tauri build -- --no-sign` 會跳過上面全部的簽章，只印警告。它也會連第 2 集的更新檔簽章一起跳過。更新檔簽章是另一把金鑰簽的另一種簽章：CLI 等 bundler 把安裝檔與 Windows 簽章都做完，才替成品簽 `.sig`，所以 `.sig` 簽的是已經帶著 Windows 簽章的安裝檔。

### 不用 `signtool`：`signCommand`

`signtool` 只有 Windows 有。想用別的工具，像雲端簽章服務提供的命令列程式，或是交叉打包時在別的系統上跑的簽章工具（例如 `osslsigncode`），就改設 `signCommand`。下面的 `sign-cli` 是示範用的名字，不是真的工具：

```json,ignore
{
  "bundle": {
    "windows": {
      "signCommand": {
        "cmd": "sign-cli",
        "args": ["--arg1", "--arg2", "%1"]
      }
    }
  }
}
```

規則如下：

- 每個要簽的檔案，各執行一次這個指令。內容剛好是 `%1` 的那個參數，換成檔案路徑；其他參數照原樣傳過去，只有一個例外：如果參數是一個已經存在的相對路徑，會被換成絕對路徑，因為 NSIS 簽 uninstaller 時的工作目錄不一樣。
- `signCommand` 也可以寫成一個字串，如 `"sign-cli --arg1 --arg2 %1"`。Tauri 只用單一空格去切，所以參數裡有空白就得用上面的物件寫法。
- 兩種都設的話，`signCommand` 優先，指紋、`digestAlgorithm`、`timestampUrl`、`tsp` 不會被用到。
- 指令結束代碼不是 0，打包就中斷，錯誤訊息帶著指令的輸出。
- `args` 寫在 `tauri.conf.json`，那個檔案通常會進版本控制，密碼別寫進去。

### 在 Windows 以外的機器上

預設的 `signtool` 只在 Windows 上存在。bundler 能在別的系統上交叉打出 NSIS 安裝檔（依註解是在 Linux 上用 `cargo-xwin`，打包時也會警告交叉編譯還是實驗性的），這本書不展開，但這種情況有一個容易踩到的地方：只設 `certificateThumbprint`，bundler 不報錯，也不會簽，因為在非 Windows 上，沒有 `signCommand` 的話 bundler 直接略過簽章、回報成功。「預設只支援 Windows 主機」那行警告，只在兩個欄位都沒設時才出現。要在別的系統上簽，一定得設 `signCommand`。這一點我們也只讀了原始碼。

### 付費憑證的流程

程式碼簽章憑證要花錢買，所以這本書沒有範例依賴它，只說流程。以下是從設定欄位推得出來的順序，細節（怎麼查指紋、憑證怎麼匯入、需不需要硬體權杖）是 Windows 與供應商的規矩，不在 Tauri 的原始碼裡，我們也沒有試過，請以他們的文件為準：

1. 向憑證供應商申請程式碼簽章憑證，並取得時間戳記伺服器的網址與 `tsp` 該不該開。
2. 把憑證裝到打包用的那台 Windows，讓 `signtool` 找得到。
3. 查出憑證的 SHA1 指紋，填進 `certificateThumbprint`，再填 `timestampUrl`（與需要時的 `tsp`）。
4. 在那台機器上打包。要確認簽好了沒有，可以對安裝檔執行 `signtool verify /pa`，bundler 內部判斷檔案簽過沒有，用的也是這個指令。
5. 供應商若不給你憑證，只提供它自己的簽章工具或服務，就改走 `signCommand`。

指紋只是憑證的識別碼，不是機密，寫進設定檔沒關係；該保護的是私鑰與供應商給你的密碼。

## 重點整理

- `bundle.windows.certificateThumbprint` 或 `signCommand` 至少設一個，打包才會簽章；兩個都沒設就不簽，兩個都設則 `signCommand` 優先。
- 預設用 Windows SDK 的 `signtool sign`：`digestAlgorithm` 預設 `sha256`，`timestampUrl` 選用，`tsp` 決定用 `/t` 還是 `/tr`。
- 簽的不只安裝檔：App 執行檔、資源檔裡的 `.exe` 與 `.dll`、NSIS 的 uninstaller、bundler 帶的 DLL 都會簽；`--no-sign` 全部跳過，連更新檔簽章也一起。
- `signCommand` 取代 `signtool`，`%1` 是檔案路徑，結束代碼不是 0 就中斷；在非 Windows 上只設指紋，什麼都不會簽。
- 這一集沒在 Windows 上跑過，付費憑證的部分只說流程。
