# `npm run tauri dev` 與熱重載

## 本集目標

用 `npm run tauri dev` 把 App 跑起來，並看到改了前端和改了 Rust 之後，畫面各自怎麼更新。

## 正文

### 啟動開發模式

在專案資料夾執行：

```bash
npm run tauri dev
```

`npm run tauri` 會執行 `package.json` 的 `scripts` 裡叫 `tauri` 的那一行，也就是專案裡裝的 Tauri 命令列工具，後面的 `dev` 是交給它的子命令。第一次要下載並編譯兩百多個 crate，視電腦而定，要幾十秒到幾分鐘。

`npm run tauri dev` 的輸出（節錄，中間大量的編譯訊息省略）：

```text
> tauri-app@0.1.0 tauri
> tauri dev

     Running BeforeDevCommand (`npm run dev`)

> tauri-app@0.1.0 dev
> vite


  VITE v8.3.8  ready in 324 ms

  ➜  Local:   http://localhost:1420/
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info Watching ~/tauri-app/src-tauri for changes...
   Compiling proc-macro2 v1.0.107
   Compiling tauri v2.12.6
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.80s
     Running `target/debug/tauri-app`
```

編譯完會跳出一個 800×600 的視窗，標題是 `tauri-app`，上面有 Welcome to Tauri、三個 logo、一個輸入框和 Greet 按鈕。在輸入框打 `Tauri` 再按 Greet，下方會出現 `Hello, Tauri! You've been greeted from Rust!`。這句話是 Rust 產生的，第 7 集會一路追它的來歷。

如果你用 macOS，終端機後面可能還會多出 `TSM AdjustCapsLockLEDForKeyTransitionHandling ...` 和 `error messaging the mach port for IMKCFRunLoopWakeUpReliable` 這類訊息。它們是系統的輸入法元件印的，雖然有 `error` 字樣，跟你的程式無關，可以忽略。

### 同時跑了兩個東西

輸出裡有兩個 `Running`：

- `BeforeDevCommand`：先啟動 Vite 的開發伺服器，網址是 `http://localhost:1420/`。
- `DevCommand`：再編譯並執行 Rust 專案。程式一啟動就開一個視窗，載入剛剛那個網址。

所以視窗裡的畫面不是從檔案讀出來的，而是本機伺服器送來的網頁。前端一有改動，Vite 就能通知視窗更新。這兩個指令和網址都寫在 `tauri.conf.json`，第 6 集會看到。

### 改前端：存檔就更新

讓 `tauri dev` 繼續開著，把 `index.html` 的標題改掉：

`index.html`（標題改成中文）：

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="/src/styles.css" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Tauri App</title>
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>你好，Tauri！</h1>

      <div class="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/src/assets/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img
            src="/src/assets/tauri.svg"
            class="logo tauri"
            alt="Tauri logo"
          />
        </a>
        <a href="https://www.typescriptlang.org/docs" target="_blank">
          <img
            src="/src/assets/typescript.svg"
            class="logo typescript"
            alt="typescript logo"
          />
        </a>
      </div>
      <p>Click on the Tauri logo to learn more about the framework</p>

      <form class="row" id="greet-form">
        <input id="greet-input" placeholder="Enter a name..." />
        <button type="submit">Greet</button>
      </form>
      <p id="greet-msg"></p>
    </main>
  </body>
</html>
```

存檔，視窗馬上就換了，不必重開 App。終端機多了一行 Vite 的訊息：

```text
上午11:01:59 [vite] (client) page reload index.html
```

時間的格式會跟著你的系統語言走。如果改的是 `src/styles.css`，訊息會變成 `hmr update /src/styles.css?direct`，表示 Vite 連頁面都不重新載入，只把新樣式換進去。這種存檔就看到結果的機制叫熱重載（hot reload）。

### 改 Rust：重新編譯並重啟

Rust 沒有熱重載。把 `src-tauri/src/lib.rs` 的問候語改掉：

`src-tauri/src/lib.rs`（問候語改成中文）：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好，{name}！這句話是 Rust 說的。")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

存檔後，Tauri 發現 `src-tauri/` 裡有檔案變了，會把 App 關掉、重新編譯、再開起來：

`npm run tauri dev` 的輸出（改了 `lib.rs` 之後）：

```text
        Info File src-tauri/src/lib.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.27s
     Running `target/debug/tauri-app`
```

這次只重新編譯你的 crate，所以幾秒就好。視窗會閃一下再出現，剛才在輸入框打的字也不見了，因為整個 App 是重新啟動的。

要結束開發模式，在終端機按 `Ctrl + C`；直接關掉 App 視窗，`tauri dev` 也會跟著結束。

## 重點整理

- `npm run tauri dev` 會啟動 Vite 開發伺服器和 Rust 程式，視窗載入 `http://localhost:1420/`。
- 第一次編譯要下載、編譯很多 crate，之後只重編改過的部分。
- 改前端存檔就更新畫面，不用重開 App。
- 改 Rust 會自動重新編譯並重啟整個 App，狀態歸零。
- 用 `Ctrl + C` 或關閉視窗結束。
