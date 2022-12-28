use crate::digest::ElementDigest;
use crate::errors::Result;
use crate::f3g::F3G;
use num_bigint::BigUint;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;
use winter_math::StarkField;
use crate::starkinfo_codegen::Node;
use crate::starkinfo::StarkInfo;
use crate::constant::SHIFT;
use crate::pil2circom::StarkOption;

fn header() -> String {

    let header = r#"
pragma circom 2.1.0;
pragma custom_templates;

include "cmul.circom";
include "cinv.circom";
include "poseidon.circom";
include "bitify.circom";
include "fft.circom";
include "merklehash.circom";
include "evalpol.circom";
include "treeselector.circom";
"#;

    header
}


#[derive(Default)]
struct Transcript {
    state: [String; 4],
    pending: Vec<String>,
    out: Vec<String>,
    st_cnt: usize,
    h_cnt: usize,
    n2b_cnt: usize,
    code: Vec<String>,
}

impl Transcript {
    pub fn new() {
        Self {
            state: [String::from("0"); 4],
            pending: vec![],
            out: vec![],
            st_cnt: 0,
            h_cnt: 0,
            n2b_cnt: 0,
            code: vec![],
        }
    }

    fn getField(&mut self, v: &str) {
        self.code.push(format!("{}[0] <== {}", v, self.getFields1()));
        self.code.push(format!("{}[1] <== {}", v, self.getFields1()));
        self.code.push(format!("{}[2] <== {}", v, self.getFields1()));
    }

    fn getFields1(&mut self) -> String {
        if (self.out.length == 0) {
            while (self.pending.length<8) {
                self.pending.push(String::from("0"));
            }
            self.code.push(format!("component tcHahs_{} = Poseidon(12);", self.h_cnt));
            self.h_cnt += 1;

            for i in 0..8 {
                self.code.push(format!("tcHahs_{}.in[{}] <== {};", self.h_cnt-1, i, self.pending[i]));
            }
            for i in 0..12 {
                self.out[i] = format!("tcHahs_{}.out[{}]", self.h_cnt-1, i);
            }
            for i in 0..4 {
                self.code.push(format!("tcHahs_{}.capacity[{}] <== {};", self.h_cnt-1, i, self.state[i]));
                self.state[i] = format!("tcHahs_{}.out[{}]", self.h_cnt-1, i);
            }
            self.pending = vec![];
        }
        let res = self.out[0];
        self.out.remove(0);
        res
    }

    pub fn put(&mut self, a: String, l: usize) {
        if l > 0 {
            for i in 0..l {
                self._add1(format!("{}[{}]", a, i));
            }
        } else {
            self._add1(a);
        }
    }

    pub fn _add1(&mut self, a: String) {
        self.out = vec![];
        self.pending.push(a);
        if self.pending.len() == 8 {
            self.code.push(format!("component tcHahs_{} = Poseidon(12);", self.h_cnt));
            self.h_cnt += 1;
            for i in 0..8 {
                self.code.push(format!("tcHahs_{}.in[{}] <== {};", self.h_cnt - 1, i, self.pending[i]));
            }
            for i in 0..12 {
                self.out[i] = format!("tcHahs_{}.out[{}]", self.h_cnt-1, i);
            }
            for i in 0..4 {
                self.code.push(format!("tcHahs_{}.capacity[{}] <== {};", self.h_cnt-1, i, self.state[i]));
                self.state[i] = format!("tcHahs_{}.out[{}]", self.h_cnt-1, i);
            }
            self.pending = vec![];
        }
    }

    pub fn getPermutations(&mut self, v: String, n: usize, nBits: usize) {
        let totalBits = n*nBits;
        let NFields = (totalBits - 1)/63+1;
        let mut n2b: <String> = vec![];
        for i in 0..NFields {
            let f = self.getFields1();
            n2b[i] = format!("tcN2b_{}", self.n2b_cnt);
            self.n2b_cnt += 1;
            self.code.push(format!("component {} = Num2Bits_strict();", self.n2b[i]));
            self.code.push(format!("{}.in <== {};", self.n2b[i], f));
        }
        let mut curField =0;
        let mut curBit =0;
        for i in 0..n {
            let a = 0;
            for j in 0..nBits {
                self.code.push(format!("{}[{}][{}] <== {}.out[{}];", v, i, j, self.n2b[curField], curBit));
                curBit += 1;
                if (curBit == 63) {
                    curBit = 0;
                    curField += 1;
                }
            }
        }
    }

    pub fn getCode() -> String {
        for c in self.code.iter_mut() {
            *c = "    " + *c;
        }
        self.code.iter().join("\n");
    }
}

