#![allow(non_snake_case)]

use crate::constant::{MG, SHIFT};
use crate::digest::ElementDigest;
use crate::f3g::F3G;
use crate::pil2circom::StarkOption;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::Node;
use crate::starkinfo_codegen::Section;
use crate::traits::FieldExtension;
use crate::traits::MTNodeType;
use crate::types::{StarkStruct, PIL};
use profiler_macro::time_profiler;

fn header(options: &StarkOption) -> String {
    let mut header = r#"pragma circom 2.1.0;
pragma custom_templates;

include "cmuladd.circom";
include "cinv.circom";
include "poseidon.circom";
include "bitify.circom";
include "fft.circom";
include "merklehash.circom";
include "evalpol.circom";
include "treeselector.circom";
"#
    .to_string();
    if options.agg_stage {
        header += r#"
include "mux1.circom";
include "iszero.circom";
"#;
    }

    header
}

#[derive(Default)]
struct Transcript {
    state: [String; 4],
    pending: Vec<String>,
    out: Vec<String>,
    h_cnt: usize,
    n2b_cnt: usize,
    code: Vec<String>,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            state: [String::from("0"), String::from("0"), String::from("0"), String::from("0")],
            pending: vec![],
            out: vec![],
            h_cnt: 0,
            n2b_cnt: 0,
            code: vec![],
        }
    }

    fn getField(&mut self, v: &str, _l: usize) {
        let tmp = self.getFields1();
        self.code.push(format!("{}[0] <== {};", v, tmp));
        let tmp = self.getFields1();
        self.code.push(format!("{}[1] <== {};", v, tmp));
        let tmp = self.getFields1();
        self.code.push(format!("{}[2] <== {};", v, tmp));
    }

    fn getFields1(&mut self) -> String {
        if self.out.is_empty() {
            while self.pending.len() < 8 {
                self.pending.push(String::from("0"));
            }
            self.code.push(format!(
                "signal tcHahs_{}[12] <==  Poseidon(12)([{}], [{}]);",
                self.h_cnt,
                self.pending.join(","),
                self.state.join(",")
            ));
            self.h_cnt += 1;
            for i in 0..12 {
                self.out.push(format!("tcHahs_{}[{}]", self.h_cnt - 1, i));
            }
            for i in 0..4 {
                self.state[i] = format!("tcHahs_{}[{}]", self.h_cnt - 1, i);
            }
            self.pending = vec![];
        }
        let res = self.out[0].to_owned();
        self.out.remove(0);
        res
    }

    pub fn put(&mut self, a: &str, l: i32) {
        if l >= 0 {
            for i in 0..l {
                self._add1(&format!("{}[{}]", a, i));
            }
        } else {
            self._add1(a);
        }
    }

    pub fn _add1(&mut self, a: &str) {
        self.out = vec![];
        self.pending.push(a.to_string());
        if self.pending.len() == 8 {
            self.code.push(format!(
                "signal tcHahs_{}[12] <== Poseidon(12)([{}], [{}]);",
                self.h_cnt,
                self.pending.join(","),
                self.state.join(",")
            ));
            self.h_cnt += 1;
            self.out = vec![];
            for i in 0..12 {
                self.out.push(format!("tcHahs_{}[{}]", self.h_cnt - 1, i));
            }
            for i in 0..4 {
                self.state[i] = format!("tcHahs_{}[{}]", self.h_cnt - 1, i);
            }
            self.pending = vec![];
        }
    }

    pub fn getPermutations(&mut self, v: &str, n: usize, nBits: usize) {
        let totalBits = n * nBits;
        let NFields = (totalBits - 1) / 63 + 1;
        let mut n2b: Vec<String> = vec![];
        for i in 0..NFields {
            let f = self.getFields1();
            n2b.push(format!("tcN2b_{}", self.n2b_cnt));
            self.n2b_cnt += 1;
            self.code.push(format!("component {} = Num2Bits_strict();", n2b[i]));
            self.code.push(format!("{}.in <== {};", n2b[i], f));
        }
        let mut curField = 0;
        let mut curBit = 0;
        for i in 0..n {
            for j in 0..nBits {
                self.code
                    .push(format!("{}[{}][{}] <== {}.out[{}];", v, i, j, n2b[curField], curBit));
                curBit += 1;
                if curBit == 63 {
                    curBit = 0;
                    curField += 1;
                }
            }
        }
    }

    pub fn getCode(&self) -> String {
        let mut tmp: Vec<String> = vec![];
        for i in 0..self.code.len() {
            tmp.push("    ".to_owned() + &self.code[i]);
        }
        tmp.join("\n")
    }
}

