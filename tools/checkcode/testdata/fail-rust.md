結尾的隱含檢查點必須成功，這裡編譯不過：

`src-tauri/src/lib.rs`：

```rust
pub fn run() {
    let total: i32 = "3";
    println!("{total}");
}
```
