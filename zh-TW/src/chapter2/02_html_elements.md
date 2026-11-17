# HTML 元素與屬性

## 本集目標

看懂 HTML 的元素（element）與屬性（attribute）怎麼寫，並自己搭出一個有清單、圖片、連結和輸入框的頁面。

## 正文

### 先換一個頁面

把 `index.html` 整個換成下面這份。這一集只動這一個檔案，`src/main.ts` 和 `src/styles.css` 維持範本原樣：

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
      <h1>今天要做的事</h1>
      <p>這一頁只用 HTML 搭出結構，還不會動。</p>

      <ul id="todo-list">
        <li>讀完第 2 章</li>
        <li class="done">安裝 Node.js</li>
      </ul>

      <img src="/src/assets/tauri.svg" class="logo tauri" alt="Tauri logo" />
      <a href="https://tauri.app" target="_blank">Tauri 官網</a>

      <div class="row">
        <input id="todo-input" placeholder="新增一件事" />
        <button type="button">新增</button>
      </div>
    </main>
  </body>
</html>
```

存檔後視窗會更新：標題、一段文字、兩行清單、Tauri 的 logo、一個連結，最下面是輸入框和按鈕。接下來拆開看它怎麼寫。

### 元素：起始標籤、內容、結束標籤

```html
<p>這一頁只用 HTML 搭出結構，還不會動。</p>
```

HTML 的基本單位是**元素**。`<p>` 是起始標籤（start tag），`</p>` 是結束標籤（end tag），中間是內容，三個合起來是一個元素。`p` 是段落（paragraph），`h1` 是最大的標題，`li` 是清單的一項。

元素可以放在另一個元素裡面：`<ul>` 裡有兩個 `<li>`，`<body>` 裡有 `<main>`，`<main>` 裡又有一堆東西。縮排只是方便人看，瀏覽器不在乎，但照巢狀的深度縮排，結構一目了然。整份頁面就是一棵由外往內的樹，第 7 集會用到這個樣子。

### 沒有內容的元素

`<img>` 和 `<input>` 沒有「中間」可以放內容，所以沒有結束標籤，只有一個標籤，寫成 `<img ... />`。圖片的位置、輸入框的提示文字，都改放在屬性裡。

### 屬性：寫在起始標籤裡的設定

```html
<img src="/src/assets/tauri.svg" class="logo tauri" alt="Tauri logo" />
```

屬性的格式是 `名稱="值"`，寫在起始標籤裡，用空白隔開。這一行有三個屬性：

- `src`：圖片檔的位置。
- `class`：分類名稱。值可以有好幾個，用空白隔開，這裡是 `logo` 和 `tauri` 兩個。
- `alt`：圖片的文字說明。圖片載入失敗或使用者用螢幕閱讀器時會用到。

頁面裡還有這幾個：

- `href`：連結要去的網址，用在 `<a>`。`target="_blank"` 表示在新的地方開。範本裝了 opener plugin，所以在 Tauri 的視窗裡，這類連結會交給系統的瀏覽器開，第 7 章會正式介紹 opener。
- `type`：`<button type="button">` 是一般按鈕。範本的按鈕是 `type="submit"`，會送出所在的表單，第 9 集再說。
- `placeholder`：輸入框空著時顯示的提示文字。
- `lang`：寫在 `<html>` 上，說明整份網頁的語言。

### `id` 與 `class`

這兩個屬性不影響外觀，作用是替元素取名字，讓別人找得到它：

- `id`：這個元素獨有的名字，一個頁面裡不能重複。例如 `todo-input` 就是那個輸入框。
- `class`：一類元素共用的名字，可以重複，也可以一個元素掛好幾個。

第 3 集的 CSS 靠它們挑出要換外觀的元素，第 7 集的 TypeScript 靠它們找到要操作的元素。

### 常見的元素

| 元素 | 用途 |
| --- | --- |
| `h1` | 最大的標題 |
| `p` | 段落 |
| `a` | 連結 |
| `img` | 圖片 |
| `ul`、`li` | 清單，以及清單的項目 |
| `button`、`input` | 按鈕、輸入框 |
| `div`、`main` | 容器：`div` 沒有特別意義，只拿來把元素分組；`main` 標示頁面的主要內容 |

### 整份網頁的骨架

最外層是 `<html>`，裡面分兩塊：`<head>` 放關於這份頁面的資訊，例如標題、字元編碼、要載入的 CSS 與 TypeScript，不會顯示在畫面上；`<body>` 放使用者看得到的東西。第一行的 `<!doctype html>` 只是告訴瀏覽器「這是現在的 HTML 格式」。

## 重點整理

- 元素由起始標籤、內容、結束標籤組成，可以巢狀擺放，整份頁面是一棵樹。
- `<img>`、`<input>` 沒有內容，只有一個標籤。
- 屬性寫在起始標籤裡，格式是 `名稱="值"`。
- `id` 是獨一無二的名字，`class` 是可重複的分類名稱，CSS 和 TypeScript 都靠它們找元素。
- `<head>` 放頁面的資訊，`<body>` 放看得到的內容。
