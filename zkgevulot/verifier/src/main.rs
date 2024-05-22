extern crate clap;
use clap::{command, Parser};


use starky::prove::stark_prove;
use std::time::Instant;
use std::fs;
use gevulot_common::WORKSPACE_PATH;
use gevulot_shim::{Task, TaskResult};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    gevulot_shim::run(run_task)
}

fn run_task(task: Task) -> Result<TaskResult> {

    //env_logger::init();
    println!("prover: task.args: {:?}", &task.args);
    //TODO define some provided context

    //the gevulot cmd parameters when running prover
    /*
     {"name":"--out_file","value":"/workspace/proof.json"},
     {"name":"--piljson","value":"/workspace/fibonacci.recursive2.pil.json"},
      {"name":"--const_pols","value":"/workspace/fibonacci.recursive2.const"},
      {"name":"--cm_pols","value":"/workspace/fibonacci.recursive2.cm"},
      {"name":"--stark_stuct","value":"/gevulot/starkStruct.json"},

    */
   let _ = stark_prove(
       "/gevulot/starkStruct.json", //just testing  with fixed name
       "/workspace/fibonacci.recursive2.pil.json",
        true,
        false,
        false,
        "/workspace/fibonacci.recursive2.const",
        "/workspace/fibonacci.recursive2.cm",
        "/workspace/final.circom",
        "/workspace/proof.json",
        "273030697313060285579891744179749754319274977764",
    );



    // Return TaskResult with reference to the generated proof file.
    task.result(vec![], vec![String::from("/workspace/proof.json")])


}