fn unrollCode(code: &Vec<Section>, starkinfo: &StarkInfo) -> String {
    let ref_ = |r: Node| -> String {
        match r.type_.as_str() {
            "eval" => format!("evals[{}]", r.id),
            "challenge" => format!("challenges[{}]", r.id),
            "public" => format!("publics[{}]", r.id),
            "x" => format!("challenges[7]"),
            "Z" => format!("Z"),
            "xDivXSubXi" => format!("xDivXSubXi"),
            "xDivXSubWXi" => format!("xDivXSubWXi"),
            "tmp" => format!("tmp_{}", r.id),
            "tree1" => format!("mapValues.tree1_{}", r.id),
            "tree2" => format!("mapValues.tree2_{}", r.id - starkinfo.n_cm1),
            "tree3" => format!("mapValues.tree3_{}", r.id - starkinfo.n_cm1 - starkinfo.n_cm2),
            "tree4" => format!("mapValues.tree4_{}", r.id - starkinfo.n_cm1 - starkinfo.n_cm2, starkinfo.n_cm3),
            "const" => format!("consts[{}]", r.id),
            "number" => format!("{}", r.value.unwrap()),
            _ => panic!("Invalid ref: {}", r.type_),
        }
    };
    let mut tmpNameId = 0;
    let mut str_code = String::from("");
    for i in 0..code.len() {
      let inst = &code[i];
      if (inst.dest.type_.as_str() == "tmp") {
          if (inst.dest.dim == 1) {
              str_code.push_str(format!("\t\tsignal tmp_{};\n", inst.dest.id));
          } else if (inst.dest.dim == 3)  {
              str_code.push_str(format!("\t\tsignal tmp_{}[3];\n", inst.dest.id));
          } else {
              panic!("Invalid dimension");
          }
      }

      match inst.op.as_str() {
          "add" => {
              if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                  str_code.push_str(format!("\t{} <== {} + {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
              } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                  str_code.push_str(format!("\t{}[0] <== {} + {}[0]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {}[1]\n", ref_(inst.dest), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[2] <== {}[2]\n", ref_(inst.dest), ref_(inst.src[1])));
              } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                  str_code.push_str(format!("\t{}[0] <== {}[0] + {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {}[1]\n", ref_(inst.dest), ref_(inst.src[0])));
                  str_code.push_str(format!("\t{}[2] <== {}[2]\n", ref_(inst.dest), ref_(inst.src[0])));
              } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                  str_code.push_str(format!("\t{}[0] <== {}[0] + {}[0]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {}[1] + {}[1]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[2] <== {}[2] + {}[2]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
              } else {
                  panic!("Invalid src dimensions");
              }
          },
          "sub" => {
              if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                  str_code.push_str(format!("\t{} <== {} - {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
              } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                  str_code.push_str(format!("\t{}[0] <== {} - {}[0]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {}[1]\n", ref_(inst.dest), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[2] <== {}[2]\n", ref_(inst.dest), ref_(inst.src[1])));
              } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                  str_code.push_str(format!("\t{}[0] <== {}[0] - {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {}[1]\n", ref_(inst.dest), ref_(inst.src[0])));
                  str_code.push_str(format!("\t{}[2] <== {}[2]\n", ref_(inst.dest), ref_(inst.src[0])));
              } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {
                  str_code.push_str(format!("\t{}[0] <== {}[0] - {}[0]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {}[1] - {}[1]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[2] <== {}[2] - {}[2]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
              } else {
                  panic!("Invalid src dimensions");
              }
          },
          "mul" => {
              if inst.src[0].dim == 1 && inst.src[1].dim == 1 {
                  str_code.push_str(format!("\t{} <== {} * {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
              } else if inst.src[0].dim == 1 && inst.src[1].dim == 3 {
                  str_code.push_str(format!("\t{}[0] <== {} * {}[0]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {} * {}[1]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[2] <== {} * {}[2]\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
              } else if inst.src[0].dim == 3 && inst.src[1].dim == 1 {
                  str_code.push_str(format!("\t{}[0] <== {}[0] * {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[1] <== {}[1] * {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
                  str_code.push_str(format!("\t{}[2] <== {}[2] * {}\n", ref_(inst.dest), ref_(inst.src[0]), ref_(inst.src[1])));
              } else if inst.src[0].dim == 3 && inst.src[1].dim == 3 {

                let cmpName = format!("cmul_{}", tmpNameId);
                tmpNameId += 1;
                str_code.push_str(format!("component {} = CMul();\n", cmpName));
                str_code.push_str(format!("{}.ina[0] <== {}[0];\n", cmpName, ref_(inst.src[0])));
                str_code.push_str(format!("{}.ina[1] <== {}[1];\n", cmpName, ref_(inst.src[0])));
                str_code.push_str(format!("{}.ina[2] <== {}[2];\n", cmpName, ref_(inst.src[0])));
                str_code.push_str(format!("{}.inb[0] <== {}[0];\n", cmpName, ref_(inst.src[1])));
                str_code.push_str(format!("{}.inb[1] <== {}[1];\n", cmpName, ref_(inst.src[1])));
                str_code.push_str(format!("{}.inb[2] <== {}[2];\n", cmpName, ref_(inst.src[1])));
                str_code.push_str(format!("{}[0] <== {}.out[0];\n", ref_(inst.dest), cmpName));
                str_code.push_str(format!("{}[1] <== {}.out[1];\n", ref_(inst.dest), cmpName));
                str_code.push_str(format!("{}[2] <== {}.out[2];\n", ref_(inst.dest), cmpName));
              } else {
                  panic!("Invalid src dimensions");
              }
          },
          "copy" => {
              if inst.src[0].dim == 1 {
                  str_code.push_str(format!("\t{} <== {}\n", ref_(inst.dest), ref_(inst.src[0])));
              } else if inst.src[0].dim == 3 {
                  str_code.push_str(format!("\t{}[0] <== {}[0]\n", ref_(inst.dest), ref_(inst.src[0])));
                  str_code.push_str(format!("\t{}[1] <== {}[1]\n", ref_(inst.dest), ref_(inst.src[0])));
                  str_code.push_str(format!("\t{}[2] <== {}[2]\n", ref_(inst.dest), ref_(inst.src[0])));
              } else {
                  panic!("Invalid src dimensions");
              }
          },
          _ => panic!("Invalid op"),
      }
    }
    ref_(code[code.length-1].dest)
}

fn verify_evaluations(starkinfo: &StarkInfo, prorgam: &Program, pil: &PIL, stark_struct: &StarkStruct) -> String {

    let mut res = format!(
    r#"
template VerifyEvaluations() {
    signal input challenges[8][3];
    signal input evals[{}][3];
    signal input publics[{}];
    signal input enable;
"#, starkinfo.ev_map.len(), pil.publics.len());

    res.push_str(
        format!("\tcomponent zMul[<%- starkStruct.nBits %>];", stark_struct.nBits)
    );

    res.push_str(
        format!(r#"
    for (var i=0; i< {}; i++) {
        zMul[i] = CMul();
        if (i==0) {
            zMul[i].ina[0] <== challenges[7][0];
            zMul[i].ina[1] <== challenges[7][1];
            zMul[i].ina[2] <== challenges[7][2];
            zMul[i].inb[0] <== challenges[7][0];
            zMul[i].inb[1] <== challenges[7][1];
            zMul[i].inb[2] <== challenges[7][2];
        } else {
            zMul[i].ina[0] <== zMul[i-1].out[0];
            zMul[i].ina[1] <== zMul[i-1].out[1];
            zMul[i].ina[2] <== zMul[i-1].out[2];
            zMul[i].inb[0] <== zMul[i-1].out[0];
            zMul[i].inb[1] <== zMul[i-1].out[1];
            zMul[i].inb[2] <== zMul[i-1].out[2];
        }
    }
        "#, stark_struct.nBits)
            );

    res.push_str(
        format!(r#"
    signal Z[3];

    Z[0] <== zMul[{}].out[0] -1;
    Z[1] <== zMul[{}].out[1];
    Z[2] <== zMul[{}].out[2];

    signal xN[3] <== zMul[{}].out;
            "#, stark_struct.nBits - 1, stark_struct.nBits - 1, stark_struct.nBits - 1, stark_struct.nBits - 1)
        );

    let evalP = unrollCode(program.verifier_code.first);

    res.push_str(
        format!(r#"
    signal xAcc[{}][3];
    signal qStep[{}][3];
    signal qAcc[{}][3];
    for (var i=0; i< {}; i++) {
        if (i==0) {
            xAcc[0] <== [1, 0, 0];
            qAcc[0] <== evals[{}+i];
        } else {
            xAcc[i] <== CMul()(xAcc[i-1], xN);
            qStep[i-1] <== CMul()(xAcc[i], evals[{}+i]);

            qAcc[i][0] <== qAcc[i-1][0] + qStep[i-1][0];
            qAcc[i][1] <== qAcc[i-1][1] + qStep[i-1][1];
            qAcc[i][2] <== qAcc[i-1][2] + qStep[i-1][2];
        }
    }"#, starkinfo.q_deg, starkinfo.q_deg - 1, starkinfo.q_deg, starkinfo.q_deg, starkinfo.ev_idx.cm[0][starkinfo.qs[0]], starkinfo.ev_idx.cm[0][starkinfo.qs[0]])
    );

    res.push_str(
        format!(r#"
    signal qZ[3] <== CMul()(qAcc[{}], Z);

// Final Verification
    enable * ({}[0] - qZ[0]) === 0;
    enable * ({}[1] - qZ[1]) === 0;
    enable * ({}[2] - qZ[2]) === 0;
}
        "#, starkinfo.q_deg-1,  evalP, evalP, evalP)
    );
    res
}

fn verify_query(starkinfo: &StarkInfo, prorgam: &Program, pil: &PIL, stark_struct: &StarkStruct) -> String {

    let mut res = format!(
    r#"
template VerifyQuery() {
    signal input ys[{}];
    signal input challenges[8][3];
    signal input evals[{}][3];
    signal input tree1[{}];
    "#,
    stark_struct.steps[0].nBits,
    starkinfo.ev_map.len(),
    starkinfo.map_sectionsN.cm1_2ns,
);

    if stark_struct.map_sectionsN.cm2_2ns > 0 {
        res.push_str(
            format!(r#"
    signal input tree2[{}];
            "#, starkinfo.map_sectionsN.cm2_2ns)
        );
    }

    if stark_struct.map_sectionsN.cm3_2ns > 0 {
        res.push_str(
            format!(r#"
    signal input tree3[{}];
            "#, starkinfo.map_sectionsN.cm3_2ns)
        );
    }

    res.push_str(
        format!(r#"
    signal input tree4[{}];
    signal input consts[{}];
    signal output out[3];
        "#, starkinfo.map_sectionsN.cm4_2ns, starkinfo.n_constants)
    );

///////////
// Mapping
///////////

    res.push_str(format!(r#"
    component mapValues = MapValues();

    for (var i=0; i< {}; i++ ) {
        mapValues.vals1[i] <== tree1[i];
    }
    "#, starkinfo.map_sectionsN.cm1_2ns));

    if starkinfo.map_sectionsN.cm2_2ns > 0 {
        res.push_str(format!(r#"
    for (var i=0; i< {}; i++ ) {
        mapValues.vals2[i] <== tree2[i];
    }"#, starkinfo.map_sectionsN.cm2_2ns));
    }

    if starkinfo.map_sectionsN.cm3_2ns > 0 {
        res.push_str(format!(r#"
    for (var i=0; i< {}; i++ ) {
        mapValues.vals3[i] <== tree3[i];
    }"#, starkinfo.map_sectionsN.cm3_2ns));
    }


    if starkinfo.map_sectionsN.cm4_2ns > 0 {
        res.push_str(format!(r#"
    for (var i=0; i< {}; i++ ) {
        mapValues.vals4[i] <== tree4[i];
    }"#, starkinfo.map_sectionsN.cm4_2ns));
    }

    res.push_str(format!(r#"
    signal xacc[{}];
    xacc[0] <== ys[0]*({} * roots({})-{}) + {};
    for (var i=1; i<{}; i++ ) {
        xacc[i] <== xacc[i-1] * ( ys[i]*(roots({} - i) - 1) +1);
    }
    ));"#, stark_struct.steps[0].nBits, SHIFT.as_int(), stark_struct.steps[0].nBits, SHIFT.as_int(), SHIFT.as_int(), stark_struct.steps[0].nBits, stark_struct.steps[0].nBits));


    res.push_str(format!(r#"
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

    res.push_str(format!(r#"
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

    let evalQ = unrollCode(&program.verifier_query_code.first);

    res.push_str(format!(r#"
    out[0] <== {}[0];
    out[1] <== {}[1];
    out[2] <== {}[2];
}
    "#, evalQ, evalQ, evalQ));
}


fn map_values(&self, starkinfo: &StarkInfo) {
    let mut res = format!(
        r#"
template MapValues() {
    signal input vals1[{}];
"#, starkinfo.map_sectionsN.cm1_2ns);

    if starkinfo.map_sectionsN.cm2_2ns > 0 {
        res.push_str(format!(r#"
    signal input vals2[{}];
"#, starkinfo.map_sectionsN.cm2_2ns));
    }

    if starkinfo.map_sectionsN.cm3_2ns > 0 {
        res.push_str(format!(r#"
    signal input vals3[{}];
"#, starkinfo.map_sectionsN.cm3_2ns));
    }

    res.push_str(format!(r#"
    signal input vals4[{}];
"#, starkinfo.map_sectionsN.cm4_2ns));


    let sNames = vec!["", "cm1_2ns", "cm2_2ns", "cm3_2ns", "cm4_2ns"];
    for t in 1..=4 {
        for ms in starkinfo.map_sections.get(sNames[i]).iter() {
            let p = starkinfo.var_pol_map[*ms];
            if p.dim == 1 {
                res.push_str(format!(r#"
    signal output tree{}_{};
                "#, t, i));
            } else if p.dim == 3 {
                res.push_str(format!(r#"
    signal output tree{}_{}[3];
                "#, t, i));
            } else {
                panic!("Invalid dim");
            }
        }
    }

    for t in 1..=4 {
        for ms in starkinfo.map_sections.get(sNames[i]).iter() {
            let p = starkinfo.var_pol_map[*ms];
            if p.dim == 1 {
                res.push_str(format!(r#"
    tree<{}_{} <== vals{}[{}];
                "#, t, i, t, p.section_pos));
            } else if p.dim == 3 {
                res.push_str(format!(r#"
    tree{}_{}[0] <== vals{}[{}];
    tree{}_{}[1] <== vals{}[{}];
    tree{}_{}[2] <== vals{}[{}];
}"#,
                t, i, t, p.section_pos,
                t, i, t, p.section_pos + 1,
                t, i, t, p.section_pos + 2,
                ));
            } else {
                panic!("Invalid dim");
            }
        }
    }
}

fn stark_verifier(starkinfo: &StarkInfo, prorgam: &Program, pil: &PIL, stark_struct: &StarkStruct, const_root: &ElementDigest, options: &StarkOption) -> String {
    let mut res = format!(r#"
template StarkVerifier() {
    signal input publics[{}];
    signal input root1[4];
    signal input root2[4];
    signal input root3[4];
    signal input root4[4];
"#, pil.publics.len());

    if options.verkey_input {
        res.push_str(format!(r#"
    signal input rootC[4];
"#));
    } else {
        let const_roots = const_root.as_elements();
        res.push_str(format!(r#"
    signal rootC[4];
    rootC[0] <== {};
    rootC[1] <== {};
    rootC[2] <== {};
    rootC[3] <== {};
"#,
   const_root[0].as_int(),
   const_root[1].as_int(),
   const_root[2].as_int(),
   const_root[3].as_int()));
    }

    res.push_str(format!(r#"
    signal input evals[{}][3];
    signal input s0_vals1[{}][{}];
    "#, starkinfo.ev_map.len(), stark_struct.n_queries, starkinfo.map_sectionsN.cm1_2ns));

    if starkinfo.map_sectionsN.cm2_2ns > 0 {
        res.push_str(format!());
    }

<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { -%>
    signal input s0_vals2[<%- starkStruct.nQueries %>][<%- starkInfo.mapSectionsN.cm2_2ns %>];
<% }                                         -%>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { -%>
    signal input s0_vals3[<%- starkStruct.nQueries %>][<%- starkInfo.mapSectionsN.cm3_2ns %>];
<% }                                         -%>
    signal input s0_vals4[<%- starkStruct.nQueries %>][<%- starkInfo.mapSectionsN.cm4_2ns %>];
    signal input s0_valsC[<%- starkStruct.nQueries %>][<%- starkInfo.nConstants %>];
    signal input s0_siblings1[<%- starkStruct.nQueries %>][<%- starkStruct.steps[0].nBits %>][4];
<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { -%>
    signal input s0_siblings2[<%- starkStruct.nQueries %>][<%- starkStruct.steps[0].nBits %>][4];
<% }                                         -%>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { -%>
    signal input s0_siblings3[<%- starkStruct.nQueries %>][<%- starkStruct.steps[0].nBits %>][4];
<% }                                         -%>
    signal input s0_siblings4[<%- starkStruct.nQueries %>][<%- starkStruct.steps[0].nBits %>][4];
    signal input s0_siblingsC[<%- starkStruct.nQueries %>][<%- starkStruct.steps[0].nBits %>][4];

<% for (let s=0; s<starkStruct.steps.length-1; s++) {   -%>
    signal input s<%- s+1 %>_root[4];
<% }                                                     -%>

<% for (let s=1; s<starkStruct.steps.length; s++) {   -%>
    signal input s<%- s %>_vals[<%- starkStruct.nQueries %>][<%- (1 << (starkStruct.steps[s-1].nBits - starkStruct.steps[s].nBits))*3 %>];
    signal input s<%- s %>_siblings[<%- starkStruct.nQueries %>][<%- starkStruct.steps[s].nBits %>][4];
<% }                                                  -%>

    signal input finalPol[<%- 1 << starkStruct.steps[starkStruct.steps.length-1].nBits %>][3];

<% if (options.enableInput) { -%>
    signal input enable;
    enable * (enable -1 ) === 0;
<% } else { -%>
    signal enable;
    enable <== 1;
<% } -%>


    signal challenges[8][3];
<% for (let s=0; s<starkStruct.steps.length; s++) {   -%>
    signal s<%- s %>_specialX[3];
<% }                                                    -%>

    signal ys[<%- starkStruct.nQueries %>][<%- starkStruct.steps[0].nBits %>];


///////////
// challenge calculation
///////////

<%
const transcript = new Transcript();
transcript.put("publics", pil.publics.length);
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
for (let i=0; i<starkInfo.evMap.length; i++) {
    transcript.put(`evals[${i}]`, 3);
}
transcript.getField("challenges[5]", 3);
transcript.getField("challenges[6]", 3);
for (let si=0; si<starkStruct.steps.length; si++) {
    transcript.getField(`s${si}_specialX`, 3);
    if (si < starkStruct.steps.length-1) {
        transcript.put(`s${si+1}_root`, 4);
    } else {
        for (let j=0; j< 1<<starkStruct.steps[starkStruct.steps.length-1].nBits; j++ ) {
            transcript.put(`finalPol[${j}]`, 3);
        }
    }
}
transcript.getPermutations("ys", starkStruct.nQueries, starkStruct.steps[0].nBits);

-%>

<%- transcript.getCode() %>

///////////
// Constrain polynomial check in vauations
///////////
    component verifyEvaluations = VerifyEvaluations();
    verifyEvaluations.enable <== enable;
    for (var i=0; i<8; i++) {
        for (var k=0; k<3; k++) {
            verifyEvaluations.challenges[i][k] <== challenges[i][k];
        }
    }
    for (var i=0; i<<%- pil.publics.length %>; i++) {
        verifyEvaluations.publics[i] <== publics[i];
    }
    for (var i=0; i<<%- starkInfo.evMap.length %>; i++) {
        for (var k=0; k<3; k++) {
            verifyEvaluations.evals[i][k] <== evals[i][k];
        }
    }

///////////
// Step0 Check and evaluate queries
///////////

    component verifyQueries[<%- starkStruct.nQueries %>];
    component s0_merkle1[<%- starkStruct.nQueries %>];
<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { %>
    component s0_merkle2[<%- starkStruct.nQueries %>];
<% }                                         %>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { %>
    component s0_merkle3[<%- starkStruct.nQueries %>];
<% }                                         %>
    component s0_merkle4[<%- starkStruct.nQueries %>];
    component s0_merkleC[<%- starkStruct.nQueries %>];
    component s0_lowValues[<%- starkStruct.nQueries %>];

    for (var q=0; q<<%- starkStruct.nQueries %>; q++) {
        verifyQueries[q] = VerifyQuery();
        s0_merkle1[q] = MerkleHash(1, <%- starkInfo.mapSectionsN.cm1_2ns %>, <%- 1 << starkStruct.steps[0].nBits %>);
<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { %>
        s0_merkle2[q] = MerkleHash(1, <%- starkInfo.mapSectionsN.cm2_2ns %>, <%- 1 << starkStruct.steps[0].nBits %>);
<% }                                         %>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { %>
        s0_merkle3[q] = MerkleHash(1, <%- starkInfo.mapSectionsN.cm3_2ns %>, <%- 1 << starkStruct.steps[0].nBits %>);
<% }                                         %>
        s0_merkle4[q] = MerkleHash(1, <%- starkInfo.mapSectionsN.cm4_2ns %>, <%- 1 << starkStruct.steps[0].nBits %>);
        s0_merkleC[q] = MerkleHash(1, <%- starkInfo.nConstants %>, <%- 1 << starkStruct.steps[0].nBits %>);
        s0_lowValues[q] = TreeSelector(<%- starkStruct.steps[0].nBits - ((0< starkStruct.steps.length-1) ? starkStruct.steps[1].nBits : 0)  %>, 3) ;

        for (var i=0; i<<%- starkStruct.steps[0].nBits %>; i++ ) {
            verifyQueries[q].ys[i] <== ys[q][i];
            s0_merkle1[q].key[i] <== ys[q][i];
<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { %>
            s0_merkle2[q].key[i] <== ys[q][i];
<% }                                         %>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { %>
            s0_merkle3[q].key[i] <== ys[q][i];
<% }                                         %>
            s0_merkle4[q].key[i] <== ys[q][i];
            s0_merkleC[q].key[i] <== ys[q][i];
        }
        for (var i=0; i<<%- starkInfo.mapSectionsN.cm1_2ns %>; i++ ) {
            verifyQueries[q].tree1[i] <== s0_vals1[q][i];
            s0_merkle1[q].values[i][0] <== s0_vals1[q][i];
        }
<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { %>
        for (var i=0; i<<%- starkInfo.mapSectionsN.cm2_2ns %>; i++ ) {
            verifyQueries[q].tree2[i] <== s0_vals2[q][i];
            s0_merkle2[q].values[i][0] <== s0_vals2[q][i];
        }
<% }                                         %>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { %>
        for (var i=0; i<<%- starkInfo.mapSectionsN.cm3_2ns %>; i++ ) {
            verifyQueries[q].tree3[i] <== s0_vals3[q][i];
            s0_merkle3[q].values[i][0] <== s0_vals3[q][i];
        }
<% }                                         %>
        for (var i=0; i<<%- starkInfo.mapSectionsN.cm4_2ns %>; i++ ) {
            verifyQueries[q].tree4[i] <== s0_vals4[q][i];
            s0_merkle4[q].values[i][0] <== s0_vals4[q][i];
        }
        for (var i=0; i<<%- starkInfo.nConstants %>; i++ ) {
            verifyQueries[q].consts[i] <== s0_valsC[q][i];
            s0_merkleC[q].values[i][0] <== s0_valsC[q][i];
        }
        for (var i=0; i<8; i++) {
            for (var e=0; e<3; e++) {
                verifyQueries[q].challenges[i][e] <== challenges[i][e];
            }
        }
        for (var i=0; i<<%- starkInfo.evMap.length %>; i++) {
            for (var e=0; e<3; e++) {
                verifyQueries[q].evals[i][e] <== evals[i][e];
            }
        }
        for (var i=0; i<<%- starkStruct.steps[0].nBits %>;i++) {
            for (var j=0; j<4; j++) {
                s0_merkle1[q].siblings[i][j] <== s0_siblings1[q][i][j];
<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { %>
                s0_merkle2[q].siblings[i][j] <== s0_siblings2[q][i][j];
<% }                                         %>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { %>
                s0_merkle3[q].siblings[i][j] <== s0_siblings3[q][i][j];
<% }                                         %>
                s0_merkle4[q].siblings[i][j] <== s0_siblings4[q][i][j];
                s0_merkleC[q].siblings[i][j] <== s0_siblingsC[q][i][j];
            }
        }

        for (var j=0; j<4; j++) {
            enable * (s0_merkle1[q].root[j] - root1[j]) === 0;
<% if (starkInfo.mapSectionsN.cm2_2ns > 0) { %>
            enable * (s0_merkle2[q].root[j] - root2[j]) === 0;
<% }                                         %>
<% if (starkInfo.mapSectionsN.cm3_2ns > 0) { %>
            enable * (s0_merkle3[q].root[j] - root3[j]) === 0;
<% }                                         %>
            enable * (s0_merkle4[q].root[j] - root4[j]) === 0;
            enable * (s0_merkleC[q].root[j] - rootC[j]) === 0;
        }

<% if (0 < starkStruct.steps.length-1) {            -%>
        for (var i=0; i<<%- 1 << (starkStruct.steps[0].nBits - starkStruct.steps[1].nBits) %>; i++) {
            for (var e=0; e<3; e++) {
                s0_lowValues[q].values[i][e] <== s1_vals[q][i*3+e];
            }
        }
        for (var i=0; i<<%- (starkStruct.steps[0].nBits - starkStruct.steps[1].nBits) %>; i++) {
            s0_lowValues[q].key[i] <== ys[q][i + <%- starkStruct.steps[1].nBits %>];
        }
<% } else { -%>
        for (var i=0; i<<%- 1 << (starkStruct.steps[0].nBits) %>; i++) {
            for (var e=0; e<3; e++) {
                s0_lowValues[q].values[i][e] <== finalPol[i][e];
            }
        }
        for (var i=0; i<<%- (starkStruct.steps[0].nBits) %>; i++) {
            s0_lowValues[q].key[i] <== ys[q][i];
        }
<% }      -%>
        for (var e=0; e<3; e++) {
            enable * (s0_lowValues[q].out[e] - verifyQueries[q].out[e]) === 0;
        }

    }

<% for (let s=1; s<starkStruct.steps.length; s++) {   -%>
    component s<%- s %>_merkle[<%- starkStruct.nQueries %>];
    component s<%- s %>_fft[<%- starkStruct.nQueries %>];
    component s<%- s %>_evalPol[<%- starkStruct.nQueries %>];
    component s<%- s %>_lowValues[<%- starkStruct.nQueries %>];
    signal s<%- s %>_sx[<%- starkStruct.nQueries %>][<%- starkStruct.steps[s].nBits %>];

    for (var q=0; q<<%- starkStruct.nQueries %>; q++) {
        s<%- s %>_merkle[q] = MerkleHash(3, <%- 1 << (starkStruct.steps[s-1].nBits - starkStruct.steps[s].nBits) %>, <%- 1 << starkStruct.steps[s].nBits %>);
        s<%- s %>_fft[q] = FFT(<%- starkStruct.steps[s-1].nBits - starkStruct.steps[s].nBits %>, 3, 1, 1);
        s<%- s %>_evalPol[q] = EvalPol(<%- 1 << starkStruct.steps[s-1].nBits - starkStruct.steps[s].nBits %>);
        s<%- s %>_lowValues[q] = TreeSelector(<%- starkStruct.steps[s].nBits - ((s< starkStruct.steps.length-1) ? starkStruct.steps[s+1].nBits : 0)  %>, 3) ;
        for (var i=0; i< <%- 1 << (starkStruct.steps[s-1].nBits - starkStruct.steps[s].nBits) %>; i++) {
            for (var e=0; e<3; e++) {
                s<%- s %>_merkle[q].values[i][e] <== s<%- s %>_vals[q][i*3+e];
                s<%- s %>_fft[q].in[i][e] <== s<%- s %>_vals[q][i*3+e];
            }
        }
        for (var i=0; i<<%- starkStruct.steps[s].nBits %>; i++) {
            for (var j=0; j<4; j++) {
                s<%- s %>_merkle[q].siblings[i][j] <== s<%- s %>_siblings[q][i][j];
            }
            s<%- s %>_merkle[q].key[i] <== ys[q][i];
        }
        s<%- s %>_sx[q][0] <==  <%- F.inv(F.exp(F.shift, 1 << (starkStruct.nBitsExt -starkStruct.steps[s-1].nBits) ) ) %> *  ( ys[q][0] * <%- F.sub(F.inv(F.w[starkStruct.steps[s-1].nBits]), 1n) %> +1);
        for (var i=1; i<<%- starkStruct.steps[s].nBits %>; i++) {
            s<%- s %>_sx[q][i] <== s<%- s %>_sx[q][i-1] *  ( ys[q][i] * ((1/roots(<%- starkStruct.steps[s-1].nBits %> -i)) -1) +1);
        }
        for (var i=0; i< <%- 1 << (starkStruct.steps[s-1].nBits - starkStruct.steps[s].nBits) %>; i++) {
            for (var e=0; e<3; e++) {
                s<%- s %>_evalPol[q].pol[i][e] <== s<%- s %>_fft[q].out[i][e];
            }
        }
        for (var e=0; e<3; e++) {
            s<%- s %>_evalPol[q].x[e] <== s<%- s %>_specialX[e] *  s<%- s %>_sx[q][<%- starkStruct.steps[s].nBits-1 %>];
        }
<% if (s < starkStruct.steps.length-1) {            -%>
        for (var i=0; i<<%- 1 << (starkStruct.steps[s].nBits - starkStruct.steps[s+1].nBits) %>; i++) {
            for (var e=0; e<3; e++) {
                s<%- s %>_lowValues[q].values[i][e] <== s<%- s+1 %>_vals[q][i*3+e];
            }
        }
        for (var i=0; i<<%- (starkStruct.steps[s].nBits - starkStruct.steps[s+1].nBits) %>; i++) {
            s<%- s %>_lowValues[q].key[i] <== ys[q][i + <%- starkStruct.steps[s+1].nBits %>];
        }
<% } else { -%>
        for (var i=0; i<<%- 1 << (starkStruct.steps[s].nBits) %>; i++) {
            for (var e=0; e<3; e++) {
                s<%- s %>_lowValues[q].values[i][e] <== finalPol[i][e];
            }
        }
        for (var i=0; i<<%- (starkStruct.steps[s].nBits) %>; i++) {
            s<%- s %>_lowValues[q].key[i] <== ys[q][i];
        }
<% }      -%>
        for (var e=0; e<3; e++) {
            enable * (s<%- s %>_lowValues[q].out[e] - s<%- s %>_evalPol[q].out[e]) === 0;
        }

        enable * (s<%- s %>_merkle[q].root[0] - s<%- s %>_root[0]) === 0;
        enable * (s<%- s %>_merkle[q].root[1] - s<%- s %>_root[1]) === 0;
        enable * (s<%- s %>_merkle[q].root[2] - s<%- s %>_root[2]) === 0;
        enable * (s<%- s %>_merkle[q].root[3] - s<%- s %>_root[3]) === 0;
    }
<% }                                                  -%>

///////
// Check Degree last pol
///////
// Last FFT
<% const nLastBits = starkStruct.steps[ starkStruct.steps.length-1].nBits;  -%>
<% const maxDegBits =  nLastBits -  (starkStruct.nBitsExt - starkStruct.nBits); -%>
    component lastIFFT = FFT(<%- nLastBits %>, 3, 1, 1 );

    for (var k=0; k< <%- 1 << nLastBits %>; k++ ){
        for (var e=0; e<3; e++) {
            lastIFFT.in[k][e] <== finalPol[k][e];
        }
    }

    for (var k= <%- 1 << maxDegBits %>; k< <%- 1 << nLastBits %>; k++ ) {
        for (var e=0; e<3; e++) {
            enable * lastIFFT.out[k][e] === 0;
        }
    }

}


}

<% if (!options.skipMain) {  -%>
component main {public [publics]}= StarkVerifier();
<% } -%>

}