fn unrollCode(code: &Vec<Section>, starkinfo: &StarkInfo) -> (String, String) {
    let ref_ = |r: &Node| -> String {
        match r.type_.as_str() {
            "eval" => format!("evals[{}]", r.id),
            "challenge" => format!("challenges[{}]", r.id),
            "public" => format!("publics[{}]", r.id),
            "x" => "challenges[7]".to_string(),
            "Z" => "Z".to_string(),
            "xDivXSubXi" => "xDivXSubXi".to_string(),
            "xDivXSubWXi" => "xDivXSubWXi".to_string(),
            "tmp" => format!("tmp_{}", r.id),
            "tree1" => format!("mapValues.tree1_{}", r.id),
            "tree2" => format!("mapValues.tree2_{}", r.id - starkinfo.n_cm1),
            "tree3" => format!("mapValues.tree3_{}", r.id - starkinfo.n_cm1 - starkinfo.n_cm2),
            "tree4" => format!(
                "mapValues.tree4_{}",
                r.id - starkinfo.n_cm1 - starkinfo.n_cm2 - starkinfo.n_cm3
            ),
            "const" => format!("consts[{}]", r.id),
            "number" => r.value.as_ref().unwrap().to_string(),
            _ => panic!("Invalid ref: {}", r.type_),
        }
    };
    let mut str_code = String::from("");

    for inst in code {
        /*
            if inst.dest.type_.as_str() == "tmp" {
                if inst.dest.dim == 1 {
                    str_code.push_str(&format!(
                        r#"
        signal tmp_{};"#,
                        inst.dest.id
                    ));
                } else if inst.dest.dim == 3 {
                    str_code.push_str(&format!(
                        r#"
        signal tmp_{}[3];"#,
                        inst.dest.id
                    ));
                } else {
                    panic!("Invalid dimension");
                }
            }
            */

        match inst.op.as_str() {
            "add" => {
                if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {} <== {} + {};"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{} + {}[0], {}[1], {}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[1]),
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{}[0] + {}, {}[1], {}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[0])
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{}[0] + {}[0], {}[1] + {}[1], {}[2] + {}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else {
                    panic!("Invalid src dimensions");
                }
            }
            "sub" => {
                if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {} <== {} - {};"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{} - {}[0], -{}[1], -{}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{}[0] - {}, {}[1], {}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[0]),
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{}[0] - {}[0], {}[1] - {}[1], {}[2] - {}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else {
                    panic!("Invalid src dimensions");
                }
            }
            "mul" => {
                if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {} <== {} * {};"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{} * {}[0], {} * {}[1], {} * {}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{}[0] * {}, {}[1] * {}, {}[2] * {}];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== CMul()({}, {});"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else {
                    panic!("Invalid src dimensions");
                }
            }
            "copy" => {
                if inst.src[0].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {} <== {};"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0])
                    ));
                } else if inst.src[0].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== {};"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0])
                    ));
                } else {
                    panic!("Invalid src dimensions");
                }
            }
            "muladd" => {
                if inst.src[2].dim == 1 {
                    if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                        str_code.push_str(&format!(
                            r#"
    signal {} <== {} * {} + {};"#,
                            ref_(&inst.dest),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[2])
                        ));
                    } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                        str_code.push_str(&format!(
                            r#"
    signal {}[3] <== [{} * {}[0] + {}, {} * {}[1], {} * {}[2]];"#,
                            ref_(&inst.dest),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[2]),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1])
                        ));
                    } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                        str_code.push_str(&format!(
                            r#"
    signal {}[3] <== [{}[0] * {} + {}, {}[1] * {}, {}[2] * {}];"#,
                            ref_(&inst.dest),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[2]),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1])
                        ));
                    } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                        str_code.push_str(&format!(
                            r#"
    signal {}[3] <== CMulAdd()({}, {}, [{}, 0, 0]);"#,
                            ref_(&inst.dest),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[2])
                        ));
                    } else {
                        panic!("Invalid src dimensions")
                    }
                } else if inst.src[2].dim == 3 {
                    if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                        str_code.push_str(&format!(
                            r#"
    signal {}[3] <== [{}*{} + {}[0], {}[1], {}[2]];"#,
                            ref_(&inst.dest),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[2]),
                            ref_(&inst.src[2]),
                            ref_(&inst.src[2])
                        ));
                    } else {
                        let ina = match inst.src[0].dim {
                            3 => ref_(&inst.src[0]),
                            1 => format!("[{}, 0, 0]", ref_(&inst.src[0])),
                            _ => panic!("Invalid src dimensions"),
                        };

                        let inb = match inst.src[1].dim {
                            3 => ref_(&inst.src[1]),
                            1 => format!("[{}, 0, 0]", ref_(&inst.src[1])),
                            _ => panic!("Invalid src dimensions"),
                        };

                        let inc = match inst.src[2].dim {
                            3 => ref_(&inst.src[2]),
                            1 => format!("[{}, 0, 0]", ref_(&inst.src[2])),
                            _ => panic!("Invalid src dimensions"),
                        };
                        str_code.push_str(&format!(
                            r#"
    signal {}[3] <== CMulAdd()({}, {}, {});"#,
                            ref_(&inst.dest),
                            ina,
                            inb,
                            inc
                        ));
                    }
                } else {
                    panic!("Invalid src dimensions")
                }
            }
            _ => panic!("Invalid op"),
        }
    }
    (str_code, ref_(&code[code.len() - 1].dest))
}

fn verify_evaluations(
    starkinfo: &StarkInfo,
    program: &Program,
    pil: &PIL,
    stark_struct: &StarkStruct,
) -> String {
    let mut res = format!(
        r#"
template VerifyEvaluations() {{
    signal input challenges[8][3];
    signal input evals[{}][3];
    signal input publics[{}];
    signal input enable;
"#,
        starkinfo.ev_map.len(),
        pil.publics.len()
    );

    res.push_str(&format!(
        r#"
    signal zMul[{}][3];
    "#,
        stark_struct.nBits
    ));

    res.push_str(&format!(
        r#"
    for (var i=0; i< {}; i++) {{
        if (i==0) {{
            zMul[i] <== CMul()(challenges[7], challenges[7]);
        }} else {{
            zMul[i] <== CMul()(zMul[i-1], zMul[i-1]);
        }}
    }}
        "#,
        stark_struct.nBits
    ));

    res.push_str(&format!(
        r#"
    signal Z[3];

    Z[0] <== zMul[{}][0] -1;
    Z[1] <== zMul[{}][1];
    Z[2] <== zMul[{}][2];"#,
        stark_struct.nBits - 1,
        stark_struct.nBits - 1,
        stark_struct.nBits - 1,
    ));

    let (tmpCode, evalP) = unrollCode(&program.verifier_code.first, starkinfo);
    res.push_str(&tmpCode);

    res.push_str(&format!(
        r#"
    signal xN[3] <== zMul[{}];

    signal xAcc[{}][3];
    signal qStep[{}][3];
    signal qAcc[{}][3];
    for (var i=0; i< {}; i++) {{
        if (i==0) {{
            xAcc[0] <== [1, 0, 0];
            qAcc[0] <== evals[{}+i];
        }} else {{
            xAcc[i] <== CMul()(xAcc[i-1], xN);
            qStep[i-1] <== CMul()(xAcc[i], evals[{}+i]);

            qAcc[i][0] <== qAcc[i-1][0] + qStep[i-1][0];
            qAcc[i][1] <== qAcc[i-1][1] + qStep[i-1][1];
            qAcc[i][2] <== qAcc[i-1][2] + qStep[i-1][2];
        }}
    }}"#,
        stark_struct.nBits - 1,
        starkinfo.q_deg,
        starkinfo.q_deg - 1,
        starkinfo.q_deg,
        starkinfo.q_deg,
        starkinfo.ev_idx.cm.get(&(0, starkinfo.qs[0])).unwrap(),
        starkinfo.ev_idx.cm.get(&(0, starkinfo.qs[0])).unwrap(),
    ));

    res.push_str(&format!(
        r#"
    signal qZ[3] <== CMul()(qAcc[{}], Z);

// Final Verification
    enable * ({}[0] - qZ[0]) === 0;
    enable * ({}[1] - qZ[1]) === 0;
    enable * ({}[2] - qZ[2]) === 0;
}}
        "#,
        starkinfo.q_deg - 1,
        evalP,
        evalP,
        evalP
    ));
    res
}

