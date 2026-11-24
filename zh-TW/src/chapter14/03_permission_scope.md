# permission 與 scope

## 本集目標

讀懂一項 permission 由「放行哪些 command」加上「限制哪些路徑」（scope）組成：會查 plugin 的 permission 放行了什麼，用 `fs:scope` 的 `allow`、`deny` 限制 fs plugin 能碰的路徑（`deny` 一定優先），也分得清 scope 掛在全域和掛在單一 command 的差別。

## 正文

### permission 的組成

第 7 章已經用過 permission 與 scope，這集把它拆開來看，主線是 scope；查 permission 內容的指令、全域與 command 兩種 scope、`deny-` 開頭的 permission，都是為了用好 scope 補上的。一項 permission 由兩種東西組成，各自可以有也可以沒有：

- `commands`：放行（`allow`）或禁止（`deny`）哪些 command。
- `scope`：一份 `allow` 清單和一份 `deny` 清單，每一項是一個物件，意思由 plugin 決定。fs 的物件寫 `path`，opener 的 `allow-open-path` 也一樣（第 7 章第 3 集）。

再加上第三種：set，把好幾項 permission 打包成一個名字。每個 plugin 都有的 `default`（像 `core:default`、`dialog:default`）也是一組打包好的 permission。

要看一個 plugin 的 permission 裡有什麼，不用翻原始碼。編譯過一次之後，`src-tauri/gen/schemas/acl-manifests.json` 就收著所有 plugin 的 permission。`node -p` 會執行一小段 JavaScript 並印出結果，用它在專案根目錄印出第 10 章用過的 `fs:allow-appdata-write-recursive`：

```bash
node -p 'JSON.stringify(require("./src-tauri/gen/schemas/acl-manifests.json").fs.permission_sets["allow-appdata-write-recursive"])'
```

```text
{"identifier":"allow-appdata-write-recursive","description":"This allows full recursive write access to the complete `$APPDATA` folder, files and subdirectories.","permissions":["write-all","scope-appdata-recursive"]}
```

它是一個 set，由 `write-all` 和 `scope-appdata-recursive` 組成。把 `permission_sets` 換成 `permissions`，看後者，它只有 scope、沒有 command：

```bash
node -p 'JSON.stringify(require("./src-tauri/gen/schemas/acl-manifests.json").fs.permissions["scope-appdata-recursive"])'
```

```text
{"identifier":"scope-appdata-recursive","description":"This scope permits recursive access to the complete `$APPDATA` folder, including sub directories and files.","commands":{"allow":[],"deny":[]},"scope":{"allow":[{"path":"$APPDATA"},{"path":"$APPDATA/**"}]}}
```

`write-all` 也是 set，裡面是所有寫入的 command（`allow-write-text-file`、`allow-remove`、`allow-rename`⋯⋯）。所以第 10 章那一行放得比較寬：所有寫入的功能，加上整個 `$APPDATA`。這集改成只放行需要的 command，再自己決定 scope。

### 範例：只放行 `*.txt`，但 `secret.txt` 不行

頁面沿用範本的表單：輸入框填檔名，按下按鈕就在 App 資料夾寫一個檔案，再讀回來。

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src/main.ts`：

```ts
import {
  BaseDirectory,
  mkdir,
  readTextFile,
  writeTextFile,
} from "@tauri-apps/plugin-fs";

const dir = BaseDirectory.AppData;

const nameEl: HTMLInputElement | null = document.querySelector("#greet-input");
const msgEl: HTMLElement | null = document.querySelector("#greet-msg");
if (!nameEl || !msgEl) {
  throw new Error("找不到 #greet-input 或 #greet-msg");
}

document.querySelector("#greet-form")?.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await mkdir("", { baseDir: dir, recursive: true });
    await writeTextFile(nameEl.value, "你好，scope", { baseDir: dir });
    const text = await readTextFile(nameEl.value, { baseDir: dir });
    msgEl.textContent = `寫好了，讀回來是：${text}`;
  } catch (error) {
    msgEl.textContent = `出錯了：${error}`;
  }
});
```

capability 只放行用得到的三個 command：建資料夾、寫、讀。範圍由 `fs:scope` 決定：它是 fs plugin 留給你自己填 scope 的 permission，沒有 command，只有你寫進去的 `allow` 與 `deny`。這裡允許 App 資料夾本身（`mkdir` 要用）和底下的 `.txt` 檔，但 `secret.txt` 不行：

`src-tauri/capabilities/default.json`：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "主視窗",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:allow-mkdir",
    "fs:allow-write-text-file",
    "fs:allow-read-text-file",
    {
      "identifier": "fs:scope",
      "allow": [{ "path": "$APPDATA" }, { "path": "$APPDATA/*.txt" }],
      "deny": [{ "path": "$APPDATA/secret.txt" }]
    }
  ]
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
   Compiling tauri-app v0.1.0 (~/tauri-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.15s
     Running `target/debug/tauri-app`
```

