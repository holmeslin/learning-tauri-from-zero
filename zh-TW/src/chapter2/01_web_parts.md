# 網頁由什麼組成

## 本集目標

知道一個網頁由 HTML、CSS、TypeScript 三層組成，各自管什麼，以及瀏覽器怎麼把三個檔案組成畫面。

## 正文

### 三個檔案，三種工作

第 1 章看過，前端只有三個檔案值得注意。它們各管一件事：

| 層 | 負責 | 範本裡的檔案 |
| --- | --- | --- |
| HTML | 結構與內容：頁面上有哪些東西 | `index.html` |
| CSS | 外觀：顏色、大小、位置 | `src/styles.css` |
| TypeScript | 行為：按下去、輸入之後要做什麼 | `src/main.ts` |

HTML（HyperText Markup Language）是一份用文字描述的結構：這裡有一個標題、那裡有一個按鈕。CSS（Cascading Style Sheets）是一組規則，規定這些東西長什麼樣子。這兩個都只是「描述」，不是程式語言。真正會執行的只有第三層：TypeScript 是程式語言，能回應使用者的操作，還能改頁面上的內容。

用 Rust 的角度看：HTML 像資料，CSS 像「這份資料要怎麼畫出來」的規則，TypeScript 是處理資料的程式。

### 瀏覽器怎麼把它們組起來

視窗裡的 WebView 就是一個瀏覽器（第 1 章第 1 集）。它先載入 `index.html`，從上讀到下。範本的 `<head>` 裡有這兩行：

```html
<link rel="stylesheet" href="/src/styles.css" />
<script type="module" src="/src/main.ts" defer></script>
```

讀到 `<link>`，瀏覽器去抓 `src/styles.css` 當作外觀規則；讀到 `<script>`，去抓 `src/main.ts`，等整份 HTML 讀完才執行，為什麼是這個順序，第 7 集再說。檔案怎麼從開發伺服器送到瀏覽器，第 11 集再說。

`main.ts` 是 TypeScript，瀏覽器不能直接執行，中間會先轉換，細節第 4 集再說。

結構、外觀、行為分在三個檔案，改一層不會動到另外兩層：想換顏色只改 CSS，想多一個按鈕只改 HTML，想改按下去之後的反應只改 TypeScript。

### 拿掉一層看看

讓 `npm run tauri dev` 開著，打開 `index.html`，把載入 CSS 的那一行用 HTML 的註解 `<!-- -->` 包起來。註解裡的東西瀏覽器會忽略：

```html
<!-- <link rel="stylesheet" href="/src/styles.css" /> -->
```

存檔，頁面會變成完全沒有外觀的樣子：標題、三個 logo、輸入框和按鈕都還在，但 logo 變回圖片原本的大小，有的大到占滿視窗，文字也全部靠左。結構還在，外觀不見了。

把這一行復原，改把載入 TypeScript 的那一行包起來：

```html
<!-- <script type="module" src="/src/main.ts" defer></script> -->
```

這次畫面看起來跟原本一樣，因為 HTML 和 CSS 都沒動。但輸入名字再按 Greet，不會出現問候語，因為處理按鈕的程式沒有載入。把這行也復原，頁面就恢復正常。

## 重點整理

- 網頁由三層組成：HTML 管結構與內容，CSS 管外觀，TypeScript 管行為。
- HTML 與 CSS 只是描述，TypeScript 才是會執行的程式語言。
- 瀏覽器讀 `index.html`，遇到 `<link>` 載入 CSS，遇到 `<script>` 載入程式，等 HTML 讀完再執行。
- 三層各自獨立，拿掉任何一層，其他兩層還是在。
