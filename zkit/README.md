# eigen zkit

A universal commandline for [plonky](../plonky) and [starky](../starky).

## Usage

```
eigen-zkit 0.1.6

USAGE:
    eigen-zkit <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    aggregation_check                    Check aggregation proof
    aggregation_prove                    Proof aggregation for plonk
    aggregation_verify                   Verify aggregation proof
    analyse                              Analyse circuits
    calculate_witness                    Calculate witness and save to output file
    compile                              Compile circom circuits to r1cs, and generate witness
    compressor12_exec                    Exec compressor12 for converting R1CS to PIL
    compressor12_setup                   Setup compressor12 for converting R1CS to PIL
    export_aggregation_verification_key  Export aggregation proof's verification key
    export_verification_key              Export proof's verification key
    generate_aggregation_verifier        A subcommand for generating a Solidity aggregation verifier smart contract
    generate_verifier                    Generate solidity verifier
    groth16_prove                        Prove with groth16
    groth16_setup                        Setup groth16
    groth16_verify                       Verify with groth16
    help                                 Print this message or the help of the given subcommand(s)
    join_zkin                            generate the input1.zkin.json and input2.zkin.json into out.zkin.json
    prove                                Prove by Plonk
    setup                                Trust setup for Plonk
    stark_prove                          Stark proving and verifying all in one
    verify                               Verify the Plonk proof
```

The recursive proof example can be found [here](../starkjs).
