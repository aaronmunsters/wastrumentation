pub mod std_lib_compile;
mod std_lib_gen;
pub mod wasm_constructs;

use wasm_constructs::Signature;

#[derive(Clone, Copy)]
pub enum Language {
    AssemblyScript,
}

#[must_use]
pub fn generate_lib(language: Language, signatures: &[Signature]) -> String {
    match language {
        Language::AssemblyScript => std_lib_gen::assemblyscript::generate_lib(signatures),
    }
}
