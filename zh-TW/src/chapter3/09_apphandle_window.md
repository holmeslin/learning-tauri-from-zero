# 在 command 裡拿到 `AppHandle` 與 `WebviewWindow`

## 本集目標

學會在 command 的參數裡直接寫 `AppHandle` 與 `WebviewWindow`，拿到整個 App 的資訊，以及是哪個視窗呼叫了這個 command。

## 正文

### 不是前端傳的參數

前面所有參數都是前端用 `invoke` 的物件傳來的。但 command 的參數還有另一種：Tauri 自己從這次呼叫的脈絡（context）填進去的。你只要在參數列寫上型別，Tauri 就會幫你備好，前端不用傳，傳了也會被忽略。這一集認識兩個：

- `tauri::AppHandle`：整個 App 的把手（handle）。拿它可以讀 App 的設定與版本資訊；管理狀態、送事件、操作視窗也都靠它，分別在第 4、5、6 章會正式介紹。它可以 `clone()`，所以能複製一份交給別處使用。
- `tauri::WebviewWindow`：**呼叫這個 command 的那個視窗**。用它可以知道是哪個視窗在問，也能對那個視窗做事。

### 範例：App 與視窗的資訊

`app_info` 一個參數從前端來（`name`），另外兩個由 Tauri 填：

`src-tauri/src/lib.rs`：

```rust
use tauri::{AppHandle, WebviewWindow};

#[tauri::command]
fn app_info(name: &str, app: AppHandle, window: WebviewWindow) -> String {
    let info = app.package_info();
    format!(
        "你好，{name}！這裡是 {} {}（identifier：{}，視窗 label：{}）",
        info.name,
        info.version,
        app.config().identifier,
        window.label()
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `app.package_info()` 是 App 的名稱與版本，來源是 `tauri.conf.json` 的 `productName` 與 `version`。
- `app.config()` 是整份 `tauri.conf.json` 讀進來的設定，這裡取 `identifier`（第 1 章第 6 集）。
- `window.label()` 是視窗的 label，第 1 章第 6 集說過，範本的視窗沒有指定，預設是 `main`。視窗還能做哪些事，第 6 章會正式介紹。

前端用範本現成的 Greet 表單就好，不必改頁面。打開 `src/main.ts`，把 `invoke("greet", ...` 的 `"greet"` 改成 `"app_info"`，其他一個字都不動：

```ts,ignore
greetMsgEl.textContent = await invoke("app_info", {
  name: greetInputEl.value,
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.39s
     Running `target/debug/tauri-app`
```

輸入 `Ada` 按 Greet，段落出現：

```text
你好，Ada！這裡是 tauri-app 0.1.0（identifier：tw.example.tauri-app，視窗 label：main）
```

前端只傳了 `name`，`app` 與 `window` 不在 `invoke` 的物件裡，Tauri 自己補上了。這類參數跟一般參數可以混著寫，順序隨意。

### 這些參數從哪裡來

這兩種型別是從「這次呼叫是誰發的」推出來的，不是從前端傳的 JSON 還原的，所以不經過 `Deserialize`。這也是為什麼同一個 command，從不同視窗呼叫，`WebviewWindow` 會是不同的視窗；範本只有一個視窗，要到第 6 章才會看到多個視窗。

它們也都是擁有所有權的型別，不是借用，所以 `async` command 可以放心使用（第 8 集的借用限制不會碰到）：

```rust,ignore
#[tauri::command]
async fn app_name(app: AppHandle, window: WebviewWindow) -> String {
    format!("{} {}", app.package_info().name, window.label())
}
```

## 重點整理

- command 的參數除了前端傳的資料，也可以寫 `AppHandle`、`WebviewWindow` 這類 Tauri 自己填的型別，前端不用傳。
- `AppHandle` 是整個 App 的把手，可以讀設定與版本資訊，可以 `clone()`。
- `WebviewWindow` 是呼叫這個 command 的視窗，`label()` 是它的名字標記。
- 這類參數跟一般參數可以混用，也能用在 `async` command。
