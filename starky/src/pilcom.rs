// Rust version of pilcom.compile(js), which compiles a .pil file to its json form and also tries to generate
// constants and committed polynomials.
// @returns a compilation result, containing witness and fixed columns
// More test see compiler::test::pil::verify_pil
pub use compiler::compile_pil;
pub use compiler::compile_pil_ast;
// Rust version of pilcom.verifyPil(js)
pub use compiler::verify;
