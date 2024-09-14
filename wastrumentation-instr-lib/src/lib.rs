mod analysis_gen_assemblyscript;
pub mod std_lib_compile;
mod std_lib_gen;

use std::marker::PhantomData;

use std_lib_compile::assemblyscript::compiler_options::CompilerOptions as AssemblyScriptCompilerOptions;
use std_lib_compile::rust::CompilerOptions as RustCompilerOptions;
use std_lib_compile::rust::RustSource;
use std_lib_gen::assemblyscript::generate_lib as generate_AS_lib;
use std_lib_gen::rust::generate_lib as generate_RS_lib;
use wastrumentation::compiler::LibGeneratable;
use wastrumentation::compiler::Library;
use wastrumentation::compiler::SourceCodeBound;
use wastrumentation::wasm_constructs::Signature;

// Languages
#[derive(Debug, Clone)]
pub struct AssemblyScript;

#[derive(Debug, Clone)]
pub struct Rust;

impl LibGeneratable for AssemblyScript {
    fn generate_lib(signatures: &[Signature]) -> Library<Self> {
        Library::<Self> {
            content: generate_AS_lib(signatures),
            language: PhantomData,
        }
    }
}

impl LibGeneratable for Rust {
    fn generate_lib(signatures: &[Signature]) -> Library<Self> {
        let (manifest_source, rust_source) = generate_RS_lib(signatures);
        Library::<Self> {
            content: RustSource::SourceCode(manifest_source, rust_source),
            language: PhantomData,
        }
    }
}

impl SourceCodeBound for Rust {
    type DefaultCompiler = RustCompilerOptions;
    type SourceCode = RustSource;
}

impl SourceCodeBound for AssemblyScript {
    type DefaultCompiler = AssemblyScriptCompilerOptions;
    type SourceCode = String;
}

// impl From<String> for Library<AssemblyScript> {
//     fn from(value: String) -> Self {
//         Library {
//             content: value,
//             language: Default::default(),
//         }
//     }
// }

// impl From<(ManifestSource, RustSourceCode)> for Library<Rust> {
//     fn from(value: (ManifestSource, RustSourceCode)) -> Self {
//         let (manifest_source, rust_source_code) = value;
//         Library {
//             content: RustSource::SourceCode(manifest_source, rust_source_code),
//             language: Default::default(),
//         }
//     }
// }
