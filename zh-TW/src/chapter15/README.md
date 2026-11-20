# 打包與安裝檔

前面十四章，App 都是用 `tauri dev` 在自己的電腦上跑。要給別人用，得把它做成能安裝的檔案：macOS 是 `.app` 與 `.dmg`，Windows 是 MSI 或 NSIS 的安裝程式，Linux 是 AppImage、deb、rpm。這個動作叫打包（bundle），指令是 `npm run tauri build`。

這一章先看這個指令做了什麼、怎麼做出 debug 版、怎麼替 release 版打開開發者工具；再換掉預設圖示，整理 `tauri.conf.json` 的 `bundle` 與版本號；然後分平台看各種安裝檔，最後把資源檔一起打包進去。程式碼簽章、公證與自動更新留到第 16 章，用 GitHub Actions 一次打三個平台的安裝檔在第 17 章。前面章節說「第 15 章會正式介紹」的東西，都在這裡兌現。
