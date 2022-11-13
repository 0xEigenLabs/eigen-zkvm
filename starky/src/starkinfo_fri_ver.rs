use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::f3g::F3G;
use crate::helper::get_ks;
use crate::starkinfo::StarkInfo;
use crate::starkinfo::{CICTX, PECTX};
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Calculated, Context, ContextF, EVIdx, Node,
};
use crate::types::PolIdentity;
use std::collections::HashMap;

impl StarkInfo {
    pub fn generate_fri_verifier(&mut self, ctx: &mut Context) -> Result<()> {
        pil_code_gen(ctx, self.fri_exp_id, false, &"".to_string())?;
        Ok(())
    }
}
