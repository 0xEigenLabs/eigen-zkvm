#![allow(dead_code, non_snake_case)]
use crate::constant::CHALLENGE_MAP;
use crate::types::Expression;

pub struct ExpressionOps;

impl ExpressionOps {
    pub fn add(a: &Expression, b: &Expression) -> Expression {
        Expression::new("add".to_string(), 0, None, None, Some(vec![a.clone(), b.clone()]))
    }

    pub fn sub(a: &Expression, b: &Expression) -> Expression {
        Expression::new("sub".to_string(), 0, None, None, Some(vec![a.clone(), b.clone()]))
    }

    pub fn mul(a: &Expression, b: &Expression) -> Expression {
        Expression::new("mul".to_string(), 0, None, None, Some(vec![a.clone(), b.clone()]))
    }

    pub fn neg(a: &Expression) -> Expression {
        Expression::new("neg".to_string(), 0, None, None, Some(vec![a.clone()]))
    }

    pub fn exp(id: usize, next: Option<bool>) -> Expression {
        let mut exp = Expression::new("exp".to_string(), 0, Some(id), None, None);
        exp.next = next;
        exp
    }

    pub fn cm(id: usize, next: Option<bool>) -> Expression {
        let mut exp = Expression::new("cm".to_string(), 0, Some(id), None, None);
        exp.next = next;
        exp
    }

    pub fn const_(id: usize, next: Option<bool>) -> Expression {
        let mut exp = Expression::new("const".to_string(), 0, Some(id), None, None);
        exp.next = next;
        exp
    }

    pub fn q(id: usize, next: Option<bool>) -> Expression {
        let mut exp = Expression::new("q".to_string(), 0, Some(id), None, None);
        exp.next = next;
        exp
    }

    pub fn challenge(name: String) -> Expression {
        if CHALLENGE_MAP.get(&name.as_str()).is_none() {
            panic!("challenge not defined");
        }
        Expression::new(
            "challenge".to_string(),
            0,
            Some(*CHALLENGE_MAP.get(&name.as_str()).unwrap()),
            None,
            None,
        )
    }

    pub fn number(n: String) -> Expression {
        Expression::new("number".to_string(), 0, None, Some(n), None)
    }

    pub fn eval(n: usize) -> Expression {
        Expression::new("eval".to_string(), 0, Some(n), None, None)
    }

    pub fn tmp(n: usize) -> Expression {
        Expression::new("tmp".to_string(), 0, Some(n), None, None)
    }

    pub fn xDivXSubXi() -> Expression {
        Expression::new("xDivXSubXi".to_string(), 0, None, None, None)
    }

    pub fn xDivXSubWXi() -> Expression {
        Expression::new("xDivXSubWXi".to_string(), 0, None, None, None)
    }

    pub fn x() -> Expression {
        Expression::new("x".to_string(), 0, None, None, None)
    }

    pub fn nop() -> Expression {
        Expression::new("nop".to_string(), 0, None, None, None)
    }

    pub fn is_nop(e: &Expression) -> bool {
        e.op.as_str() == "nop"
    }
}
