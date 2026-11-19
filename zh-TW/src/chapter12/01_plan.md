# 規劃

## 本集目標

決定 Markdown 編輯器要做什麼、要記住哪些狀態、程式怎麼分工，並先用 HTML 與 CSS 把畫面排出來。

## 正文

### 要做什麼

這個 App 要能：

- 左邊寫 Markdown，右邊即時看到轉好的結果。
- 轉換在 Rust 端做，用 pulldown-cmark（第 2 集）。
- 開啟 `.md` 檔、儲存、另存新檔。
- 停下來一秒鐘，就自動儲存。
- 還沒存的內容，在關視窗之前先問一聲。
- 在檔案總管或 Finder 裡雙擊 `.md` 檔，用這個 App 開啟（第 7 集）。

畫面是上中下三塊：最上面一排按鈕和「自動儲存」的勾選框，中間左右兩欄，最下面一行狀態列，顯示儲存、開啟的結果，出錯時也在這裡說明。

### 要記住哪些狀態

編輯器一次只開一份文件：就是編輯區裡的內容，它可能對應一個檔案，也可能還沒存過。要記的只有兩樣：

- `path`：這份文件對應的檔案路徑，還沒存過就是 `null`，視窗標題會顯示「未命名」。
- `savedText`：最後一次從檔案讀進來、或寫進檔案的內容。

編輯區的文字不另外存一份：`<textarea>` 的 `value` 就是答案。「有沒有沒存的修改」不用旗標去記，直接比較：編輯區現在的文字跟 `savedText` 不一樣，就是還沒存。這樣打了字又刪回原樣，就不會被當成有修改。

後面幾集會用到這幾條規則，先在這裡說定：

- 沒有 `path` 的文件不會自動儲存，因為不知道該存到哪裡。要先按一次「儲存」，選好檔名與位置。
- 自動儲存開著的時候，有 `path` 的文件，「沒存」的狀態只會持續到停手後的一秒鐘；沒有 `path` 的文件，會一直是「沒存」，直到你存檔。
- 要關視窗（或用別的檔案取代編輯區）的時候，只要有「沒存」，就先問。

### 為什麼檔案讀寫放在 Rust 端

第 10 章的筆記 App 用 fs plugin，在前端直接讀寫檔案。這一章不這樣做，原因在第 7 集：雙擊 `.md` 檔時，路徑是作業系統交給 App 的，不是使用者在 dialog 裡選的。第 7 章第 5 集說過，fs plugin 預設只放行 app 資料夾，加上 dialog 選出來的路徑，所以這種路徑前端讀不到。Rust 端用標準函式庫讀檔，不受 fs plugin 的 scope 限制。

所以這一章的 dialog 只負責「問使用者要哪個檔案」，拿到路徑後，讀、寫都交給兩個我們自己寫的 command。這兩個 command 會讀寫前端給的任何路徑，對只在自己電腦上用的編輯器沒問題，要發佈給別人用的話就該限制範圍，第 14 章會正式介紹。

轉 Markdown 也放在 Rust 端：哪些語法要支援、檔案裡夾的 HTML 與連結的網址怎麼處理，規則全部集中在一個地方，前端只負責把結果放進畫面。

### 程式怎麼分工

- `src-tauri/src/markdown.rs`：`to_html`，Markdown 進去、HTML 出來。不碰 Tauri 的型別，所以可以直接寫測試。
- `src-tauri/src/lib.rs`：command、plugin，還有第 7 集的檔案關聯。
- `src/commands.ts`：包住 `invoke` 的幾個函式，跟第 11 章的 `todos.ts` 一樣。
- `src/main.ts`：畫面與文件的狀態。

### 先排畫面

這一集只做靜態的畫面：左邊的 Markdown 和右邊的結果都是寫死的，`src/main.ts` 與 `lib.rs` 維持範本原樣，按任何按鈕都不會有反應。

