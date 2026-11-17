# 只送給某個視窗

## 本集目標

學會用 `emit_to` 指定事件的接收者，並知道「收」的一方也有自己的目標，沒指定就什麼都收。

## 正文

### 不是每個事件都要給所有人

`emit` 送給所有接收者。一個 App 如果有好幾個視窗，常常只想通知其中一個。多視窗第 6 章會正式介紹，這一集只用範本唯一的視窗：它的 label 是 `main`（第 1 章第 6 集）。`emit_to(目標, 事件名稱, payload)` 就是用 label 指定接收者：

`src-tauri/src/lib.rs`：

```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn send_notice(app: AppHandle, target: Option<String>) -> tauri::Result<()> {
    match target {
        Some(label) => {
            let text = format!("只送給 {label}");
            app.emit_to(label, "notice", text)
        }
        None => app.emit("notice", "送給所有人"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_notice])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`target` 是 `Option<String>`（第 3 章第 3 集）：前端沒傳就 `emit`，傳了就 `emit_to`，兩邊的 payload 都是字串。`emit_to` 的第一個參數放字串，意思是「label 等於這個字串的視窗」。沒有任何視窗叫這個名字也不會報錯，只是沒人收到。`emit`、`emit_to` 回傳 `tauri::Result<()>`，這裡直接當成 command 的回傳值（第 3 章第 6 集），`tauri::Error` 本來就能 `Serialize`。

前端放兩個 `notice` 的監聽，一個照舊，一個用第三個參數指定 `target`（用法在後面說明），再加三個按鈕：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>emit_to</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>emit_to</h1>
      <div class="row">
        <button id="all-btn" type="button">所有人</button>
        <button id="main-btn" type="button">main</button>
        <button id="report-btn" type="button">report</button>
      </div>
      <ul id="log"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const logEl: HTMLElement | null = document.querySelector("#log");

function addLog(text: string): void {
  if (logEl) {
    const itemEl = document.createElement("li");
    itemEl.textContent = text;
    logEl.append(itemEl);
  }
}

listen<string>("notice", (event) => {
  addLog(`沒指定 target：${event.payload}`);
});

listen<string>(
  "notice",
  (event) => {
    addLog(`target 是 main：${event.payload}`);
  },
  { target: "main" },
);

document.querySelector("#all-btn")?.addEventListener("click", () => {
  invoke("send_notice");
});

document.querySelector("#main-btn")?.addEventListener("click", () => {
  invoke("send_notice", { target: "main" });
});

document.querySelector("#report-btn")?.addEventListener("click", () => {
  invoke("send_notice", { target: "report" });
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.55s
     Running `target/debug/tauri-app`
```

### 結果：「report」也收得到

分別按三個按鈕：

- 「所有人」：兩個監聽都收到。
- 「main」：兩個監聽都收到。
- 「report」：沒指定 `target` 的那個**還是收到了**，只有 `target` 是 `main` 的沒收到。

原因是收的一方也有目標。`listen` 沒指定 `target`，就是「任何目標」，什麼事件都收，`emit_to` 擋不住它。指定了 `target: "main"`，才只收送給 label 是 `main` 的、或沒有指定目標的 `emit`：

| Rust 的送法 | 沒指定 `target` | `target: "main"` |
| --- | --- | --- |
| `emit` | 收到 | 收到 |
| `emit_to("main", …)` | 收到 | 收到 |
| `emit_to("report", …)` | 收到 | 沒收到 |

所以想讓事件真的只到某個視窗，兩邊要配合：Rust 用 `emit_to` 指定，前端的 `listen` 也要指定 `target`。視窗物件自己的 `listen` 也一樣，目標就是那個視窗：收得到 `emit` 與送給它的 `emit_to`，規則跟表格最右欄相同。

### Rust 端也一樣

第 3 集的 `app.listen` 目標是 App 本身，所以收得到 `emit`，收不到 `emit_to("main", …)`。

### 跟 Electron 比一比

Electron 的 `win.webContents.send(...)` 本來就只送給那一個視窗。Tauri 的 `emit` 是送給所有人，要縮小範圍得用 `emit_to`，而且收的一方沒指定 `target` 的話，照樣什麼都收：只靠送的一方，擋不住。

## 重點整理

- `emit_to(目標, 事件名稱, payload)` 指定接收者；目標是字串時，代表 label 等於它的視窗，沒有這個視窗也不會報錯。
- `listen` 也有目標：沒指定 `target` 就是什麼都收，包括送給別的 label 的。
- 要讓事件只到某個視窗，Rust 用 `emit_to`，前端 `listen` 加上 `{ target: "label" }`。
- Rust 的 `app.listen` 目標是 App：收得到 `emit`，收不到送給視窗 label 的 `emit_to`。
