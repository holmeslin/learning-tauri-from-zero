# 自訂錯誤型別（`thiserror` 加 `Serialize`）

## 本集目標

用 `thiserror` 定義自己的錯誤型別，替它實作 `Serialize`，讓 command 能用 `?` 直接回傳各種錯誤。

## 正文

### 為什麼要自己的錯誤型別

command 的錯誤來源通常不只一種：文字轉數字失敗、讀檔失敗、資料不合理。每一種都 `map_err(|e| e.to_string())`，很快就煩了，也看不出錯在哪一類。Rust 的慣例是定義一個 enum，每個變體代表一類錯誤，再用 `?` 自動轉換。

這裡用 `thiserror`，一個替錯誤型別省掉樣板程式的 crate。它的 `#[derive(thiserror::Error)]` 會幫你實作 `std::error::Error` 與 `Display`，`#[error("...")]` 指定錯誤訊息，`#[from]` 順便實作 `From`，`?` 就能自動轉換。範本沒有它，要先在專案裡加上：

```bash
cd src-tauri
cargo add thiserror@2
```

### 範例：檢查年齡

延續上一集，多加一種錯誤：年齡超過 150 也不合理。

`src-tauri/src/lib.rs`：

```rust
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("不是數字：{0}")]
    NotANumber(#[from] std::num::ParseIntError),
    #[error("年齡 {0} 不在 0 到 150 之間")]
    OutOfRange(u8),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
fn parse_age(text: &str) -> Result<u8, AppError> {
    let age: u8 = text.trim().parse()?;
    if age > 150 {
        return Err(AppError::OutOfRange(age));
    }
    Ok(age)
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

兩個變體各有一句訊息。`text.trim().parse()?` 裡，`ParseIntError` 靠 `#[from]` 自動變成 `AppError::NotANumber`，不用 `map_err`。

### 替它實作 `Serialize`

上一集說過，`Err` 的型別要能序列化。`AppError` 沒辦法 `#[derive(Serialize)]`，因為 `NotANumber` 裡面的 `ParseIntError` 不是 `Serialize`。這裡手動實作：送給前端的就是 `to_string()` 的結果，也就是 `#[error(...)]` 寫的那句訊息，序列化成一個字串。`serialize_str` 是 serde 的 `Serializer` 提供的方法。

前端不用改，仍然是上一集的頁面：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>自訂錯誤</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>自訂錯誤</h1>
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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.70s
     Running `target/debug/tauri-app`
```

輸入 `abc`，顯示「出錯了：不是數字：invalid digit found in string」；輸入 `200`，顯示「出錯了：年齡 200 不在 0 到 150 之間」。前端拿到的仍然是字串，只是現在由 Rust 的錯誤型別決定內容。`Serialize` 送出什麼，前端就收到什麼：想讓前端依錯誤種類做不同處理，也可以送出物件而不是字串。

## 重點整理

- 用 enum 表示 command 的各類錯誤；`thiserror` 的 `#[derive(thiserror::Error)]` 提供 `Error` 與 `Display`，`#[error("...")]` 寫訊息，`#[from]` 讓 `?` 自動轉換。
- 範本沒有 `thiserror`，要用 `cargo add thiserror@2` 加。
- `Err` 的型別要實作 `Serialize`。含有不能序列化的欄位時，手動實作，送出 `to_string()` 的字串最簡單。
- 前端 `catch` 拿到的是那個字串。
