# 用 `Mutex` 包住可變狀態

## 本集目標

讓狀態可以修改：把它包進 `std::sync::Mutex`，在 command 裡上鎖、修改，並看到這份狀態是跟著 Rust 端活著的。

## 正文

### `State` 只借你唯讀

寫一個計數器，每次 `increment` 把數字加一。照直覺，直接改 `State` 裡的欄位：

`src-tauri/src/lib.rs`：

```rust
use tauri::State;

struct Counter {
    count: u32,
}

#[tauri::command]
fn increment(counter: State<Counter>) -> u32 {
    counter.count += 1;
    counter.count
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Counter { count: 0 })
        .invoke_handler(tauri::generate_handler![increment])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
error[E0594]: cannot assign to data in dereference of `tauri::State<'_, Counter>`
 --> src/lib.rs:9:5
  |
9 |     counter.count += 1;
  |     ^^^^^^^^^^^^^^^^^^ cannot assign
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `tauri::State<'_, Counter>`
```

`State` 只能當 `&Counter` 用，沒有 `DerefMut`。這是故意的：所有 command 共用同一份狀態，它們可能同時在不同執行緒上跑，Tauri 不能把 `&mut` 發給每一個人。

### 包進 `Mutex`

這個問題你在 Rust 裡見過：多執行緒共用、又要修改，就用 `Mutex`。把 `Counter` 包起來，`manage` 與 `State` 的型別都跟著改成 `Mutex<Counter>`：

`src-tauri/src/lib.rs`：

```rust
use std::sync::Mutex;
use tauri::State;

struct Counter {
    count: u32,
}

#[tauri::command]
fn increment(state: State<Mutex<Counter>>) -> u32 {
    let mut counter = state.lock().unwrap();
    counter.count += 1;
    counter.count
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(Counter { count: 0 }))
        .invoke_handler(tauri::generate_handler![increment])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `lock()` 回傳 `Result`，只有在別的執行緒拿著鎖 panic（鎖「中毒」了）時才會是 `Err`，範例直接 `unwrap()`。
- 鎖的守衛（guard）離開作用域就自動解鎖，`increment` 的 `counter` 在函式結束時放掉；上鎖的時間越短越好。

前端放一行文字和一個按鈕：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Mutex</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>Mutex</h1>
      <p id="count">還沒按過</p>
      <div class="row">
        <button id="inc-btn" type="button">加一</button>
      </div>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const countEl: HTMLElement | null = document.querySelector("#count");
const incBtnEl: HTMLButtonElement | null = document.querySelector("#inc-btn");

incBtnEl?.addEventListener("click", async () => {
  const count = await invoke<number>("increment");
  if (countEl) {
    countEl.textContent = `目前的計數：${count}`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.52s
     Running `target/debug/tauri-app`
```

每按一次「加一」，文字就變成「目前的計數：1」「目前的計數：2」……這個計數不在網頁裡，在 Rust 的行程裡：多按幾次之後，改一下 `src/main.ts` 存檔，頁面重新載入，文字回到「還沒按過」，再按一次卻是接著算，不是從 1 重來。整個 App 重啟（例如改了 Rust 程式，第 1 章第 5 集），`Mutex::new(Counter { count: 0 })` 才會重新執行，計數才歸零。

### 跟 Electron 比一比

Electron 的主行程跑 JavaScript，只有一條執行緒，在模組最上層寫一個 `let count = 0`，每個處理函式都能直接 `count += 1`。Tauri 的 command 可能同時在多條執行緒上跑，Rust 不讓你隨手共用可變的資料，要用 `Mutex` 明確上鎖。

## 重點整理

- `State<T>` 只借出 `&T`，不能直接修改；要修改就把值包進 `std::sync::Mutex`：`manage(Mutex::new(值))`、參數寫 `State<Mutex<T>>`，用 `lock().unwrap()` 拿到可修改的守衛。
- 守衛離開作用域就解鎖，上鎖的時間越短越好。
- 狀態活在 Rust 行程裡，重新載入頁面不影響它；App 重啟才會歸零。
