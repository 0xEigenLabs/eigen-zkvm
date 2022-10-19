# SM
State Machine based zkVM prototype abstracted from [zkevm-proverjs](https://github.com/0xPolygonHermez/zkevm-proverjs/tree/main/pil).
We're reimplementing the pil-stark by Rust.

# Run

```
npm run buildrom
npm run buildstoragerom
npm run genstarkstruct
node src/main.js -w circuits/
```
