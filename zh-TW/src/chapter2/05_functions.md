# `let`、`const` 與函式、箭頭函式

## 本集目標

學會用 `let` 與 `const` 宣告變數，並寫出有型別的函式與箭頭函式，也知道函式可以當成值傳給別的函式。

## 正文

### `const` 與 `let`

```ts,ignore
const limit = 3;
let total = 0;
total = total + limit;
```

- `const`：名字綁定之後，不能再指向別的值。
- `let`：之後可以重新指定別的值。

這跟 Rust 剛好相反：Rust 預設不可變，要改才加 `mut`；TypeScript 每個變數都得二選一，慣例是**優先用 `const`**，真的要重新指定值才改成 `let`。還有一個更老的 `var`，它的規則比較亂，不用就好。

### 函式

```ts,ignore
function add(a: number, b: number): number {
  return a + b;
}
```

跟 Rust 的 `fn add(a: i32, b: i32) -> i32` 對照：關鍵字是 `function`，參數型別寫在名字後面，回傳型別接在括號後面的冒號，要回傳值就**一定要寫 `return`**，沒有「最後一行就是回傳值」這回事。參數的型別一定要寫；回傳型別省略的話會被推論，但本書自己寫的函式宣告（function declaration，不含下面的箭頭函式）都會把它寫出來。範本的 `greet` 是例外，它沒有寫。

函式如果不回傳任何值，回傳型別就寫 `void`，意思是「沒有值」，對應 Rust 的 `()`，函式裡也不用 `return`。底下完整範例的最後有一個 `reset`。

### 箭頭函式

```ts,ignore
const double = (n: number) => n * 2;
const isLimit = (n: number) => n === limit;
```

箭頭函式（arrow function）是更短的寫法：`(參數) => 結果`。`=>` 右邊如果是一個運算式，它的值就是回傳值，不用寫 `return`。要寫多行就用大括號，這時要自己寫 `return`：

```ts,ignore
const add = (a: number, b: number) => {
  return a + b;
};
```

`n === limit` 是比較兩個值相不相等。JavaScript 有 `==` 和 `===` 兩種，`==` 會先偷偷轉換型別再比，結果常出人意料，所以一律用三個等號 `===`（不相等是 `!==`）。

跟 Rust 的閉包（closure）一樣，箭頭函式可以用到外面的變數：`isLimit` 就用了外面的 `limit`。

### 函式也是值

函式可以放進變數，也能當成參數傳給另一個函式。這種被傳進去、等著被別人呼叫的函式叫 callback。它的型別寫成 `(n: number) => number`，讀作「接收一個 `number`，回傳 `number` 的函式」。第 8 集的事件處理，就是把函式交給瀏覽器，在使用者按下按鈕時呼叫。

### 先故意寫錯

把 `src/main.ts` 換成這份，裡面有兩個錯：對 `const` 重新指定值，以及傳錯型別的參數：

`src/main.ts`（故意放錯值）：

```ts
const limit = 3;
limit = 4;

function add(a: number, b: number): number {
  return a + b;
}

const total = add("1", 2);
```

`npx tsc --noEmit` 的輸出：

```text
src/main.ts:2:1 - error TS2588: Cannot assign to 'limit' because it is a constant.

2 limit = 4;
  ~~~~~

src/main.ts:8:19 - error TS2345: Argument of type 'string' is not assignable to parameter of type 'number'.

8 const total = add("1", 2);
                    ~~~


Found 2 errors in the same file, starting at: src/main.ts:2
```

JavaScript 本身會讓 `add("1", 2)` 照跑，算出字串 `"12"`；TypeScript 在執行前就擋下來。

### 改成正確的完整範例

再把 `src/main.ts` 換成這份。這集不會在畫面上看到東西，只要通過型別檢查就好：

`src/main.ts`：

```ts
const limit = 3;
let total = 0;
total = total + limit;

function add(a: number, b: number): number {
  return a + b;
}

const double = (n: number) => n * 2;
const isLimit = (n: number) => n === limit;

function applyTwice(f: (n: number) => number, x: number): number {
  return f(f(x));
}

const sum = add(total, 1); // 4
const twice = applyTwice(double, 5); // 20
const atLimit = isLimit(total); // true
const message = `上限 ${limit}，目前 ${total}，加一是 ${sum}`; // 上限 3，目前 3，加一是 4

function reset(): void {
  total = 0;
}

reset(); // total 回到 0
```

最後一行用反引號包住字串，裡面的 `${...}` 會換成運算式的值，這是範本字串（template literal），就像 Rust 的 `format!("{x}")`。註解寫的是各個變數的值。

## 重點整理

- `const` 的名字不能再指向別的值，`let` 可以；預設用 `const`。
- 函式寫成 `function add(a: number, b: number): number { ... }`，要回傳值就明確 `return`；不回傳值的函式，回傳型別是 `void`。
- 箭頭函式 `(n: number) => n * 2` 是更短的寫法，運算式的值就是回傳值。
- 函式是值，可以放進變數或當 callback 傳給別的函式，型別寫成 `(n: number) => number`。
- 比較用 `===` 與 `!==`，字串內插用反引號加 `${...}`。
