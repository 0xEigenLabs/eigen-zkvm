#![allow(clippy::to_string_trait_impl)]

pub enum CompressorNameSpace {
    Global,
    Compressor,
}

#[allow(dead_code, clippy::upper_case_acronyms, non_camel_case_types)]
pub enum CompressorPolName {
    // pols name under namespace `Global`
    L,
    // pols name under namespace `Compressor`
    S,
    C,
    a,
    PARTIAL,
    POSEIDON12,
    GATE,
    CMULADD,
    EVPOL4,
    FFT4,
}

// impl
impl ToString for CompressorPolName {
    fn to_string(&self) -> String {
        match self {
            Self::L => "L".to_string(),
            Self::S => String::from("S"),
            Self::C => String::from("C"),
            Self::a => String::from("a"),
            Self::PARTIAL => String::from("PARTIAL"),
            Self::POSEIDON12 => String::from("POSEIDON12"),
            Self::GATE => String::from("GATE"),
            Self::CMULADD => String::from("CMULADD"),
            Self::EVPOL4 => String::from("EVPOL4"),
            Self::FFT4 => String::from("FFT4"),
        }
    }
}
impl ToString for CompressorNameSpace {
    fn to_string(&self) -> String {
        match self {
            Self::Global => String::from("Global"),
            Self::Compressor => String::from("Compressor"),
        }
    }
}

