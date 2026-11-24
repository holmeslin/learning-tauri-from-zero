# isolation pattern

## 本集目標

知道 isolation pattern 在 IPC 路上多放了什麼、什麼情況值得開，並寫一個會擋下訊息的 hook，實際跑一次。

## 正文

### 為什麼要多一道

前幾集的防線，permission 管的是「能不能呼叫這個 command」，參數對不對，只能靠 command 自己檢查（第 4 集）。如果你的前端用了很多第三方程式（npm 套件、載入進來的外部腳本），想在所有 command 前面統一加一道檢查，就可以開 isolation pattern：每一則 IPC 訊息，在送到 Rust 之前，先經過一段你寫的 JavaScript。這段程式跑在頁面碰不到的地方，頁面裡的惡意程式改不到它。

### 它怎麼運作

打開 isolation 之後，Tauri 會在你的頁面裡放一個隱藏的 `<iframe>`，網址是一個專用的 `isolation-…://localhost/`，裡面載入的是你提供的另一個小網頁。頁面每次 `invoke`，訊息都先交給這個 iframe：

1. iframe 裡的 `window.__TAURI_ISOLATION_HOOK__` 收到訊息，可以檢查、修改，丟出錯誤就是不放行。
2. hook 回傳的訊息，由 iframe 用一把金鑰加密，再送給 Rust。這把金鑰是 App 每次啟動時產生的，只給了 iframe。Rust 解得開才會執行，解不開的一律丟掉。

iframe 是 sandbox 的，頁面的程式讀不到、改不到裡面的東西，也拿不到金鑰，所以繞不過去。

### 範例：擋下太長的名字

我們沿用範本的 greet 表單，不改 `lib.rs` 與前端，只加一段 hook：`greet` 的 `name` 必須是 10 個字以內的字串。isolation 的小網頁放在專案根目錄的 `dist-isolation/`，跟 `src-tauri/` 同一層。

`dist-isolation/index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <title>isolation</title>
  </head>

  <body>
    <script src="index.js"></script>
  </body>
</html>
```

`dist-isolation/index.js`：

```js
const MAX_NAME_LENGTH = 10;

window.__TAURI_ISOLATION_HOOK__ = (message) => {
  if (message.cmd === "greet") {
    const name = message.payload.name;
    if (typeof name !== "string" || [...name].length > MAX_NAME_LENGTH) {
      throw new Error(`greet 的 name 要是 ${MAX_NAME_LENGTH} 個字以內的字串`);
    }
  }
  return message;
};
```

`message` 就是準備送出的那則訊息：`cmd` 是 command 的名字（plugin 的是 `plugin:fs|read_text_file` 這種），`payload` 是參數物件。回傳什麼，Rust 就收到什麼。

算「幾個字」用 `[...name].length`，不用 `name.length`：後者算的是 UTF-16 單位，`𠮷` 或 emoji 這類字會算成 2，10 個這樣的字會被當成 20 而被擋掉。`[...name]` 把字串展開成一個個 code point，一個字算 1。（仍然不等於螢幕上看到的一個字：`👨‍👩‍👧` 是三個 emoji 加兩個零寬連接字元，會算成 5。這裡的目的只是擋掉離譜的長度，不求精確。）

接著在 `tauri.conf.json` 的 `app.security` 加上 `pattern`，告訴 Tauri 去哪裡找這個小網頁。`dir` 是相對於 `tauri.conf.json` 的路徑：

