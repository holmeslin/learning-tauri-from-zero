# `Channel`：串流資料

## 本集目標

學會用 `Channel` 讓 command 在執行期間，一則一則把資料傳回前端，並知道它跟事件有什麼不同。

## 正文

### command 只能回答一次

command 的回傳值只有一個：函式結束時 `return` 一次。如果工作要做很久，想一邊做一邊回報（下載到幾 % 了、剛讀到哪一行），就需要另一條路。用事件可以，但事件靠名字配對，所有在聽的人都會收到，也分不出是哪一次呼叫送的。

`Channel` 是專門為此準備的：前端建一個 `Channel`，當作參數傳給 command；Rust 拿到它，可以呼叫任意多次 `send`，每次前端的 `onmessage` 就被呼叫一次。

### 範例：倒數

`count_down` 從 `start` 倒數到 0，每 0.5 秒 `send` 一個數字：

`src-tauri/src/lib.rs`：

```rust
use std::time::Duration;
use tauri::ipc::Channel;

#[tauri::command]
async fn count_down(start: u32, on_tick: Channel<u32>) -> tauri::Result<()> {
    for n in (0..=start).rev() {
        on_tick.send(n)?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![count_down])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `on_tick: Channel<u32>`：`Channel` 在 `tauri::ipc` 裡，`<u32>` 是每一則訊息的型別，要能 `Serialize`（不必 `Clone`）。
- `send(n)` 回傳 `tauri::Result<()>`，所以函式也回傳它，用 `?` 處理。
- 要等 0.5 秒，所以寫成 `async fn` 加 `tokio::time::sleep`（第 3 章第 8 集），同步 command 會卡住主執行緒。

前端按下按鈕時建一個 `Channel`，設好 `onmessage`，再當成參數交給 `invoke`：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Channel</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>Channel</h1>
      <div class="row">
        <button id="start-btn" type="button">開始倒數</button>
      </div>
      <ul id="log"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { Channel, invoke } from "@tauri-apps/api/core";

const logEl: HTMLElement | null = document.querySelector("#log");

function addLog(text: string): void {
  if (logEl) {
    const itemEl = document.createElement("li");
    itemEl.textContent = text;
    logEl.append(itemEl);
  }
}

document.querySelector("#start-btn")?.addEventListener("click", async () => {
  const onTick = new Channel<number>();
  onTick.onmessage = (n) => {
    addLog(`收到 ${n}`);
  };
  await invoke("count_down", { start: 5, onTick: onTick });
  addLog("count_down 結束了");
});
```

`new Channel<number>()` 建出一個 `Channel`，`<number>` 跟 `invoke<T>` 一樣是你對 TypeScript 的承諾，說明每則訊息是數字。要在 `invoke` 之前設好 `onmessage`，先到的訊息不會補送。`invoke` 的參數鍵是 `onTick`，對應 Rust 的 `on_tick`，規則同第 3 章第 3 集。

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.61s
     Running `target/debug/tauri-app`
```

按下按鈕，清單每 0.5 秒多一行，從「收到 5」到「收到 0」，最後是「count_down 結束了」。

### Channel 跟事件的不同

- 配對的方式：事件靠名字，Channel 靠「這次 `invoke` 傳進去的那一個」。只有那次呼叫收得到，兩次呼叫各用各的，不會混在一起。
- 順序：同一個 Channel 的訊息，到前端時的順序一定跟 `send` 的順序一樣。
- 收尾：Channel 不用像 `listen` 那樣自己取消。Rust 端的 Channel 被丟掉（這個範例是 command 結束時），Tauri 會通知前端收尾。

### 別用 `invoke` 的回傳判斷結束

訊息之間有順序，但訊息和 `invoke` 的回傳值之間沒有：兩者走不同的路，Tauri 沒有保證誰先到。所以不能靠 `invoke` 的 Promise 判斷「訊息都收完了」。要知道的話，自己在訊息裡帶記號；下一集的做法則是：進度只管畫面上的數字，結果以 `invoke` 的回傳為準，之後晚到的進度直接忽略。

## 重點整理

- 前端 `new Channel<T>()`、設好 `onmessage`，當成參數交給 `invoke`；Rust 的參數型別是 `Channel<T>`。
- Rust 端 `send(資料)` 一次傳一則，`T` 要 `Serialize`，回傳 `tauri::Result<()>`。
- 同一個 Channel 的訊息保證順序，也只有傳進去的那次呼叫收得到。
- 訊息和 `invoke` 的回傳值之間沒有順序保證，不能用 `invoke` 回來當作訊息都收完了。
