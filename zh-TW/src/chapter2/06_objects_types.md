# 物件、陣列、`type` 與 `interface`

## 本集目標

學會用物件和陣列放資料，用 `interface` 與 `type` 描述資料的形狀，並處理「可能沒有值」的型別。

## 正文

### 物件與 `interface`

JavaScript 的物件用大括號直接寫出來，裡面是一個個 `欄位名: 值`（欄位就是物件的 property）：

```ts,ignore
const note = { id: 1, title: "讀完第 2 章", done: false };
```

在 TypeScript 裡，用 `interface` 替這種物件的形狀取名字：

```ts,ignore
interface Note {
  id: number;
  title: string;
  done: boolean;
}
```

它像 Rust 的 `struct` 定義，但不用另外「建立 struct」：任何欄位和型別都對得上的物件，就是一個 `Note`。這叫結構型別（structural typing）：TypeScript 比的是形狀，也就是有哪些欄位、各是什麼型別，不看名字。Rust 是名義型別（nominal typing）：兩個欄位一模一樣、名字不同的 struct 是不同的型別，不能互相代替；TypeScript 裡，欄位相同的另一個 `interface Task` 就能當 `Note` 用。多出欄位的物件也能當 `Note` 用，只有直接寫在那個位置的新物件字面值，多寫的欄位會被當成打錯字而擋下來。

用 `.` 讀寫欄位：`note.title`、`note.done = true`。

### 陣列

```ts,ignore
const notes: Note[] = [];
notes.push({ id: 1, title: "讀完第 2 章", done: false });
```

`Note[]` 是「裝 `Note` 的陣列」，對應 Rust 的 `Vec<Note>`。`push` 在尾端加一個，`notes.length` 是長度，`for (const note of notes) { ... }` 逐一走過每個元素。

`const` 只管名字不能再指向別的值，不代表內容不能改：`const notes` 照樣可以 `push`，物件的欄位也照樣可以改。這跟 Rust 不同，Rust 的 `let notes` 沒加 `mut` 就不能 `push`。

### `type` 與聯集

`type` 能替任何型別取別名（alias），最常用在**聯集（union）**，也就是「這幾種型別之一」：

```ts,ignore
type Filter = "all" | "todo" | "done";
```

字串本身也能當型別，`Filter` 的值只能是這三個字串之一，用起來像沒有資料的 Rust `enum`。`type` 和 `interface` 描述物件形狀時差不多，本書的習慣是：物件的形狀用 `interface`，別名與聯集用 `type`。

### 可能沒有值

聯集也用來表達「可能沒有」，對應 Rust 的 `Option<T>`：寫成 `T | undefined` 或 `T | null`。`undefined` 與 `null` 都表示「沒有」，習慣上陣列的 `find` 找不到時回傳 `undefined`，第 7 集會看到網頁的 API 用 `null`。範本裡的 `HTMLInputElement | null` 就是這個寫法。

TypeScript 要你先檢查，才讓你用裡面的值。故意不檢查看看：

`src/main.ts`（故意放錯值）：

```ts
interface Note {
  id: number;
  title: string;
  done: boolean;
}

const notes: Note[] = [{ id: 1, title: "Read" }];

const found = notes.find((note) => note.id === 1);
found.done = true;
```

`npx tsc --noEmit` 的輸出：

```text
src/main.ts:7:24 - error TS2741: Property 'done' is missing in type '{ id: number; title: string; }' but required in type 'Note'.

7 const notes: Note[] = [{ id: 1, title: "Read" }];
                         ~~~~~~~~~~~~~~~~~~~~~~~~

  src/main.ts:4:3
    4   done: boolean;
        ~~~~
    'done' is declared here.

src/main.ts:10:1 - error TS18048: 'found' is possibly 'undefined'.

10 found.done = true;
   ~~~~~


Found 2 errors in the same file, starting at: src/main.ts:7
```

第一則：物件少了 `done` 欄位。第二則：`find` 的回傳型別是 `Note | undefined`，直接用 `found.done` 不行。解法是先用 `if` 檢查：

```ts,ignore
if (found) {
  found.done = true;
}
```

進了 `if (found)` 的大括號，TypeScript 就知道 `found` 不是 `undefined`，這叫型別縮窄（narrowing）。要同時檢查兩個值，可以用 `&&` 接起來：`if (a && b)`，範本的 `greet` 就是這樣寫的。

### 完整的範例

把 `src/main.ts` 換成改好的版本。這集一樣不會在畫面上看到東西，只要通過型別檢查：

`src/main.ts`：

```ts
interface Note {
  id: number;
  title: string;
  done: boolean;
}

const notes: Note[] = [
  { id: 1, title: "讀完第 2 章", done: false },
  { id: 2, title: "安裝 Node.js", done: true },
];
notes.push({ id: 3, title: "寫第一個 Tauri App", done: false });

let doneCount = 0;
for (const note of notes) {
  if (note.done) {
    doneCount += 1; // 跑完是 1
  }
}

const found = notes.find((note) => note.id === 2);
if (found) {
  found.done = false;
}
```

## 重點整理

- 物件用 `{ 欄位: 值 }` 寫，`interface` 替物件的形狀取名字，欄位用 `.` 存取。
- 陣列型別寫成 `Note[]`，常用 `push`、`length`、`for...of` 與 `find`。
- `const` 不代表內容不能改。
- `type` 能取別名，聯集 `A | B` 表示「其中一種」；字串也能當型別，像沒有資料的 `enum`。
- `T | undefined`、`T | null` 表示可能沒有值，要先用 `if` 檢查（型別縮窄）才能用。
