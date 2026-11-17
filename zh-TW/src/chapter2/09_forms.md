# 表單與輸入值

## 本集目標

學會監聽表單的 `submit` 事件、用 `preventDefault()` 擋掉預設行為（default action），並讀出輸入框裡的值。

## 正文

### 一個打招呼的表單

這一集做一個跟範本 Greet 長得一樣的表單，只是問候語由 TypeScript 自己組，不去找 Rust：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>打招呼</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1>打招呼</h1>
      <form class="row" id="greet-form">
        <input id="greet-input" placeholder="輸入名字" />
        <button type="submit">送出</button>
      </form>
      <p id="greet-msg"></p>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
const formEl: HTMLFormElement | null = document.querySelector("#greet-form");
const inputEl: HTMLInputElement | null = document.querySelector("#greet-input");
const msgEl: HTMLElement | null = document.querySelector("#greet-msg");

formEl?.addEventListener("submit", (event) => {
  event.preventDefault();
  if (inputEl && msgEl) {
    msgEl.textContent = `你好，${inputEl.value}！`;
  }
});
```

存檔後，在輸入框打 `Tauri`，按「送出」，下面出現「你好，Tauri！」。

### `submit` 事件

`<form>` 把輸入框和按鈕包成一組。表單裡 `type="submit"` 的按鈕被按下，或是在輸入框裡按 Enter，表單就會發出 `submit` 事件。所以要監聽的是整個表單的 `submit`，而不是按鈕的 `click`，這樣兩種送出方式才都接得到。

### `preventDefault()`

瀏覽器對很多事件都有預設行為。表單送出的預設行為，是帶著輸入的內容重新載入頁面。對這種不離開頁面的 App 來說，重新載入就是畫面上的東西全部歸零，所以要在監聽函式裡呼叫 `event.preventDefault()` 取消預設行為。範本的 `submit` 監聽函式（不是 `greet()`）裡那一行 `e.preventDefault()`，用途就在這裡。可以試試把這一行拿掉再送出：頁面重新載入，問候語不會留在畫面上。

這個監聽函式的事件物件是 `SubmitEvent`，TypeScript 看事件名 `"submit"` 就推論出來了，跟上一集一樣。

### 輸入框的值

`HTMLInputElement` 的 `value` 欄位，是輸入框**現在**的內容，型別是 `string`，可以讀也可以寫。它跟 HTML 裡寫的 `value="..."` 屬性不一樣：那個屬性只是初始值，使用者一打字，變的是 `inputEl.value`。

範例裡 `if (inputEl && msgEl)` 先確認兩個元素都找得到，才用它們。這是第 6 集的型別縮窄，範本也是這樣寫。

回頭看範本的 `main.ts`：取出輸入框的值、擋掉預設行為、把結果寫回段落，這些你現在都看得懂。還沒教的有三樣：`async` 與 `await` 在下一集，`import` 在第 11 集，`invoke` 第 3 章會正式介紹。

## 重點整理

- 監聽表單的 `submit` 事件，按送出鈕與在輸入框按 Enter 都接得到。
- `event.preventDefault()` 取消預設行為；表單預設會重新載入頁面。
- `inputEl.value` 是輸入框現在的內容，型別是 `string`。
