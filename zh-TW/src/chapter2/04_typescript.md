# TypeScript 是什麼

## 本集目標

知道 TypeScript 是「加了型別的 JavaScript」，型別只在寫程式時檢查、執行時並不存在，並學會讀 `tsc` 報的型別錯誤。

## 正文

### JavaScript 與 TypeScript

瀏覽器（也就是 WebView）只會執行 JavaScript。JavaScript 是動態型別的語言：同一個變數先放數字、再放字串，都合法，型別錯了要等到程式跑到那一行才知道。

TypeScript 是 JavaScript 加上型別（type）。它是 JavaScript 的超集（superset）：JavaScript 能寫的它都能寫，再多出一套型別的寫法，和一個叫 `tsc` 的檢查程式，會在你執行之前把型別對不上的地方挑出來。

跟 Rust 比起來，兩個地方很像：都是靜態型別，型別寫在名字後面加冒號，編譯器也會推論型別。差別有三個：

- TypeScript 的型別只活在寫程式的時候。程式交給瀏覽器前，型別會被整個拿掉，執行時看不到任何型別資訊。
- 沒有所有權與借用。
- 型別檢查是可以被繞過的。有個型別叫 `any`，意思是「這個值別檢查了」，用了就等於放棄保護。本書不用它。

### 型別註記與推論

```ts,ignore
let count: number = 1;
let title = "第 2 章";
```

`let` 宣告一個變數，細節第 5 集再講。第一行的 `: number` 是型別註記（type annotation），寫在名字後面，跟 Rust 一樣。第二行沒寫型別，TypeScript 看到右邊是字串，自己推論 `title` 是 `string`，之後整個程式裡 `title` 都只能放字串。

最常見的三個基本型別：

| Rust | TypeScript |
| --- | --- |
| `i32`、`f64` 等數字型別 | `number`：整數和小數都是它 |
| `String`、`&str` | `string` |
| `bool` | `boolean` |

### 型別錯誤長什麼樣子

把 `src/main.ts` 換成這份，故意放錯值：

`src/main.ts`（故意放錯值）：

```ts
let count: number = 1;
count = "two";

let title = "第 2 章";
title = 2;
```

在專案資料夾執行 `npx tsc --noEmit`。`--noEmit` 是只檢查、不產生 JavaScript 檔：

`npx tsc --noEmit` 的輸出：

```text
src/main.ts:2:1 - error TS2322: Type 'string' is not assignable to type 'number'.

2 count = "two";
  ~~~~~

src/main.ts:5:1 - error TS2322: Type 'number' is not assignable to type 'string'.

5 title = 2;
  ~~~~~


Found 2 errors in the same file, starting at: src/main.ts:2
```

每則錯誤依序是：檔案與行列、錯誤碼 `TS2322`、說明，下面用 `~` 標出出問題的地方。第二則錯誤的 `title` 沒有寫型別，但它被推論成 `string`，所以放進數字一樣不行。Rust 的 `error[E0308]` 長得很像，讀法也一樣。

### 它不會擋住執行

這點跟 Rust 不同：型別有錯，`npm run tauri dev` 還是照跑。Vite 把 TypeScript 轉成 JavaScript 時，只是拿掉型別，不檢查，所以上面那份有錯的程式仍然會在視窗裡執行。

檢查要靠 `tsc`，有三個時機：

- 編輯器：VS Code 等編輯器內建 TypeScript 的檢查，錯的地方會直接畫紅線。
- 手動執行 `npx tsc --noEmit`。
- `npm run build`：範本裡這個 script 是 `tsc && vite build`，先檢查再打包，所以有型別錯誤就會失敗。`tauri build` 打包前會執行它（第 1 章第 6 集看過）。

看不到錯誤不代表沒錯，記得常看編輯器的紅線，或在存檔前跑一次 `npx tsc --noEmit`。

### 範本開了 strict

範本的 `tsconfig.json` 有 `"strict": true`。它是一個總開關，一次打開一整組嚴格檢查（例如不准悄悄冒出 `any`、可能是 `null` 的值要先檢查），不是單一規則。另外還單獨開了 `noUnusedLocals`（函式裡沒用到的變數算錯）。本書的程式碼都是在這個設定下通過的。把錯的地方改對：

`src/main.ts`：

```ts
let count: number = 1;
count = 2;

let title = "第 2 章";
title = "第 3 章";
```

## 重點整理

- TypeScript 是加了型別的 JavaScript，由 `tsc` 檢查，瀏覽器執行的是拿掉型別後的 JavaScript。
- 型別寫在名字後面（`let count: number`），沒寫就由右邊的值推論。
- 基本型別：`number`（整數與小數都是）、`string`、`boolean`。
- 型別錯誤有錯誤碼（如 `TS2322`），讀法跟 Rust 的編譯錯誤一樣。
- 型別錯誤不會擋住 `tauri dev`，要靠編輯器、`npx tsc --noEmit` 與 `npm run build` 才看得到。
