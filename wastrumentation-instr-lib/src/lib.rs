pub mod std_lib_compile;
mod std_lib_gen;
pub mod wasm_constructs;

use wasm_constructs::Signature;

#[derive(Clone, Copy)]
pub enum Langauge {
    AssemblyScript,
}

#[must_use]
pub fn generate_lib(language: Langauge, signatures: &[Signature]) -> String {
    match language {
        Langauge::AssemblyScript => std_lib_gen::assemblyscript::generate_lib(signatures),
    }
}
