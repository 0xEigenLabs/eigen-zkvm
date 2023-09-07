#[derive(Default)]
pub struct PlonkSetup {
    pub(crate) pil_str: String,
    pub(crate) const_pols: u64,
    pub(crate) s_map: vec![vec![]],
    pub(crate) plonk_additions: vec![vec![]],
}

impl PlonkSetup {
    pub fn plonk_setup(r1cs: &R1CS<GL>, opts: &Options) -> Self {
        // 0. r1cs to plonk
        // const [plonkConstraints, plonkAdditions] = r1cs2plonk(F, r1cs);
        // const plonkConstraints = new BigArray();
        // const plonkAdditions = new BigArray();

        // 1. get normal plonk info
        // const plonkInfo = getNormalPlonkInfo();

        // 2. get custom gate info
        // const customGatesInfo = getCustomGatesInfo();

        // 3. calculate columns,rows,constraints info.

        // 4. render .pil file by template.
        //      And save as a file.
        // const pilStr = ejs.render(template ,  obj);

        // 5. compile pil and init ConstantPolsArray
        //      And construct it.
        // const pil = await compile(F, pilFile);
        // const constPols =  newConstantPolsArray(pil);

        // 6. init sMap and construct it.

        // 7. generate custom gates

        // 8. calculate S polynomial

        // 9. Fill unused rows.

        Self::Default()
    }
}
