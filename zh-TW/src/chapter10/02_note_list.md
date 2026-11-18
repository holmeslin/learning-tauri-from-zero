# 筆記清單

## 本集目標

用一個 `Note[]` 陣列畫出清單，點一則就把它選起來，內容顯示在右邊。

## 正文

### 資料決定畫面

上一集的清單是寫死在 `index.html` 的。這一集改成：資料放在 TypeScript 的陣列裡，畫面由資料畫出來。資料換了，重畫一次畫面就跟著變。目前資料是三則假筆記，存檔留到第 4 集。

`index.html` 的清單改成空的，等程式來填。兩個輸入框先加上 `readonly`，只能看、不能改；它跟第 6 章的 `hidden` 一樣，是不用寫值的屬性。編輯要等下一集：

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>筆記</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <div class="sidebar">
      <ul id="note-list"></ul>
    </div>
    <main>
      <input id="title-input" placeholder="標題" readonly />
      <textarea id="body-input" placeholder="內容" readonly></textarea>
    </main>
  </body>
</html>
```

`src/styles.css`、`src/notes.ts` 跟上一集一樣，這一集為了能獨立開始，再貼一次：

`src/styles.css`：

```css
body {
  display: flex;
  height: 100vh;
  margin: 0;
  font-family: sans-serif;
}

.sidebar {
  width: 200px;
  padding: 8px;
  overflow-y: auto;
  background-color: #eee;
}

main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px;
}

#note-list {
  padding: 0;
  list-style: none;
}

#note-list li {
  padding: 6px 8px;
  cursor: pointer;
}

#note-list .selected {
  background-color: #cde;
}

#body-input {
  flex: 1;
}
```

`src/notes.ts`：

```ts
export interface Note {
  title: string;
  body: string;
}
```

### 畫出清單，選一則

`src/main.ts`：

```ts
import { Note } from "./notes";

const listEl: HTMLUListElement | null = document.querySelector("#note-list");
const titleEl: HTMLInputElement | null = document.querySelector("#title-input");
const bodyEl: HTMLTextAreaElement | null = document.querySelector("#body-input");
if (!listEl || !titleEl || !bodyEl) {
  throw new Error("找不到必要的元素");
}

const notes: Note[] = [
  { title: "購物清單", body: "牛奶\n雞蛋" },
  { title: "讀書計畫", body: "讀完第 10 章" },
  { title: "", body: "沒有標題的筆記" },
];
let current: Note = notes[0];

const render = () => {
  listEl.textContent = "";
  for (const note of notes) {
    const itemEl = document.createElement("li");
    itemEl.textContent = note.title;
    if (note.title === "") {
      itemEl.textContent = "（沒有標題）";
    }
    if (note === current) {
      itemEl.className = "selected";
    }
    itemEl.addEventListener("click", () => select(note));
    listEl.append(itemEl);
  }
};

const select = (note: Note) => {
  current = note;
  titleEl.value = note.title;
  bodyEl.value = note.body;
  render();
};

select(current);
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.51s
     Running `target/debug/tauri-app`
```

視窗左邊出現三則筆記，第一則是藍底，右邊顯示它的標題和內容。點「讀書計畫」，藍底移到它身上，右邊跟著換；點第三則，清單上顯示「（沒有標題）」，右邊的標題框是空的。

### 這段程式怎麼跑

- `current` 記著目前選到哪一則，一開始是 `notes[0]`，也就是第一則（陣列的編號從 0 算起，跟 Rust 一樣）。它存的是那個物件本身，不是編號，所以 `note === current` 就能判斷清單上的哪一列要加底色。
- `render` 先用 `listEl.textContent = ""` 清空清單，再為每則筆記建一個 `li`。每次都整份重畫：筆記不多的時候，這比「只改有變的那幾列」簡單得多，也不容易出錯。
- `itemEl.className = "selected"` 設定元素的 `class` 屬性（第 2 章第 2 集）。`.selected` 那條 CSS 規則就會套上去。
- 每個 `li` 的點擊函式 `() => select(note)`，用到的是迴圈裡那一輪的 `note`（箭頭函式可以用外面的變數，第 2 章第 5 集），所以點哪一列，選到的就是哪一則。
- `select` 把 `current` 換掉，把標題與內容填進輸入框，再 `render` 一次，讓藍底移過去。最後一行 `select(current)` 讓一開始就有選到第一則。

### 為什麼 `render` 不寫成 `function`

前面的程式檢查過 `listEl`、`titleEl`、`bodyEl` 不是 `null`，後面才用。但 `function render() { ... }` 這種宣告會被提到檔案最前面，TypeScript 不敢假設那時候檢查已經做完，會說 `'listEl' is possibly 'null'`。`const render = () => { ... }` 是檢查之後才建立的箭頭函式，TypeScript 看得出此時 `listEl` 一定有東西，所以 `main.ts` 裡會用到這些檢查結果的函式，都寫成這種形式。

## 重點整理

- 資料放在陣列，畫面由資料畫出來；資料變了，就清空清單、整份重畫。
- 用 `textContent = ""` 清空清單，`createElement`、`append` 加上每一列，`className` 設定 `class`。
- 用 `current` 存目前選到的筆記物件，`note === current` 判斷哪一列要標示。
- 函式要用到前面做過的 `null` 檢查，就寫成 `const f = () => { ... }`，不要用 `function` 宣告。
