#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::errors::Result;

#[derive(Serialize, Deserialize)]
pub struct Public {
    pub polType: String,
    pub polId: i32,
    pub idx: i32,
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Reference {
    pub polType: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub id: i32,
    pub polDeg: i32,
    pub isArray: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Expression {
    pub op: String, // number, cm, add, sub, ...
    pub deg: i32,
    pub next: Option<bool>,
    pub value: Option<String>,
    pub values: Option<Vec<Expression>>,
}

#[derive(Serialize, Deserialize)]
pub struct PolIdentity {
    pub e: i32,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PlookupIdentity {
    pub f: Option<Vec<i32>>,
    pub t: Option<Vec<i32>>,
    pub selF: Option<i32>, //selector
    pub selT: Option<i32>,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PermutationIdentity {
    pub f: Option<Vec<i32>>,
    pub t: Option<Vec<i32>>,
    pub selF: Option<i32>, //selector
    pub selT: Option<i32>,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ConnectionIdentity {
    pub pols: Option<Vec<i32>>,
    pub connections: Option<Vec<i32>>,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PILJson {
    pub nCommitments: i32,
    pub nQ: i32,
    pub nIm: i32,
    pub nConstants: i32,
    pub publics: Vec<Public>,
    pub references: HashMap<String, Reference>,
    pub expressions: Vec<Expression>,
    pub polIdentities: Vec<PolIdentity>,
    pub plookupIdentities: Vec<PlookupIdentity>,
    pub permutationIdentities: Vec<PermutationIdentity>,
    pub connectionIdentities: Vec<ConnectionIdentity>,
}

pub fn read_pil(filename: &str) -> Result<PILJson> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let json: PILJson = serde_json::from_str(&data)?;
    Ok(json)
}

#[test]
pub fn test_read_pil() {
    read_pil("data/fib.pil.json").unwrap();
}
