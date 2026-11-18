# 前端 mock IPC：`@tauri-apps/api/mocks`

## 本集目標

用 `@tauri-apps/api/mocks` 假裝 Rust 的回覆，在不啟動 App 的情況下，用 vitest 測前端呼叫 command 的程式。

## 正文

### 測試裡沒有 Rust

前端的 `invoke` 靠 Tauri 放進頁面的 `window.__TAURI_INTERNALS__` 跟 Rust 溝通。前端測試跑在 Node 裡，沒有 WebView，也沒有那個東西，直接呼叫 `invoke` 會失敗。`mockIPC` 就是替它裝一個假的：測試給一個函式，`invoke` 一送出請求，就改由這個函式回答。

需要兩個套件：vitest 是跑測試的工具，跟 Vite 同一個生態；jsdom 在 Node 裡模擬瀏覽器的 `window`，`mockIPC` 要把假的東西放在 `window` 上。

```bash
npm install -D vitest jsdom
```

`-D` 把它們記在 `package.json` 的 `devDependencies`，只在開發時用，像 Cargo 的 `[dev-dependencies]`。本書用 vitest 5.1 與 jsdom 31，它們要求 Node.js 22.22.2、24.15.0 或 26 以上；照本書的慣例用目前的 LTS 就符合，`node -v` 可以看版本。

### 把呼叫 Rust 的程式獨立出來

跟第 1 集同一個想法：呼叫 command 的程式放進不碰 DOM 的 `api.ts`，才好測。`addItem` 呼叫 `add_item`，把結果換成要顯示的訊息；`main.ts` 再把訊息放上畫面。Rust 這一側就是第 1 集的 `add_item`，這集不改它（前兩集的測試用不到，所以沒有貼）：

`src-tauri/src/lib.rs`：

```rust
use std::sync::Mutex;
use tauri::State;

fn push_trimmed(items: &mut Vec<String>, text: &str) -> Result<usize, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("項目不能是空的".to_string());
    }
    items.push(text.to_string());
    Ok(items.len())
}

#[tauri::command]
fn add_item(item_text: &str, state: State<Mutex<Vec<String>>>) -> Result<usize, String> {
    push_trimmed(&mut state.lock().unwrap(), item_text)
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
```

`src/api.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

export async function addItem(text: string): Promise<string> {
  try {
    const count = await invoke<number>("add_item", { itemText: text });
    return `已加入，目前共 ${count} 項`;
  } catch (error) {
    return `加入失敗：${error}`;
  }
}
```

`src/main.ts`（沿用範本 `index.html` 的輸入框、按鈕與段落，畫面上的字還是 Greet，只是借來用）：

```ts
import { addItem } from "./api";

const inputEl: HTMLInputElement | null = document.querySelector("#greet-input");
const msgEl: HTMLElement | null = document.querySelector("#greet-msg");

document.querySelector("#greet-form")?.addEventListener("submit", async (e) => {
  e.preventDefault();
  if (inputEl && msgEl) {
    msgEl.textContent = await addItem(inputEl.value);
  }
});
```

### 寫測試

`src/api.test.ts`：

```ts
// @vitest-environment jsdom
import { afterEach, expect, test } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { addItem } from "./api";

afterEach(() => {
  clearMocks();
});

test("addItem 成功時顯示項目數，並用 itemText 傳參數", async () => {
  let received: unknown;
  mockIPC((cmd, payload) => {
    if (cmd === "add_item") {
      received = payload;
      return 3;
    }
  });

  expect(await addItem("牛奶")).toBe("已加入，目前共 3 項");
  expect(received).toEqual({ itemText: "牛奶" });
});

test("addItem 在 Rust 回傳 Err 時顯示失敗訊息", async () => {
  mockIPC(() => {
    throw "項目不能是空的";
  });

  expect(await addItem("")).toBe("加入失敗：項目不能是空的");
});
```

- 第一行是註解，告訴 vitest 這個檔案用 jsdom 環境，才有 `window`。`test` 與 `expect` 的用法跟 Rust 的 `#[test]` 與 `assert_eq!` 很像。每個測試之後，`afterEach` 裡的 `clearMocks()` 把假的東西清掉，免得影響下一個。
- `mockIPC` 收一個函式，參數是 command 的名字與參數物件。函式的回傳值就是 `invoke` 成功時的值；丟出（`throw`）的值，就是 Promise 被拒絕時收到的值，等於 Rust 的 `Err`。`Err(String)` 到前端是字串（第 3 章第 6 集），所以這裡丟的字串，跟 `push_trimmed` 回傳的 `Err` 一樣。第一個測試把收到的參數存進 `received`，檢查前端傳的鍵是 `itemText`，跟上一集 Rust 測試用的是同一個。

### 跑測試

`npx vitest run` 的輸出（節錄）：

```text
 RUN  v5.1.0 ~/tauri-app

 Test Files  1 passed (1)
      Tests  2 passed (2)
```

把 `api.ts` 裡的 `itemText` 改成 `item_text` 再跑，第一個測試會失敗，輸出會列出 `Expected` 與 `Received` 的差異，指出鍵不一樣。

測試檔放在 `src/`，跟 `api.ts` 並排。`npm run build` 會跑 `tsc`，連 `src/api.test.ts` 也做型別檢查；但 Vite 只打包從 `main.ts` 一路 `import` 得到的檔案，沒有人 import 測試檔，它不會進 App。

### mock 回什麼就是什麼

`mockIPC` 回 `3`，不代表 Rust 真的會回 3：就算 Rust 這一側根本沒有 `add_item`，這些測試照樣通過。所以兩邊要各測一次同一份約定：Rust 端用上一集的 mock runtime，前端用這一集的 mock IPC，兩邊都要驗證 command 的名字、參數的鍵，以及錯誤訊息。像上面丟的「項目不能是空的」，就要是 Rust 真的會回傳的那一句。最後仍要開 App 按一次，確認兩邊真的接得起來。

## 重點整理

- 前端測試用 vitest 加 jsdom；`mockIPC` 替 `invoke` 裝一個假的回答函式，`throw` 等於 Rust 的 `Err`。
- 把呼叫 command 的程式獨立成一個檔案，不碰 DOM，才好用 `mockIPC` 測；用 `let` 存下收到的參數，就能檢查前端傳的鍵。
- 每個測試之後呼叫 `clearMocks()` 清掉 mock；`npx vitest run` 跑一次測試就結束；測試檔會被 `tsc` 檢查，但不會被打包。
- mock 的名字、鍵、錯誤訊息要跟 Rust 端是同一份約定，最後仍要開 App 實際按一次。
