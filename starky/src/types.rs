#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::errors::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Public {
    pub polType: String,
    pub polId: i32,
    pub idx: i32,
    pub id: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reference {
    pub polType: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub id: usize,
    pub polDeg: usize,
    pub isArray: bool,
    pub elementType: Option<String>, // "field, s8, s16, s32, s64, u16, u8"
    pub len: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expression {
    pub op: String, // number, cm, add, sub, ...
    pub deg: i32,
    pub id: Option<i32>,
    pub next: Option<bool>, // None is false, the other would be true. same as others with type Option<bool>
    pub value: Option<String>,
    pub values: Option<Vec<Expression>>,
    pub keep: Option<bool>,
    pub keep2ns: Option<bool>,
    pub idQ: Option<i32>,
    pub const_: Option<i64>,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.op == other.op && self.deg == other.deg && self.id == other.id
    }
}

impl Expression {
    pub fn new(
        op: String,
        deg: i32,
        id: Option<i32>,
        value: Option<String>,
        values: Option<Vec<Expression>>,
    ) -> Self {
        Expression {
            op: op,
            deg: deg,
            id: id,
            next: None,
            value: value,
            values: values,
            keep: None,
            keep2ns: None,
            idQ: None,
            const_: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PolIdentity {
    pub e: i32,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlookupIdentity {
    pub f: Option<Vec<i32>>,
    pub t: Option<Vec<i32>>,
    pub selF: Option<i32>, //selector
    pub selT: Option<i32>,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PermutationIdentity {
    pub f: Option<Vec<i32>>,
    pub t: Option<Vec<i32>>,
    pub selF: Option<i32>, //selector
    pub selT: Option<i32>,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionIdentity {
    pub pols: Option<Vec<i32>>,
    pub connections: Option<Vec<i32>>,
    pub fileName: String,
    pub line: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PIL {
    pub nCommitments: i32,
    pub nQ: i32,
    pub nIm: i32,
    pub nConstants: i32,
    pub publics: Vec<Public>,
    pub references: HashMap<String, Reference>,
    pub expressions: Vec<Expression>,
    pub polIdentities: Vec<PolIdentity>,
    pub plookupIdentities: Vec<PlookupIdentity>,
    pub permutationIdentities: Option<Vec<PermutationIdentity>>,
    pub connectionIdentities: Option<Vec<ConnectionIdentity>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StarkStruct {
    pub nBits: i32,
    pub nBitsExt: i32,
    pub nQueries: i32,
    pub verificationHashType: String,
    pub steps: Vec<HashMap<String, i32>>,
}

pub fn load_json<T>(filename: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    read_json(data)
}

pub fn read_json<T>(data: String) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_json::from_str(&data)?)
}

#[test]
pub fn test_read_pil() {
    load_json::<PIL>("data/fib.pil.json").unwrap();
    println!(
        "arrays.pil.json: {:?}",
        load_json::<PIL>("data/arrays.pil.json").unwrap()
    );
}

#[test]
pub fn test_read_struct() {
    let json_str = r#"
    {
        "nBits": 23,
        "nBitsExt": 24,
        "nQueries": 4,
        "verificationHashType": "BN128",
        "steps": [
        {
            "nBits": 24
        },
        {
            "nBits": 20
        },
        {
            "nBits": 16
        },
        {
            "nBits": 12
        },
        {
            "nBits": 8
        }
        ]
    }"#;
    read_json::<StarkStruct>(json_str.to_string()).unwrap();
}
