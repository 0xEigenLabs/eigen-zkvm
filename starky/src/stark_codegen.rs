use std::collections::HashMap;

pub struct Context<'a> {
    pil: &'a PIL,
    clculated: HashMap<String, >,
    exp_id: i32,
    tmp_used: u32,
    code: Vec<Code>,
}

pub struct Node {
    pub type_: String,
    id: i32,
    dim: i32,
    prime: Option<bool>,
    tree_pos: Option<i32>,
}

pub struct Code {
    pub op: String,
    pub dest: Node,
    pub src: Vec<Node>,
}

pub struct StepCode {
    pub first: Vec<Code>,
    pub i: Vec<Code>,
    pub last: Vec<Code>,
}


struct MapSections {
    cm1_n: i32,
    cm1_2ns: i32,
    cm2_n: i32,
    cm2_2ns: i32,
    cm3_n: i32,
    cm3_2ns: i32,
    q_2ns: i32,
    exps_withq_n: i32,
    exps_withq_2ns: i32,
    exps_withoutq_n: i32,
    exps_withoutq_2ns: i32,
}


pub fn pil_code_gen(ctx: &Context, pol_id: i32, bool: prime, mode: String) {

}

pub fn build_code(ctx: &Context) {

}
