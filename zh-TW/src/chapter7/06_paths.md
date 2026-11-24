# 路徑：App 的資料夾

## 本集目標

知道 App 的資料該放在哪，會用 `appDataDir()` 與 `join()`（前端）、`app.path().app_data_dir()`（Rust）拿到這個資料夾的路徑，也看懂第 5 集的 `baseDir` 指到哪裡。

## 正文

### 每個 App 一個資料夾

App 要存檔，不能把檔案亂丟。每個作業系統都有慣例的位置，Tauri 在這些位置底下，用 `tauri.conf.json` 的 `identifier`（第 1 章第 6 集）當資料夾名字，替每個 App 分出專屬的 App 資料夾。不同 App 的 identifier 不同，檔案就不會互相覆蓋。反過來說，identifier 改了，資料夾也跟著換，舊資料就找不到了。

App 資料夾依用途分成幾種。這一集看兩種：`AppData` 放使用者的資料（筆記、設定），`AppLog` 放日誌（第 10 集會用到）。

### 前端與 Rust 怎麼拿路徑

前端用 `@tauri-apps/api/path` 的 `appDataDir()`、`appLogDir()`。它們屬於 Tauri 核心的 `@tauri-apps/api`，不是 plugin，不用安裝、註冊，`core:default` 也已經放行。每個函式回傳 `Promise<string>`，要 `await`。`join()` 把路徑片段接起來，用目前平台的分隔符號。

Rust 用 `app.path().app_data_dir()`，名字跟前端一一對應，回傳 `Result<PathBuf>`。`path()` 定義在 `Manager` 上，要先 `use`（第 4 章第 4 集）。

### 範例：看路徑，也寫一個檔案

`app_data_dir()` 只算出路徑，**不會替你建資料夾**。所以下面的 command 先 `create_dir_all`，再寫 `hello.txt`。Rust 端用標準函式庫寫檔，不需要 plugin，也不需要 permission：

`src-tauri/src/lib.rs`：

```rust
use tauri::{AppHandle, Manager};

#[tauri::command]
fn write_hello_file(app: AppHandle) -> tauri::Result<String> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    let file = dir.join("hello.txt");
    std::fs::write(&file, "你好，app 資料夾")?;
    Ok(file.display().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![write_hello_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`create_dir_all` 與 `write` 回傳的是 `std::io::Result`，`tauri::Error` 能從 `io::Error` 轉換，所以同樣用 `?`。

前端印出兩個資料夾，再比較前端 `join()` 算出的路徑，和 Rust 實際寫入的路徑：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>路徑</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>路徑</h1>
      <div class="row">
        <button id="show-btn" type="button">顯示路徑</button>
      </div>
      <pre id="paths"></pre>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { appDataDir, appLogDir, join } from "@tauri-apps/api/path";

const pathsEl: HTMLElement | null = document.querySelector("#paths");
if (!pathsEl) {
  throw new Error("找不到 #paths");
}

document.querySelector("#show-btn")?.addEventListener("click", async () => {
  const dataDir = await appDataDir();
  pathsEl.textContent = `appDataDir: ${dataDir}\n`;
  pathsEl.textContent += `appLogDir: ${await appLogDir()}\n`;
  pathsEl.textContent += `前端 join: ${await join(dataDir, "hello.txt")}\n`;
  pathsEl.textContent += `Rust 寫入: ${await invoke("write_hello_file")}`;
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.18s
     Running `target/debug/tauri-app`
```

按下「顯示路徑」，在 macOS 上畫面是（`ada` 是範例的使用者名稱）：

```text
appDataDir: /Users/ada/Library/Application Support/tw.example.tauri-app
appLogDir: /Users/ada/Library/Logs/tw.example.tauri-app
前端 join: /Users/ada/Library/Application Support/tw.example.tauri-app/hello.txt
Rust 寫入: /Users/ada/Library/Application Support/tw.example.tauri-app/hello.txt
```

最後兩行一模一樣：前端與 Rust 說的是同一個資料夾。到終端機確認檔案真的在那裡：

```bash
cat "$HOME/Library/Application Support/tw.example.tauri-app/hello.txt"
```

```text
你好，app 資料夾
```

這一集只在 macOS 實測，其他平台的實際路徑沒有驗證，所以要用函式問，不要自己拼路徑。

### 第 5 集的 `baseDir`

第 5 集的 `baseDir: BaseDirectory.AppData`，意思是「路徑從 `appDataDir()` 算起」。`BaseDirectory` 的成員跟這些函式一一對應，例如 `AppLog` 是 `appLogDir()`。所以寫 `"hello.txt"` 加 `BaseDirectory.AppData`，檔案就會出現在 `前端 join` 那一行的位置。

## 重點整理

- 每個 App 在系統慣例的位置底下，有一個用 `identifier` 命名的資料夾；使用者資料放 `AppData`，日誌放 `AppLog`。
- 前端用 `@tauri-apps/api/path` 的 `appDataDir()`、`join()`；Rust 用 `app.path().app_data_dir()`。它們是核心 API，不是 plugin。
- 這些函式只算路徑，不建資料夾；要寫檔，先建資料夾。
- `baseDir: BaseDirectory.AppData` 就是從 `appDataDir()` 算起；macOS 上是 `~/Library/Application Support/<identifier>`。
