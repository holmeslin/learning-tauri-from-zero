# 回傳 `Result`

## 本集目標

學會讓 command 回傳 `Result`，並在前端用 `try`/`catch` 接住 `Err`，知道 `catch` 拿到的是什麼。

## 正文

### `Ok` 成功，`Err` 拒絕

command 會失敗的時候（檔案打不開、輸入不合法），回傳 `Result<T, E>` 就好，跟平常寫 Rust 一樣。到了前端：

- `Ok(值)`：Promise 成功，`await` 拿到那個值，跟上一集一樣。
- `Err(錯誤)`：Promise 被拒絕（第 2 章第 10 集），`await` 會丟出例外，要用 `try`/`catch` 接。

最簡單的 `E` 是 `String`。我們寫一個把文字轉成年齡的 command，轉不成就回傳錯誤訊息：

`src-tauri/src/lib.rs`：

```rust
#[tauri::command]
fn parse_age(text: &str) -> Result<u8, String> {
    text.trim().parse::<u8>().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![parse_age])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`parse::<u8>()` 的錯誤型別是 `ParseIntError`，用 `map_err` 轉成 `String`，原因下面說。前端是一個輸入框，成功顯示年齡，失敗顯示錯誤：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Result</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>Result</h1>
      <form class="row" id="age-form">
        <input id="age-input" placeholder="輸入年齡" />
        <button type="submit">檢查</button>
      </form>
      <p id="age-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const formEl: HTMLFormElement | null = document.querySelector("#age-form");
const ageInputEl: HTMLInputElement | null = document.querySelector("#age-input");
const ageMsgEl: HTMLElement | null = document.querySelector("#age-msg");

formEl?.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!ageInputEl || !ageMsgEl) {
    return;
  }
  try {
    const age = await invoke<number>("parse_age", { text: ageInputEl.value });
    ageMsgEl.textContent = `年齡是 ${age}`;
  } catch (error) {
    ageMsgEl.textContent = `出錯了：${error}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.90s
     Running `target/debug/tauri-app`
```

輸入 `42`，顯示「年齡是 42」。輸入 `abc`，顯示「出錯了：invalid digit found in string」；輸入 `300`，是「出錯了：number too large to fit in target type」，因為 `u8` 最大是 255。

### `catch` 拿到的是 Rust 放進 `Err` 的東西

`catch (error)` 的 `error`，就是 Rust 的 `Err` 裡那個值，經過 serde 變成的 JSON。`Err(String)` 到前端就是一個字串，不是 `Error` 物件，所以第 2 章第 10 集 `error instanceof Error` 的檢查在這裡永遠是 `false`。範例直接把它放進範本字串（template literal）裡，字串就照原樣印出來。

### 為什麼要 `map_err`

`Err` 裡的型別跟回傳值一樣，要能序列化。如果把回傳型別寫成 `Result<u8, std::num::ParseIntError>`，不轉換直接回傳，編譯會失敗（節錄）：

```text
error[E0599]: the method `blocking_kind` exists for reference `&Result<u8, ParseIntError>`, but its trait bounds were not satisfied
  --> src/lib.rs:1:1
...
   = note: the following trait bounds were not satisfied:
           `ParseIntError: Into<InvokeError>`
           which is required by `Result<u8, ParseIntError>: tauri::ipc::private::ResultKind`
```

`InvokeError` 是 Tauri 裝錯誤用的型別，所有實作 `Serialize` 的型別都能轉進去，`ParseIntError` 不是，所以不行。`map_err(|e| e.to_string())` 把它變成 `String`，就滿足了。下一集會寫一個自己的錯誤型別，不用每次都轉成字串。

## 重點整理

- command 回傳 `Result<T, E>`：`Ok` 讓 Promise 成功，`Err` 讓它被拒絕，前端用 `try`/`catch` 接。
- `E` 要實作 `Serialize`，最簡單的是 `String`，用 `map_err(|e| e.to_string())` 轉。
- `catch` 拿到的是 `Err` 序列化後的值；`Err(String)` 就是字串，不是 `Error` 物件。
- 編譯錯誤裡出現 `Into<InvokeError>`，代表 `Err` 的型別不能序列化。