`<textarea>` 的內容寫在起始與結束標籤中間（第 10 章說過），所以範例文字緊貼著 `>`，換行都算內容。`spellcheck="false"` 是 HTML 的屬性，關掉瀏覽器對拼字的紅色底線，寫 Markdown 時那些底線只會礙眼。`<header>` 與 `<article>` 是沒有特別外觀的容器元素，名字只是說明用途：工具列與一篇內容。

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Markdown 編輯器</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <header>
      <button id="new-btn" type="button">新增</button>
      <button id="open-btn" type="button">開啟</button>
      <button id="save-btn" type="button">儲存</button>
      <button id="save-as-btn" type="button">另存新檔</button>
      <label><input id="autosave" type="checkbox" checked /> 自動儲存</label>
    </header>
    <main>
      <textarea id="editor" spellcheck="false"># 你好，Markdown

這是 **粗體**，這是[連結](https://tauri.app)。

- 第一項
- 第二項</textarea>
      <article id="preview">
        <h1>你好，Markdown</h1>
        <p>這是 <strong>粗體</strong>，這是<a href="https://tauri.app">連結</a>。</p>
        <ul>
          <li>第一項</li>
          <li>第二項</li>
        </ul>
      </article>
    </main>
    <p id="status">這裡顯示儲存、開啟的結果與錯誤</p>
  </body>
</html>
```

排版用的是第 2 章第 3 集的 flex：

- `body` 是由上往下排的 flex，`height: 100vh` 讓它剛好是視窗那麼高。工具列、中間、狀態列三塊裡，只有 `main` 設了 `flex: 1`，它吃掉剩下的高度。
- `main` 本身是 flex，左右並排；`min-height: 0` 不能省：flex 的子元素預設不肯縮得比內容還矮，文字一多，`main` 就會把整個頁面撐高。設成 `0`，內容太長時才會在 `main` 裡面捲動。
- `#editor` 與 `#preview` 都是 `flex: 1`，各佔一半寬。`min-width: 0` 跟第 11 章第 1 集的輸入框一樣，讓它們在很窄的視窗裡也能縮；`box-sizing: border-box` 讓 `padding` 算在寬度之內，不然兩欄加起來會比視窗寬；`overflow: auto` 讓內容太長時出現捲軸。
- `#editor` 用等寬字型，`resize: none` 拿掉右下角拖曳改大小的把手。

`src/styles.css`：

```css
body {
  display: flex;
  flex-direction: column;
  height: 100vh;
  margin: 0;
  font-family: sans-serif;
}

header,
#status {
  padding: 8px;
  background-color: #eee;
}

header {
  display: flex;
  align-items: center;
  gap: 8px;
}

main {
  flex: 1;
  display: flex;
  min-height: 0;
}

#editor,
#preview {
  flex: 1;
  min-width: 0;
  box-sizing: border-box;
  margin: 0;
  padding: 12px;
  overflow: auto;
}

#editor {
  border: none;
  border-right: 1px solid #ddd;
  resize: none;
  font-family: monospace;
  font-size: 14px;
}

#preview pre {
  padding: 8px;
  overflow: auto;
  background-color: #f4f4f4;
}

#preview table {
  border-collapse: collapse;
}

#preview th,
#preview td {
  padding: 4px 8px;
  border: 1px solid #ccc;
}

#status {
  margin: 0;
  min-height: 1.5em;
  font-size: 12px;
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/tauri-app`
```

視窗標題還是 `tauri-app`，大小 800×600，這兩樣到第 8 集才改。上面一排按鈕，中間左右兩欄一樣寬，左邊是 Markdown 原文，右邊是排好的標題、粗體、連結和清單，最下面是那一行狀態列的字。現在點什麼都沒有反應，因為還沒有任何程式。

## 重點整理

- 文件的狀態只有 `path`（對應的檔案，沒存過是 `null`）和 `savedText`（最後一次讀進來或寫出去的內容）；有沒有沒存的修改，是比較編輯區的文字與 `savedText`，不另外記旗標。
- 沒有 `path` 的文件不自動儲存；有沒存的內容，在關視窗、換檔案之前要先問。
- 檔案讀寫與 Markdown 轉換都放在 Rust 端；dialog 只負責選路徑，因為第 7 集的路徑來自作業系統，不在 fs plugin 的 scope 內。
- 檔案分工：`markdown.rs` 轉換，`lib.rs` 放 command 與 plugin，`commands.ts` 包住 `invoke`，`main.ts` 管畫面與文件狀態。
- 畫面是上中下三塊的 flex，中間兩欄各佔一半，`min-height: 0` 讓內容太長時在欄裡捲動，不撐高整個頁面。
