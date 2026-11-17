# 從零開始學 Tauri

你好！這份教學寫給**讀完《[從零開始學 Rust](https://holmeslin.github.io/learning-rust-from-zero/zh-TW/)》、但還沒寫過桌面 App** 的人。Rust 你已經會了，所以我們不會再講所有權或 trait，而是把力氣花在兩件新的事情上：寫一點前端，以及用 Tauri 把前端和 Rust 接起來。

## 為什麼是 Tauri？

Tauri 讓你用網頁技術（HTML、CSS、TypeScript）畫介面，用 Rust 處理檔案、資料庫、系統功能，最後打包成 macOS、Windows、Linux 的桌面 App，甚至 Android 與 iOS App。它借用作業系統內建的 WebView，不必把整個瀏覽器塞進安裝檔，所以 App 小、啟動快；權限也預設關閉，要用什麼功能得明確打開。

本教學以 **Tauri 2.12**、**Rust 1.99（edition 2024）**、Node.js LTS 與 npm 為準，前端只用 TypeScript，不用任何框架。

## 這份教學怎麼讀

- 每一「集」只教一個小概念，篇幅很短，請照順序讀。
- 第一部是基礎：第 1 章先補足寫 Tauri 需要的前端知識，之後一章一個 Tauri 主題。
- 第二部是實戰：筆記 App、SQLite 待辦清單、Markdown 編輯器、系統匣小工具，由簡單到複雜。
- 第三部是上線：安全性、打包、自動更新、用 GitHub Actions 發佈，最後把 App 搬到手機上。
- 書中的範例都從同一個起始專案出發（第 2 章會帶你建立），貼出來的檔案就是完整內容，可以直接覆蓋。

---

本教學的初稿由 AI 協助撰寫，再經人工審閱修改。
