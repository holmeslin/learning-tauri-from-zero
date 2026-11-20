# Windows：MSI 與 NSIS

## 本集目標

知道 Windows 上 `npm run tauri build` 會做出哪兩種安裝檔、各自放在哪裡、差在哪裡，以及打 MSI 之前要先準備什麼。

## 正文

### 先說清楚：這一集沒有實測

我們沒有 Windows 機器，而 MSI 只有在 Windows 上才打得出來，所以這一集沒有貼任何打包的輸出。你要照著做，需要一台 Windows：實體機、虛擬機，或是第 17 章的 CI。

這一集的檔名、路徑、預設值，都是對照 Tauri CLI 2.12 內建的 tauri-bundler 2.10 原始碼查出來的。只有一件事原始碼裡查不到，就是第 1 章第 2 集留下的 VBSCRIPT，下面會標出來。

### 兩種安裝檔

範本的 `bundle.targets` 是 `"all"`。在 Windows 上，它展開成 `msi` 與 `nsis` 兩種，各放在 `src-tauri\target\release\bundle\` 底下自己的資料夾（用 `--debug` 打包就是 `debug`）：

| | MSI | NSIS |
| --- | --- | --- |
| 資料夾 | `bundle\msi\` | `bundle\nsis\` |
| 檔名格式 | `產品名_版本_架構_語言.msi` | `產品名_版本_架構-setup.exe` |
| 起始專案會得到 | `tauri-app_0.1.0_x64_en-US.msi` | `tauri-app_0.1.0_x64-setup.exe` |
| 用什麼做的 | WiX Toolset 3.14 | NSIS 3.13 |
| 預設安裝位置 | `Program Files` 底下 | `%LOCALAPPDATA%` 底下 |
| 預設語言（只有這一種） | `en-US` | `English` |

產品名是 `tauri.conf.json` 的 `productName`，版本是 `version`。架構欄是 `x64`，ARM 版的 Windows 則是 `arm64`。

如果設了 `bundle.fileAssociations`（第 12 章第 7 集），兩種安裝檔都會在安裝時把檔案關聯寫進系統，雙擊檔案時執行的指令是 `"App 路徑" "%1"`，`%1` 是被雙擊的檔案。這一點同樣只讀了原始碼，沒有在 Windows 上試過。

### MSI 只能在 Windows 打，還有兩件事要先準備

MSI 的打包程式，原始碼裡有 `#[cfg(target_os = "windows")]`，只在 Windows 上編進去。在 macOS 或 Linux 上跑 `tauri build`，沒有指定 `--target` 的話，`"all"` 只會展開成那個平台自己的格式，不會有任何 Windows 安裝檔。

NSIS 另有一條路：原始碼的註解寫著，NSIS 安裝檔可以在 Linux 與 macOS 上交叉打包，但打包時會警告這是實驗性功能。這本書不走這條路，跨平台打包交給第 17 章。

在 Windows 上打包，要先準備兩件事：

- **網路**：第一次打包時，工具要從網路下載。MSI 是用 WiX 做的，Tauri 從 GitHub 下載 WiX 3.14；NSIS 是另一套工具（`makensis`），不經過 WiX，Tauri 下載的是 NSIS 3.13 和一個輔助用的 `nsis_tauri_utils.dll`。下載的東西存在使用者快取資料夾底下的 `tauri` 資料夾，之後就不用再下載。
- **VBSCRIPT**：Tauri 官方文件說，打 MSI 需要啟用 Windows 的 VBSCRIPT 選用功能（optional feature）。這一條是文件寫的，tauri 的原始碼裡一個字也搜不到，我們也沒有驗證過；原始碼只看得出 MSI 是呼叫 WiX 的 `candle.exe` 與 `light.exe` 做的。

### MSI 的版本號只吃數字

MSI 要求版本是數字。`version` 要是 `1.0.0-beta.1` 這種帶字母的預發行版本，打 MSI 時會直接報錯；全數字的，像 `1.0.0-4`，會被轉成 `1.0.0.4`。NSIS 的原始碼沒有同樣的檢查。真的要用字母標籤，可以另外設 `bundle.windows.wix.version`（也要是數字格式）專門給 MSI 用。版本號本身怎麼設，見第 3 集。

### 裝在哪裡、用什麼語言

MSI 固定是「每台機器」的安裝，裝進 `Program Files` 底下以 `productName` 命名的資料夾。NSIS 預設只裝給目前的使用者，位置是 `%LOCALAPPDATA%\<productName>`，原始碼的說明是「不需要系統管理員權限的位置」。

NSIS 的 `installMode` 可以改：`currentUser`（預設）、`perMachine`（裝進 `Program Files`），或 `both`（安裝時讓使用者選，這個模式一定要系統管理員權限）。

語言也是各改各的。繁體中文在 MSI 叫 `zh-TW`，在 NSIS 叫 `TradChinese`：

```json,ignore
{
  "bundle": {
    "windows": {
      "wix": { "language": "zh-TW" },
      "nsis": {
        "installMode": "perMachine",
        "languages": ["TradChinese"]
      }
    }
  }
}
```

NSIS 的安裝畫面預設用作業系統的語言，不在 `languages` 清單裡，就用清單的第一個。NSIS 內建了 `TradChinese` 的翻譯；MSI 這邊，Tauri 自己加的幾句文字（像 `Launch tauri-app`）預設是英文，要翻成中文，得用 `language` 的 `localePath` 另外給一份翻譯檔。

### WebView2

第 1 章第 1 集說過，Windows 上的 WebView 是 WebView2。兩種安裝檔裝到一半，都會先去讀登錄檔，看電腦裡有沒有 WebView2，有就跳過。

沒有的話怎麼辦，由 `bundle.windows.webviewInstallMode` 決定，預設是 `downloadBootstrapper`：安裝時從網路下載微軟的安裝程式來裝。安裝檔最小，但那台電腦要連得上網路。NSIS 的原始碼裡，下載失敗就會中止安裝。

其他選項把安裝程式直接放進安裝檔，用安裝檔變大換掉網路：

- `embedBootstrapper`：放 bootstrapper，大約多 1.8 MB，還是要連網。
- `offlineInstaller`：放完整的安裝程式，大約多 127 MB，不用連網。
- `fixedRuntime`：放固定版本的 WebView2，大約多 180 MB，要用 `path` 指到資料夾。
- `skip`：不處理 WebView2。

這些大小是原始碼註解寫的概數。設定的寫法是一個帶 `type` 的物件：

```json,ignore
{
  "bundle": {
    "windows": {
      "webviewInstallMode": { "type": "offlineInstaller" }
    }
  }
}
```

安裝檔的簽章，第 16 章會正式介紹。

## 重點整理

- Windows 上 `"all"` 展開成 `msi` 與 `nsis`，分別放在 `bundle\msi\` 與 `bundle\nsis\`，檔名格式不同。
- MSI 只能在 Windows 上打，第一次要下載 WiX；官方文件說還要啟用 VBSCRIPT，這一條原始碼裡查不到，我們沒有驗證。
- MSI 版本不能帶字母預發行標籤；NSIS 預設裝進 `%LOCALAPPDATA%`，MSI 裝進 `Program Files`。
- 預設的 `downloadBootstrapper` 要求安裝的電腦連得上網路，離線環境改用 `offlineInstaller` 或 `fixedRuntime`。
- 這一集沒有在 Windows 上實測，內容依 tauri-bundler 2.10 的原始碼整理。
