# 回傳值與 serde

## 本集目標

知道 command 的回傳值怎麼變成 JSON 送到前端，並學會常見的 Rust 型別在前端各對應到什麼。

## 正文

### serde 與 `Serialize`

command 的回傳值要穿過 IPC，到前端變成 JavaScript 的值，中間需要一種大家都懂的格式：JSON。Rust 把值轉成某種格式叫序列化（serialize），反過來叫反序列化（deserialize）。做這件事的 crate 叫 serde，Tauri 用它搭配 `serde_json`。範本的 `Cargo.toml` 已經有這兩個相依套件，不用另外加。

serde 的核心是 `Serialize` 這個 trait：型別有實作它，就知道自己該怎麼變成 JSON。**command 的回傳型別必須實作 `Serialize`**。`String`、數字、`Vec<T>` 這些標準函式庫的型別都已經實作好了，自己定義的型別要靠 `#[derive(Serialize)]`，第 5 集會用到。

### 四種常見的回傳型別

下面四個 command，各回傳一種型別。`fruits` 與 `fruit_at` 要共用同一份資料，所以抽成一個普通函式 `fruit_list`，不讓 command 直接呼叫另一個 command：

`src-tauri/src/lib.rs`：

```rust
fn fruit_list() -> Vec<String> {
    vec!["apple".to_string(), "banana".to_string()]
}

#[tauri::command]
fn fruits() -> Vec<String> {
    fruit_list()
}

#[tauri::command]
fn fruit_at(index: usize) -> Option<String> {
    fruit_list().into_iter().nth(index)
}

#[tauri::command]
fn prices() -> std::collections::BTreeMap<String, u32> {
    std::collections::BTreeMap::from([("apple".to_string(), 30), ("banana".to_string(), 20)])
}

#[tauri::command]
fn no_value() {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![fruits, fruit_at, prices, no_value])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

前端按一下按鈕，用 `for...of` 逐一走過四個結果（第 2 章第 6 集），每個結果轉成 JSON 文字，一行一個印出來。`JSON.stringify(值)` 是瀏覽器內建的函式，把任何值轉成 JSON 文字，這裡只是為了把收到的值原樣印出來看，照抄就好：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>回傳值</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>回傳值</h1>
      <button id="run-btn" type="button">呼叫四個 command</button>
      <pre id="results"></pre>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

interface Prices {
  apple: number;
  banana: number;
}

const runBtnEl: HTMLButtonElement | null = document.querySelector("#run-btn");
const resultsEl: HTMLElement | null = document.querySelector("#results");

async function runAll(): Promise<void> {
  const fruits = await invoke<string[]>("fruits");
  const found = await invoke<string | null>("fruit_at", { index: 5 });
  const prices = await invoke<Prices>("prices");
  const noValueResult = await invoke<null>("no_value");
  if (resultsEl) {
    let text = "";
    for (const value of [fruits, found, prices, noValueResult]) {
      text += `${JSON.stringify(value)}\n`;
    }
    resultsEl.textContent = text;
  }
}

runBtnEl?.addEventListener("click", runAll);
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.36s
     Running `target/debug/tauri-app`
```

按下按鈕，頁面印出：

```text
["apple","banana"]
null
{"apple":30,"banana":20}
null
```

### Rust 型別對照

| Rust 型別 | JSON | `invoke<T>` 的 `T` |
| --- | --- | --- |
| `String` | 字串 | `string` |
| 整數、浮點數 | 數字 | `number` |
| `bool` | 布林 | `boolean` |
| `Vec<T>`、陣列 | 陣列 | `T[]` |
| `Option<T>` | `None` 是 `null`（不是 `undefined`），`Some(x)` 是 `x` | `T \| null`（第 2 章第 6 集），用之前先判斷 |
| `(A, B)`（tuple） | 兩個元素的陣列 | `[A, B]`（恰好兩個元素的陣列） |
| `BTreeMap<String, V>`、`HashMap<String, V>` | 物件 | 用 `interface` 寫出欄位，如 `Prices`；鍵不固定時寫 `Record<string, V>`，照抄即可 |
| `()` | `null` | `null` |

數字有一個限制：JavaScript 的 `number` 只能精確表示到 2 的 53 次方左右。Rust 的 `u64` 或 `i64` 超過這個範圍，前端會失去精確度，例如 Rust 回傳 `9007199254740993`，前端收到 `9007199254740992`。這種數字（像資料庫的 ID）要改用字串傳。

### 跟 Electron 比一比

Electron 在行程之間傳資料，用的是 HTML 標準的結構化複製演算法（Structured Clone Algorithm），所以 `Date`、`Map`、`Set` 這類物件可以直接傳。Tauri 傳的是 JSON，沒有這些型別：日期要先轉成字串或時間戳記，`Map` 要改成物件或陣列，才傳得過去。

## 重點整理

- 回傳值經過 serde 變成 JSON，所以 command 的回傳型別必須實作 `Serialize`；標準函式庫的常見型別都已經實作。
- `Vec<T>` 對應陣列、`Option<T>` 對應 `T | null`、tuple 對應固定長度的陣列、map 對應物件、`()` 對應 `null`；`invoke<T>` 的 `T` 要自己對照型別表寫。
- JSON 有它的限制：超過 2 的 53 次方的整數在前端會失真，改用字串傳；也傳不了 `Date`、`Map`，不像 Electron 的結構化複製。
