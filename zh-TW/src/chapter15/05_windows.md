# Windows：MSI 與 NSIS

## 本集目標

知道 Windows 上 `npm run tauri build` 會做出哪兩種安裝檔、各自放在哪裡、差在哪裡，以及打 MSI 之前要先準備什麼、版本號有什麼限制。

## 正文

### 先說清楚：這一集沒有實測

我們沒有 Windows 機器，而 MSI 只有在 Windows 上才打得出來，所以這一集沒有貼任何打包的輸出。你要照著做，需要一台 Windows：實體機、虛擬機，或是第 17 章的 CI。

檔名、路徑、預設值與版本號的限制，都是對照 Tauri CLI 2.12 內建的 tauri-bundler 2.10 原始碼查出來的。VBSCRIPT 出自官方文件，WebView2 的檔案大小出自原始碼的註解，下面各自標出來。

### 兩種安裝檔

範本的 `bundle.targets` 是 `"all"`。在 Windows 上，它展開成 `msi` 與 `nsis` 兩種，各放在 `src-tauri\target\release\bundle\` 底下自己的資料夾（用 `--debug` 打包就是 `debug`）：

| | MSI | NSIS |
| --- | --- | --- |
| 資料夾 | `bundle\msi\` | `bundle\nsis\` |
| 檔名格式 | `產品名_版本_架構_語言.msi` | `產品名_版本_架構-setup.exe` |
| 起始專案會得到 | `tauri-app_0.1.0_x64_en-US.msi` | `tauri-app_0.1.0_x64-setup.exe` |
| 用什麼做的 | WiX Toolset 3.14 | NSIS 3.13 |
| 預設安裝位置 | `Program Files` 底下 | `%LOCALAPPDATA%` 底下 |

產品名是 `tauri.conf.json` 的 `productName`，版本是 `version`。架構欄是 `x64`，ARM 版的 Windows 則是 `arm64`。檔名裡的語言是安裝畫面的語言，兩種安裝檔預設都是英文，要換語言各有自己的設定，這本書不展開。

如果設了 `bundle.fileAssociations`（第 12 章第 7 集），兩種安裝檔都會在安裝時把檔案關聯寫進系統，雙擊檔案時執行的指令是 `"App 路徑" "%1"`，`%1` 是被雙擊的檔案。這一點同樣只讀了原始碼，沒有在 Windows 上試過。

### 打 MSI 要先準備兩件事

MSI 的打包程式，原始碼裡有 `#[cfg(target_os = "windows")]`，只在 Windows 上編進去。在 macOS 或 Linux 上跑 `tauri build`，`"all"` 只會展開成那個平台自己的格式，不會有任何 Windows 安裝檔。要一次打好各平台，是第 17 章 GitHub Actions 的事。

在 Windows 上打包，要先準備兩件事：

- **網路**：第一次打包時，工具要從網路下載。MSI 是用 WiX 做的，Tauri 從 GitHub 下載 WiX 3.14；NSIS 是另一套工具，Tauri 下載的是 NSIS 3.13 和一個輔助用的 `nsis_tauri_utils.dll`。下載的東西存在使用者快取資料夾底下的 `tauri` 資料夾，之後就不用再下載。
- **VBSCRIPT**：官方文件說，打 MSI 需要 Windows 的 VBSCRIPT 選用功能（optional feature）。多數 Windows 預設是開的；被關掉的話，打 MSI 會出現 `failed to run light.exe` 這類錯誤（`light.exe` 是 WiX 的工具）。遇到的話，到 Windows 設定的選用功能裡把 VBSCRIPT 加回來。這一條是官方文件的說法，tauri 的原始碼裡一個字也搜不到，我們沒有驗證過。

### MSI 的版本號有限制

MSI 要求版本是 `主.次.修` 的數字（後面可以再接一組建置號碼），而且有上限：主版號與次版號不能超過 255，修訂號與建置號碼不能超過 65535。所以拿年份當主版號的 `2026.1.0` 不行，打 MSI 時會報 `app version major number cannot be greater than 255`。

帶字母的預發行版本，像 `1.0.0-beta.1`，也會直接報錯；全數字的，像 `1.0.0-4`，會被轉成 `1.0.0.4`。NSIS 的原始碼沒有同樣的檢查。真的要用年份或字母標籤，可以另外設 `bundle.windows.wix.version` 專門給 MSI 用（一樣要符合上面的限制）。版本號本身怎麼設，見第 3 集。

### 裝在哪裡

MSI 固定是「每台機器」的安裝，裝進 `Program Files` 底下以 `productName` 命名的資料夾。NSIS 預設只裝給目前的使用者，位置是 `%LOCALAPPDATA%\<productName>`，原始碼的說明是「不需要系統管理員權限的位置」。

NSIS 的 `installMode` 可以改：`currentUser`（預設）、`perMachine`（裝進 `Program Files`），或 `both`（安裝時讓使用者選，這個模式一定要系統管理員權限）。

### WebView2

第 1 章第 1 集說過，Windows 上的 WebView 是 WebView2。兩種安裝檔裝到一半，都會先去讀登錄檔，看電腦裡有沒有 WebView2，有就跳過。沒有的話，由 `bundle.windows.webviewInstallMode` 決定怎麼辦，預設是 `downloadBootstrapper`：安裝時從網路下載微軟的 bootstrapper 來裝。安裝檔最小，但那台電腦要連得上網路。

離線環境要改成 `offlineInstaller`，把完整的 WebView2 安裝檔放進你的安裝檔，原始碼註解說大約多 127 MB：

```json,ignore
{
  "bundle": {
    "windows": {
      "webviewInstallMode": { "type": "offlineInstaller" }
    }
  }
}
```

其他選項（`embedBootstrapper`、`fixedRuntime`、`skip`）這裡不展開。安裝檔的簽章，第 16 章會正式介紹。

## 重點整理

- Windows 上 `"all"` 展開成 `msi` 與 `nsis`，分別放在 `bundle\msi\` 與 `bundle\nsis\`，檔名格式不同。
- MSI 只能在 Windows 上打，第一次要下載 WiX；打包時出現 `failed to run light.exe`，先確認 VBSCRIPT 選用功能有開。
- MSI 的版本號是數字，主、次版號不超過 255、修訂號不超過 65535，不能帶字母預發行標籤；NSIS 預設裝進 `%LOCALAPPDATA%`，MSI 裝進 `Program Files`。
- 預設的 `downloadBootstrapper` 要求安裝的電腦連得上網路，離線環境改用 `offlineInstaller`。