fn verify_query(starkinfo: &StarkInfo, program: &Program, stark_struct: &StarkStruct) -> String {
    let mut res = format!(
        r#"
template parallel VerifyQuery() {{
    signal input ys[{}];
    signal input challenges[8][3];
    signal input evals[{}][3];
    signal input tree1[{}];
    "#,
        stark_struct.steps[0].nBits,
        starkinfo.ev_map.len(),
        starkinfo.map_sectionsN.get("cm1_2ns"),
    );

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input tree2[{}];
            "#,
            starkinfo.map_sectionsN.get("cm2_2ns")
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input tree3[{}];
            "#,
            starkinfo.map_sectionsN.get("cm3_2ns")
        ));
    }

    res.push_str(&format!(
        r#"
    signal input tree4[{}];
    signal input consts[{}];
    signal output out[3];
        "#,
        starkinfo.map_sectionsN.get("cm4_2ns"),
        starkinfo.n_constants
    ));

    ///////////
    // Mapping
    ///////////

    res.push_str(&format!(
        r#"
    component mapValues = MapValues();

    for (var i=0; i< {}; i++ ) {{
        mapValues.vals1[i] <== tree1[i];
    }}"#,
        starkinfo.map_sectionsN.get("cm1_2ns")
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
    for (var i=0; i< {}; i++ ) {{
        mapValues.vals2[i] <== tree2[i];
    }}"#,
            starkinfo.map_sectionsN.get("cm2_2ns")
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
    for (var i=0; i< {}; i++ ) {{
        mapValues.vals3[i] <== tree3[i];
    }}"#,
            starkinfo.map_sectionsN.get("cm3_2ns")
        ));
    }

    if starkinfo.map_sectionsN.get("cm4_2ns") > 0 {
        res.push_str(&format!(
            r#"
    for (var i=0; i< {}; i++ ) {{
        mapValues.vals4[i] <== tree4[i];
    }}"#,
            starkinfo.map_sectionsN.get("cm4_2ns")
        ));
    }

    res.push_str(&format!(
        r#"
    signal xacc[{}];
    xacc[0] <== ys[0]*({} * roots({})-{}) + {};
    for (var i=1; i<{}; i++ ) {{
        xacc[i] <== xacc[i-1] * ( ys[i]*(roots({} - i) - 1) +1);
    }}"#,
        stark_struct.steps[0].nBits,
        SHIFT.as_int(),
        stark_struct.steps[0].nBits,
        SHIFT.as_int(),
        SHIFT.as_int(),
        stark_struct.steps[0].nBits,
        stark_struct.steps[0].nBits
    ));

    res.push_str(&format!(
        r#"
    component den1inv = CInv();
    den1inv.in[0] <== xacc[{}] - challenges[7][0];
    den1inv.in[1] <== -challenges[7][1];
    den1inv.in[2] <== -challenges[7][2];
    signal xDivXSubXi[3];
    xDivXSubXi[0] <== xacc[{}] * den1inv.out[0];
    xDivXSubXi[1] <== xacc[{}] * den1inv.out[1];
    xDivXSubXi[2] <== xacc[{}] * den1inv.out[2];
    "#,
        stark_struct.steps[0].nBits - 1,
        stark_struct.steps[0].nBits - 1,
        stark_struct.steps[0].nBits - 1,
        stark_struct.steps[0].nBits - 1,
    ));

    res.push_str(&format!(
        r#"
    component den2inv = CInv();
    den2inv.in[0] <== xacc[{}] - roots({})*challenges[7][0];
    den2inv.in[1] <== -roots({})*challenges[7][1];
    den2inv.in[2] <== -roots({})*challenges[7][2];
    signal xDivXSubWXi[3];
    xDivXSubWXi[0] <== xacc[{}] * den2inv.out[0];
    xDivXSubWXi[1] <== xacc[{}] * den2inv.out[1];
    xDivXSubWXi[2] <== xacc[{}] * den2inv.out[2];
    "#,
        stark_struct.steps[0].nBits - 1,
        stark_struct.nBits,
        stark_struct.nBits,
        stark_struct.nBits,
        stark_struct.steps[0].nBits - 1,
        stark_struct.steps[0].nBits - 1,
        stark_struct.steps[0].nBits - 1,
    ));

    let (tmpCode, evalQ) = unrollCode(&program.verifier_query_code.first, starkinfo);
    res.push_str(&tmpCode);

    res.push_str(&format!(
        r#"
    out[0] <== {}[0];
    out[1] <== {}[1];
    out[2] <== {}[2];
}}
    "#,
        evalQ, evalQ, evalQ
    ));

    res
}

fn map_values(starkinfo: &StarkInfo) -> String {
    let mut res = format!(
        r#"
template MapValues() {{
    signal input vals1[{}];"#,
        starkinfo.map_sectionsN.get("cm1_2ns")
    );

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input vals2[{}];"#,
            starkinfo.map_sectionsN.get("cm2_2ns")
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input vals3[{}];"#,
            starkinfo.map_sectionsN.get("cm3_2ns")
        ));
    }

    res.push_str(&format!(
        r#"
    signal input vals4[{}];"#,
        starkinfo.map_sectionsN.get("cm4_2ns")
    ));

    let sNames = ["", "cm1_2ns", "cm2_2ns", "cm3_2ns", "cm4_2ns"];
    for (t, s_name) in sNames.iter().enumerate().skip(1) {
        for (i, ms) in starkinfo.map_sections.get(s_name).iter().enumerate() {
            let p = &starkinfo.var_pol_map[*ms];
            if p.dim == 1 {
                res.push_str(&format!(
                    r#"
    signal output tree{}_{};"#,
                    t, i
                ));
            } else if p.dim == 3 {
                res.push_str(&format!(
                    r#"
    signal output tree{}_{}[3];"#,
                    t, i
                ));
            } else {
                panic!("Invalid dim");
            }
        }
    }

    for (t, s_name) in sNames.iter().enumerate().skip(1) {
        for (i, ms) in starkinfo.map_sections.get(s_name).iter().enumerate() {
            let p = &starkinfo.var_pol_map[*ms];
            if p.dim == 1 {
                res.push_str(&format!(
                    r#"
    tree{}_{} <== vals{}[{}];"#,
                    t, i, t, p.section_pos
                ));
            } else if p.dim == 3 {
                res.push_str(&format!(
                    r#"
    tree{}_{}[0] <== vals{}[{}];
    tree{}_{}[1] <== vals{}[{}];
    tree{}_{}[2] <== vals{}[{}];"#,
                    t,
                    i,
                    t,
                    p.section_pos,
                    t,
                    i,
                    t,
                    p.section_pos + 1,
                    t,
                    i,
                    t,
                    p.section_pos + 2,
                ));
            } else {
                panic!("Invalid dim");
            }
        }
    }
    res.push_str(
        r#"
}"#,
    );
    res
}

