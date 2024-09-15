pub mod lib_compile;
pub mod lib_gen;

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
