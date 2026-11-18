# `tauri::test` 的 mock runtime

## 本集目標

用 `tauri::test` 在不開視窗的情況下，把一份 IPC 請求送給 command，檢查回傳值、狀態與錯誤訊息。

## 正文

### 不開視窗，也能「呼叫」command

上一集留下的缺口是接線：command 有沒有登記、狀態有沒有 `manage`、參數的鍵對不對。Tauri 在 `tauri::test` 提供一個假的執行環境，叫 mock runtime：不建立真的視窗與 WebView，`Builder`、`manage`、`invoke_handler` 卻照常運作。測試拿到一個假的視窗，把一份請求交給它，得到的回應跟前端 `invoke` 收到的一樣。官方把這個模組標為不穩定（unstable），之後 API 可能調整。

### 打開 `test` feature

`tauri::test` 預設不編進來，要打開 `test` feature。放在 `[dev-dependencies]`，只有建置測試時才會用到：

```bash
cargo add --manifest-path src-tauri/Cargo.toml --dev tauri@2 --features test
```

它在 `src-tauri/Cargo.toml` 多加一段 `[dev-dependencies]`，內容是 `tauri = { version = "2", features = ["test"] }`。`cargo test` 與 `cargo clippy --all-targets` 會建置測試，才打開這個 feature；`npm run tauri dev` 與 `npm run tauri build` 不會，測試用的程式不會進 App。

### 先把組裝抽成函式

測試要用 mock runtime 組出跟真的 App 一樣的 `Builder`，所以 `manage` 與 `invoke_handler` 抽成函式 `configure`：收進任何 runtime 的 `Builder<R>`，加上設定再還回去。`run()` 傳入預設的，測試傳入 `mock_builder()`。plugin 留在 `run()`，測試只管自己的 command 與狀態。`add` 跟上一集一樣，它的單元測試這裡省略：

`src-tauri/src/lib.rs`：

```rust
use std::sync::Mutex;
use tauri::State;

fn add(items: &mut Vec<String>, text: &str) -> Result<usize, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("項目不能是空的".to_string());
    }
    items.push(text.to_string());
    Ok(items.len())
}

#[tauri::command]
fn add_item(item_text: &str, state: State<Mutex<Vec<String>>>) -> Result<usize, String> {
    add(&mut state.lock().unwrap(), item_text)
}

fn configure<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .manage(Mutex::new(Vec::<String>::new()))
        .invoke_handler(tauri::generate_handler![add_item])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    configure(tauri::Builder::default())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{
        INVOKE_KEY, MockRuntime, assert_ipc_response, get_ipc_response, mock_builder, mock_context,
        noop_assets,
    };
    use tauri::webview::InvokeRequest;
    use tauri::{WebviewWindow, WebviewWindowBuilder};

    fn mock_window() -> WebviewWindow<MockRuntime> {
        let app = configure(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    }

    fn request(cmd: &str, body: Value) -> InvokeRequest {
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: if cfg!(any(windows, target_os = "android")) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    #[test]
    fn add_item_keeps_state_between_calls() {
        let window = mock_window();
        let milk = json!({ "itemText": "牛奶" });
        let eggs = json!({ "itemText": "雞蛋" });
        assert_ipc_response(&window, request("add_item", milk), Ok(json!(1)));
        assert_ipc_response(&window, request("add_item", eggs), Ok(json!(2)));
    }

    #[test]
    fn snake_case_key_is_rejected() {
        let window = mock_window();
        let body = json!({ "item_text": "牛奶" });
        let error = get_ipc_response(&window, request("add_item", body)).unwrap_err();
        assert!(error.to_string().contains("missing required key itemText"));
    }
}
```

- `mock_window` 用 `mock_builder()` 與 `configure` 組出 App，`mock_context(noop_assets())` 是空的設定與空的前端資源，再用 `WebviewWindowBuilder`（第 6 章第 3 集）開一個 label 叫 `main` 的假視窗。
- `request` 組出前端 `invoke` 會送的請求：`cmd` 是 command 的名字，`body` 是參數的 JSON。`callback` 與 `error` 是回呼編號，測試用不到。`url` 要是 App 自己的位址，Windows 與 Android 是 `http://tauri.localhost`，其他平台是 `tauri://localhost`。`invoke_key` 要用 `INVOKE_KEY`。
- 第一個測試對同一個視窗呼叫兩次，第二次接著第一次數：狀態活在 App 裡。`assert_ipc_response` 比較回應跟期待值，`Ok` 是成功的回傳值，`Err` 是前端 Promise 被拒絕時收到的值；兩邊要同一種型別，所以用 `json!` 包成 `Value`。
- 第二個測試把鍵寫成 `item_text`。`get_ipc_response` 回傳 `Result`，`Err` 裡就是第 3 章第 3 集看過的 `invalid args` 訊息，測試檢查它有沒有 `missing required key itemText`。

### 跑測試

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

兩個測試都會通過（`2 passed`）。現在接線出錯也抓得到：把 `configure` 裡的 `.manage(...)` 那行刪掉，第一個測試會失敗：

`cargo test --manifest-path src-tauri/Cargo.toml` 的輸出（刪掉 `manage` 之後，節錄）：

```text
thread 'tests::add_item_keeps_state_between_calls' (123456) panicked at ~/.cargo/registry/src/.../tauri-2.12.6/src/test/mod.rs:243:3:
assertion `left == right` failed
  left: Err(String("state not managed for field `state` on command `add_item`. You must call `.manage()` before using this command"))
 right: Ok(Number(1))
```

這跟第 4 章第 1 集在視窗上看到的是同一種錯誤，只是現在由測試先擋下來。

mock 測不到的有兩樣：沒有真的 WebView，前端的程式不會執行（下一集處理）；`mock_context` 是空設定，不讀 `tauri.conf.json` 與 `capabilities/`，permission 有沒有設對也測不到。

## 重點整理

- `tauri::test` 要打開 `test` feature；放在 `[dev-dependencies]`，只有建置測試時才打開，不影響 App。
- 把 `manage` 與 `invoke_handler` 抽成收 `Builder<R>` 的函式，真的 App 與測試共用同一份設定。
- `mock_builder()`、`mock_context(noop_assets())` 與 `WebviewWindowBuilder` 組出假視窗；`get_ipc_response`、`assert_ipc_response` 送出 `InvokeRequest`，取得跟前端一樣的回應。
- 能測 command 的名字、參數的鍵、狀態、回傳值與錯誤；測不到前端程式與 permission。