#[time_profiler()]
fn stark_verifier<F: ff::PrimeField + Default>(
    starkinfo: &StarkInfo,
    pil: &PIL,
    stark_struct: &StarkStruct,
    const_root: &ElementDigest<4, F>,
    options: &StarkOption,
) -> String {
    let mut res = format!(
        r#"
template StarkVerifier() {{
    signal input publics[{}];
    signal input root1[4];
    signal input root2[4];
    signal input root3[4];
    signal input root4[4];
"#,
        pil.publics.len()
    );

    if options.verkey_input {
        res.push_str(
            r#"
    signal input rootC[4];
"#,
        );
    } else {
        let const_roots = const_root.as_elements();
        res.push_str(&format!(
            r#"
    signal rootC[4];
    rootC[0] <== {};
    rootC[1] <== {};
    rootC[2] <== {};
    rootC[3] <== {};
"#,
            const_roots[0].as_int(),
            const_roots[1].as_int(),
            const_roots[2].as_int(),
            const_roots[3].as_int()
        ));
    }

    res.push_str(&format!(
        r#"
    signal input evals[{}][3];
    signal input s0_vals1[{}][{}];
    "#,
        starkinfo.ev_map.len(),
        stark_struct.nQueries,
        starkinfo.map_sectionsN.get("cm1_2ns")
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input s0_vals2[{}][{}];
        "#,
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm2_2ns")
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input s0_vals3[{}][{}];
        "#,
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm3_2ns")
        ));
    }

    res.push_str(&format!(
        r#"
    signal input s0_vals4[{}][{}];
    signal input s0_valsC[{}][{}];
    signal input s0_siblings1[{}][{}][4];
"#,
        stark_struct.nQueries,
        starkinfo.map_sectionsN.get("cm4_2ns"),
        stark_struct.nQueries,
        starkinfo.n_constants,
        stark_struct.nQueries,
        stark_struct.steps[0].nBits
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input s0_siblings2[{}][{}][4];
        "#,
            stark_struct.nQueries, stark_struct.steps[0].nBits
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input s0_siblings3[{}][{}][4];
        "#,
            stark_struct.nQueries, stark_struct.steps[0].nBits
        ));
    }

    res.push_str(&format!(
        r#"
    signal input s0_siblings4[{}][{}][4];
    signal input s0_siblingsC[{}][{}][4];
        "#,
        stark_struct.nQueries,
        stark_struct.steps[0].nBits,
        stark_struct.nQueries,
        stark_struct.steps[0].nBits
    ));

    for s in 0..(stark_struct.steps.len() - 1) {
        res.push_str(&format!(
            r#"
    signal input s{}_root[4];
        "#,
            s + 1
        ));
    }

    for s in 1..stark_struct.steps.len() {
        res.push_str(&format!(
            r#"
    signal input s{}_vals[{}][{}];
    signal input s{}_siblings[{}][{}][4];
        "#,
            s,
            stark_struct.nQueries,
            (1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits)) * 3,
            s,
            stark_struct.nQueries,
            stark_struct.steps[s].nBits
        ));
    }

    res.push_str(&format!(
        r#"
    signal input finalPol[{}][3];
    "#,
        1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits
    ));

    if options.enable_input {
        res.push_str(
            r#"
    signal input enable;
    enable * (enable -1 ) === 0;
    "#,
        );
    } else {
        res.push_str(
            r#"
    signal enable;
    enable <== 1;
    "#,
        );
    }

    res.push_str(
        r#"
    signal challenges[8][3];
    "#,
    );

    for s in 0..stark_struct.steps.len() {
        res.push_str(&format!(
            r#"
    signal s{}_specialX[3];
    "#,
            s
        ));
    }

    res.push_str(&format!(
        r#"
    signal ys[{}][{}];
    "#,
        stark_struct.nQueries, stark_struct.steps[0].nBits
    ));

    ///////////
    // challenge calculation
    ///////////

    let mut transcript = Transcript::new();
    transcript.put("publics", pil.publics.len() as i32);
    transcript.put("root1", 4);
    transcript.getField("challenges[0]", 3);
    transcript.getField("challenges[1]", 3);
    transcript.put("root2", 4);
    transcript.getField("challenges[2]", 3);
    transcript.getField("challenges[3]", 3);
    transcript.put("root3", 4);
    transcript.getField("challenges[4]", 3);
    transcript.put("root4", 4);
    transcript.getField("challenges[7]", 3);
    for i in 0..starkinfo.ev_map.len() {
        transcript.put(&format!("evals[{}]", i), 3);
    }
    transcript.getField("challenges[5]", 3);
    transcript.getField("challenges[6]", 3);
    for si in 0..stark_struct.steps.len() {
        transcript.getField(&format!("s{}_specialX", si), 3);
        if si < stark_struct.steps.len() - 1 {
            transcript.put(&format!("s{}_root", si + 1), 4);
        } else {
            for j in 0..(1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits) {
                transcript.put(&format!("finalPol[{}]", j), 3);
            }
        }
    }
    transcript.getPermutations("ys", stark_struct.nQueries, stark_struct.steps[0].nBits);
    res.push_str(&transcript.getCode());

    ///////////
    // Constrain polynomial check in valuations
    ///////////

    res.push_str(&format!(
        r#"
    component verifyEvaluations = VerifyEvaluations();
    verifyEvaluations.enable <== enable;
    for (var i=0; i<8; i++) {{
        for (var k=0; k<3; k++) {{
            verifyEvaluations.challenges[i][k] <== challenges[i][k];
        }}
    }}
    for (var i=0; i<{}; i++) {{
        verifyEvaluations.publics[i] <== publics[i];
    }}
    for (var i=0; i<{}; i++) {{
        for (var k=0; k<3; k++) {{
            verifyEvaluations.evals[i][k] <== evals[i][k];
        }}
    }}
    "#,
        pil.publics.len(),
        starkinfo.ev_map.len()
    ));
    ///////////
    // Step0 Check and evaluate queries
    ///////////

    res.push_str(&format!(
        r#"
    component verifyQueries[{}];
    component s0_merkle1[{}];
    "#,
        stark_struct.nQueries, stark_struct.nQueries
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
    component s0_merkle2[{}];
    "#,
            stark_struct.nQueries
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
    component s0_merkle3[{}];
    "#,
            stark_struct.nQueries
        ));
    }

    res.push_str(&format!(
        r#"
    component s0_merkle4[{}];
    component s0_merkleC[{}];
    component s0_lowValues[{}];
    "#,
        stark_struct.nQueries, stark_struct.nQueries, stark_struct.nQueries
    ));

    res.push_str(&format!(
        r#"
    for (var q=0; q<{}; q++) {{
        verifyQueries[q] = VerifyQuery();
        s0_merkle1[q] = MerkleHash(1, {}, {});
    "#,
        stark_struct.nQueries,
        starkinfo.map_sectionsN.get("cm1_2ns"),
        1 << stark_struct.steps[0].nBits
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
        s0_merkle2[q] = MerkleHash(1, {}, {});
    "#,
            starkinfo.map_sectionsN.get("cm2_2ns"),
            1 << stark_struct.steps[0].nBits
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
        s0_merkle3[q] = MerkleHash(1, {}, {});
    "#,
            starkinfo.map_sectionsN.get("cm3_2ns"),
            1 << stark_struct.steps[0].nBits
        ));
    }
    res.push_str(&format!(
        r#"
        s0_merkle4[q] = MerkleHash(1, {}, {});
        s0_merkleC[q] = MerkleHash(1, {}, {});
        s0_lowValues[q] = TreeSelector({}, 3) ;
    "#,
        starkinfo.map_sectionsN.get("cm4_2ns"),
        1 << stark_struct.steps[0].nBits,
        starkinfo.n_constants,
        1 << stark_struct.steps[0].nBits,
        stark_struct.steps[0].nBits
            - (if 0 < stark_struct.steps.len() - 1 { stark_struct.steps[1].nBits } else { 0 })
    ));

    res.push_str(&format!(
        r#"
        for (var i=0; i<{}; i++ ) {{
            verifyQueries[q].ys[i] <== ys[q][i];
            s0_merkle1[q].key[i] <== ys[q][i];
    "#,
        stark_struct.steps[0].nBits
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(
            r#"
            s0_merkle2[q].key[i] <== ys[q][i];
    "#,
        );
    }
    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(
            r#"
            s0_merkle3[q].key[i] <== ys[q][i];
    "#,
        );
    }

    res.push_str(&format!(
        r#"
            s0_merkle4[q].key[i] <== ys[q][i];
            s0_merkleC[q].key[i] <== ys[q][i];
        }}
        for (var i=0; i<{}; i++ ) {{
            verifyQueries[q].tree1[i] <== s0_vals1[q][i];
            s0_merkle1[q].values[i][0] <== s0_vals1[q][i];
        }}
    "#,
        starkinfo.map_sectionsN.get("cm1_2ns")
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
        for (var i=0; i<{}; i++ ) {{
            verifyQueries[q].tree2[i] <== s0_vals2[q][i];
            s0_merkle2[q].values[i][0] <== s0_vals2[q][i];
        }}
    "#,
            starkinfo.map_sectionsN.get("cm2_2ns")
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
        for (var i=0; i<{}; i++ ) {{
            verifyQueries[q].tree3[i] <== s0_vals3[q][i];
            s0_merkle3[q].values[i][0] <== s0_vals3[q][i];
        }}
    "#,
            starkinfo.map_sectionsN.get("cm3_2ns")
        ));
    }

    res.push_str(&format!(
        r#"
        for (var i=0; i<{}; i++ ) {{
            verifyQueries[q].tree4[i] <== s0_vals4[q][i];
            s0_merkle4[q].values[i][0] <== s0_vals4[q][i];
        }}
        for (var i=0; i<{}; i++ ) {{
            verifyQueries[q].consts[i] <== s0_valsC[q][i];
            s0_merkleC[q].values[i][0] <== s0_valsC[q][i];
        }}
        for (var i=0; i<8; i++) {{
            for (var e=0; e<3; e++) {{
                verifyQueries[q].challenges[i][e] <== challenges[i][e];
            }}
        }}
        for (var i=0; i<{}; i++) {{
            for (var e=0; e<3; e++) {{
                verifyQueries[q].evals[i][e] <== evals[i][e];
            }}
        }}
        for (var i=0; i<{};i++) {{
            for (var j=0; j<4; j++) {{
                s0_merkle1[q].siblings[i][j] <== s0_siblings1[q][i][j];
    "#,
        starkinfo.map_sectionsN.get("cm4_2ns"),
        starkinfo.n_constants,
        starkinfo.ev_map.len(),
        stark_struct.steps[0].nBits
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(
            r#"
                s0_merkle2[q].siblings[i][j] <== s0_siblings2[q][i][j];
        "#,
        );
    }
    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(
            r#"
                s0_merkle3[q].siblings[i][j] <== s0_siblings3[q][i][j];
        "#,
        );
    }

    res.push_str(
        r#"
                s0_merkle4[q].siblings[i][j] <== s0_siblings4[q][i][j];
                s0_merkleC[q].siblings[i][j] <== s0_siblingsC[q][i][j];
            }
        }
        "#,
    );

    if 0 < stark_struct.steps.len() - 1 {
        res.push_str(&format!(
            r#"
        for (var i=0; i<{}; i++) {{
            for (var e=0; e<3; e++) {{
                s0_lowValues[q].values[i][e] <== s1_vals[q][i*3+e];
            }}
        }}
        for (var i=0; i<{}; i++) {{
            s0_lowValues[q].key[i] <== ys[q][i + {}];
        }}
        "#,
            1 << (stark_struct.steps[0].nBits - stark_struct.steps[1].nBits),
            (stark_struct.steps[0].nBits - stark_struct.steps[1].nBits),
            stark_struct.steps[1].nBits
        ));
    } else {
        res.push_str(&format!(
            r#"
        for (var i=0; i<{}; i++) {{
            for (var e=0; e<3; e++) {{
                s0_lowValues[q].values[i][e] <== finalPol[i][e];
            }}
        }}
        for (var i=0; i<{}; i++) {{
            s0_lowValues[q].key[i] <== ys[q][i];
        }}
        "#,
            1 << stark_struct.steps[0].nBits,
            stark_struct.steps[0].nBits
        ));
    }

    res.push_str(
        r#"
    }
        "#,
    );

    for s in 1..stark_struct.steps.len() {
        res.push_str(&format!(
            r#"
    component s{}_merkle[{}];
    component s{}_fft[{}];
    component s{}_evalPol[{}];
    component s{}_lowValues[{}];
    signal s{}_sx[{}][{}];
        "#,
            s,
            stark_struct.nQueries,
            s,
            stark_struct.nQueries,
            s,
            stark_struct.nQueries,
            s,
            stark_struct.nQueries,
            s,
            stark_struct.nQueries,
            stark_struct.steps[s].nBits,
        ));

        let nbits =
            if s < stark_struct.steps.len() - 1 { stark_struct.steps[s + 1].nBits } else { 0 };
        let selector = stark_struct.steps[s].nBits - nbits;

        res.push_str(&format!(
            r#"
    for (var q=0; q<{}; q++) {{
        s{}_merkle[q] = MerkleHash(3, {}, {});
        s{}_fft[q] = FFT({}, 3, 1);
        s{}_evalPol[q] = EvalPol({});
        s{}_lowValues[q] = TreeSelector({}, 3) ;
        for (var i=0; i< {}; i++) {{
            for (var e=0; e<3; e++) {{
                s{}_merkle[q].values[i][e] <== s{}_vals[q][i*3+e];
                s{}_fft[q].in[i][e] <== s{}_vals[q][i*3+e];
            }}
        }}
        "#,
            stark_struct.nQueries,
            s,
            1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits),
            1 << stark_struct.steps[s].nBits,
            s,
            stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits,
            s,
            1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits),
            s,
            selector,
            1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits),
            s,
            s,
            s,
            s
        ));

        res.push_str(&format!(
            r#"
        for (var i=0; i<{}; i++) {{
            for (var j=0; j<4; j++) {{
                s{}_merkle[q].siblings[i][j] <== s{}_siblings[q][i][j];
            }}
            s{}_merkle[q].key[i] <== ys[q][i];
        }}
        s{}_sx[q][0] <==  {} *  ( ys[q][0] * {} +1);
        for (var i=1; i<{}; i++) {{
            s{}_sx[q][i] <== s{}_sx[q][i-1] *  ( ys[q][i] * ((1/roots({} -i)) -1) +1);
        }}
        for (var i=0; i< {}; i++) {{
            for (var e=0; e<3; e++) {{
                s{}_evalPol[q].pol[i][e] <== s{}_fft[q].out[i][e];
            }}
        }}
        for (var e=0; e<3; e++) {{
            s{}_evalPol[q].x[e] <== s{}_specialX[e] *  s{}_sx[q][{}];
        }}
        "#,
            stark_struct.steps[s].nBits,
            s,
            s,
            s,
            s,
            // we need to use F3G::from(constant.clone()) as the ownership of constant(Here we mean SHIFT) will be moved into the from function
            F3G::from(*SHIFT)
                .exp(1 << (stark_struct.nBitsExt - stark_struct.steps[s - 1].nBits))
                .inv()
                .as_int(),
            (F3G::from(MG.0[stark_struct.steps[s - 1].nBits]).inv() - F3G::ONE).as_int(),
            stark_struct.steps[s].nBits,
            s,
            s,
            stark_struct.steps[s - 1].nBits,
            1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits),
            s,
            s,
            s,
            s,
            s,
            stark_struct.steps[s].nBits - 1
        ));

        if s < stark_struct.steps.len() - 1 {
            res.push_str(&format!(
                r#"
        for (var i=0; i<{}; i++) {{
            for (var e=0; e<3; e++) {{
                s{}_lowValues[q].values[i][e] <== s{}_vals[q][i*3+e];
            }}
        }}
        for (var i=0; i<{}; i++) {{
            s{}_lowValues[q].key[i] <== ys[q][i + {}];
        }}
        "#,
                1 << (stark_struct.steps[s].nBits - stark_struct.steps[s + 1].nBits),
                s,
                s + 1,
                stark_struct.steps[s].nBits - stark_struct.steps[s + 1].nBits,
                s,
                stark_struct.steps[s + 1].nBits
            ));
        } else {
            res.push_str(&format!(
                r#"
        for (var i=0; i<{}; i++) {{
            for (var e=0; e<3; e++) {{
                s{}_lowValues[q].values[i][e] <== finalPol[i][e];
            }}
        }}
        for (var i=0; i<{}; i++) {{
            s{}_lowValues[q].key[i] <== ys[q][i];
        }}
        "#,
                1 << stark_struct.steps[s].nBits,
                s,
                stark_struct.steps[s].nBits,
                s
            ));
        }

        //// Checks
        let enable2 = if starkinfo.map_sectionsN.cm2_2ns > 0 {
            "enable * (s0_merkle2[q].root[j] - root2[j]) === 0;"
        } else {
            ""
        };
        let enable3 = if starkinfo.map_sectionsN.cm3_2ns > 0 {
            "enable * (s0_merkle3[q].root[j] - root3[j]) === 0;"
        } else {
            ""
        };
        res.push_str(&format!(
            r#"
        for(var q = 0; q < {}; q ++) {{
            for(var j = 0; j < 4; j ++) {{
                enable * (s0_merkle1[q].root[j] - root1[j]) === 0;
                {}
                {}
                enable * (s0_merkle4[q].root[j] - root4[j]) === 0;
                enable * (s0_merkleC[q].root[j] - rootC[j]) === 0;
            }}
            for (var e = 0; e < 3; e ++) {{
                enable * (s0_lowValues[q].out[e] - verifyQueries[q].out[e]) === 0;
            }}
        }}
        "#,
            stark_struct.nQueries, enable2, enable3
        ));

        res.push_str(&format!(
            r#"
        for (var e=0; e<3; e++) {{
            enable * (s{}_lowValues[q].out[e] - s{}_evalPol[q].out[e]) === 0;
        }}

        enable * (s{}_merkle[q].root[0] - s{}_root[0]) === 0;
        enable * (s{}_merkle[q].root[1] - s{}_root[1]) === 0;
        enable * (s{}_merkle[q].root[2] - s{}_root[2]) === 0;
        enable * (s{}_merkle[q].root[3] - s{}_root[3]) === 0;
    }}
        "#,
            s, s, s, s, s, s, s, s, s, s
        ));
    }

    ///////
    // Check Degree last pol
    ///////
    // Last FFT
    let nLastBits = stark_struct.steps[stark_struct.steps.len() - 1].nBits;
    let maxDegBits = nLastBits - (stark_struct.nBitsExt - stark_struct.nBits);

    res.push_str(&format!(
        r#"
    component lastIFFT = FFT({}, 3, 1);

    for (var k=0; k< {}; k++ ){{
        for (var e=0; e<3; e++) {{
            lastIFFT.in[k][e] <== finalPol[k][e];
        }}
    }}

    for (var k= {}; k< {}; k++ ) {{
        for (var e=0; e<3; e++) {{
            enable * lastIFFT.out[k][e] === 0;
        }}
    }}
}}

