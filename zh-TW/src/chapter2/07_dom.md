# DOM：`querySelector` 與修改內容

## 本集目標

學會用 `querySelector` 找到頁面上的元素，用 `textContent` 改它的文字、用 `createElement` 加新元素，並用 `console.log` 把值印到 Console。

## 正文

### 頁面是一棵 DOM 樹

瀏覽器讀完 `index.html` 之後，會在記憶體裡建出一棵樹，每個元素都是樹上的一個物件，這棵樹叫 DOM（Document Object Model）。第 1 章第 8 集在 Elements 分頁看到的，就是它。`document` 是樹的根，TypeScript 靠它找元素、改元素；改了 DOM，畫面就跟著變，不需要動 `index.html` 這個檔案。

### 先換兩個檔案

`index.html` 換成一個標題加一個空的清單，`src/styles.css` 維持範本原樣：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>待辦清單</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main class="container">
      <h1 id="title">今天要做的事</h1>
      <ul id="todo-list"></ul>
    </main>
  </body>
</html>
```

`src/main.ts`：

```ts
interface Note {
  id: number;
  title: string;
}

const notes: Note[] = [
  { id: 1, title: "讀完第 2 章" },
  { id: 2, title: "安裝 Node.js" },
];

const titleEl: HTMLElement | null = document.querySelector("#title");
const listEl: HTMLUListElement | null = document.querySelector("#todo-list");

console.log(`共 ${notes.length} 件事`);
console.log(titleEl?.textContent);
console.log(document.querySelector("#nope"));

if (titleEl) {
  titleEl.textContent = `今天要做的 ${notes.length} 件事`;
}

if (listEl) {
  for (const note of notes) {
    const item = document.createElement("li");
    item.textContent = note.title;
    listEl.append(item);
  }
}
```

存檔後，標題變成「今天要做的 2 件事」，下面多了兩個清單項目。

### 找元素：`querySelector`

`document.querySelector("#title")` 的參數是第 3 集的 CSS 選擇器，回傳**第一個**符合的元素；一個都沒有就回傳 `null`。所以它的型別是「元素或 `null`」，要像上一集那樣先檢查再用。

型別上有一點要注意：每種元素在 TypeScript 裡有自己的型別。`<ul>` 是 `HTMLUListElement`，`<input>` 是 `HTMLInputElement`，它們都屬於比較籠統的 `HTMLElement`。`querySelector` 會照你宣告的變數型別回傳，所以 `const listEl: HTMLUListElement | null = ...` 一行就搞定。但 TypeScript 看不到你的 HTML，不會幫你確認 `#todo-list` 真的是 `<ul>`，寫錯了要到執行時才出問題。

`titleEl?.textContent` 的 `?.` 叫選用串接（optional chaining）：前面是 `null`（或 `undefined`）就整個運算式變成 `undefined`，不會報錯；不是才繼續往後取。範本的 `document.querySelector("#greet-form")?.addEventListener(...)` 就是這個寫法。

### 改內容：`textContent` 與 `createElement`

`textContent` 是元素裡的文字，可以讀也可以寫。範例用 `if (titleEl)` 確定不是 `null` 才改它。

要加新元素，先 `document.createElement("li")` 做出一個還沒放進頁面的 `<li>`，設好它的 `textContent`，再用 `listEl.append(item)` 放到清單的最後面，它才會出現在畫面上。

還有一個 `innerHTML` 可以直接塞一段 HTML 字串。字串裡如果有使用者輸入的內容，就可能被塞進惡意的程式，所以本書一律用 `textContent`。

### 印出值：`console.log`

`console.log(...)` 把值印到開發者工具的 Console（第 1 章第 8 集打開過的那個）。它的角色像 Rust 的 `println!`，只是印在 Console，不在終端機。範例印了三行，打開 Console 會看到：

```text
共 2 件事
今天要做的事
null
```

第二行印的是標題被改掉**之前**的文字，因為印的那一行在 `if (titleEl)` 前面。第三行的 `null` 是找不到 `#nope` 的結果。

### 為什麼不用等頁面載好

範本把程式包在 `DOMContentLoaded` 裡，這集的程式直接寫在最外層，也能找到元素。因為 `<script type="module">` 預設等 HTML 讀完才執行，執行時元素已經都在了。`DOMContentLoaded` 是什麼，下一集會說明。

## 重點整理

- 瀏覽器把頁面建成一棵 DOM 樹，`document` 是根，改 DOM 畫面就跟著變。
- `document.querySelector(選擇器)` 回傳第一個符合的元素，找不到是 `null`，要先檢查或用 `?.`。
- 元素有各自的型別（`HTMLUListElement`、`HTMLInputElement`），用變數的型別註記告訴 `querySelector` 要哪一種。
- `textContent` 讀寫文字；`createElement` 加 `append` 加新元素；不要用 `innerHTML` 塞使用者輸入。
- `console.log` 把值印到 Console，用來看前端程式在做什麼。
