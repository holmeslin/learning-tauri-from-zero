# Tauri 的安全模型

## 本集目標

說得出 Tauri 把信任的界線畫在哪裡，認得四道防線各擋哪一段，並親眼看一個 Tauri 預設不會替你擋的缺口：自己寫的 command。

## 正文

### 前端是信不過的一端

前面的章節裡，前端都是我們自己寫的，看起來沒什麼好防。但跑在 WebView 裡的程式，不只有你寫的那幾行：

- npm 套件（第 2 章第 11 集）：任何一個套件被人動了手腳，它的程式就跟你的程式跑在同一個頁面裡。
- 使用者給的內容：第 12 章的預覽把 Markdown 轉成 HTML 放進頁面，要是轉換沒擋乾淨，別人檔案裡的惡意程式就會在你的頁面裡執行，這種攻擊叫 XSS（cross-site scripting）。
- 頁面自己載入的網路資源，例如圖片和腳本。

只要有一段惡意的 JavaScript 跑進頁面，它就有你的前端同樣的身分，前端能做的它都能做。反過來，Rust 這一側是編譯好的程式，這些東西改不到。所以 Tauri 的安全模型只有一個出發點：**前端是信不過的一端，Rust 是信得過的一端，界線就是 IPC**（第 1 章第 1 集）。前端要做什麼，都得送訊息過界線，由 Rust 決定放不放行。

### 範例：一個什麼都沒擋的 command

先寫一個最單純的 command：前端給路徑，Rust 就把檔案讀出來，是第 12 章那兩個檔案 command 的簡化版。

`src-tauri/src/lib.rs`：

```rust
#[tauri::command]
fn read_text(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("{path}：{e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_text])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

頁面沿用範本的表單，輸入框（`#greet-input`）填路徑，按鈕（寫著 Greet）改成讀檔，結果顯示在 `#greet-msg`，`index.html` 不用動：

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const pathEl: HTMLInputElement | null = document.querySelector("#greet-input");
const msgEl: HTMLElement | null = document.querySelector("#greet-msg");
if (!pathEl || !msgEl) {
  throw new Error("找不到 #greet-input 或 #greet-msg");
}

document.querySelector("#greet-form")?.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    msgEl.textContent = await invoke<string>("read_text", { path: pathEl.value });
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.77s
     Running `target/debug/tauri-app`
```

在輸入框填 `/etc/hosts`（Windows 可以填 `C:\Windows\System32\drivers\etc\hosts`），按按鈕，檔案內容就出現在畫面上（節錄前幾行）：

```text
##
# Host Database
#
# localhost is used to configure the loopback interface
# when the system is booting.  Do not change this entry.
##
127.0.0.1	localhost
255.255.255.255	broadcasthost
```

這裡沒有任何 permission，也沒動 `capabilities/default.json`，`read_text` 直接就能用，第 3 章第 1 集說過。

### 不需要你的按鈕

要呼叫 `read_text`，其實不需要那個按鈕。打開開發者工具（第 1 章第 8 集）的 Console，貼上：

```js,ignore
window.__TAURI__.core.invoke("read_text", { path: "/etc/hosts" }).then(console.log);
```

Console 一樣印出檔案內容。`window.__TAURI__` 是範本的 `withGlobalTauri: true`（第 1 章第 6 集）放進頁面的。把它改成 `false`，`window.__TAURI__` 就不存在了，但頁面裡還有 `window.__TAURI_INTERNALS__`（第 9 章第 3 集看過），`window.__TAURI_INTERNALS__.invoke("read_text", …)` 照樣讀得到檔案，所以關掉它不算防線。

Console 只是方便示範。任何一段跑在這個頁面裡的 JavaScript，不管是你寫的、套件帶進來的，還是 XSS 注入的，都能這樣呼叫；路徑換成使用者自己的重要檔案，也一樣讀得到。這就是第 12 章說「要發佈給別人，就該限制範圍」的原因。

### 四道防線

Tauri 替這道界線準備了四道防線，後面五集會一一介紹：

| 防線 | 擋什麼 | 設定在哪裡 | 哪一集 |
| --- | --- | --- | --- |
| capability | 哪個視窗拿得到哪些 permission | `src-tauri/capabilities/*.json` | 第 2 集 |
| permission 與 scope | 能呼叫哪些 command、能碰哪些路徑或網址 | plugin 內建、`permissions/*.toml` | 第 3、4 集 |
| CSP | 頁面能載入、能執行什麼 | `tauri.conf.json` 的 `app.security.csp` | 第 5 集 |
| isolation | 每則 IPC 訊息送到 Rust 之前，先過一個你寫的檢查 | `tauri.conf.json` 的 `app.security.pattern` | 第 6 集 |

前三項其實是同一套存取控制清單（ACL，access control list）：capability 決定「誰」，也就是哪些視窗；permission 決定「能呼叫什麼」；scope 決定「碰得到什麼」。

一個 `invoke` 抵達 Rust 時，大致是這樣走：

1. 如果開了 isolation，訊息先送進隔離的 iframe，通過你的檢查才繼續（第 6 集）。
2. Tauri 查 ACL：發出訊息的視窗，有沒有被某個 capability 放行這個 command。沒有，直接回 `not allowed`，command 不會執行。
3. command 開始執行。有 scope 的話，由 command 對照 scope 檢查參數（第 3、4 集）。

CSP 不在這條路上，它管的是頁面本身：能載入哪些資源、能不能執行行內腳本。

第 2 步有個例外，也是這集的重點：plugin 與 core 的 command（`plugin:fs|read_text_file` 這種名字）一定要查；自己寫的 command 要等你替它們定義過 permission（第 4 集）才會查。定義之前，沒人擋。所以前面幾章的 `greet`、`read_text` 才能不用設定就呼叫，而 plugin 的函式沒放行就是會被擋。

### 放行得越少越好

四道防線有一個共同的原則：只給用得到的，也就是最小權限（least privilege）。前端出事時，能做的事只剩你放行的那些。範本的 `capabilities/default.json` 一開頭就有兩項：`core:default` 與 `opener:default`。`opener:default` 是為了範本附的 opener plugin 放的，如果你的 App 用不到，留著就只是多一個缺口。第 2 集會把它拿掉。

### 跟 Electron 比一比

Electron 的頁面現在預設碰不到 Node.js，開發者用 preload 腳本和 `contextBridge` 挑幾個函式交給頁面，沒交出去的，頁面就碰不到。Tauri 反過來：`invoke` 在每個頁面裡都有，註冊進 `generate_handler!` 的 command 預設都能呼叫，要靠 permission 去收緊。所以寫 command 的時候，要把「任何頁面都能拿任何參數呼叫它」當成預設，參數一律當成不可信的輸入。

## 重點整理

- 前端是信不過的一端，Rust 是信得過的一端，界線是 IPC。Rust 端要假設前端可能被 XSS 或有問題的套件控制。
- 四道防線：capability（哪個視窗）、permission 與 scope（能呼叫什麼、碰得到什麼）、CSP（頁面能載入什麼）、isolation（IPC 訊息先過檢查）。
- plugin 與 core 的 command 一定要被放行才能呼叫；自己寫的 command 在你定義 permission 之前，沒人擋，任何頁面都能拿任何參數呼叫。
- 最小權限：用到什麼放行什麼。
