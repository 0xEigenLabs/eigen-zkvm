#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::errors::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Public {
    pub polType: String,
    pub polId: usize,
    pub idx: usize,
    pub id: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polType: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub id: usize,
    pub polDeg: usize,
    pub isArray: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elementType: Option<String>, // "field, s8, s16, s32, s64, u16, u8"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub len: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expression {
    pub op: String, // number, cm, add, sub, ...
    pub deg: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<bool>, // None is false, the other would be true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<Expression>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep2ns: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idQ: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub const_: Option<i64>,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj = json!(self);
        write!(f, "{}", serde_json::to_string_pretty(&obj).unwrap())
    }
}

impl Expression {
    pub fn next(&self) -> bool {
        return !(self.next.is_none() || !self.next.unwrap());
    }
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.op == other.op && self.deg == other.deg && self.id == other.id
    }
}

impl Expression {
    pub fn new(
        op: String,
        deg: usize,
        id: Option<usize>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolIdentity {
    pub e: usize,
    pub fileName: String,
    pub line: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlookupIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selF: Option<usize>, //selector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selT: Option<usize>,
    pub fileName: String,
    pub line: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PermutationIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selF: Option<usize>, //selector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selT: Option<usize>,
    pub fileName: String,
    pub line: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pols: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<usize>>,
    pub fileName: String,
    pub line: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PIL {
    pub nCommitments: usize,
    pub nQ: usize,
    pub nIm: usize,
    pub nConstants: usize,
    pub publics: Vec<Public>,
    pub references: HashMap<String, Reference>,
    pub expressions: Vec<Expression>,
    pub polIdentities: Vec<PolIdentity>,
    pub plookupIdentities: Vec<PlookupIdentity>,
    pub permutationIdentities: Option<Vec<PermutationIdentity>>,
    pub connectionIdentities: Option<Vec<ConnectionIdentity>>,

    #[serde(skip)]
    pub cm_dims: Vec<usize>,
    #[serde(skip)]
    pub q2exp: Vec<usize>,
}

impl fmt::Display for PIL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj = json!(self);
        write!(f, "{}", serde_json::to_string_pretty(&obj).unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Step {
    pub nBits: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StarkStruct {
    pub nBits: usize,
    pub nBitsExt: usize,
    pub nQueries: usize,
    pub verificationHashType: String,
    pub steps: Vec<Step>,
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
    log::info!(
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
