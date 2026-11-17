# struct 當參數與回傳值

## 本集目標

學會用 `#[derive(Serialize, Deserialize)]` 讓自己定義的 struct 能當 command 的參數與回傳值，並分清楚「參數名稱」與「欄位名稱」各自怎麼轉換。

## 正文

### 前端的物件，Rust 的 struct

資料一多，一個一個參數傳很累。前端傳一個物件，Rust 收一個 struct，更自然。兩個方向各需要一個 serde 的 trait，兩個都要就一起 derive：

- struct 當**參數**，前端的 JSON 要變成 struct，需要 `Deserialize`。
- struct 當**回傳值**，struct 要變成 JSON 送回去，需要 `Serialize`。

### 範例：過生日

`birthday` 收一個 `Profile`，把年齡加一，再回傳新的 `Profile`：

`src-tauri/src/lib.rs`：

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    display_name: String,
    age: u8,
}

#[tauri::command]
fn birthday(profile: Profile) -> Profile {
    Profile {
        age: profile.age.saturating_add(1),
        ..profile
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![birthday])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`#[serde(rename_all = "camelCase")]` 先放著，下一節解釋。前端這邊，用 `interface` 描述同一個形狀（第 2 章第 6 集）：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>struct</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>struct</h1>
      <div class="row">
        <button id="birthday-btn" type="button">過生日</button>
      </div>
      <p id="profile-msg">Ada：36 歲</p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

interface Profile {
  displayName: string;
  age: number;
}

const birthdayBtnEl: HTMLButtonElement | null = document.querySelector("#birthday-btn");
const profileMsgEl: HTMLElement | null = document.querySelector("#profile-msg");

let profile: Profile = { displayName: "Ada", age: 36 };

birthdayBtnEl?.addEventListener("click", async () => {
  profile = await invoke<Profile>("birthday", { profile: profile });
  if (profileMsgEl) {
    profileMsgEl.textContent = `${profile.displayName}：${profile.age} 歲`;
  }
});
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.35s
     Running `target/debug/tauri-app`
```

畫面一開始是「Ada：36 歲」，每按一次「過生日」加一歲。

### 兩層名稱，兩套規則

`invoke` 的第二個參數是 `{ profile: profile }`，這裡有兩層名稱，由不同的人負責轉換：

| | 例子 | 誰轉換 | 規則 |
| --- | --- | --- | --- |
| 參數名稱 | `profile` | Tauri | 一律把 `snake_case` 換成 `camelCase`（第 3 集） |
| struct 的欄位名稱 | `display_name` | serde | 照 Rust 的欄位名稱，除非自己用 `#[serde(...)]` 指定 |

這就是 `#[serde(rename_all = "camelCase")]` 的用途。拿掉它，欄位名稱就是 Rust 的 `display_name`：前端收到的物件是 `{"display_name":"Ada","age":37}`，傳 `{ displayName: ... }` 進去則會失敗，錯誤訊息是：

```text
invalid args `profile` for command `birthday`: missing field `display_name`
```

本書的 struct 一律加上 `rename_all = "camelCase"`，前端的 `interface` 就能統一用 `camelCase`。

### 少了 `Serialize` 或 `Deserialize`

如果上面的 `Profile` 只寫了 `#[derive(Deserialize)]`，回傳值就送不出去，`npm run tauri dev` 的錯誤（節錄）：

```text
error[E0599]: the method `blocking_kind` exists for reference `&Profile`, but its trait bounds were not satisfied
   --> src/lib.rs:10:1
    |
  5 | struct Profile {
    | -------------- doesn't satisfy `Profile: IpcResponse`
...
    = note: the following trait bounds were not satisfied:
            `Profile: IpcResponse`
```

訊息繞了一圈，沒有直接說 `Serialize`。`IpcResponse` 是 Tauri 內部的 trait，每個實作了 `Serialize` 的型別都自動是 `IpcResponse`，所以意思就是「這個型別不是 `Serialize`」。反過來，只寫 `#[derive(Serialize)]` 時，參數收不進來，錯誤碼是 `E0277`，說 `Profile` 沒有實作 `Deserialize`。

## 重點整理

- struct 當參數要 `Deserialize`，當回傳值要 `Serialize`，通常一起 derive。
- 前端傳物件，鍵是欄位名稱；用 `interface` 描述它的形狀。
- 參數名稱由 Tauri 轉成 `camelCase`；struct 的欄位名稱由 serde 決定，預設不轉換，要用 `#[serde(rename_all = "camelCase")]`。
- 回傳型別沒有 `Serialize` 時，編譯錯誤會提到 `IpcResponse`，意思就是缺 `Serialize`。
