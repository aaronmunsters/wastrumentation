// Wastrumentation imports
use rust_to_wasm_compiler::{Profile, WasiSupport};
use wastrumentation::compiler::Compiles;
use wastrumentation_lang_rust::compile::compiler::Compiler;
use wastrumentation_lang_rust::compile::options::*;

const INPUT_PROGRAM_MANIFEST: &str = r#"
package.name = "rust-denan-input-program"
package.version = "0.1.0"
package.edition = "2021"
lib.crate-type = ["cdylib"]
profile.release.strip = true
profile.release.lto = true
profile.release.panic = "abort"
[workspace]
"#;

pub enum Source {
    Rust(&'static str, WasiSupport, Profile),
    // Wasm(Box<Path>),
}

impl Source {
    pub fn to_input_program(&self) -> Vec<u8> {
        match self {
            Source::Rust(source, wasi_support, profile) => Compiler::setup_compiler()
                .unwrap()
                .compile(&CompilerOptions {
                    profile: *profile,
                    source: RustSource::SourceCode(
                        *wasi_support,
                        ManifestSource(INPUT_PROGRAM_MANIFEST.into()),
                        RustSourceCode(source.to_string()),
                    ),
                })
                .unwrap(),
        }
    }
}
