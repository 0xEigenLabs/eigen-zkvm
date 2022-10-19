# Generate input executor

## Usage
- `node generate-txs.js -t <nTx> -e <flagRun> -r <rom.json> -p <pil.json> -o <flagOnlyExecutor>`
  - `-t <nTx>`: number of transaction to process. Creates a json file with the inputs ready to be processed by the executor. Input file would have the following format: `input-${nTx}.json`
  - `-e <flagRun>`: flag to run executor written in javascript with the input generated. If this flag is activated, `rom.json` and `pil.json` must be provided
  - `-r <rom.json>`: path to rom json file
  - `-p <pil.json>`: path to pil json file
  - `-o <flagOnlyExecutor>`: flag to skip building input for `nTx`. This flag assumes that `input-${nTx}.json` is already created and its main purpose is to run the executor without generate again the input.

## Example
```
node run-gen-txs.js -t 20 -e true -r ../../zkrom/build/rom.json -p ../../circuits/vm.pil.json
```
