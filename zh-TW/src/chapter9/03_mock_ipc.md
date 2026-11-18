# 前端 mock IPC：`@tauri-apps/api/mocks`

## 本集目標

用 `@tauri-apps/api/mocks` 假裝 Rust 的回覆，在不啟動 App 的情況下，用 vitest 測前端呼叫 command 的程式。

## 正文

### 測試裡沒有 Rust

前端的 `invoke` 靠 Tauri 放進頁面的 `window.__TAURI_INTERNALS__` 跟 Rust 溝通。前端測試跑在 Node 裡，沒有 WebView，也沒有那個東西，直接呼叫 `invoke` 會失敗。`mockIPC` 就是替它裝一個假的：測試給一個函式，`invoke` 一送出請求，就改由這個函式回答。

### 裝測試工具

需要兩個套件：vitest 是跑測試的工具，跟 Vite 同一個生態；jsdom 在 Node 裡模擬瀏覽器的 `window`，`mockIPC` 要把假的東西放在 `window` 上。

```bash
npm install -D vitest jsdom
```

`-D` 把它們記在 `package.json` 的 `devDependencies`，只在開發時用，像 Cargo 的 `[dev-dependencies]`。本書用 vitest 5.1 與 jsdom 31。

### 把呼叫 Rust 的程式獨立出來

跟第 1 集同一個想法：呼叫 command 的程式放進不碰 DOM 的 `api.ts`，才好測。`addItem` 呼叫上一集的 `add_item`，把結果換成要顯示的訊息；`showCount` 用 `getCurrentWindow().setTitle`（第 6 章第 2 集）把項目數放進視窗標題。`main.ts` 只要 `import { addItem } from "./api"` 就能用，這一集不改它。

`src/api.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export async function addItem(text: string): Promise<string> {
  try {
    const count = await invoke<number>("add_item", { itemText: text });
    return `已加入，目前共 ${count} 項`;
  } catch (error) {
    return `加入失敗：${error}`;
  }
}

export async function showCount(count: number): Promise<void> {
  await getCurrentWindow().setTitle(`清單（${count} 項）`);
}
```

### 寫測試

`src/api.test.ts`：

```ts
// @vitest-environment jsdom
import { afterEach, expect, test } from "vitest";
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { addItem, showCount } from "./api";

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
  mockIPC((cmd) => {
    if (cmd === "add_item") {
      throw "「牛奶」已經在清單裡了";
    }
  });

  expect(await addItem("牛奶")).toBe("加入失敗：「牛奶」已經在清單裡了");
});

test("showCount 把項目數放進視窗標題", async () => {
  mockWindows("main");
  let received: unknown;
  mockIPC((cmd, payload) => {
    if (cmd === "plugin:window|set_title") {
      received = payload;
    }
  });

  await showCount(2);
  expect(received).toEqual({ label: "main", value: "清單（2 項）" });
});
```

- 第一行是註解，告訴 vitest 這個檔案用 jsdom 環境，才有 `window`。`test` 與 `expect` 的用法跟 Rust 的 `#[test]` 與 `assert_eq!` 很像。
- `mockIPC` 收一個函式，參數是 command 的名字與參數物件。函式的回傳值就是 `invoke` 成功時的值；丟出（`throw`）的值，就是 Promise 被拒絕時收到的值，等於 Rust 的 `Err`。`Err(String)` 到前端是字串（第 3 章第 6 集），所以這裡丟字串。
- 第一個測試把收到的參數存進 `received`，檢查前端傳的鍵是 `itemText`，跟上一集 Rust 測試用的是同一個。
- `mockWindows("main")` 讓 `getCurrentWindow()` 知道目前視窗叫 `main`。它只模擬視窗存在，視窗的方法照樣走 `mockIPC`：`setTitle` 送出的請求叫 `plugin:window|set_title`，參數裡有視窗的 label 與新標題。
- `afterEach` 裡的 `clearMocks()` 把假的東西清掉，免得一個測試的 mock 影響下一個。

### 跑測試

```bash
npx vitest run
```

`npx vitest run` 的輸出：

```text
 RUN  v5.1.0 ~/tauri-app


 Test Files  1 passed (1)
      Tests  3 passed (3)
   Start at  12:55:49
   Duration  397ms (environment 92%, transform 4%, import 2%, worker 1%, tests 1%)
```

把 `api.ts` 裡的 `itemText` 改成 `item_text` 再跑，第一個測試會失敗，並指出差在哪裡（節錄）：

```text
 FAIL  src/api.test.ts > addItem 成功時顯示項目數，並用 itemText 傳參數
AssertionError: expected { item_text: '牛奶' } to deeply equal { itemText: '牛奶' }

- Expected
+ Received

  {
-   "itemText": "牛奶",
+   "item_text": "牛奶",
  }
```

### mock 回什麼就是什麼

`mockIPC` 回 `3`，不代表 Rust 真的會回 3：就算 Rust 這一側根本沒有 `add_item`，這些測試照樣通過。所以兩邊要各測一次同一份約定：Rust 端用上一集的 mock runtime，前端用這一集的 mock IPC，command 的名字與參數的鍵兩邊各驗一次。最後仍要開 App 按一次，確認兩邊真的接得起來。

## 重點整理

- 前端測試用 vitest 加 jsdom；`mockIPC` 替 `invoke` 裝一個假的回答函式，`throw` 等於 Rust 的 `Err`。
- 呼叫 command 的程式獨立成不碰 DOM 的檔案，才好用 `mockIPC` 測；用 `let` 存下收到的參數，就能檢查前端傳的鍵。
- 用到 `getCurrentWindow()` 的程式要先 `mockWindows("main")`；它只模擬視窗存在，視窗的方法仍靠 `mockIPC`。
- 每個測試之後呼叫 `clearMocks()` 清掉 mock；`npx vitest run` 跑一次測試就結束。
