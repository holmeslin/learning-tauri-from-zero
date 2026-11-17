# CSS 選擇器與 flex 排版

## 本集目標

學會寫 CSS 規則，用選擇器（selector）挑出元素；再用 flex 排版把一組元素排成直的或橫的。

## 正文

### 先換兩個檔案

這一集換掉 `index.html` 和 `src/styles.css`，`src/main.ts` 維持範本原樣。頁面是一份三項的清單，加上一列輸入框和按鈕：

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

      <ul id="todo-list">
        <li>讀完第 2 章</li>
        <li class="done">安裝 Node.js</li>
        <li>寫第一個 Tauri App</li>
      </ul>

      <div class="row">
        <input id="todo-input" placeholder="新增一件事" />
        <button type="button">新增</button>
      </div>
    </main>
  </body>
</html>
```

`src/styles.css`：

```css
body {
  font-family: sans-serif;
  color: #222;
  background-color: #f6f6f6;
}

h1 {
  text-align: center;
}

.container {
  max-width: 420px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

#todo-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* .done 寫在 li 前面，但它比較具體，所以還是它贏 */
.done {
  color: #888;
  text-decoration: line-through;
}

li {
  padding: 8px 12px;
  color: #222;
  background-color: white;
  border-radius: 8px;
}

.row {
  display: flex;
  gap: 8px;
}

.row input {
  flex: 1;
}
```

存檔後，清單變成三張白色卡片，第二張是灰字加刪除線，輸入框和按鈕並排在最下面。

### 一條規則的樣子

拿 `h1` 那一條來看：`h1 { text-align: center; }`。CSS 由一條條這樣的規則組成。大括號前面是**選擇器**，說明規則套用在哪些元素；大括號裡是一行行宣告（declaration），格式是 `屬性: 值;`。這裡的屬性（property）是 CSS 的，跟上一集 HTML 元素的屬性（attribute）是兩回事。這條的意思是：所有 `h1` 的文字置中。

### 三種挑元素的方式

上一集的 `id` 和 `class` 在這裡派上用場：

| 選擇器 | 寫法 | 挑中誰 |
| --- | --- | --- |
| 元素名稱 | `li` | 所有 `<li>` |
| `class` | `.done` | 所有 `class` 裡有 `done` 的元素 |
| `id` | `#todo-list` | `id` 是 `todo-list` 的那一個 |

選擇器可以接起來。`.row input` 中間有空白，意思是「在 `.row` 裡面的 `input`」。

兩條規則設了同一個屬性時，比較具體的贏：`id` 勝過 `class`，`class` 勝過元素名稱；一樣具體才看誰寫在後面。這份 CSS 裡，第二張卡片同時符合 `.done` 和 `li`，兩條都設了 `color`，而 `li` 還寫在 `.done` 後面。如果只看順序，字會是 `#222`；但 `.done` 比 `li` 具體，所以字是灰的 `#888`。可以自己把這兩條規則的順序對調試試，字還是灰的。

### 排版：flex

預設的版面是由上往下排。把容器設成 `display: flex`，它的子元素就會改用 flex 排版，沿著一條軸線排好。這份 CSS 裡用到這幾個設定：

- `display: flex`：開啟 flex。子元素預設沿著水平方向排成一橫列。
- `flex-direction: column`：改成直的，由上往下排。`.container` 和 `#todo-list` 用它。
- `gap`：子元素之間的空隙。
- `flex: 1`：這個子元素把一列剩下的空間全部吃掉。`.row` 裡輸入框被拉長，按鈕只佔自己需要的寬度。

### 其他用到的屬性

剩下的屬性看名字就懂大半，不必背：`color` 文字顏色、`background-color` 背景色、`border-radius` 圓角、`text-decoration: line-through` 刪除線、`list-style: none` 拿掉清單的圓點。`padding` 是內容與邊框之間的空白，`margin` 是元素外面的空白；`margin: 0 auto` 讓有 `max-width` 的元素左右置中。

## 重點整理

- CSS 規則的形狀是 `選擇器 { 屬性: 值; }`。
- 選擇器用元素名稱（`li`）、`class`（`.done`）、`id`（`#todo-list`）挑元素，也可以接起來（`.row input`）。
- 同一個屬性被兩條規則設定時，`id` 勝過 `class`，`class` 勝過元素名稱，一樣就看誰在後面。
- `display: flex` 讓子元素沿著一條軸線排列，`flex-direction`、`gap`、`flex: 1` 控制方向、間距與伸縮。
