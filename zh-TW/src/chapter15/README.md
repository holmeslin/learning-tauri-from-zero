# 打包與安裝檔

前面十四章，App 都是用 `tauri dev` 在自己的電腦上跑。要給別人用，得把它做成能安裝的檔案：macOS 是 `.app` 與 `.dmg`，Windows 是 MSI 或 NSIS 的安裝檔，Linux 是 AppImage、deb、rpm。這個動作叫打包（bundle），指令是 `npm run tauri build`。

打包要面對的是每個平台各自的規矩：圖示要哪些尺寸、版本號能怎麼寫、安裝檔有哪幾種、資源檔放進去之後要怎麼找。這一章把這些整理清楚，做出能交給別人安裝的檔案。程式碼簽章、公證與自動更新留到第 16 章，用 GitHub Actions 一次打三個平台的安裝檔在第 17 章。