"#,
        nLastBits,
        1 << nLastBits,
        1 << maxDegBits,
        1 << nLastBits
    ));

    // Normalization Stage

    if !options.skip_main && !options.verkey_input {
        res.push_str(&format!(
            r#"
template Main() {{
    signal input publics[{}];
    signal input root1[4];
    signal input root2[4];
    signal input root3[4];
    signal input root4[4];

    signal input rootC[4];
    "#,
            pil.publics.len()
        ));

        res.push_str(&format!(
            r#"
    signal input evals[{}][3];
    signal input s0_vals1[{}][{}];
        "#,
            starkinfo.ev_map.len(),
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm1_2ns")
        ));

        if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_vals2[{}][{}];
            "#,
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm2_2ns")
            ));
        }

        if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_vals3[{}][{}];
            "#,
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm3_2ns")
            ));
        }

        res.push_str(&format!(
            r#"
    signal input s0_vals4[{}][{}];
    signal input s0_valsC[{}][{}];
    signal input s0_siblings1[{}][{}][4];
    "#,
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm4_2ns"),
            stark_struct.nQueries,
            starkinfo.n_constants,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits
        ));

        if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_siblings2[{}][{}][4];
            "#,
                stark_struct.nQueries, stark_struct.steps[0].nBits
            ));
        }

        if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_siblings3[{}][{}][4];
            "#,
                stark_struct.nQueries, stark_struct.steps[0].nBits
            ));
        }

        res.push_str(&format!(
            r#"
    signal input s0_siblings4[{}][{}][4];
    signal input s0_siblingsC[{}][{}][4];
            "#,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits
        ));

        for s in 0..(stark_struct.steps.len() - 1) {
            res.push_str(&format!(
                r#"
        signal input s{}_root[4];
            "#,
                s + 1
            ));
        }

        for s in 1..stark_struct.steps.len() {
            res.push_str(&format!(
                r#"
    signal input s{}_vals[{}][{}];
    signal input s{}_siblings[{}][{}][4];
            "#,
                s,
                stark_struct.nQueries,
                (1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits)) * 3,
                s,
                stark_struct.nQueries,
                stark_struct.steps[s].nBits
            ));
        }

        res.push_str(&format!(
            r#"
    signal input finalPol[{}][3];
        "#,
            1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits
        ));

        res.push_str(
            r#"
    component vA = StarkVerifier();

    vA.publics <== publics;

    vA.root1 <== root1;
    vA.root2 <== root2;
    vA.root3 <== root3;
    vA.root4 <== root4;
    vA.evals <== evals;
    vA.s0_vals1 <== s0_vals1;
    vA.s0_vals3 <== s0_vals3;
    vA.s0_vals4 <== s0_vals4;
    vA.s0_valsC <== s0_valsC;
    vA.s0_siblings1 <== s0_siblings1;
    vA.s0_siblings3 <== s0_siblings3;
    vA.s0_siblings4 <== s0_siblings4;
    vA.s0_siblingsC <== s0_siblingsC;

    vA.finalPol <== finalPol;
            "#,
        );

        for s in 1..(stark_struct.steps.len()) {
            res.push_str(&format!(
                r#"
    vA.s{}_root <== s{}_root;
    vA.s{}_vals <== s{}_vals;
    vA.s{}_siblings <== s{}_siblings;
            "#,
                s, s, s, s, s, s,
            ));
        }

        res.push_str(
            r#"
}
            "#,
        )
    }

    if options.verkey_input && !options.agg_stage {
        res.push_str(&format!(
            r#"
template Main() {{
    signal input publics[{}];
    signal input root1[4];
    signal input root2[4];
    signal input root3[4];
    signal input root4[4];

    signal input rootC[4];
    "#,
            pil.publics.len()
        ));

        res.push_str(&format!(
            r#"
    signal input evals[{}][3];
    signal input s0_vals1[{}][{}];
        "#,
            starkinfo.ev_map.len(),
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm1_2ns")
        ));

        if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_vals2[{}][{}];
            "#,
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm2_2ns")
            ));
        }

        if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_vals3[{}][{}];
            "#,
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm3_2ns")
            ));
        }

        res.push_str(&format!(
            r#"
    signal input s0_vals4[{}][{}];
    signal input s0_valsC[{}][{}];
    signal input s0_siblings1[{}][{}][4];
    "#,
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm4_2ns"),
            stark_struct.nQueries,
            starkinfo.n_constants,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits
        ));

        if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_siblings2[{}][{}][4];
            "#,
                stark_struct.nQueries, stark_struct.steps[0].nBits
            ));
        }

        if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_siblings3[{}][{}][4];
            "#,
                stark_struct.nQueries, stark_struct.steps[0].nBits
            ));
        }

        res.push_str(&format!(
            r#"
    signal input s0_siblings4[{}][{}][4];
    signal input s0_siblingsC[{}][{}][4];
            "#,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits
        ));

        for s in 0..(stark_struct.steps.len() - 1) {
            res.push_str(&format!(
                r#"
        signal input s{}_root[4];
            "#,
                s + 1
            ));
        }

        for s in 1..stark_struct.steps.len() {
            res.push_str(&format!(
                r#"
    signal input s{}_vals[{}][{}];
    signal input s{}_siblings[{}][{}][4];
            "#,
                s,
                stark_struct.nQueries,
                (1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits)) * 3,
                s,
                stark_struct.nQueries,
                stark_struct.steps[s].nBits
            ));
        }

        res.push_str(&format!(
            r#"
    signal input finalPol[{}][3];
        "#,
            1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits
        ));

        res.push_str(
            r#"
    component vA = StarkVerifier();

    vA.publics <== publics;

    vA.root1 <== root1;
    vA.root2 <== root2;
    vA.root3 <== root3;
    vA.root4 <== root4;
    vA.rootC <== rootC;
    vA.evals <== evals;
    vA.s0_vals1 <== s0_vals1;
    vA.s0_vals3 <== s0_vals3;
    vA.s0_vals4 <== s0_vals4;
    vA.s0_valsC <== s0_valsC;
    vA.s0_siblings1 <== s0_siblings1;
    vA.s0_siblings3 <== s0_siblings3;
    vA.s0_siblings4 <== s0_siblings4;
    vA.s0_siblingsC <== s0_siblingsC;

    vA.finalPol <== finalPol;
            "#,
        );

        for s in 1..(stark_struct.steps.len()) {
            res.push_str(&format!(
                r#"
    vA.s{}_root <== s{}_root;
    vA.s{}_vals <== s{}_vals;
    vA.s{}_siblings <== s{}_siblings;
            "#,
                s, s, s, s, s, s,
            ));
        }

        res.push_str(
            r#"
}
            "#,
        )
    }

    ///////
    // Aggregation Stage
    ///////

    if options.agg_stage {
        //let const_roots = const_root.as_elements();
        res.push_str(&format!(
            r#"
template Main() {{

    signal input publics[{}];
    signal input rootC[4];

    "#,
            pil.publics.len() - 4,
        ));
        res.push_str(&format!(
            r#"
    signal input a_publics[{}];
    signal input a_root1[4];
    signal input a_root2[4];
    signal input a_root3[4];
    signal input a_root4[4];
    signal input a_rootC[4];

    signal input b_publics[{}];
    signal input b_root1[4];
    signal input b_root2[4];
    signal input b_root3[4];
    signal input b_root4[4];
    signal input b_rootC[4];
    "#,
            pil.publics.len(),
            pil.publics.len()
        ));

        res.push_str(&format!(
            r#"
    signal input a_evals[{}][3];
    signal input a_s0_vals1[{}][{}];

    signal input b_evals[{}][3];
    signal input b_s0_vals1[{}][{}];
        "#,
            starkinfo.ev_map.len(),
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm1_2ns"),
            starkinfo.ev_map.len(),
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm1_2ns")
        ));

        if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input a_s0_vals2[{}][{}];
    signal input b_s0_vals2[{}][{}];
            "#,
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm2_2ns"),
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm2_2ns")
            ));
        }

        if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input a_s0_vals3[{}][{}];

    signal input b_s0_vals3[{}][{}];
            "#,
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm3_2ns"),
                stark_struct.nQueries,
                starkinfo.map_sectionsN.get("cm3_2ns")
            ));
        }

        res.push_str(&format!(
            r#"
    signal input a_s0_vals4[{}][{}];
    signal input a_s0_valsC[{}][{}];
    signal input a_s0_siblings1[{}][{}][4];

    signal input b_s0_vals4[{}][{}];
    signal input b_s0_valsC[{}][{}];
    signal input b_s0_siblings1[{}][{}][4];
    "#,
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm4_2ns"),
            stark_struct.nQueries,
            starkinfo.n_constants,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits,
            stark_struct.nQueries,
            starkinfo.map_sectionsN.get("cm4_2ns"),
            stark_struct.nQueries,
            starkinfo.n_constants,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits
        ));

        if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input a_s0_siblings2[{}][{}][4];

    signal input b_s0_siblings2[{}][{}][4];
            "#,
                stark_struct.nQueries,
                stark_struct.steps[0].nBits,
                stark_struct.nQueries,
                stark_struct.steps[0].nBits
            ));
        }

        if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
            res.push_str(&format!(
                r#"
    signal input a_s0_siblings3[{}][{}][4];

    signal input b_s0_siblings3[{}][{}][4];
            "#,
                stark_struct.nQueries,
                stark_struct.steps[0].nBits,
                stark_struct.nQueries,
                stark_struct.steps[0].nBits,
            ));
        }

        res.push_str(&format!(
            r#"
    signal input a_s0_siblings4[{}][{}][4];
    signal input a_s0_siblingsC[{}][{}][4];

    signal input b_s0_siblings4[{}][{}][4];
    signal input b_s0_siblingsC[{}][{}][4];
            "#,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits,
            stark_struct.nQueries,
            stark_struct.steps[0].nBits
        ));

        for s in 0..(stark_struct.steps.len() - 1) {
            res.push_str(&format!(
                r#"
        signal input a_s{}_root[4];

        signal input b_s{}_root[4];
            "#,
                s + 1,
                s + 1
            ));
        }

        for s in 1..stark_struct.steps.len() {
            res.push_str(&format!(
                r#"
    signal input a_s{}_vals[{}][{}];
    signal input a_s{}_siblings[{}][{}][4];

    signal input b_s{}_vals[{}][{}];
    signal input b_s{}_siblings[{}][{}][4];
            "#,
                s,
                stark_struct.nQueries,
                (1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits)) * 3,
                s,
                stark_struct.nQueries,
                stark_struct.steps[s].nBits,
                s,
                stark_struct.nQueries,
                (1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits)) * 3,
                s,
                stark_struct.nQueries,
                stark_struct.steps[s].nBits
            ));
        }

        res.push_str(&format!(
            r#"
    signal input a_finalPol[{}][3];

    signal input b_finalPol[{}][3];
        "#,
            1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits,
            1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits,
        ));

        res.push_str(&format!(
            r#"
    component vA = StarkVerifier();

    for (var i=0; i<{}; i++) {{
        vA.publics[i] <== a_publics[i];
    }}

    vA.root1 <== a_root1;
    vA.root2 <== a_root2;
    vA.root3 <== a_root3;
    vA.root4 <== a_root4;
    vA.rootC <== a_rootC;
    vA.evals <== a_evals;
    vA.s0_vals1 <== a_s0_vals1;
    vA.s0_vals3 <== a_s0_vals3;
    vA.s0_vals4 <== a_s0_vals4;
    vA.s0_valsC <== a_s0_valsC;
    vA.s0_siblings1 <== a_s0_siblings1;
    vA.s0_siblings3 <== a_s0_siblings3;
    vA.s0_siblings4 <== a_s0_siblings4;
    vA.s0_siblingsC <== a_s0_siblingsC;

    vA.finalPol <== a_finalPol;
            "#,
            pil.publics.len()
        ));
        // component isOneBatchA = IsZero();
        // isOneBatchA.in  <== a_publics[43] - a_publics[16] - 1; a_publics[43]-> newBatchNum;  a_publics[16]-> oldBatchNum
        // TODO: "vA.rootC <== rootCSingle;" this need to change!!!

        for s in 1..(stark_struct.steps.len()) {
            res.push_str(&format!(
                r#"
    vA.s{}_root <== a_s{}_root;
    vA.s{}_vals <== a_s{}_vals;
    vA.s{}_siblings <== a_s{}_siblings;
            "#,
                s, s, s, s, s, s,
            ));
        }

        res.push_str(&format!(
            r#"
    component vB = StarkVerifier();
    for (var i=0; i<{}; i++) {{
        vB.publics[i] <== b_publics[i];
    }}

    vB.root1 <== b_root1;
    vB.root2 <== b_root2;
    vB.root3 <== b_root3;
    vB.root4 <== b_root4;
    vB.rootC <== b_rootC;
    vB.evals <== b_evals;
    vB.s0_vals1 <== b_s0_vals1;
    vB.s0_vals3 <== b_s0_vals3;
    vB.s0_vals4 <== b_s0_vals4;
    vB.s0_valsC <== b_s0_valsC;
    vB.s0_siblings1 <== b_s0_siblings1;
    vB.s0_siblings3 <== b_s0_siblings3;
    vB.s0_siblings4 <== b_s0_siblings4;
    vB.s0_siblingsC <== b_s0_siblingsC;

    vB.finalPol <== b_finalPol;
            "#,
            pil.publics.len()
        ));

        for s in 1..(stark_struct.steps.len()) {
            res.push_str(&format!(
                r#"
    vB.s{}_root <== b_s{}_root;
    vB.s{}_vals <== b_s{}_vals;
    vB.s{}_siblings <== b_s{}_siblings;
            "#,
                s, s, s, s, s, s,
            ));
        }

        res.push_str(
            r#"
}
            "#,
        )
    }

    // generate the main component
    if !options.skip_main {
        // if options.agg_stage {
        res.push_str(
            r#"
component main {public [publics, rootC]}= Main();
    "#,
        );
        // }
    } else {
        res.push_str(
            r#"
component main {public [publics]}= StarkVerifier();
"#,
        );
    }
    res
}

// Support goldilocks
pub fn render<F: ff::PrimeField + Default>(
    starkinfo: &StarkInfo,
    prorgam: &Program,
    pil: &PIL,
    stark_struct: &StarkStruct,
    const_root: &ElementDigest<4, F>,
    options: &StarkOption,
) -> String {
    let mut res = header(options);
    res.push_str(&verify_evaluations(starkinfo, prorgam, pil, stark_struct));
    res.push_str(&verify_query(starkinfo, prorgam, stark_struct));
    res.push_str(&map_values(starkinfo));
    res.push_str(&stark_verifier(starkinfo, pil, stark_struct, const_root, options));
    res
}
