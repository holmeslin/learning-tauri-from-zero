# `manage` 與 `State`

## 本集目標

學會用 `manage` 把一個值交給 Tauri 保管，並在 command 的參數裡用 `State` 取用；也要知道忘了 `manage` 時會看到什麼。

## 正文

### 先 `manage`，再 `State`

要讓好幾次 command 呼叫共用同一份資料，那份資料就不能是 command 裡的區域變數。做法是兩個動作，跟第 3 章的 command 一樣是「登記」加「使用」：

1. 在 `Builder` 上呼叫 `.manage(值)`，把值交給 Tauri 保管。這個值就是狀態，整個 App 活著的期間都在。
2. 在 command 的參數裡寫 `State<型別>`。跟第 3 章第 9 集的 `AppHandle` 一樣，這是 Tauri 從呼叫的脈絡幫你填的參數，前端不用傳。

### 範例：唯讀的設定

用範本的 `greet` 改寫：問候語放進狀態，`name` 還是前端傳來的。

`src-tauri/src/lib.rs`：

```rust
use tauri::State;

struct Settings {
    greeting: String,
}

#[tauri::command]
fn greet(name: &str, settings: State<Settings>) -> String {
    format!("{}，{name}！", settings.greeting)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Settings {
            greeting: "你好".to_string(),
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `State<Settings>` 可以直接當 `&Settings` 用，所以寫 `settings.greeting`。
- `State` 只借給你唯讀的參考，改不了裡面的值；怎麼改，下一集說。
- 交給 Tauri 的值一律要是 `Send + Sync + 'static`，因為狀態可能被好幾條執行緒同時讀到：async command 在工作執行緒上跑（第 3 章第 8 集），你自己開的執行緒也拿得到它（第 5 集會看到）。

前端改用 `try`/`catch` 接住失敗（第 3 章第 6 集），下一節會用到：

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    try {
      greetMsgEl.textContent = await invoke("greet", {
        name: greetInputEl.value,
      });
    } catch (error) {
      greetMsgEl.textContent = `出錯了：${error}`;
    }
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.36s
     Running `target/debug/tauri-app`
```

輸入 `Ada` 按 Greet，段落出現「你好，Ada！」。

### 忘了 `manage` 會怎樣

把 `lib.rs` 裡 `.manage(Settings { ... })` 那三行拿掉，其他不動，再執行 `npm run tauri dev`。編譯照樣通過：編譯器不知道你有沒有 `manage`。按 Greet，段落變成：

```text
出錯了：state not managed for field `settings` on command `greet`. You must call `.manage()` before using this command
```

這不是 Rust 端 panic，而是 command 沒有執行就被拒絕，前端的 Promise 被 reject，`catch` 拿到一個字串，跟第 3 章第 6 集的 `Err(String)` 一樣。訊息裡有欄位名（`settings`）與 command 名（`greet`），照著找就是了。看完記得把那三行放回去。

### 一種型別只能管一份

Tauri 是用型別來找狀態的：`State<Settings>` 就是「給我那個 `Settings`」。所以同一種型別只能 `manage` 一次，再來一次，App 一啟動就 panic：

```text
thread 'main' (123456) panicked at ~/.cargo/registry/src/.../tauri-2.12.6/src/app.rs:2060:5:
state for type 'tauri_app_lib::Settings' is already being managed
```

需要兩份同樣是 `String` 的資料，就各包進自己的 struct，讓它們是不同的型別。

## 重點整理

- `Builder::manage(值)` 把值交給 Tauri 保管；command 用參數 `State<型別>` 取用，前端不用傳。
- `State<T>` 可以當 `&T` 用，只能讀，不能改。
- 忘了 `manage` 編譯會過，執行時 command 被拒絕，前端收到 `state not managed for field ...` 的字串。
- 狀態是依型別找的，同一種型別只能 `manage` 一次；要分開就包成不同的 struct。
