# 規劃

## 本集目標

決定「快速便條」要做什麼、資料各放在哪裡、程式怎麼分工，並先用 HTML 與 CSS 把便條的畫面排出來。

## 正文

### 要做什麼

「快速便條」是一個平常看不到視窗的小工具。想到什麼，按一個快捷鍵，小視窗跳出來，寫幾個字；再按一次，它又收起來。它要做到：

- 啟動時不顯示視窗，只在系統匣放一個圖示；關掉視窗不是結束 App，只是把視窗藏起來（第 2 集）。
- 系統匣圖示的選單有「顯示便條」「設定…」「結束」（第 3 集放前面與最後一項，「設定…」第 5 集才加）。
- 全域快捷鍵 `CommandOrControl+Alt+Shift+N` 叫出或收起便條視窗（第 4 集）。
- 設定視窗可以改便條的字型大小，存在 store（第 5 集）。
- 設定視窗有一個「開機時自動啟動」的勾選框（第 6 集）。

### 兩個視窗

- `main`：便條視窗，裡面一個 `<textarea>`。它寫在 `tauri.conf.json` 裡，App 啟動就建好，只是不顯示（第 2 集）。
- `settings`：設定視窗，要用的時候才由 Rust 建（第 6 章第 3 集）。

兩個視窗載入同一頁 `index.html`，頁面用 `getCurrentWindow().label` 分辨自己是誰，做法跟第 6 章第 3 集一樣。

### 資料各放在哪裡

這個 App 要記住三樣東西，每一樣都只放在一個地方：

| 資料 | 放在哪裡 |
| --- | --- |
| 便條的文字 | 便條視窗頁面裡的 `<textarea>` |
| 字型大小 | store 的 `settings.json`，key 是 `fontSize`（第 5 集） |
| 要不要開機自動啟動 | 作業系統（第 6 集），我們不另外存一份 |

便條的文字有一個限制：藏起來的視窗，頁面還活著，所以再叫出來，文字都還在；但 App 一結束，文字就沒了。它是便條，不是筆記 App。要存下來的話，第 10 章做過檔案的版本，第 7 集的「還能怎麼改」也會提到。

開機自動啟動是作業系統在管的事，要知道現在有沒有開，就去問作業系統，不要自己再記一份。兩份資料不一致的時候，會不知道該相信哪一份。

### 程式怎麼分工

- `src-tauri/src/lib.rs`：視窗、系統匣、快捷鍵、plugin，全部都在這裡。這個 App 沒有自己的 command，前端不需要呼叫 Rust。
- `src/main.ts`：看自己是哪個視窗，決定要畫便條還是設定。
- `src/settings.ts`：讀寫 `fontSize`（第 5 集）。

### 先排便條的畫面

這一集只排便條視窗的畫面，`lib.rs` 與 `main.ts` 維持範本的樣子。便條就是一個占滿整個視窗的 `<textarea>`：`placeholder` 是還沒打字時顯示的灰色提示，`spellcheck="false"` 關掉拼字檢查的紅色底線。

`index.html`：

```html
<!doctype html>
<html lang="zh-TW">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>快速便條</title>
    <link rel="stylesheet" href="/src/styles.css" />
    <script type="module" src="/src/main.ts" defer></script>
  </head>

  <body>
    <main id="note-view">
      <textarea id="note" placeholder="想到什麼，寫下來" spellcheck="false"></textarea>
    </main>
  </body>
</html>
```

排版跟第 12 章第 1 集的 `#editor` 差不多：`#note-view` 用 `height: 100vh` 撐滿視窗，`<textarea>` 填滿它。`box-sizing: border-box` 讓 `padding` 算在寬高之內，不然 `100%` 加上 `padding` 會比視窗大；`resize: none` 拿掉右下角拖曳改大小的把手；`outline: none` 拿掉取得焦點時的藍色外框，整個視窗都是輸入區，不需要它。`<textarea>` 預設用等寬字型，這裡改成 `inherit`，跟 `body` 的 `sans-serif` 一樣。

`src/styles.css`：

```css
body {
  margin: 0;
  font-family: sans-serif;
}

#note-view {
  height: 100vh;
}

#note {
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 12px;
  border: none;
  outline: none;
  resize: none;
  font-family: inherit;
  font-size: 16px;
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.02s
     Running `target/debug/tauri-app`
```

視窗標題還是 `tauri-app`，大小 800×600，這兩樣到第 2 集才改。整個視窗是一塊白色的輸入區，還沒打字的時候顯示灰色的「想到什麼，寫下來」。現在能打字，但沒有任何程式，也還沒有系統匣與快捷鍵。

## 重點整理

- 快速便條是一個平常藏起來的小工具：系統匣圖示與全域快捷鍵是入口，視窗只在需要時出現。
- 兩個視窗：`main` 是便條，寫在設定檔；`settings` 是設定，要用才建。兩個視窗載入同一個 `index.html`，用 `label` 分辨。
- 每一份資料只放一個地方：便條文字在 `<textarea>`、字型大小在 store、開機自動啟動交給作業系統。
- 這個 App 沒有自己的 command，程式幾乎都在 `lib.rs`，前端只畫兩個很薄的畫面。
- 便條的文字只留在頁面裡，結束 App 就沒了。
