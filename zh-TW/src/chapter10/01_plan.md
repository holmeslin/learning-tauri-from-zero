# 規劃畫面與資料

## 本集目標

決定筆記 App 的畫面、資料和程式分工，並先用 HTML 與 CSS 把畫面排出來。

## 正文

### 要做什麼

這個 App 要能：

- 左邊列出所有筆記，點一則，右邊就能編輯。
- 新增、編輯、刪除筆記。
- 筆記存在 App 資料夾，關掉再開還在。
- 把一則筆記匯出成 `.md` 檔，或從 `.md` 檔匯入。
- 出錯時（檔案壞了、寫不進去）在畫面上說明，不默默吞掉。

畫面分成兩欄：

- 左欄：新增、匯入兩個按鈕，下面是筆記清單。
- 右欄：標題輸入框、內容輸入框、匯出與刪除兩個按鈕，最下面一行是狀態列，顯示成功或出錯的訊息。

### 資料：一則筆記

一則筆記只有標題 `title` 和內容 `body` 兩個字串欄位，型別 `Note` 寫在後面的 `src/notes.ts`。整份資料是 `Note[]`，存成 App 資料夾裡的一個 JSON 檔 `notes.json`：整份讀進來、整份寫回去，最單純。檔案會長這樣：

```json,ignore
[
  {
    "title": "購物清單",
    "body": "牛奶\n雞蛋"
  }
]
```

這裡刻意不替筆記編號。`===` 用在兩個物件上，比的是「是不是同一個物件」，不是欄位的值一不一樣。所以程式只要記住「選到的那個物件」，用 `===` 就能找出它，兩則內容一模一樣的筆記也不會搞混。筆記只有這個 App 自己用，不需要 `id`。

### 程式放在哪

這個專案不寫自己的 command。讀寫檔案有 fs plugin，選檔案有 dialog plugin，前端直接呼叫就行（第 7 章）；筆記的邏輯只是在整理一個陣列，放在前端最省事。Rust 這邊只要註冊 plugin、放行 permission。資料多到整份讀寫太慢時，要改用資料庫，第 11 章會正式介紹。

前端分成幾個檔案：

- `index.html`、`src/styles.css`：畫面。
- `src/notes.ts`：`Note` 型別，之後加上讀寫檔案的函式。它不碰畫面。
- `src/main.ts`：畫面的程式，把使用者的操作接到 `notes.ts`。

接下來每一集接上一個零件：清單、新增與編輯、存檔、匯入匯出、錯誤處理，最後收尾。

### 先排畫面

這一集只做靜態的畫面，清單裡的字是寫死的。`src/main.ts` 用不到，維持範本原樣，下一集會換掉。

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
      <ul id="note-list">
        <li class="selected">購物清單</li>
        <li>讀書計畫</li>
        <li>（沒有標題）</li>
      </ul>
    </div>
    <main>
      <input id="title-input" placeholder="標題" value="購物清單" />
      <textarea id="body-input" placeholder="內容">牛奶、雞蛋</textarea>
    </main>
  </body>
</html>
```

`<textarea>` 是多行的輸入框。它跟 `<input>` 一樣有 `placeholder`，但內容寫在起始與結束標籤中間，不用 `value`。

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

排版用的是第 2 章第 3 集的 flex：

- `body` 設成 `display: flex`，左欄、右欄並排。`height: 100vh` 的 `vh` 是視窗高度的百分之一，所以 `100vh` 是整個視窗的高度；`margin: 0` 去掉 `body` 預設的邊距。
- `.sidebar` 固定 200px 寬，清單太長時 `overflow-y: auto` 會出現捲軸。
- `main` 用 `flex: 1` 吃掉剩下的寬度，自己再用 `flex-direction: column` 由上往下排。
- `#body-input` 的 `flex: 1` 讓內容輸入框撐滿右欄剩下的高度。
- `.selected` 是選到的那則的底色，`cursor: pointer` 讓滑鼠移上去變成手指。

資料的形狀放進 `notes.ts`。`export` 之後，別的檔案才能 `import` 它（第 2 章第 11 集）：

`src/notes.ts`：

```ts
export interface Note {
  title: string;
  body: string;
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.30s
     Running `target/debug/tauri-app`
```

視窗左邊是灰底的欄，裡面有三則筆記，第一則是藍底；右邊是標題輸入框和內容輸入框，內容框一路撐到視窗底部。現在點什麼都沒有反應，因為還沒有任何程式。

## 重點整理

- 筆記 App 的資料是 `Note[]`，一則筆記只有 `title` 與 `body`，整份存成 App 資料夾裡的 `notes.json`。
- 這個專案不寫自己的 command：讀寫檔案與選檔案都由前端呼叫 fs、dialog plugin。
- 檔案分工：`notes.ts` 放型別與讀寫檔案的函式，`main.ts` 負責畫面，兩者分開。
- 畫面用 flex 排成左右兩欄，先用寫死的內容確認版面，下一集再讓清單由資料畫出來。
