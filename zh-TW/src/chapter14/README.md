# 安全性

前面十三章為了先讓程式跑起來，不少地方是放寬的：範本的 CSP 是 `null`，自己寫的 command 誰都能呼叫，plugin 的 permission 能放就放。App 只在自己電腦上用，這樣沒問題；要發佈給別人，就得回頭把門一扇一扇關好。

這一章從 Tauri 的安全模型講起：前端是信不過的一端，Rust 是信得過的一端，中間隔著 IPC。我們會看 Tauri 在這條界線上設的幾道防線，一道一道用起來：capability 與 permission 決定哪個視窗能呼叫什麼，scope 限制能碰哪些路徑，替自己的 command 定義 permission，再到 CSP 與 isolation pattern。前面章節說「第 14 章會正式介紹」的東西，都在這裡兌現。