pub fn render(n_bits: usize, n_publics: usize) -> String {
    let mut res = String::from("");
    res.push_str(&format!(
        r#"
let N: int = 2**{n_bits};
namespace Global(N);
    pol constant L1 = [0]*;
    "#
    ));
    for i in (12..n_publics).step_by(12) {
        res.push_str(&format!(
            r#"
    pol constant L{} = [0]*;
            "#,
            i / 12 + 1
        ));
    }

    res.push_str(
        r#"
namespace Compressor(N);
    pol constant S_0 = [0]*;
    pol constant S_1 = [0]*;
    pol constant S_2 = [0]*;
    pol constant S_3 = [0]*;
    pol constant S_4 = [0]*;
    pol constant S_5 = [0]*;
    pol constant S_6 = [0]*;
    pol constant S_7 = [0]*;
    pol constant S_8 = [0]*;
    pol constant S_9 = [0]*;
    pol constant S_10 = [0]*;
    pol constant S_11 = [0]*;
    pol constant C_0 = [0]*;
    pol constant C_1 = [0]*;
    pol constant C_2 = [0]*;
    pol constant C_3 = [0]*;
    pol constant C_4 = [0]*;
    pol constant C_5 = [0]*;
    pol constant C_6 = [0]*;
    pol constant C_7 = [0]*;
    pol constant C_8 = [0]*;
    pol constant C_9 = [0]*;
    pol constant C_10 = [0]*;
    pol constant C_11 = [0]*;
    pol constant PARTIAL = [0]*;
    pol constant POSEIDON12 = [0]*;
    pol constant GATE = [0]*;
    pol constant CMULADD = [0]*;
    pol constant EVPOL4 = [0]*;
    pol constant FFT4 = [0]*;
    pol commit a[12];
            "#,
    );

    for i in 0..n_publics {
        res.push_str(&format!(
            r#"
    public pub{} = a[{}]({});
            "#,
            i,
            i % 12,
            i / 12
        ));
    }

    for i in 0..n_publics {
        res.push_str(&format!(
            r#"
    Global::L{} * (a[{}] - :pub{}) = 0;
            "#,
            i / 12 + 1,
            i % 12,
            i
        ));
    }

    // Normal plonk gate
    res.push_str(
        r#"
    pol a01 = a[0] * a[1];
    pol g012 = C_3 * a01 + C_0 * a[0] + C_1 * a[1] + C_2 * a[2] + C_4;
    g012 * GATE = 0;
    pol a34 = a[3] * a[4];
    pol g345 = C_3 * a34 + C_0 * a[3] + C_1 * a[4] + C_2 * a[5] + C_4;
    g345 * GATE = 0;
    pol a67 = a[6] * a[7];
    pol g678 = C_9 * a67 + C_6 * a[6] + C_7 * a[7] + C_8 * a[8] + C_10;
    g678 * GATE = 0;
    pol a910 = a[9] * a[10];
    pol g91011 = C_9 * a910 + C_6 * a[9] + C_7 * a[10] + C_8 * a[11] + C_10;
    g91011 * GATE = 0;
    "#,
    );

    // POSEIDON12 GATE
    for i in 0..12 {
        res.push_str(&format!(
            r#"
    pol a{i}_1 = a[{i}] + C_{i};
        "#
        ));

        res.push_str(&format!(
            r#"
    pol a{i}_2 = a{i}_1 * a{i}_1;
        "#
        ));
        res.push_str(&format!(
            r#"
    pol a{i}_4 = a{i}_2 * a{i}_2;
        "#
        ));
        res.push_str(&format!(
            r#"
    pol a{i}_6 = a{i}_4 * a{i}_2;
        "#
        ));
        res.push_str(&format!(
            r#"
    pol a{i}_7 = a{i}_6 * a{i}_1;
        "#
        ));
        if i == 0 {
            res.push_str(&format!(
                r#"
    pol a{i}_R = a{i}_7;
        "#
            ));
        } else {
            res.push_str(&format!(
                r#"
    pol a{i}_R = PARTIAL * (a{i}_1 - a{i}_7) + a{i}_7;
        "#
            ));
        }
    }
    res.push_str(
        r#"
    POSEIDON12 * (a[0]' - (25 * a0_R + 15 * a1_R + 41 * a2_R + 16 * a3_R + 2 * a4_R + 28 * a5_R + 13 * a6_R + 13 * a7_R + 39 * a8_R + 18 * a9_R + 34 * a10_R + 20 * a11_R)) = 0;
    POSEIDON12 * (a[1]' - (20 * a0_R + 17 * a1_R + 15 * a2_R + 41 * a3_R + 16 * a4_R + 2 * a5_R + 28 * a6_R + 13 * a7_R + 13 * a8_R + 39 * a9_R + 18 * a10_R + 34 * a11_R)) = 0;
    POSEIDON12 * (a[2]' - (34 * a0_R + 20 * a1_R + 17 * a2_R + 15 * a3_R + 41 * a4_R + 16 * a5_R + 2 * a6_R + 28 * a7_R + 13 * a8_R + 13 * a9_R + 39 * a10_R + 18 * a11_R)) = 0;
    POSEIDON12 * (a[3]' - (18 * a0_R + 34 * a1_R + 20 * a2_R + 17 * a3_R + 15 * a4_R + 41 * a5_R + 16 * a6_R + 2 * a7_R + 28 * a8_R + 13 * a9_R + 13 * a10_R + 39 * a11_R)) = 0;
    POSEIDON12 * (a[4]' - (39 * a0_R + 18 * a1_R + 34 * a2_R + 20 * a3_R + 17 * a4_R + 15 * a5_R + 41 * a6_R + 16 * a7_R + 2 * a8_R + 28 * a9_R + 13 * a10_R + 13 * a11_R)) = 0;
    POSEIDON12 * (a[5]' - (13 * a0_R + 39 * a1_R + 18 * a2_R + 34 * a3_R + 20 * a4_R + 17 * a5_R + 15 * a6_R + 41 * a7_R + 16 * a8_R + 2 * a9_R + 28 * a10_R + 13 * a11_R)) = 0;
    POSEIDON12 * (a[6]' - (13 * a0_R + 13 * a1_R + 39 * a2_R + 18 * a3_R + 34 * a4_R + 20 * a5_R + 17 * a6_R + 15 * a7_R + 41 * a8_R + 16 * a9_R + 2 * a10_R + 28 * a11_R)) = 0;
    POSEIDON12 * (a[7]' - (28 * a0_R + 13 * a1_R + 13 * a2_R + 39 * a3_R + 18 * a4_R + 34 * a5_R + 20 * a6_R + 17 * a7_R + 15 * a8_R + 41 * a9_R + 16 * a10_R + 2 * a11_R)) = 0;
    POSEIDON12 * (a[8]' - (2 * a0_R + 28 * a1_R + 13 * a2_R + 13 * a3_R + 39 * a4_R + 18 * a5_R + 34 * a6_R + 20 * a7_R + 17 * a8_R + 15 * a9_R + 41 * a10_R + 16 * a11_R)) = 0;
    POSEIDON12 * (a[9]' - (16 * a0_R + 2 * a1_R + 28 * a2_R + 13 * a3_R + 13 * a4_R + 39 * a5_R + 18 * a6_R + 34 * a7_R + 20 * a8_R + 17 * a9_R + 15 * a10_R + 41 * a11_R)) = 0;
    POSEIDON12 * (a[10]' - (41 * a0_R + 16 * a1_R + 2 * a2_R + 28 * a3_R + 13 * a4_R + 13 * a5_R + 39 * a6_R + 18 * a7_R + 34 * a8_R + 20 * a9_R + 17 * a10_R + 15 * a11_R)) = 0;
    POSEIDON12 * (a[11]' - (15 * a0_R + 41 * a1_R + 16 * a2_R + 2 * a3_R + 28 * a4_R + 13 * a5_R + 13 * a6_R + 39 * a7_R + 18 * a8_R + 34 * a9_R + 20 * a10_R + 17 * a11_R)) = 0;
    pol ca0 = (a[0] + C_0) * C_9;
    pol ca1 = (a[1] + C_1) * C_9;
    pol ca2 = (a[2] + C_2) * C_9;
    pol ca3 = a[3] + C_3;
    pol ca4 = a[4] + C_4;
    pol ca5 = a[5] + C_5;
    pol ca6 = (a[6] + C_6) * C_10;
    pol ca7 = (a[7] + C_7) * C_10;
    pol ca8 = (a[8] + C_8) * C_10;
    pol ca9 = a[9];
    pol ca10 = a[10];
    pol ca11 = a[11];
    pol cA = (ca0 + ca1) * (ca3 + ca4);
    pol cB = (ca0 + ca2) * (ca3 + ca5);
    pol cC = (ca1 + ca2) * (ca4 + ca5);
    pol cD = ca0 * ca3;
    pol cE = ca1 * ca4;
    pol cF = ca2 * ca5;
    CMULADD * (ca9 - (cC + cD - cE - cF) - ca6) = 0;
    CMULADD * (ca10 - (cA + cC - 2 * cE - cD) - ca7) = 0;
    CMULADD * (ca11 - (cB - cD + cE) - ca8) = 0;
    pol g0 = C_0 * a[0] + C_1 * a[3] + C_2 * a[6] + C_3 * a[9] + C_6 * a[0] + C_7 * a[3];
    pol g1 = C_0 * a[1] + C_1 * a[4] + C_2 * a[7] + C_3 * a[10] + C_6 * a[1] + C_7 * a[4];
    pol g2 = C_0 * a[2] + C_1 * a[5] + C_2 * a[8] + C_3 * a[11] + C_6 * a[2] + C_7 * a[5];
    pol g3 = C_0 * a[0] - C_1 * a[3] + C_4 * a[6] - C_5 * a[9] + C_6 * a[0] - C_7 * a[3];
    pol g4 = C_0 * a[1] - C_1 * a[4] + C_4 * a[7] - C_5 * a[10] + C_6 * a[1] - C_7 * a[4];
    pol g5 = C_0 * a[2] - C_1 * a[5] + C_4 * a[8] - C_5 * a[11] + C_6 * a[2] - C_7 * a[5];
    pol g6 = C_0 * a[0] + C_1 * a[3] - C_2 * a[6] - C_3 * a[9] + C_6 * a[6] + C_8 * a[9];
    pol g7 = C_0 * a[1] + C_1 * a[4] - C_2 * a[7] - C_3 * a[10] + C_6 * a[7] + C_8 * a[10];
    pol g8 = C_0 * a[2] + C_1 * a[5] - C_2 * a[8] - C_3 * a[11] + C_6 * a[8] + C_8 * a[11];
    pol g9 = C_0 * a[0] - C_1 * a[3] - C_4 * a[6] + C_5 * a[9] + C_6 * a[6] - C_8 * a[9];
    pol g10 = C_0 * a[1] - C_1 * a[4] - C_4 * a[7] + C_5 * a[10] + C_6 * a[7] - C_8 * a[10];
    pol g11 = C_0 * a[2] - C_1 * a[5] - C_4 * a[8] + C_5 * a[11] + C_6 * a[8] - C_8 * a[11];
    FFT4 * (a[0]' - g0) = 0;
    FFT4 * (a[1]' - g1) = 0;
    FFT4 * (a[2]' - g2) = 0;
    FFT4 * (a[3]' - g3) = 0;
    FFT4 * (a[4]' - g4) = 0;
    FFT4 * (a[5]' - g5) = 0;
    FFT4 * (a[6]' - g6) = 0;
    FFT4 * (a[7]' - g7) = 0;
    FFT4 * (a[8]' - g8) = 0;
    FFT4 * (a[9]' - g9) = 0;
    FFT4 * (a[10]' - g10) = 0;
    FFT4 * (a[11]' - g11) = 0;
       "#
    );

    // CMulAdd
    let mut c_mul_add = |r0: &str,
                         r1: &str,
                         r2: &str,
                         a0: &str,
                         a1: &str,
                         a2: &str,
                         b0: &str,
                         b1: &str,
                         b2: &str,
                         c0: &str,
                         c1: &str,
                         c2: &str| {
        res.push_str(
            format!(
                r#"
        pol {r0}_A = ({a0} + {a1}) * ({b0} + {b1});
            "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r0}_B = ({a0} + {a2}) * ({b0} + {b2});
            "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r0}_C = ({a1} + {a2}) * ({b1} + {b2});
            "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r0}_D = {a0} * {b0};
        "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r0}_E = {a1} * {b1};
        "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r0}_F = {a2} * {b2};
        "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r0} = {r0}_C + {r0}_D - {r0}_E - {r0}_F + {c0};
        "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r1} = {r0}_A + {r0}_C - 2 * {r0}_E - {r0}_D + {c1};
        "#
            )
            .as_str(),
        );
        res.push_str(
            format!(
                r#"
        pol {r2} = {r0}_B - {r0}_D + {r0}_E + {c2};
        "#
            )
            .as_str(),
        );
    };

    c_mul_add(
        "acc1_0", "acc1_1", "acc1_2", "a[0]'", "a[1]'", "a[2]'", "a[3]'", "a[4]'", "a[5]'", "a[9]",
        "a[10]", "a[11]",
    );
    c_mul_add(
        "acc2_0", "acc2_1", "acc2_2", "acc1_0", "acc1_1", "acc1_2", "a[3]'", "a[4]'", "a[5]'",
        "a[6]", "a[7]", "a[8]",
    );
    c_mul_add(
        "acc3_0", "acc3_1", "acc3_2", "acc2_0", "acc2_1", "acc2_2", "a[3]'", "a[4]'", "a[5]'",
        "a[3]", "a[4]", "a[5]",
    );
    c_mul_add(
        "acc4_0", "acc4_1", "acc4_2", "acc3_0", "acc3_1", "acc3_2", "a[3]'", "a[4]'", "a[5]'",
        "a[0]", "a[1]", "a[2]",
    );

    // EVPOL4
    res.push_str(
        r#"
    EVPOL4 * (a[6]' - acc4_0 ) = 0;
    EVPOL4 * (a[7]' - acc4_1 ) = 0;
    EVPOL4 * (a[8]' - acc4_2 ) = 0;
    [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8], a[9], a[10], a[11]] connect [S_0, S_1, S_2, S_3, S_4, S_5, S_6, S_7, S_8, S_9, S_10, S_11];
    "#,
    );

    res
}

#[macro_export]
macro_rules! c_mul_add {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(test)]
mod test {
    use crate::compressor12_pil::render;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_render() {
        let pil_string = render(5, 5);

        let mut file = File::create(Path::new("/tmp/render_pil_rs.pil")).unwrap();
        file.write_all(pil_string.as_bytes()).unwrap();
    }

    #[test]
    fn test_render_and_compile() {
        let pil_string = render(5, 5);
        let mut file = File::create(Path::new("/tmp/render_pil_rs.pil")).unwrap();
        write!(file, "{}", pil_string).unwrap();
    }
}
