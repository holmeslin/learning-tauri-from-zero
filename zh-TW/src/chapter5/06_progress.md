# 長時間工作回報進度

## 本集目標

用 `Channel` 替一個要跑很久的 command 回報進度，讓兩個工作同時進行也不會混在一起，並學會處理晚到的進度。

## 正文

### 範例：一步 0.3 秒的工作

長時間工作的畫面有兩樣東西：跑到哪了，以及最後的結果。進度走 `Channel`，每做完一步就 `send` 一則 `Progress`；結果走回傳值，函式結束時 `return`，前端的 `invoke` 拿到，成功或失敗都在這裡。`run_job` 做 `total` 步，每步睡 0.3 秒假裝在忙：

`src-tauri/src/lib.rs`：

```rust
use serde::Serialize;
use std::time::Duration;
use tauri::ipc::Channel;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    done: u32,
    total: u32,
}

#[tauri::command]
async fn run_job(total: u32, on_progress: Channel<Progress>) -> Result<String, String> {
    for done in 1..=total {
        tokio::time::sleep(Duration::from_millis(300)).await;
        on_progress
            .send(Progress { done, total })
            .map_err(|error| error.to_string())?;
    }
    Ok(format!("{total} 步全部完成"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_job])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`send` 失敗的錯誤用 `map_err` 轉成字串（第 3 章第 6 集），讓函式能回傳 `Result<String, String>`。`Progress` 只要 `Serialize`，不必 `Clone`。

頁面有兩個按鈕，一個跑 5 步、一個跑 15 步。每按一次就在清單加一行，那一行專屬於這次工作：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>進度</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>進度</h1>
      <div class="row">
        <button id="short-btn" type="button">短工作（5 步）</button>
        <button id="long-btn" type="button">長工作（15 步）</button>
      </div>
      <ul id="job-list"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { Channel, invoke } from "@tauri-apps/api/core";

interface Progress {
  done: number;
  total: number;
}

const jobListEl: HTMLElement | null = document.querySelector("#job-list");
let jobCount = 0;

async function startJob(total: number): Promise<void> {
  if (!jobListEl) {
    return;
  }
  jobCount += 1;
  const jobNo = jobCount;
  const itemEl = document.createElement("li");
  itemEl.textContent = `工作 ${jobNo}：準備中`;
  jobListEl.append(itemEl);

  let finished = false;
  const onProgress = new Channel<Progress>();
  onProgress.onmessage = (progress) => {
    if (!finished) {
      itemEl.textContent = `工作 ${jobNo}：${progress.done} / ${progress.total}`;
    }
  };

  let message: string;
  try {
    message = await invoke<string>("run_job", { total: total, onProgress: onProgress });
  } catch (error) {
    message = `失敗（${error}）`;
  }
  finished = true;
  itemEl.textContent = `工作 ${jobNo}：${message}`;
}

document.querySelector("#short-btn")?.addEventListener("click", () => startJob(5));
document.querySelector("#long-btn")?.addEventListener("click", () => startJob(15));
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.66s
     Running `target/debug/tauri-app`
```

先按「長工作」，馬上再按「短工作」：兩行各自往前走，「工作 2」先到「5 步全部完成」，「工作 1」繼續跑到「15 步全部完成」。每次 `startJob` 都 `new` 一個自己的 `Channel`，`onmessage` 用到的 `itemEl` 與 `jobNo` 是那次呼叫專屬的（第 2 章第 5 集的閉包），所以進度不會串到別行。

### 為什麼要 `finished`

上一集說過，訊息和 `invoke` 的回傳值之間沒有順序保證。沒有 `finished` 的話，萬一最後的「15 / 15」晚到，就會把剛顯示的「15 步全部完成」蓋掉。所以進度只負責中途的數字，結果一出來，就以 `invoke` 的回傳為準，之後到的進度直接忽略。

### 用事件還是 Channel

- 進度、下載的資料這種屬於某一次呼叫的連續資料，用 `Channel`：只有那次呼叫的人收到，順序有保證，不用取消。
- 不屬於哪一次呼叫的通知，像背景執行緒偵測到變化、多個地方都想知道的狀態改變，用事件。
- 用事件回報進度也行，但同時有兩個工作時，進度都在同一個事件名稱底下，得自己在 payload 裡帶工作編號來分辨。

## 重點整理

- 進度走 `Channel`，結果走 `invoke` 的回傳值（成功或失敗）。
- 每次呼叫建自己的 `Channel`，同時進行的工作各收各的進度。
- 最後的進度可能比 `invoke` 的回傳晚到，結果出來後要忽略晚到的進度，免得蓋掉結果。
- 屬於某次呼叫的連續資料用 `Channel`，不屬於哪次呼叫的通知用事件。
