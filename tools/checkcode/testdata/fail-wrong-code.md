有錯，但錯誤碼跟書上不同：

`src-tauri/src/lib.rs`：

```rust
pub fn run() {
    let total: i32 = "3";
    println!("{total}");
}
```

`cargo check` 的輸出：

```text
error[E0425]: cannot find value
```
