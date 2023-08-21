use crate::pilcom::compile_pil;
// todo
// 1. compile(pilFile) -> .pil.json
// compile_pil
// const pil = await compile(F, pilFile, null, pilConfig);
// await fs.promises.writeFile(pilFile+ ".json", JSON.stringify(pil, null, 1) + "\n", "utf8");


// 2. pil -> cm
// const cmPols = newCommitPolsArray(pil);
// 3. wasm -> wc
// const wc = await WitnessCalculatorBuilder(wasm);
// 4.inptu -> w
// const w = await wc.calculateWitness(input);
//
// for (let i=0; i<nAdds; i++) {
// w.push( F.add( F.mul( w[addsBuff[i*4]], addsBuff[i*4 + 2]), F.mul( w[addsBuff[i*4+1]],  addsBuff[i*4+3]  )));
// }
//
// const N = cmPols.Compressor.a[0].length;
//
// for (let i=0; i<nSMap; i++) {
// for (let j=0; j<12; j++) {
// if (sMapBuff[12*i+j] != 0) {
// cmPols.Compressor.a[j][i] = w[sMapBuff[12*i+j]];
// } else {
// cmPols.Compressor.a[j][i] = 0n;
// }
// }
// }
//
// for (let i=nSMap; i<N; i++) {
// for (let j=0; j<12; j++) {
// cmPols.Compressor.a[j][i] = 0n;
// }
// }
//
// await cmPols.saveToFile(commitFile);
