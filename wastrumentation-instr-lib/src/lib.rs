mod std_lib_gen;
pub mod wasm_constructs;

use wasm_constructs::Signature;

pub enum Langauge {
    AssemblyScript,
}

pub fn generate_lib(language: Langauge, signatures: &[Signature]) -> String {
    match language {
        Langauge::AssemblyScript => std_lib_gen::assemblyscript::generate_lib(signatures),
    }
}
