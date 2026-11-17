# 事件：`addEventListener`

## 本集目標

學會用 `addEventListener` 註冊函式，在使用者點擊、按鍵、或頁面載好時執行，並知道事件物件（event object）從哪裡來。

## 正文

### 事件與監聽函式

使用者按下按鈕、敲鍵盤、或頁面載好，瀏覽器都會發出一個**事件**（event）。想在事件發生時做點事，就用 `addEventListener` 註冊一個函式（監聽函式，listener）：

```ts,ignore
button.addEventListener("click", increase);
```

第一個參數是事件的名字，第二個是事件發生時要呼叫的函式。註冊的當下函式不會執行，之後每發生一次事件就呼叫一次。所以這裡寫的是 `increase`，不是 `increase()`：前者是把函式交出去（第 5 集的 callback），後者會立刻執行，再把它的回傳值交出去。

### 計數器

這一集的頁面是一個計數器：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>計數器</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>計數器</h1>
      <p id="count">0</p>
      <div class="row">
        <button id="plus" type="button">加一</button>
      </div>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
const countEl: HTMLElement | null = document.querySelector("#count");
const plusBtnEl: HTMLButtonElement | null = document.querySelector("#plus");

let count = 0;

function increase(): void {
  count += 1;
  if (countEl) {
    countEl.textContent = `${count}`;
  }
}

plusBtnEl?.addEventListener("click", increase);

document.addEventListener("keydown", (event) => {
  if (event.key === "ArrowUp") {
    increase();
  }
});

window.addEventListener("DOMContentLoaded", () => {
  console.log("頁面載好了");
});
```

存檔後，按「加一」，數字加一；按鍵盤的向上鍵，也會加一。

### 事件物件

註冊的函式可以收一個參數，瀏覽器呼叫它時會把**事件物件**傳進來，裡面是這次事件的資訊。上面 `keydown` 的函式用 `event.key` 看按下的是哪個鍵。

這個參數不用自己寫型別：TypeScript 看事件名字就知道是哪一種。`"keydown"` 是 `KeyboardEvent`，有 `key`；`"click"` 是 `PointerEvent`，有點擊位置的座標等資訊。如果在 `click` 的事件物件上取 `key`，`tsc` 會報錯，因為點擊沒有這個欄位。`increase` 不需要事件的資訊，所以它沒有參數，這是允許的；它不回傳值，回傳型別是第 5 集說的 `void`。

### 在哪個物件上註冊

`addEventListener` 不只按鈕有，`document`、`window` 也有。大部分的事件發生在某個元素上之後，會從那個元素一路往外傳：先到它自己，再到包住它的外層元素，最後到 `document` 和 `window`，這叫冒泡（bubbling）。所以註冊在哪一層，決定的是你要聽多大的範圍：

- 元素（如按鈕）：只聽那個元素，以及從它裡面冒泡上來的事件，像它的 `click`。
- `document`：整份頁面冒泡上來的事件，像鍵盤的 `keydown`（按鍵發生在目前有焦點的元素上，再往外傳）。按鈕的 `click` 也會傳到這裡。
- `window`：最外層，像 `DOMContentLoaded`（發在 `document` 上，再冒泡到 `window`）。

`DOMContentLoaded` 在 HTML 全部讀完、DOM 建好時發出一次。範本把程式包在它裡面，是為了確保元素找得到，不包也行的原因見上一集。這裡包起來，是為了讓你看到它被呼叫，打開 Console 會看到那行「頁面載好了」。

## 重點整理

- `元素.addEventListener("事件名", 函式)` 註冊函式，事件每發生一次就呼叫一次。
- 傳的是函式本身（`increase`），不是呼叫結果（`increase()`）。
- 監聽函式可以收一個事件物件，型別由事件名字決定（`"click"` 是 `PointerEvent`，`"keydown"` 是 `KeyboardEvent`）。
- 元素、`document`、`window` 都能註冊事件；大部分的事件會從元素往外冒泡到 `document` 與 `window`。`DOMContentLoaded` 在頁面讀完時發出。
