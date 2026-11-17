# 前端的 `invoke`

## 本集目標

學會在前端用 `invoke` 呼叫 command，拿到回傳值，並看懂 `invoke<string>` 裡的 `<string>` 是做什麼的。

## 正文

### `invoke` 回傳 Promise

前端這一側的入口只有一個函式 `invoke`，從 `@tauri-apps/api/core` 匯入（`import` 在第 2 章第 11 集解釋過）。它的第一個參數是 command 的名字，字串，要跟 Rust 函式的名字一模一樣。第二個參數是要傳給 command 的資料，下一集再說。

前端和 Rust 是不同的行程（第 1 章），請 Rust 做事不會立刻有結果，所以 `invoke` 回傳的是 `Promise`，用 `await` 等它（第 2 章第 10 集）。

### 第一次呼叫

我們沿用上一集的 `say_hello`。Rust 這一側不用改：

`src-tauri/src/lib.rs`：

```rust
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn say_hello() -> String {
    "哈囉，我是 Rust".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, say_hello])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

頁面放一個按鈕和一個放結果的段落：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>invoke</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>invoke</h1>
      <div class="row">
        <button id="hello-btn" type="button">呼叫 say_hello</button>
      </div>
      <p id="hello-msg"></p>
    </main>
  </body>
</html>
```

照直覺寫的第一版前端：

`src/main.ts`（第一版）：

```ts
import { invoke } from "@tauri-apps/api/core";

const helloBtnEl: HTMLButtonElement | null = document.querySelector("#hello-btn");
const helloMsgEl: HTMLElement | null = document.querySelector("#hello-msg");

async function callHello(): Promise<void> {
  const message = await invoke("say_hello");
  if (helloMsgEl) {
    helloMsgEl.textContent = message;
  }
}

helloBtnEl?.addEventListener("click", callHello);
```

`npx tsc --noEmit` 的輸出：

```text
src/main.ts:9:5 - error TS2322: Type 'unknown' is not assignable to type 'string | null'.

9     helloMsgEl.textContent = message;
      ~~~~~~~~~~~~~~~~~~~~~~


Found 1 error in src/main.ts:9
```

### `invoke<string>`：告訴 TypeScript 會收到什麼

`invoke` 只拿到一個名字字串，TypeScript 沒辦法知道那個 command 會回傳什麼，所以預設把結果當成 `unknown`（第 2 章第 4 集）。`unknown` 不能直接放進 `textContent`，於是 `tsc` 報錯。（範本的 `greetMsgEl.textContent = await invoke(...)` 沒寫 `<string>` 也能過，是因為結果直接賦值給 `textContent`，TypeScript 從 `textContent` 要的型別反推出 `T`；先存進變數，就沒有這條線索了。）

解法是在名字後面用角括號，說出你預期的型別：

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const helloBtnEl: HTMLButtonElement | null = document.querySelector("#hello-btn");
const helloMsgEl: HTMLElement | null = document.querySelector("#hello-msg");

async function callHello(): Promise<void> {
  const message = await invoke<string>("say_hello");
  if (helloMsgEl) {
    helloMsgEl.textContent = message;
  }
}

helloBtnEl?.addEventListener("click", callHello);
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.51s
     Running `target/debug/tauri-app`
```

按下按鈕，段落出現「哈囉，我是 Rust」。

`<string>` 是你對 TypeScript 的承諾，不是檢查。`tsc` 看不到 Rust 的程式，也不會驗證 Rust 真的回傳字串，它只拿你寫的 `string` 去檢查後面的程式。如果 Rust 那邊把回傳型別改了而前端沒跟著改，`tsc` 照樣通過，不一致要到執行時才會出現。所以 `<...>` 裡的型別要自己對著 Rust 函式的回傳型別寫；每種 Rust 型別對應到什麼，第 4 集會整理。

### 呼叫失敗

找不到 command，或 command 回傳錯誤時，Promise 會被拒絕（reject）。怎麼接，第 6 集再說。

## 重點整理

- `import { invoke } from "@tauri-apps/api/core"`；`invoke("名字")` 呼叫同名的 command。
- `invoke` 回傳 `Promise`，用 `await` 等結果。
- `invoke<T>("名字")` 的 `T` 是你預期的回傳型別；不寫，結果是 `unknown`，多半無法直接使用。
- `T` 只給 `tsc` 看，執行時不會驗證，要自己跟 Rust 的回傳型別對上。
- 呼叫失敗時 Promise 會被拒絕，第 6 集再接。
