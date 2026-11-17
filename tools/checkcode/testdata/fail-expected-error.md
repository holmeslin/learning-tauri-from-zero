書上貼錯誤，實際卻編譯成功：

`src/main.ts`：

```ts
console.log("ok");
```

`npx tsc --noEmit` 的輸出：

```text
src/main.ts(1,1): error TS2304: Cannot find name 'x'.
```
