// build.rs

use std::env;
use std::fs;
use std::path::Path;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let src_dir = match env::var_os("SRC_DIR") {
        Some(x) => x,
        _ => "/tmp".into(),
    };
    let out_dir = env::var_os("OUT_DIR").unwrap();
    p!("out_dir: {:?}", out_dir);
    // /tmp/ctx_public.rs  /tmp/ctx_step2prev.rs  /tmp/ctx_step3prev.rs  /tmp/ctx_step3.rs  /tmp/ctx_step4.rs  /tmp/ctx_step5.rs
    #[cfg(not(feature = "build"))]
    let file_inc = vec![
        "public",
        "step2prev",
        "step3prev",
        "step3",
        "step4",
        "step5",
    ];
    #[cfg(feature = "build")]
    let file_inc: [&str; 0] = [];

    for fi in file_inc.iter() {
        let rfn = Path::new(&src_dir).join(format!("ctx_{}.rs", fi));
        let fc = fs::read_to_string(&rfn).unwrap();
        let dest_path = Path::new(&out_dir).join(format!("ctx_{}.rs", fi));
        let body = format!(
            r#"
impl Block {{
        #[cfg(not(feature = "build"))]
        pub fn {}_fn(&self, ctx: &mut StarkContext, i: usize) -> F3G {{
            {}
        }}
}}
        "#,
            fi, fc
        );
        fs::write(&dest_path, body).unwrap();
    }
    //println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=rebuild");
}