`src-tauri/tauri.conf.json`：

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "tauri-app",
  "version": "0.1.0",
  "identifier": "tw.example.tauri-app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "tauri-app",
        "width": 800,
        "height": 600
      }
    ],
    "security": {
      "csp": null,
      "pattern": {
        "use": "isolation",
        "options": {
          "dir": "../dist-isolation"
        }
      }
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

另外，`src-tauri/Cargo.toml` 的 `tauri` 與 `tauri-build` 也要打開 `isolation` 這個 feature，兩邊必須一致，不然 `tauri-build` 會停下來，說 `Cargo.toml` 的 features 和 `tauri.conf.json` 對不上。不用自己改：第一次跑 `npm run tauri dev` 時，Tauri CLI 會發現不一致，自動替你加好。一個剛建好的專案，改完是這樣（如果你的專案還有別的 feature，像第 8 章的 `tray-icon`，會列在同一個清單裡）：

`src-tauri/Cargo.toml`（CLI 加上 `isolation` 之後）：

```toml
[package]
name = "tauri-app"
version = "0.1.0"
description = "A Tauri App"
authors = ["you"]
edition = "2024"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
# The `_lib` suffix may seem redundant but it is necessary
# to make the lib name unique and wouldn't conflict with the bin name.
# This seems to be only an issue on Windows, see https://github.com/rust-lang/cargo/issues/8519
name = "tauri_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = ["isolation"] }

[dependencies]
tauri = { version = "2", features = ["isolation"] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"


# Read the optimization guideline for more details: https://tauri.app/concept/size/#cargo-configuration
[profile.release]
codegen-units = 1
lto = true
opt-level = 3
panic = "abort"
strip = true
```

`npm run tauri dev` 的輸出（節錄，第一次要重新編譯 `tauri`，所以比較久）：

```text
   Compiling aes-gcm v0.10.3
   Compiling tauri-build v2.7.5
   Compiling tauri v2.12.6
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.22s
     Running `target/debug/tauri-app`
```

在表單輸入 `小明`，段落是「Hello, 小明! You've been greeted from Rust!」。輸入一個超過 10 個字的名字，例如 `這是一個超過十個字的很長很長的名字`，段落什麼都沒出現：訊息在 iframe 裡被擋下，根本沒到 Rust。

### 擋不掉的人會怎麼做

打開開發者工具的 Console，直接呼叫 `invoke`，也一樣：

```js,ignore
window.__TAURI__.core.invoke("greet", { name: "x".repeat(100) }).then(console.log);
```

什麼都不會印。`window.__TAURI_INTERNALS__.invoke` 也一樣，所有 `invoke` 都得先過 iframe。想改 iframe 裡的 hook，例如 `document.querySelector("iframe").contentWindow.__TAURI_ISOLATION_HOOK__ = ...`，瀏覽器會丟出 `SecurityError`。

### 要注意的事

- 被擋下的訊息，前端的 `invoke` 既不會成功也不會失敗，Promise 一直停在等待。需要的話，前端要自己設逾時。
- isolation 的小網頁是編譯時嵌進 App 的。`dist-isolation/` 裡的檔案改了，`tauri dev` 不會自動重新編譯，畫面看起來沒變。存一次 `src-tauri/` 底下的檔案，例如 `lib.rs`，它才會重新編譯，新的 hook 才生效。
- 它只檢查訊息，不取代 Rust 端的 permission 與 scope。每一條訊息要多繞一趟 iframe 和加密。Tauri 的設定說明把它標為「為了安全建議開啟」；前端用的 npm 套件越多，越值得付這個成本。
- 同時設了 CSP（第 5 集）的話，Tauri 提供頁面、附上 CSP 的時候，會自動把 isolation iframe 的來源加進 `default-src`，不用自己寫。這只發生在 Tauri 自己提供頁面的版本（打包版）；`tauri dev` 載入的是 Vite 的頁面，沒有 CSP 可加。

### 跟 Electron 比一比

Electron 也有一個叫 `contextIsolation` 的設定，但那是另一回事：它把 preload 腳本和頁面腳本放在不同的 JavaScript 環境，免得頁面改到 preload 的物件。Tauri 的 isolation pattern 不隔離你的腳本，而是讓每一則 IPC 訊息先經過 sandbox iframe 的檢查。名字像，不要混在一起。

## 重點整理

- isolation pattern 讓每一則 IPC 訊息先經過一個 sandbox iframe 裡的 `window.__TAURI_ISOLATION_HOOK__`，丟出錯誤就是不放行，通過的訊息才用只有 iframe 持有的金鑰加密送給 Rust。
- 在 `tauri.conf.json` 設 `app.security.pattern`（`use` 是 `isolation`，`options.dir` 指向小網頁的資料夾）；`Cargo.toml` 的 `isolation` feature 由 Tauri CLI 自動加。
- 頁面的程式讀不到、改不到 hook，也繞不過它；被擋下的 `invoke` 會一直等待。
- `dist-isolation/` 的內容是編譯時嵌進去的，改了要讓 Rust 重新編譯才生效。
- 前端有第三方程式碼才值得開，它是多一道防線，不取代 permission。
