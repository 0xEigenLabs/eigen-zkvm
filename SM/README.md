# SM
State Machine based zkVM prototype abstracted from [zkevm-proverjs](https://github.com/0xPolygonHermez/zkevm-proverjs/tree/main/pil).
We're reimplementing the pil-stark by Rust.

# Example

```
npm run buildrom
npm run buildstoragerom
npm run genstarkstruct
node src/main.js -w circuits/
```

## Generating custom transactions

[README](./tools/gen-input-executor/README.md)
