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

fn header() -> String {
    let header = r#"pragma circom 2.0.6;

include "gl.circom";
include "poseidon.circom";
include "bitify.circom";
include "sha256/sha256.circom";
include "fft.circom";
include "merklehash.circom";
include "evalpol.circom";
include "treeselector.circom";
include "bn1togl3.circom";
include "compconstant64.circom";
"#;

    String::from(header)
}

#[derive(Default)]
struct Transcript {
    state: String,
    pending: Vec<String>,
    out: Vec<String>,
    out3: Vec<String>,
    h_cnt: usize,
    n2b_cnt: usize,
    code: Vec<String>,
    bn1togl3Cnt: usize,

    stark_struct: StarkStruct,
}

impl Transcript {
    pub fn new(stark_struct: StarkStruct) -> Self {
        Self {
            state: String::from("0"),
            pending: vec![],
            out: vec![],
            out3: vec![],
            h_cnt: 0,
            n2b_cnt: 0,
            code: vec![],
            bn1togl3Cnt: 0,
            stark_struct,
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
        if !self.out3.is_empty() {
            let res = self.out3[0].to_owned();
            self.out3.remove(0);
            return res;
        }
        if !self.out.is_empty() {
            let cName = format!("bn1togl3_{}", self.bn1togl3Cnt);
            self.bn1togl3Cnt += 1;
            self.code.push(format!("component {} = BN1toGL3();", cName));
            let first = self.out[0].to_owned();
            self.out.remove(0);
            self.code.push(format!("{}.in <== {};", cName, first));

            self.out3.push(format!("{}.out[0]", cName));
            self.out3.push(format!("{}.out[1]", cName));
            self.out3.push(format!("{}.out[2]", cName));
            return self.getFields1();
        }
        self.updateState();
        self.getFields1()
    }

    fn getFields253(&mut self) -> String {
        if !self.out.is_empty() {
            let res = self.out[0].to_owned();
            self.out.remove(0);
            return res;
        }
        self.updateState();
        self.getFields253()
    }

    fn updateState(&mut self) {
        while self.pending.len() < 16 {
            self.pending.push("0".to_string());
        }
        self.code.push(format!("component tcHahs_{} = PoseidonEx(16,17);", self.h_cnt));
        self.h_cnt += 1;

        for i in 0..16 {
            self.code.push(format!(
                "tcHahs_{}.inputs[{}] <== {};",
                self.h_cnt - 1,
                i,
                self.pending[i]
            ));
        }

        self.out = vec![];
        for i in 0..17 {
            self.out.push(format!("tcHahs_{}.out[{}]", self.h_cnt - 1, i));
        }
        self.out3 = vec![];
        self.code.push(format!("tcHahs_{}.initialState <== {};", self.h_cnt - 1, self.state));
        self.state = format!("tcHahs_{}.out[0]", self.h_cnt - 1);
        self.pending = vec![];
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
        self.out3 = vec![];
        self.pending.push(a.to_string());
        if self.pending.len() == 16 {
            self.updateState();
        }
    }

    pub fn getPermutations(&mut self, v: &str, n: usize, nBits: usize) {
        let totalBits = n * nBits;
        let NFields = (totalBits - 1) / 253 + 1;
        let mut n2b: Vec<String> = vec![];
        let n2bt = match self.stark_struct.verificationHashType.as_str() {
            "BN128" => "Num2Bits_strict()".to_string(),
            "BLS12381" => "Num2Bits(255)".to_string(),
            _ => todo!(),
        };
        for i in 0..NFields {
            let f = self.getFields253();
            n2b.push(format!("tcN2b_{}", self.n2b_cnt));
            self.n2b_cnt += 1;
            self.code.push(format!("component {} = {};", n2b[i], n2bt));
            self.code.push(format!("{}.in <== {};", n2b[i], f));
        }
        let mut curField = 0;
        let mut curBit = 0;
        for i in 0..n {
            for j in 0..nBits {
                self.code
                    .push(format!("{}[{}][{}] <== {}.out[{}];", v, i, j, n2b[curField], curBit));
                curBit += 1;
                if curBit == 253 {
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
            "xDivXSubXi" => "xDivXSubXi.out".to_string(),
            "xDivXSubWXi" => "xDivXSubWXi.out".to_string(),
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
        match inst.op.as_str() {
            "add" => {
                if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {} <== {}[0] + {}[0];"#,
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
                        ref_(&inst.src[1])
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
    signal {}[3] <== [{} - {}[0] + p, -{}[1] + p, -{}[2] + p];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{}[0] - {} + p, {}[1], {}[2]];"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[0])
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== [{}[0] - {}[0] + p, {}[1] - {}[1] + p, {}[2] - {}[2] + p];"#,
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
    signal {} = GLCMul1()({}, {});"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== GLCMul()([{}, 0, 0], {});"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1])
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== GLCMul()({}, [{}, 0, 0]);"#,
                        ref_(&inst.dest),
                        ref_(&inst.src[0]),
                        ref_(&inst.src[1]),
                    ));
                } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== GLCMul()({}, {});"#,
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
                if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                    if inst.src[2].dim == 1 {
                        str_code.push_str(&format!(
                            r#"
    signal {} <== GLMulAdd()({}, {}, {});"#,
                            ref_(&inst.dest),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[2])
                        ));
                    } else {
                        str_code.push_str(&format!(
                            r#"
    signal {}[3] <== [GLMulAdd()({}, {}, {}[0]), {}[1], {}[2]);"#,
                            ref_(&inst.dest),
                            ref_(&inst.src[0]),
                            ref_(&inst.src[1]),
                            ref_(&inst.src[2]),
                            ref_(&inst.src[2]),
                            ref_(&inst.src[2])
                        ));
                    }
                } else {
                    let ina = match inst.src[0].dim {
                        1 => format!("[{}, 0, 0]", ref_(&inst.src[0])),
                        _ => ref_(&inst.src[0]),
                    };

                    let inb = match inst.src[1].dim {
                        1 => format!("[{}, 0, 0]", ref_(&inst.src[1])),
                        _ => ref_(&inst.src[1]),
                    };

                    let inc = match inst.src[2].dim {
                        1 => format!("[{}, 0, 0]", ref_(&inst.src[2])),
                        _ => ref_(&inst.src[2]),
                    };
                    str_code.push_str(&format!(
                        r#"
    signal {}[3] <== GLCMulAdd()({}, {}, {});"#,
                        ref_(&inst.dest),
                        ina,
                        inb,
                        inc
                    ));
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

    var p = 0xFFFFFFFF00000001;
"#,
        starkinfo.ev_map.len(),
        pil.publics.len()
    );

    res.push_str(&format!(
        r#"
    component zMul[{}];
    "#,
        stark_struct.nBits
    ));

    res.push_str(&format!(
        r#"
    for (var i=0; i< {}; i++) {{
        zMul[i] = GLCMul();
        if (i==0) {{
            zMul[i].ina[0] <== challenges[7][0];
            zMul[i].ina[1] <== challenges[7][1];
            zMul[i].ina[2] <== challenges[7][2];
            zMul[i].inb[0] <== challenges[7][0];
            zMul[i].inb[1] <== challenges[7][1];
            zMul[i].inb[2] <== challenges[7][2];
        }} else {{
            zMul[i].ina[0] <== zMul[i-1].out[0];
            zMul[i].ina[1] <== zMul[i-1].out[1];
            zMul[i].ina[2] <== zMul[i-1].out[2];
            zMul[i].inb[0] <== zMul[i-1].out[0];
            zMul[i].inb[1] <== zMul[i-1].out[1];
            zMul[i].inb[2] <== zMul[i-1].out[2];
        }}
    }}
        "#,
        stark_struct.nBits
    ));

    res.push_str(&format!(
        r#"
    signal Z[3];

    Z[0] <== zMul[{}].out[0] -1 + p;
    Z[1] <== zMul[{}].out[1];
    Z[2] <== zMul[{}].out[2];"#,
        stark_struct.nBits - 1,
        stark_struct.nBits - 1,
        stark_struct.nBits - 1,
    ));

    let (tmpCode, evalP) = unrollCode(&program.verifier_code.first, starkinfo);
    res.push_str(&tmpCode);

    res.push_str(&format!(
        r#"
    signal xN[3] <== zMul[{}].out;

    signal xAcc[{}][3];
    signal qStep[{}][3];
    signal qAcc[{}][3];
    for (var i=0; i< {}; i++) {{
        if (i==0) {{
            xAcc[0] <== [1, 0, 0];
            qAcc[0] <== evals[{}+i];
        }} else {{
            xAcc[i] <== GLCMul()(xAcc[i-1], xN);
            qStep[i-1] <== GLCMul()(xAcc[i], evals[{}+i]);

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
    signal qZ[3] <== GLCMul()(qAcc[{}], Z);

// Final Verification
    component normC = GLCNorm();
    normC.in[0] <== {}[0] - qZ[0];
    normC.in[1] <== {}[1] - qZ[1];
    normC.in[2] <== {}[2] - qZ[2];

    enable * normC.out[0] === 0;
    enable * normC.out[1] === 0;
    enable * normC.out[2] === 0;
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
    var p = 0xFFFFFFFF00000001;

    component xacc[{}-1];
    for (var i=1; i<{}; i++ ) {{
        xacc[i-1] = GLMul();
        if (i==1) {{
            xacc[i-1].ina <== ys[0]*({} * roots({})-{}) + {};
        }} else {{
            xacc[i-1].ina <== xacc[i-2].out;
        }}
        xacc[i-1].inb <== ys[i]*(roots({} - i) - 1) +1;
    }}"#,
        stark_struct.steps[0].nBits,
        stark_struct.steps[0].nBits,
        SHIFT.as_int(),
        stark_struct.steps[0].nBits,
        SHIFT.as_int(),
        SHIFT.as_int(),
        stark_struct.steps[0].nBits
    ));

    if stark_struct.steps[0].nBits > 1 {
        res.push_str(&format!(
            r#"
    signal X <== xacc[{}].out;
        "#,
            stark_struct.steps[0].nBits - 2
        ));
    } else {
        res.push_str(&format!(
            r#"
    signal X <== ys[0]*({} * roots({})-{}) + {};
        "#,
            SHIFT.as_int(),
            stark_struct.steps[0].nBits,
            SHIFT.as_int(),
            SHIFT.as_int()
        ));
    }

    res.push_str(&format!(
        r#"
    component den1inv = GLCInv();
    den1inv.in[0] <== X - challenges[7][0] + p;
    den1inv.in[1] <== -challenges[7][1] + p;
    den1inv.in[2] <== -challenges[7][2] + p;

    component xDivXSubXi = GLCMul();
    xDivXSubXi.ina[0] <== X;
    xDivXSubXi.ina[1] <== 0;
    xDivXSubXi.ina[2] <== 0;
    xDivXSubXi.inb[0] <== den1inv.out[0];
    xDivXSubXi.inb[1] <== den1inv.out[1];
    xDivXSubXi.inb[2] <== den1inv.out[2];

    component wXi = GLCMul();
    wXi.ina[0] <== roots({});
    wXi.ina[1] <== 0;
    wXi.ina[2] <== 0;
    wXi.inb[0] <== challenges[7][0];
    wXi.inb[1] <== challenges[7][1];
    wXi.inb[2] <== challenges[7][2];

    component den2inv = GLCInv();
    den2inv.in[0] <== X - wXi.out[0] + p;
    den2inv.in[1] <== -wXi.out[1] + p;
    den2inv.in[2] <== -wXi.out[2] + p;

    component xDivXSubWXi = GLCMul();
    xDivXSubWXi.ina[0] <== X;
    xDivXSubWXi.ina[1] <== 0;
    xDivXSubWXi.ina[2] <== 0;
    xDivXSubWXi.inb[0] <== den2inv.out[0];
    xDivXSubWXi.inb[1] <== den2inv.out[1];
    xDivXSubWXi.inb[2] <== den2inv.out[2];
    "#,
        stark_struct.nBits
    ));

    let (tmpCode, evalQ) = unrollCode(&program.verifier_query_code.first, starkinfo);
    res.push_str(&tmpCode);

    // Final Normalization
    res.push_str(&format!(
        r#"
    component normC = GLCNorm();
    normC.in[0] <== {}[0];
    normC.in[1] <== {}[1];
    normC.in[2] <== {}[2];

    out[0] <== normC.out[0];
    out[1] <== normC.out[1];
    out[2] <== normC.out[2];
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
    signal input root1;
    signal input root2;
    signal input root3;
    signal input root4;
"#,
        pil.publics.len()
    );

    if options.verkey_input {
        res.push_str(
            r#"
    signal input rootC;
"#,
        );
    } else {
        let repr = (*const_root).as_scalar::<F>();
        let c: F = F::from_raw_repr(repr).expect("Failed to create new Fr from_raw_repr");
        res.push_str(&format!(
            r#"
    signal rootC;
    rootC <== {};
"#,
            crate::helper::fr_to_biguint(&c)
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
    signal input s0_siblings1[{}][{}][16];
"#,
        stark_struct.nQueries,
        starkinfo.map_sectionsN.get("cm4_2ns"),
        stark_struct.nQueries,
        starkinfo.n_constants,
        stark_struct.nQueries,
        (stark_struct.steps[0].nBits - 1) / 4 + 1
    ));

    if starkinfo.map_sectionsN.get("cm2_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input s0_siblings2[{}][{}][16];
        "#,
            stark_struct.nQueries,
            (stark_struct.steps[0].nBits - 1) / 4 + 1
        ));
    }

    if starkinfo.map_sectionsN.get("cm3_2ns") > 0 {
        res.push_str(&format!(
            r#"
    signal input s0_siblings3[{}][{}][16];
        "#,
            stark_struct.nQueries,
            (stark_struct.steps[0].nBits - 1) / 4 + 1
        ));
    }

    res.push_str(&format!(
        r#"
    signal input s0_siblings4[{}][{}][16];
    signal input s0_siblingsC[{}][{}][16];
        "#,
        stark_struct.nQueries,
        (stark_struct.steps[0].nBits - 1) / 4 + 1,
        stark_struct.nQueries,
        (stark_struct.steps[0].nBits - 1) / 4 + 1
    ));

    for s in 0..(stark_struct.steps.len() - 1) {
        res.push_str(&format!(
            r#"
    signal input s{}_root;
        "#,
            s + 1
        ));
    }

    for s in 1..stark_struct.steps.len() {
        res.push_str(&format!(
            r#"
    signal input s{}_vals[{}][{}];
    signal input s{}_siblings[{}][{}][16];
        "#,
            s,
            stark_struct.nQueries,
            (1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits)) * 3,
            s,
            stark_struct.nQueries,
            (stark_struct.steps[s].nBits - 1) / 4 + 1
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

    var p = 0xFFFFFFFF00000001;
    "#,
        stark_struct.nQueries, stark_struct.steps[0].nBits
    ));

    ///////////
    // challenge calculation
    ///////////

    let mut transcript = Transcript::new(stark_struct.clone());
    transcript.put("publics", pil.publics.len() as i32);
    transcript.put("root1", -1);
    transcript.getField("challenges[0]", 3);
    transcript.getField("challenges[1]", 3);
    transcript.put("root2", -1);
    transcript.getField("challenges[2]", 3);
    transcript.getField("challenges[3]", 3);
    transcript.put("root3", -1);
    transcript.getField("challenges[4]", 3);
    transcript.put("root4", -1);
    transcript.getField("challenges[7]", 3);
    for i in 0..starkinfo.ev_map.len() {
        transcript.put(&format!("evals[{}]", i), 3);
    }
    transcript.getField("challenges[5]", 3);
    transcript.getField("challenges[6]", 3);
    for si in 0..stark_struct.steps.len() {
        transcript.getField(&format!("s{}_specialX", si), 3);
        if si < stark_struct.steps.len() - 1 {
            transcript.put(&format!("s{}_root", si + 1), -1);
        } else {
            for j in 0..(1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits) {
                transcript.put(&format!("finalPol[{}]", j), 3);
            }
        }
    }
    transcript.getPermutations("ys", stark_struct.nQueries, stark_struct.steps[0].nBits);
    res.push_str(&transcript.getCode());

    ///////////
    // Constrain polynomial check in vauations
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
            for (var j=0; j<16; j++) {{
                s0_merkle1[q].siblings[i][j] <== s0_siblings1[q][i][j];
    "#,
        starkinfo.map_sectionsN.get("cm4_2ns"),
        starkinfo.n_constants,
        starkinfo.ev_map.len(),
        (stark_struct.steps[0].nBits - 1) / 4 + 1
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
        }}"#,
            1 << stark_struct.steps[0].nBits,
            stark_struct.steps[0].nBits
        ));
    }

    res.push_str(
        r#"
    }"#,
    );

    for s in 1..stark_struct.steps.len() {
        res.push_str(&format!(
            r#"
    component s{}_merkle[{}];
    component s{}_fft[{}];
    component s{}_evalPol[{}];
    component s{}_lowValues[{}];
    component s{}_cNorm[{}];
    component s{}_sx[{}][{}];
    component s{}_evalXprime[{}];
    signal s{}_X[{}];
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
            s,
            stark_struct.nQueries,
            stark_struct.steps[s].nBits - 1,
            s,
            stark_struct.nQueries,
            s,
            stark_struct.nQueries
        ));

        let nbits =
            if s < stark_struct.steps.len() - 1 { stark_struct.steps[s + 1].nBits } else { 0 };
        let selector = stark_struct.steps[s].nBits - nbits;

        res.push_str(&format!(
            r#"
    for (var q=0; q<{}; q++) {{
        s{}_merkle[q] = MerkleHash(3, {}, {});
        s{}_fft[q] = FFT({}, 1);
        s{}_evalPol[q] = EvalPol({});
        s{}_lowValues[q] = TreeSelector({}, 3) ;
        for (var i=0; i< {}; i++) {{
            for (var e=0; e<3; e++) {{
                s{}_merkle[q].values[i][e] <== s{}_vals[q][i*3+e];
                s{}_fft[q].in[i][e] <== s{}_vals[q][i*3+e];
            }}
        }}"#,
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
            for (var j=0; j<16; j++) {{
                s{}_merkle[q].siblings[i][j] <== s{}_siblings[q][i][j];
            }}
        }}
        for (var i=0; i<{}; i++) {{
            s{}_merkle[q].key[i] <== ys[q][i];
        }}
        "#,
            (stark_struct.steps[s].nBits - 1) / 4 + 1,
            s,
            s,
            stark_struct.steps[s].nBits,
            s
        ));

        let e1 = (F3G::from(*SHIFT)
            .exp(1 << (stark_struct.nBitsExt - stark_struct.steps[s - 1].nBits))
            * F3G::from(MG.0[stark_struct.steps[s - 1].nBits]))
        .inv();
        let e0 = (F3G::from(*SHIFT)
            .exp(1 << (stark_struct.nBitsExt - stark_struct.steps[s - 1].nBits)))
        .inv();

        res.push_str(&format!(
            r#"
        for (var i=1; i<{}; i++ ) {{
            s{}_sx[q][i-1] = GLMul();
            if (i==1) {{
                s{}_sx[q][i-1].ina <== ys[q][0] * ({} - {}) + {};
            }} else {{
                s{}_sx[q][i-1].ina <== s{}_sx[q][i-2].out;
            }}
            s{}_sx[q][i-1].inb <== ys[q][i] * (_inv1(roots({} -i)) -1) +1;
        }}"#,
            stark_struct.steps[s].nBits,
            s,
            s,
            e1.as_int(),
            e0.as_int(),
            e0.as_int(),
            s,
            s,
            s,
            stark_struct.steps[s - 1].nBits
        ));

        if stark_struct.steps[0].nBits > 1 {
            res.push_str(&format!(
                r#"
        s{}_X[q] <== s{}_sx[q][{}].out;
        "#,
                s,
                s,
                stark_struct.steps[s].nBits - 2
            ))
        } else {
            res.push_str(&format!(
                r#"
        s{}_X[q] <== {} *  ( ys[q][0] * {} +1);
        "#,
                s,
                (F3G::from(*SHIFT)
                    .exp(1 << (stark_struct.nBitsExt - stark_struct.steps[s - 1].nBits)))
                .inv(),
                F3G::from(MG.0[stark_struct.steps[s - 1].nBits]) - F3G::ONE
            ));
        }

        /*
            s{}_sx[q][0] <==  {} *  ( ys[q][0] * {}+1);
            for (var i=1; i<{}; i++) {{
                s{}_sx[q][i] <== s{}_sx[q][i-1] *  ( ys[q][i] * ((1/roots({} -i)) -1) +1);
            }}
        */
        res.push_str(&format!(
            r#"
        for (var i=0; i< {}; i++) {{
            for (var e=0; e<3; e++) {{
                s{}_evalPol[q].pol[i][e] <== s{}_fft[q].out[i][e];
            }}
        }}
        s{}_evalXprime[q] = GLCMul();
        s{}_evalXprime[q].ina[0] <== s{}_specialX[0];
        s{}_evalXprime[q].ina[1] <== s{}_specialX[1];
        s{}_evalXprime[q].ina[2] <== s{}_specialX[2];
        s{}_evalXprime[q].inb[0] <== s{}_X[q];
        s{}_evalXprime[q].inb[1] <== 0;
        s{}_evalXprime[q].inb[2] <== 0;
        for (var e=0; e<3; e++) {{
            s{}_evalPol[q].x[e] <== s{}_evalXprime[q].out[e];
        }}
        "#,
            1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits),
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s,
            s
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
        }}"#,
                1 << stark_struct.steps[s].nBits,
                s,
                stark_struct.steps[s].nBits,
                s
            ));
        }

        res.push_str(&format!(
            r#"
        s{}_cNorm[q] = GLCNorm();
        for (var e=0; e<3; e++) {{
            s{}_cNorm[q].in[e] <== s{}_evalPol[q].out[e] - s{}_lowValues[q].out[e] + p;
        }}
    }}"#,
            s, s, s, s
        ));
    }
    // Checks
    res.push_str(&format!(
        r#"
    for (var q=0; q < {}; q ++) {{
        enable * (s0_merkle1[q].root - root1) === 0;"#,
        stark_struct.nQueries
    ));

    if starkinfo.map_sectionsN.cm2_2ns > 0 {
        res.push_str(
            r#"
        enable * (s0_merkle2[q].root - root2) === 0;"#,
        );
    }

    if starkinfo.map_sectionsN.cm3_2ns > 0 {
        res.push_str(
            r#"
        enable * (s0_merkle3[q].root - root3) === 0;"#,
        );
    }

    res.push_str(
        r#"
        enable * (s0_merkle4[q].root - root4) === 0;
        enable * (s0_merkleC[q].root - rootC) === 0;
        for (var e=0; e<3; e++) {
            enable * (s0_lowValues[q].out[e] - verifyQueries[q].out[e]) === 0;
        }
    }"#,
    );

    for s in 1..stark_struct.steps.len() {
        res.push_str(&format!(
            r#"
    for (var q = 0; q < {}; q ++) {{
        for (var e=0; e<3; e++) {{
            enable * s{}_cNorm[q].out[e] === 0;
        }}
        enable * (s{}_merkle[q].root - s{}_root) === 0;
    }}"#,
            stark_struct.nQueries, s, s, s
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
    component lastIFFT = FFT({}, 1);

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

    if !options.skip_main {
        res.push_str(&format!(
            r#"
template Main() {{
    signal input proverAddr;
    signal output publicsHash;

    signal input publics[{}];
    {}
    signal input root1;
    signal input root2;
    signal input root3;
    signal input root4;
    signal input evals[{}][3];

    signal input s0_vals1[{}][{}];
"#,
            pil.publics.len(),
            if options.verkey_input { "signal input rootC; " } else { "" },
            starkinfo.ev_map.len(),
            stark_struct.nQueries,
            starkinfo.map_sectionsN.cm1_2ns
        ));

        if starkinfo.map_sectionsN.cm2_2ns > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_vals2[{}][{}];
"#,
                stark_struct.nQueries, starkinfo.map_sectionsN.cm2_2ns
            ));
        }
        if starkinfo.map_sectionsN.cm3_2ns > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_vals3[{}][{}];
"#,
                stark_struct.nQueries, starkinfo.map_sectionsN.cm3_2ns
            ));
        }

        res.push_str(&format!(
            r#"
    signal input s0_vals4[{}][{}];
    signal input s0_valsC[{}][{}];
    signal input s0_siblings1[{}][{}][16];
"#,
            stark_struct.nQueries,
            starkinfo.map_sectionsN.cm4_2ns,
            stark_struct.nQueries,
            starkinfo.n_constants,
            stark_struct.nQueries,
            (stark_struct.steps[0].nBits - 1) / 4 + 1,
        ));
        if starkinfo.map_sectionsN.cm2_2ns > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_siblings2[{}][{}][16];
"#,
                stark_struct.nQueries,
                (stark_struct.steps[0].nBits - 1) / 4 + 1
            ));
        }
        if starkinfo.map_sectionsN.cm3_2ns > 0 {
            res.push_str(&format!(
                r#"
    signal input s0_siblings3[{}][{}][16];
"#,
                stark_struct.nQueries,
                (stark_struct.steps[0].nBits - 1) / 4 + 1
            ));
        }
        res.push_str(&format!(
            r#"
    signal input s0_siblings4[{}][{}][16];
    signal input s0_siblingsC[{}][{}][16];
"#,
            stark_struct.nQueries,
            (stark_struct.steps[0].nBits - 1) / 4 + 1,
            stark_struct.nQueries,
            (stark_struct.steps[0].nBits - 1) / 4 + 1
        ));

        for s in 0..(stark_struct.steps.len() - 1) {
            res.push_str(&format!(
                r#"
    signal input s{}_root;
    "#,
                s + 1
            ));
        }

        for s in 1..stark_struct.steps.len() {
            res.push_str(&format!(
                r#"
    signal input s{}_vals[{}][{}];
    signal input s{}_siblings[{}][{}][16];
"#,
                s,
                stark_struct.nQueries,
                (1 << (stark_struct.steps[s - 1].nBits - stark_struct.steps[s].nBits)) * 3,
                s,
                stark_struct.nQueries,
                (stark_struct.steps[s].nBits - 1) / 4 + 1
            ));
        }

        res.push_str(&format!(
            r#"
    signal input finalPol[{}][3];

    component sv = StarkVerifier();

    sv.publics <== publics;
    {}
    sv.root1 <== root1;
    sv.root2 <== root2;
    sv.root3 <== root3;
    sv.root4 <== root4;
    sv.evals <== evals;

    sv.s0_vals1 <== s0_vals1;
"#,
            (1 << stark_struct.steps[stark_struct.steps.len() - 1].nBits),
            if options.verkey_input { "sv.rootC <== rootC; " } else { "" }
        ));

        if starkinfo.map_sectionsN.cm2_2ns > 0 {
            res.push_str(
                r#"
    sv.s0_vals2 <== s0_vals2;
    "#,
            );
        }
        if starkinfo.map_sectionsN.cm3_2ns > 0 {
            res.push_str(
                r#"
    sv.s0_vals3 <== s0_vals3;
    "#,
            );
        }
        res.push_str(
            r#"
    sv.s0_vals4 <== s0_vals4;
    sv.s0_valsC <== s0_valsC;
    sv.s0_siblings1 <== s0_siblings1;
    "#,
        );
        if starkinfo.map_sectionsN.cm2_2ns > 0 {
            res.push_str(
                r#"
    sv.s0_siblings2 <== s0_siblings2;
    "#,
            );
        }
        if starkinfo.map_sectionsN.cm3_2ns > 0 {
            res.push_str(
                r#"
    sv.s0_siblings3 <== s0_siblings3;
    "#,
            );
        }
        res.push_str(
            r#"
    sv.s0_siblings4 <== s0_siblings4;
    sv.s0_siblingsC <== s0_siblingsC;
    "#,
        );

        for s in 0..(stark_struct.steps.len() - 1) {
            res.push_str(&format!(
                r#"
    sv.s{}_root <== s{}_root;
    "#,
                s + 1,
                s + 1
            ));
        }

        for s in 1..stark_struct.steps.len() {
            res.push_str(&format!(
                r#"
    sv.s{}_vals <== s{}_vals;
    sv.s{}_siblings <== s{}_siblings;
    "#,
                s, s, s, s
            ));
        }
        res.push_str(
            r#"
    sv.finalPol <== finalPol;
    "#,
        );

        //////
        // Calculate Publics Hash
        //////

        res.push_str(&format!(
            r#"
    component publicsHasher = Sha256({});
    component n2bProverAddr = Num2Bits(160);
    component n2bPublics[{}];
    component cmpPublics[{}];

    n2bProverAddr.in <== proverAddr;
    for (var i=0; i<160; i++) {{
        publicsHasher.in[160 - 1 -i] <== n2bProverAddr.out[i];
    }}

    var offset = 160;
    for (var i=0; i<{}; i++) {{
        n2bPublics[i] = Num2Bits(64);
        cmpPublics[i] = CompConstant64(0xFFFFFFFF00000000);
        n2bPublics[i].in <== publics[i];
        for (var j=0; j<64; j++) {{
            publicsHasher.in[offset + 64 - 1 -j] <== n2bPublics[i].out[j];
            cmpPublics[i].in[j] <== n2bPublics[i].out[j];
        }}
        cmpPublics[i].out === 0;
        offset += 64;
    }}

    component n2bPublicsHash = Bits2Num(256);
    for (var i = 0; i < 256; i++) {{
        n2bPublicsHash.in[i] <== publicsHasher.out[255-i];
    }}

    publicsHash <== n2bPublicsHash.out;
}}

component main {} = Main();
"#,
            160 + 64 * pil.publics.len(),
            pil.publics.len(),
            pil.publics.len(),
            pil.publics.len(),
            if options.verkey_input { "{public [rootC]}" } else { "" }
        ));
    }
    res
}

// Suport bn128 && bls12381
pub fn render<F: ff::PrimeField + Default>(
    starkinfo: &StarkInfo,
    prorgam: &Program,
    pil: &PIL,
    stark_struct: &StarkStruct,
    const_root: &ElementDigest<4, F>,
    options: &StarkOption,
) -> String {
    let mut res = header();
    res.push_str(&verify_evaluations(starkinfo, prorgam, pil, stark_struct));
    res.push_str(&verify_query(starkinfo, prorgam, stark_struct));
    res.push_str(&map_values(starkinfo));
    res.push_str(&stark_verifier(starkinfo, pil, stark_struct, const_root, options));
    res
}
