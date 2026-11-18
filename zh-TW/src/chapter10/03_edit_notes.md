# 新增與編輯

## 本集目標

讓筆記可以編輯、新增、刪除：輸入框一改，`current` 這則筆記跟著改，清單也隨之更新。

## 正文

### 輸入框改了，資料就改

編輯的做法很直接：使用者在輸入框打字，就把輸入框的值（第 2 章第 9 集）寫回 `current`，再重畫清單，標題的變動就馬上出現在清單上。要聽的是 `input` 事件，輸入框的內容每次改變都會發出，第 6 章第 6 集用過。兩個輸入框共用同一個 `edit`：不管改了哪一個，都把兩個欄位的值寫回去。

新增是 `push` 一則新筆記進陣列，再選起來。刪除要把 `current` 從陣列拿掉。陣列的 `filter` 跟第 2 章第 6 集的 `find` 一樣收一個函式，回傳的是「函式回答 `true` 的那些元素」組成的新陣列，所以 `notes.filter((note) => note !== current)` 就是「除了 `current` 以外的所有筆記」。因為要把新陣列放回 `notes`，`notes` 現在要用 `let`。

刪掉最後一則之後，清單就空了，右邊沒有東西可以選。為了讓程式不用處理「沒有選到任何筆記」，這個 App 規定清單裡至少有一則：空了就補一則空白的。`showFirst` 負責這件事，之後啟動時也會用到。

`index.html` 多了兩個按鈕，輸入框拿掉 `readonly`：

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
      <div>
        <button id="new-btn" type="button">新增</button>
      </div>
      <ul id="note-list"></ul>
    </div>
    <main>
      <input id="title-input" placeholder="標題" />
      <textarea id="body-input" placeholder="內容"></textarea>
      <div>
        <button id="delete-btn" type="button">刪除</button>
      </div>
    </main>
  </body>
</html>
```

`src/styles.css` 與 `src/notes.ts` 沒變：

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

### 接上輸入與按鈕

`src/main.ts`：

```ts
import { Note } from "./notes";

const listEl: HTMLUListElement | null = document.querySelector("#note-list");
const titleEl: HTMLInputElement | null = document.querySelector("#title-input");
const bodyEl: HTMLTextAreaElement | null = document.querySelector("#body-input");
if (!listEl || !titleEl || !bodyEl) {
  throw new Error("找不到必要的元素");
}

let notes: Note[] = [
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

const showFirst = () => {
  if (notes.length === 0) {
    notes.push({ title: "新筆記", body: "" });
  }
  select(notes[0]);
};

const edit = () => {
  current.title = titleEl.value;
  current.body = bodyEl.value;
  render();
};

titleEl.addEventListener("input", edit);
bodyEl.addEventListener("input", edit);

document.querySelector("#new-btn")?.addEventListener("click", () => {
  const note: Note = { title: "新筆記", body: "" };
  notes.push(note);
  select(note);
});

document.querySelector("#delete-btn")?.addEventListener("click", () => {
  notes = notes.filter((note) => note !== current);
  showFirst();
});

showFirst();
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.32s
     Running `target/debug/tauri-app`
```

試試看：

- 在標題框打字，清單上那一列的字同時跟著變；把標題清空，清單顯示「（沒有標題）」。
- 在內容框打字，換去點別的筆記再點回來，打的字還在：它們已經寫回物件了。
- 按「新增」，清單最下面多一則「新筆記」，而且已經選起來，可以直接改。
- 按「刪除」，目前這則消失，選到清單的第一則。一則一則刪到沒有，會補出一則空白的「新筆記」。

現在資料只存在記憶體裡。關掉 App 再開，又回到三則假筆記，這是下一集的事。

## 重點整理

- 兩個輸入框共用一個 `edit`：`input` 事件發生時，把 `value` 寫回 `current`，再重畫清單。
- 新增是 `push` 加 `select`；刪除用 `filter` 做出「除了 `current` 以外」的新陣列，`notes` 因此要用 `let`。
- 陣列的 `filter(函式)` 回傳函式回答 `true` 的元素組成的新陣列。
- 規定清單至少有一則筆記，程式就不用處理「沒有選到任何筆記」。