分別輸入三個檔名（路徑是 macOS 的，Windows 與 Linux 的 App 資料夾在別的地方）：

- `note.txt`：「寫好了，讀回來是：你好，scope」。
- `note.md`：`forbidden path: /Users/me/Library/Application Support/tw.example.tauri-app/note.md, maybe it is not allowed on the scope for `allow-write-text-file` permission in your capability file`。沒有任何 `allow` 比對到它。
- `secret.txt`：`forbidden path: /Users/me/Library/Application Support/tw.example.tauri-app/secret.txt`。後半句沒了。`$APPDATA/*.txt` 其實比對得到它，但 `deny` 一定優先，不管 `allow` 怎麼寫。

所以錯誤訊息有沒有 `maybe it is not allowed on the scope`，就是在告訴你是沒被 `allow`，還是被 `deny` 擋下。（這句提示只有開發模式才有，第 7 章第 5 集說過。）如果把 `fs:scope` 整段拿掉，連第一步的 `mkdir` 都會被擋，訊息是 `forbidden path: …/tw.example.tauri-app/, maybe it is not allowed on the scope for `allow-mkdir` permission in your capability file`：只有 command、沒有 scope，什麼路徑都不行。

路徑裡的 `*` 只比對一層，`**` 才會往下比對所有層。`$APPDATA/**` 比對的是 App 資料夾底下的東西，不包含資料夾本身，所以前面的 `allow` 另外寫了一行 `$APPDATA`。

### scope 掛在哪裡：全域與 command

scope 有兩種掛法，取決於那項 permission 有沒有 command：

- 沒有 command，像 `fs:scope`：全域 scope，fs plugin 的 command 會去讀它。
- 有 command，像 `fs:allow-write-text-file`：只有這個 command 適用。物件寫法跟第 7 章第 3 集一樣，`identifier` 加上 `allow`、`deny`，例如 `{ "identifier": "fs:allow-write-text-file", "allow": [{ "path": "$APPDATA/*.txt" }] }` 只限制寫檔，讀檔的範圍不受影響。

fs plugin 把兩種 scope 加在一起：`allow` 取聯集，`deny` 也取聯集，`deny` 一樣優先。（這是 fs plugin 的做法，Tauri 只是把兩種分開交給 command；第 4 集自己寫的 command 要合併就得自己來。）例如寫檔的 command scope 只有 `*.txt`，全域 scope 另外給 `*.md`，`note.md` 就寫得進去。

所以放行 `fs:default` 要小心：它除了讀取，還帶了一份全域 scope，涵蓋 App 的各個資料夾。你用 `fs:scope` 縮小範圍，它又幫你放大回去（`deny` 還是有效）。想精確控制，就像這集一樣，不放 `fs:default`，只放需要的 command 與自己寫的 scope。

### 直接禁止一個 command

每個 command 的 permission 都成對：`allow-` 開頭放行，`deny-` 開頭禁止，`deny` 贏過所有的 `allow`，連 set 裡放行的也一樣。想用一個 set，又要挖掉其中一個 command，就加上 `"fs:deny-write-text-file"`。被禁止的 command 會得到 `fs.write_text_file explicitly denied on origin local`，後面接著是哪份 capability 的哪一項禁止的。這也是開發模式才有的訊息，打包版只有 `Command plugin:fs|write_text_file not allowed by ACL`。

## 重點整理

- 一項 permission 由 `commands`（放行或禁止哪些 command）和 `scope`（`allow`、`deny` 兩份清單）組成，set 把好幾項打包成一個名字。`src-tauri/gen/schemas/acl-manifests.json` 可以查 plugin 的 permission 裡有什麼。
- 只放行 command 沒有 scope，路徑一律被擋。用 `fs:scope` 設全域 scope，或把 `allow`、`deny` 寫在某個 command 的 permission 上。fs plugin 把兩種 scope 取聯集。
- `deny` 永遠優先，不管 `allow` 怎麼寫；`deny-<command>` 能直接禁掉一個 command。
- `fs:default` 帶著 App 資料夾的 scope，要精確控制就別放，只放需要的 command。
