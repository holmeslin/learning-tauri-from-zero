# 實戰：Markdown 編輯器

這一章做一個 Markdown 編輯器：左邊寫 Markdown，右邊即時看到排好的結果。能開啟、儲存檔案，停下來一秒鐘就自動儲存；還沒存的內容在關視窗前會先問一聲；在檔案總管或 Finder 裡雙擊 `.md` 檔，也能直接用這個 App 開啟。

前面兩章，一章把檔案讀寫交給前端的 fs plugin，一章把資料庫放在 Rust 端。這一章走 Rust 這邊：Markdown 轉 HTML 用 pulldown-cmark（第 2 集），檔案讀寫也寫成自己的 command（第 4 集）。為什麼這樣分，第 1 集會說明。這一章會用到 command 與事件（第 3、5 章）、視窗事件（第 6 章）、dialog plugin（第 7 章）和 single-instance（第 8 章），最後多學一個新東西：讓作業系統把檔案交給你的 App。
