# 讀懂範本的 `greet`

## 本集目標

讀懂「按下 Greet，畫面出現一句話」經過的每個檔案，知道前端和 Rust 各做了哪一步。這集只讀、不寫，寫法留到第 2、3 章。

## 正文

### 整條路線

在輸入框打名字、按下 Greet 之後，發生的事照順序是這樣：

1. 表單送出（`index.html`）。
2. 前端程式取出輸入框的值（`src/main.ts`）。
3. 前端用 `invoke` 對 Rust 說：請執行 `greet`，參數 `name` 是這個值（`src/main.ts`）。
4. Rust 的 `greet` 函式組出一句話，回傳（`src-tauri/src/lib.rs`）。
5. 回傳值傳回前端，寫進頁面（`src/main.ts`）。

### 起點：`index.html`

頁面裡跟 Greet 有關的只有這幾行：

```html
<form class="row" id="greet-form">
  <input id="greet-input" placeholder="Enter a name..." />
  <button type="submit">Greet</button>
</form>
<p id="greet-msg"></p>
```

三個 `id` 是之後 TypeScript 找到這些元素的依據：輸入框、表單，以及一開始是空的、等著放結果的段落。按下按鈕會讓表單送出（submit）。HTML 的元素、屬性和表單會在第 2 章第 2、9 集教。

### 前端：`src/main.ts`

`src/main.ts`（範本原本的內容）：

```ts
import { invoke } from "@tauri-apps/api/core";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
```

先看形狀就好，語法之後會一一補上：

- 第一行從 Tauri 的 JavaScript 套件拿到 `invoke`，這是前端呼叫 Rust 的入口。
- `window.addEventListener("DOMContentLoaded", ...)` 在頁面載好之後，用 `querySelector` 找出輸入框和結果段落，再註冊一個「表單送出時就執行 `greet()`」的處理函式。`e.preventDefault()` 是擋掉瀏覽器預設的送出行為。這幾個用法在第 2 章第 7 到 9 集。
- `greet()` 裡的 `await invoke("greet", { name: ... })`：呼叫 Rust 端叫 `greet` 的 command，把輸入框的值當成 `name` 傳過去，等結果回來，放進段落的 `textContent`。`async`、`await` 在第 2 章第 10 集，`invoke` 的細節在第 3 章第 2 集。
- `HTMLInputElement | null` 是 TypeScript 的型別寫法，第 2 章第 4 到 6 集會教。

### 後端：`src-tauri/src/lib.rs`

`src-tauri/src/lib.rs`（範本原本的內容）：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

這段你幾乎都看得懂：

- `greet` 是一個普通的 Rust 函式，收一個 `&str`，回傳 `String`。唯一多出來的是上面的 `#[tauri::command]`，它讓前端能夠呼叫這個函式。前端傳來的 `name` 就是參數，回傳值會送回前端。第 3 章第 1 集會正式介紹。
- `run()` 把 App 組起來：`tauri::Builder::default()` 開始建構，`.plugin(tauri_plugin_opener::init())` 裝上 opener plugin（用系統預設的程式開網址或檔案，第 7 章），`.invoke_handler(tauri::generate_handler![greet])` 是登記 command 的地方，第 3 章第 1 集會講。
- `.run(tauri::generate_context!())` 把上一集看的 `tauri.conf.json` 讀進來，啟動 App。
- `#[cfg_attr(mobile, tauri::mobile_entry_point)]` 是行動裝置的入口，第 18 章才用到。

### 兩邊怎麼對起來

前端的 `"greet"` 對上 Rust 的函式名稱 `greet`，傳過去的 `{ name: ... }` 對上 Rust 的參數 `name`。名字對不上，呼叫就會失敗。參數名稱的大小寫規則留到第 3 章第 3 集。

## 重點整理

- 一次 Greet 的路線：表單送出、`main.ts` 取值、`invoke` 呼叫、Rust 的 `greet` 回傳字串、`main.ts` 寫回頁面。
- 前端用 `invoke` 呼叫 Rust，Rust 端是加了 `#[tauri::command]` 的函式。兩邊靠名稱對起來：`"greet"` 對函式名稱，`{ name }` 對參數。
- `run()` 負責組裝 App：plugin、command、`tauri.conf.json`。
- 看不懂的前端語法在第 2 章，command 與 `invoke` 的寫法在第 3 章。
