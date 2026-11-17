# `Promise` 與 `async`/`await`

## 本集目標

學會用 `async` 函式與 `await` 等一件要花時間的事，並跟 Rust 的 `Future` 比較兩者的差別。

## 正文

### 有些事要等

等使用者回應、讀檔案、問 Rust 一個問題，這些事不是立刻就有結果。這種不會立刻有結果、得等一下的作業，叫做非同步（asynchronous）。JavaScript 處理非同步的做法是：函式不傻傻等，而是馬上回傳一個 `Promise`，意思是「結果之後才會有」。`Promise<string>` 就是「將來會得到一個 `string`」，它最後會**成功**（resolve）拿到值，或是**失敗**（reject）。

`async` 與 `await` 讓你用像一般程式的寫法處理 Promise：

- `async function` 宣告的函式，回傳值自動包成 Promise。
- `await` 寫在 `async` 函式裡，意思是「在這裡等這個 Promise 有結果，再往下執行」。

### 範例：載入按鈕

頁面有兩個按鈕和一行狀態文字：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>非同步</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>非同步</h1>
      <div class="row">
        <button id="load-ok" type="button">載入</button>
        <button id="load-fail" type="button">載入（會失敗）</button>
      </div>
      <p id="status">還沒開始</p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function loadTitle(ok: boolean): Promise<string> {
  await sleep(1000);
  if (!ok) {
    throw new Error("載入失敗");
  }
  return "第 10 集";
}

const okBtnEl: HTMLButtonElement | null = document.querySelector("#load-ok");
const failBtnEl: HTMLButtonElement | null = document.querySelector("#load-fail");
const statusEl: HTMLElement | null = document.querySelector("#status");

async function run(ok: boolean): Promise<void> {
  if (!statusEl) {
    return;
  }
  statusEl.textContent = "載入中…";
  try {
    const title = await loadTitle(ok);
    statusEl.textContent = `載入完成：${title}`;
  } catch (error) {
    if (error instanceof Error) {
      statusEl.textContent = `出錯了：${error.message}`;
    }
  }
}

okBtnEl?.addEventListener("click", () => {
  run(true);
  console.log("run 已經回來了，但還沒做完");
});
failBtnEl?.addEventListener("click", () => run(false));
```

按「載入」：馬上顯示「載入中…」，一秒後變成「載入完成：第 10 集」。按「載入（會失敗）」：一秒後顯示「出錯了：載入失敗」。

### 這段程式怎麼跑

- `sleep` 是我們自己包的小工具：`setTimeout(函式, 毫秒)` 是瀏覽器提供的，時間到了呼叫那個函式。這裡把 Promise 的 `resolve` 交給它，時間到就代表「成功了」。成功時沒有值可以給，所以型別是 `Promise<void>`（`void` 見第 5 集）。
- `loadTitle` 先 `await sleep(1000)` 等一秒，再回傳字串；`ok` 是 `false` 時用 `throw` 丟出錯誤，這個 Promise 就變成失敗。
- `run` 用 `try`/`catch` 接住失敗，角色像 Rust 用 `match` 處理 `Err`：自己接下錯誤、決定怎麼辦，而不是像 `?` 那樣把錯誤往外丟給呼叫者。`catch (error)` 的 `error` 在 strict 模式下型別是 `unknown`，要先用 `error instanceof Error` 確認是 `Error` 才能讀 `message`。
- `run` 不回傳值，但 `async` 函式回傳的一定是 Promise，所以它的回傳型別是 `Promise<void>`。

`await` 只會暫停那一個函式，頁面不會卡住：按下「載入」，Console 立刻印出：

```text
run 已經回來了，但還沒做完
```

`run(true)` 一碰到 `await` 就回到呼叫它的地方，剩下的部分等一秒後再接著做。在等的這一秒，按鈕都還是能按。範例裡呼叫 `run(...)` 沒有加 `await`，因為不需要等它，範本呼叫 `greet()` 也是同一個寫法。

### 跟 Rust 的 `Future` 比一比

寫法看起來很像：`async fn` 對 `async function`，`.await` 對 `await`（只是 JavaScript 寫在前面）。但有幾處不同：

| | Rust `Future` | JavaScript `Promise` |
| --- | --- | --- |
| 什麼時候開始跑 | 惰性：沒人 `.await` 或 poll 就不會動 | 一建立就開始跑 |
| 誰負責執行 | 要選一個 runtime（例如 tokio） | 瀏覽器內建，不用選 |
| 失敗怎麼表示 | 回傳 `Result` 的 `Err` | Promise 失敗，用 `try`/`catch` 接 |
| 在幾條執行緒上 | 可以跨執行緒，有 `Send` 的限制 | 只有一條，沒有 `Send` 的問題 |

所以範例裡 `loadTitle(ok)` 一呼叫，計時就開始了；程式只在一條執行緒上跑，也不用考慮 `Send`。

範本的 `await invoke("greet", ...)` 就是這個模式：`invoke` 回傳一個 Promise，`await` 等 Rust 那邊回覆。`invoke` 本身第 3 章會正式介紹。

## 重點整理

- `Promise<T>` 表示「將來會得到 `T`」，結果是成功或失敗。
- `async function` 回傳 Promise；`await` 寫在 `async` 函式裡，等結果出來再往下。
- `await` 只暫停那個函式，不會讓頁面卡住。
- 失敗用 `throw` 丟出、`try`/`catch` 接；`catch` 拿到的 `error` 是 `unknown`，要先檢查型別。
- 跟 Rust 的 `Future` 不同：Promise 一建立就開始跑，不需要選 runtime，也只有一條執行緒。
