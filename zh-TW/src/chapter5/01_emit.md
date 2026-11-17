# Rust 送事件給前端：`emit`

## 本集目標

學會在 Rust 用 `emit` 主動送出事件，讓前端的 `listen` 收到，並知道事件名稱與 payload 的規則。

## 正文

### 事件：不用問，直接說

command 是前端問、Rust 答，呼叫一定由前端發起。可是有些訊息該由 Rust 先開口：時間到了、背景工作做完了、檔案變了。這時用事件（event）：

- 事件有一個名字（例如 `tick`），可以帶一份資料，叫 payload。
- 送出去就算了，不等回覆。
- Rust 隨時都能送，不必等前端先問。

這一集看 Rust 怎麼送。前端要收得用 `listen`，這集只借最少的寫法，下一集再細講。

### 範例：每秒送一次 `tick`

延續第 4 章第 5 集：在 `setup` 裡開一條背景執行緒，每秒一次。這次不改狀態，而是把經過的秒數送給前端。payload 寫成一個 struct，規則跟第 3 章第 5 集一樣：`Serialize` 讓它變成 JSON，`rename_all` 讓欄位在前端是 `camelCase`。另外多了 `Clone`，原因見本集最後。

`src-tauri/src/lib.rs`：

```rust
use serde::Serialize;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Tick {
    elapsed_seconds: u32,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            thread::spawn(move || {
                let mut elapsed_seconds: u32 = 0;
                loop {
                    thread::sleep(Duration::from_secs(1));
                    elapsed_seconds += 1;
                    handle.emit("tick", Tick { elapsed_seconds }).unwrap();
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `handle.emit("tick", payload)`：第一個參數是事件名稱，第二個是 payload，回傳 `tauri::Result<()>`。
- `emit` 定義在 `Emitter` trait 上，要先 `use tauri::Emitter`，道理跟第 4 章第 4 集的 `Manager` 一樣。
- 這條執行緒用的是 `AppHandle` 的複本，跟第 4 章第 5 集的做法相同。

### 前端：先借用 `listen`

頁面只要一個放訊息的段落：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>emit</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>emit</h1>
      <p id="tick-msg">等 Rust 的第一個事件…</p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { listen } from "@tauri-apps/api/event";

interface Tick {
  elapsedSeconds: number;
}

const tickMsgEl: HTMLElement | null = document.querySelector("#tick-msg");

listen<Tick>("tick", (event) => {
  if (tickMsgEl) {
    tickMsgEl.textContent = `Rust 說：已經過了 ${event.payload.elapsedSeconds} 秒`;
  }
});
```

`listen("tick", 函式)` 跟 `addEventListener`（第 2 章第 8 集）是同一個思路：`tick` 事件每來一次就呼叫一次函式，`event.payload` 就是 Rust 送的資料。`<Tick>` 跟 `invoke<T>`（第 3 章第 2 集）一樣，是你對 TypeScript 的承諾。`listen` 會回傳 Promise，這裡先不理它，下一集說明。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.52s
     Running `target/debug/tauri-app`
```

視窗出現「Rust 說：已經過了 N 秒」，數字每秒變大一次。前端沒問過任何事，是 Rust 主動推的。

### payload 與名稱的規則

- payload 的型別要同時是 `Serialize` 與 `Clone`（`emit` 的簽名是 `emit<S: Serialize + Clone>`）。數字、字串本來就都有；自己的 struct 少了 `Clone`，編譯器會說 `the trait bound `Tick: Clone` is not satisfied`，並建議加上 `#[derive(Clone)]`。
- 事件名稱只能用英數字、`-`、`/`、`:`、`_`。像 `tick.second` 有句點，`emit` 會回傳 `Err(IllegalEventName("tick.second"))`；範例的 `unwrap()` 遇到它，這條執行緒就 panic 了。
- 沒有人在聽時，`emit` 一樣回傳 `Ok(())`，不會報錯。
- `AppHandle` 之外，`App` 與視窗也能 `emit`，用法一樣。視窗第 6 章會正式介紹。
- 前端的 `listen` 需要 `core:event` 這組 permission。範本 `capabilities/default.json` 裡的 `core:default` 已經包含，不用動；permission 第 7 章會正式介紹。

## 重點整理

- 事件是有名字、可帶 payload 的訊息；Rust 可以隨時主動送，不等回覆。
- `use tauri::Emitter` 之後，用 `handle.emit("名稱", payload)` 送出，回傳 `tauri::Result<()>`。
- payload 要 `Serialize + Clone`；少了 `Clone` 編譯不過。
- 事件名稱只能用英數字與 `-`、`/`、`:`、`_`，否則 `emit` 回傳 `Err`。
- 前端用 `listen("名稱", 函式)` 收，`event.payload` 是資料；沒人在聽也不會出錯。
