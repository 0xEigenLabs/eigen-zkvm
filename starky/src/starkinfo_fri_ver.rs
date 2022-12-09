use crate::errors::Result;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context};
use crate::types::PIL;
use std::collections::HashMap;

impl StarkInfo {
    pub fn generate_fri_verifier(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        pil_code_gen(ctx, pil, self.fri_exp_id, false, "", 0)?;

        let code = build_code(ctx, pil);
        self.n_exps = pil.expressions.len();
        program.verifier_query_code = code;

        //println!("verifier_query_code: {}", program.verifier_query_code);
        Ok(())
    }
}
