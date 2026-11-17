# 應該全部通過

## 加一個 command，前端呼叫它

`src-tauri/src/lib.rs`：

```rust
#[tauri::command]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![add])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src/main.ts`：

```ts
import { invoke } from "@tauri-apps/api/core";

const sum: number = await invoke<number>("add", { a: 1, b: 2 });
console.log(sum);
```

`npm install` 的輸出不驗證：

`npm install` 的輸出：

```text
added 20 packages
```

`npm run tauri dev` 的輸出（省略編譯進度）：

```text
     Running `target/debug/tauri-app`
```

## 書上貼錯誤：必須失敗，錯誤碼要對

`src-tauri/src/lib.rs`：

```rust
#[tauri::command]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let total: i32 = "3";
    println!("{total}");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![add])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`npm run tauri dev` 的輸出（節錄）：

```text
error[E0308]: mismatched types
```

TypeScript 的錯誤也一樣：

`src/main.ts`：

```ts
const n: number = "1";
console.log(n);
```

`npx tsc --noEmit` 的輸出：

```text
src/main.ts(1,7): error TS2322: Type 'string' is not assignable to type 'number'.
```

## 片段加 ignore；沒有路徑的 html、bash 不驗證

```rust,ignore
.invoke_handler(tauri::generate_handler![add])
```

```html
<button id="go">Go</button>
```

```bash
npm run tauri dev
```

## 最後改好：檔案結尾視為必須成功

`src-tauri/src/lib.rs`：

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src/main.ts`：

```ts
console.log("ok");
```
