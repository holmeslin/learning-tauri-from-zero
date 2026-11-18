# 把邏輯從 command 抽出來測

## 本集目標

把 command 裡的判斷抽成不依賴 Tauri 的普通函式，用 `cargo test` 測它，並分清楚這樣測得到什麼、測不到什麼。

## 正文

### command 是接線，邏輯另外放

一個 command 做兩件事：先從 Tauri 拿東西（前端傳來的參數、`State`、`AppHandle`），再做真正的工作。兩件事混在一起，就很難測。`State` 這類型別只有 Tauri 能造出來（它的欄位是私有的，沒有公開的建構函式），單元測試沒辦法憑空造一個傳給函式。

所以要拆開：真正的邏輯寫成普通函式，參數只收普通的值；command 只負責接線，從 Tauri 拿到值，交給那個函式，再把結果回傳。測試只要對準普通函式。

### 範例：新增清單項目

這一章用一個小清單當例子。`add_item` 把文字加進清單，並檢查兩件事：不能是空的，也不能重複。清單放在狀態裡（第 4 章），所以 command 有一個 `State` 參數。判斷寫在 `add`，它只認得 `Vec<String>` 與 `&str`：

`src-tauri/src/lib.rs`：

```rust
use std::sync::Mutex;
use tauri::State;

fn add(items: &mut Vec<String>, text: &str) -> Result<usize, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("項目不能是空的".to_string());
    }
    if items.iter().any(|item| item == text) {
        return Err(format!("「{text}」已經在清單裡了"));
    }
    items.push(text.to_string());
    Ok(items.len())
}

#[tauri::command]
fn add_item(item_text: &str, state: State<Mutex<Vec<String>>>) -> Result<usize, String> {
    add(&mut state.lock().unwrap(), item_text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(Vec::<String>::new()))
        .invoke_handler(tauri::generate_handler![add_item])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_counts() {
        let mut items = Vec::new();
        assert_eq!(add(&mut items, "  牛奶 "), Ok(1));
        assert_eq!(add(&mut items, "雞蛋"), Ok(2));
        assert_eq!(items, ["牛奶", "雞蛋"]);
    }

    #[test]
    fn rejects_empty() {
        let mut items = Vec::new();
        assert_eq!(add(&mut items, "   "), Err("項目不能是空的".to_string()));
        assert!(items.is_empty());
    }

    #[test]
    fn rejects_duplicate() {
        let mut items = vec!["牛奶".to_string()];
        assert!(add(&mut items, "牛奶").is_err());
        assert_eq!(items.len(), 1);
    }
}
```

- `add` 先 `trim()` 去掉頭尾空白，再依序檢查空字串與重複，成功就回傳目前的項目數。它沒有任何 Tauri 的型別。
- `add_item` 只剩一行：鎖住狀態，把裡面的 `Vec` 交給 `add`。`&mut state.lock().unwrap()` 是 `&mut MutexGuard<Vec<String>>`，會自動轉成 `&mut Vec<String>`。
- 參數 `item_text` 在前端要寫成 `itemText`（第 3 章第 3 集）。

測試放在同一個檔案的 `#[cfg(test)] mod tests`，寫法跟平常一樣。三個測試各管一條規則，直接呼叫 `add`，不需要 App，也不需要視窗。

### 跑測試

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

`cargo test --manifest-path src-tauri/Cargo.toml` 的輸出（節錄）：

```text
running 3 tests
test tests::trims_and_counts ... ok
test tests::rejects_duplicate ... ok
test tests::rejects_empty ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

第一次跑要先編譯相依套件，會等一會兒。`cargo test` 不會開視窗，`npm run tauri dev` 也不會跑測試；`#[cfg(test)]` 裡的程式不會被編進 App。

### 讓測試失敗一次

把 `add` 第一行的 `let text = text.trim();` 刪掉，再跑一次：

`cargo test --manifest-path src-tauri/Cargo.toml` 的輸出（刪掉 `trim` 之後，節錄）：

```text
running 3 tests
test tests::rejects_duplicate ... ok
test tests::rejects_empty ... FAILED
test tests::trims_and_counts ... FAILED

---- tests::rejects_empty stdout ----
thread 'tests::rejects_empty' (123456) panicked at src/lib.rs:45:9:
assertion `left == right` failed
  left: Ok(1)
 right: Err("項目不能是空的")

---- tests::trims_and_counts stdout ----
thread 'tests::trims_and_counts' (123456) panicked at src/lib.rs:39:9:
assertion `left == right` failed
  left: ["  牛奶 ", "雞蛋"]
 right: ["牛奶", "雞蛋"]

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

兩條跟空白有關的規則都被抓到了，`rejects_duplicate` 沒受影響。把那一行加回去，三個測試就會再次通過。

### 測得到，也有測不到

單元測試管的是 `add` 裡的每一條規則。接線它管不到：

- command 的名字有沒有登記進 `generate_handler!`。
- 狀態有沒有 `manage`。
- 前端傳的鍵（`itemText`）跟 Rust 的參數名稱對不對得上。

這些出錯，`cargo test` 照樣全部通過，要打開 App 實際按下去才會發現。下一集會用 Tauri 提供的 mock runtime，把這一層也放進測試。

## 重點整理

- command 只負責接線；判斷寫成只收普通值的函式，`State`、`AppHandle` 留在 command 這一層。
- 拆出來的函式用 `#[cfg(test)] mod tests` 與 `cargo test` 測，不需要 App、不需要視窗，也不會被編進 App。
- 測試可以靠改壞邏輯、看它變紅來確認真的有在檢查。
- 單元測試測不到接線：名字有沒有登記、有沒有 `manage`、前端的鍵對不對得上。
