# 傳參數：`snake_case` 與 `camelCase`

## 本集目標

學會把參數從前端傳給 command，並記住一條規則：Rust 的 `snake_case` 參數名稱，在前端要寫成 `camelCase`。

## 正文

### 範例：問候全名

這一集的 command 收兩個參數，姓和名，組成一句問候：

`src-tauri/src/lib.rs`：

```rust
#[tauri::command]
fn full_greeting(family_name: &str, given_name: &str) -> String {
    format!("你好，{family_name}{given_name}！")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![full_greeting])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>傳參數</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>傳參數</h1>
      <form class="row" id="greet-form">
        <input id="family-input" placeholder="姓" />
        <input id="given-input" placeholder="名" />
        <button type="submit">問候</button>
      </form>
      <p id="greet-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const formEl: HTMLFormElement | null = document.querySelector("#greet-form");
const familyInputEl: HTMLInputElement | null = document.querySelector("#family-input");
const givenInputEl: HTMLInputElement | null = document.querySelector("#given-input");
const greetMsgEl: HTMLElement | null = document.querySelector("#greet-msg");

formEl?.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (familyInputEl && givenInputEl && greetMsgEl) {
    greetMsgEl.textContent = await invoke<string>("full_greeting", {
      familyName: familyInputEl.value,
      givenName: givenInputEl.value,
    });
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.42s
     Running `target/debug/tauri-app`
```

在「姓」打 `王`、「名」打 `小明`，按「問候」，下面出現「你好，王小明！」。

### 一個物件，鍵對參數

`invoke` 的第二個參數是一個物件，物件的每個**鍵**（key）對上 Rust 函式的一個參數。上面 `full_greeting` 有兩個參數，所以物件有兩個鍵。對應靠名字，跟順序無關。

名字有一個轉換：Rust 慣例是 `snake_case`，JavaScript 慣例是 `camelCase`，Tauri 預設把前者換成後者，所以 `family_name` 對應 `familyName`，`given_name` 對應 `givenName`。只有參數名稱會轉換，command 本身的名字不會（上一集）。Rust 參數的型別如果是 `Option<T>`，前端可以整個不傳那個鍵，Rust 收到 `None`。

### 寫錯的時候

照 Rust 的習慣，把鍵寫成 `snake_case`：

```ts,ignore
invoke<string>("full_greeting", { family_name: "王", given_name: "小明" });
```

Promise 會被拒絕，Console 看到的錯誤訊息是：

```text
invalid args `familyName` for command `full_greeting`: command full_greeting missing required key familyName
```

訊息裡出現的是轉換後的 `familyName`，也就是 Tauri 期待的鍵。看到 `missing required key`，先對照這個名字。

另一種常見的錯是型別不對，例如 `givenName` 傳了數字 `42`：

```ts,ignore
invoke<string>("full_greeting", { familyName: "王", givenName: 42 });
```

```text
invalid args `givenName` for command `full_greeting`: invalid type: integer `42`, expected a borrowed string
```

前端傳來的資料，Tauri 會用 serde 反序列化（deserialize）成 Rust 參數的型別，型別對不上就失敗；serde 在第 4 集介紹。要注意 `input.value` 一律是字串，直接傳給數字型別的參數就是上面這種錯；這種情況讓 Rust 收 `&str`、自己 `parse`，第 6 集就是這樣做的。

### 不想轉換：`rename_all`

想讓某個 command 的鍵跟 Rust 的參數名稱一模一樣，可以寫 `#[tauri::command(rename_all = "snake_case")]`，這樣前端反而要寫 `snake_case`。除非有理由，建議維持預設，整個專案的前端才一致。

## 重點整理

- `invoke("名字", { 鍵: 值 })`，物件的鍵對上 Rust 的參數，順序不重要。
- Rust 的 `snake_case` 參數預設對應前端的 `camelCase` 鍵（可用 `rename_all` 改，但建議維持預設）；command 的名字不轉換。
- 鍵寫錯，錯誤是 `missing required key …`；型別不對，是 `invalid type …`。兩種都是 Promise 被拒絕。
- `Option<T>` 參數可以不傳。
