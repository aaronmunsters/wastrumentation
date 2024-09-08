use std::collections::HashMap;

#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct CompilerOptions {
    pub source_code: String,
    pub optimization_strategy: OptimizationStrategy,
    pub enable_bulk_memory: bool,
    pub enable_sign_extension: bool,
    pub enable_nontrapping_f2i: bool,
    pub enable_export_memory: bool,
    pub flag_use: HashMap<String, String>,
    pub trap_on_abort: bool,
    pub runtime: RuntimeStrategy,
}

#[derive(Default)]
pub enum OptimizationStrategy {
    O1,
    O2,
    #[default]
    O3,
}

#[derive(Default)]
pub enum RuntimeStrategy {
    #[default]
    Incremental,
    Minimal,
    Stub,
}

impl CompilerOptions {
    #[must_use]
    pub fn new(source_code: String) -> Self {
        Self {
            source_code,
            optimization_strategy: OptimizationStrategy::O3,
            enable_bulk_memory: false,
            enable_sign_extension: false,
            enable_nontrapping_f2i: false,
            enable_export_memory: false,
            flag_use: HashMap::new(),
            runtime: RuntimeStrategy::Incremental,
            trap_on_abort: true,
        }
    }

    pub(crate) fn to_npx_command(&self, source_path: &str, output_path: &str) -> String {
        let flag_bulk_memory = if self.enable_bulk_memory {
            ""
        } else {
            "--disable bulk-memory "
        };

        let flag_sign_extension = if self.enable_sign_extension {
            ""
        } else {
            "--disable sign-extension "
        };

        let flag_non_trapping_f2i = if self.enable_nontrapping_f2i {
            ""
        } else {
            "--disable nontrapping-f2i "
        };

        let flag_export_memory = if self.enable_export_memory {
            ""
        } else {
            "--noExportMemory "
        };

        let flag_runtime = match self.runtime {
            RuntimeStrategy::Minimal => "--runtime minimal ",
            RuntimeStrategy::Incremental => "--runtime incremental ",
            RuntimeStrategy::Stub => "--runtime stub ",
        };

        let flag_optimization = match self.optimization_strategy {
            OptimizationStrategy::O1 => "-O1 ",
            OptimizationStrategy::O2 => "-O2 ",
            OptimizationStrategy::O3 => "-O3 ",
        };

        let flag_use = match (self.flag_use.is_empty(), self.trap_on_abort) {
            // No custom flags, no trap on abort
            (true, false) => String::new(),
            // Custom flags but no trap on abort
            (false, false) => format!(
                "--use {} ",
                self.flag_use
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            // Trap on abort
            (true, true) | (false, true) => {
                format!(
                    "--lib . --use {} ",
                    self.flag_use
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .chain(vec![("abort", "custom_abort")]) // include trap
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
        };

        format!(
            concat!(
                // Pass input file & output file to command
                "node ./node_modules/assemblyscript/bin/asc.js {source_path} -o {output_path} ",
                // Pas additional options to command
                "{flag_optimization}",
                "{flag_bulk_memory}",
                "{flag_sign_extension}",
                "{flag_non_trapping_f2i}",
                "{flag_runtime}",
                "{flag_export_memory}",
                "{flag_use}",
            ),
            source_path = &source_path,
            output_path = &output_path,
            flag_bulk_memory = flag_bulk_memory,
            flag_sign_extension = flag_sign_extension,
            flag_non_trapping_f2i = flag_non_trapping_f2i,
            flag_runtime = flag_runtime,
            flag_export_memory = flag_export_memory,
            flag_optimization = flag_optimization,
            flag_use = flag_use,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::std_lib_compile::assemblyscript::{
        compilation_result::CompilationError, compiler::Compiler,
    };

    use super::*;

    fn simple_compiler_option_for(source_code: String) -> CompilerOptions {
        CompilerOptions {
            source_code,
            enable_bulk_memory: false,
            enable_export_memory: false,
            enable_nontrapping_f2i: false,
            enable_sign_extension: false,
            flag_use: HashMap::new(),
            trap_on_abort: false,
            optimization_strategy: OptimizationStrategy::O3,
            runtime: RuntimeStrategy::Incremental,
        }
    }

    #[test]
    fn test_creation() {
        let conf = CompilerOptions::new("/* source code here */".into());
        assert_eq!(
            conf.to_npx_command("source_path", "output_path"),
            concat!(
                "node ./node_modules/assemblyscript/bin/asc.js source_path ",
                "-o output_path",
                " -O3 ",
                "--disable bulk-memory ",
                "--disable sign-extension ",
                "--disable nontrapping-f2i ",
                "--runtime incremental ",
                "--noExportMemory ",
                "--lib . --use abort=custom_abort ",
            )
        );
    }

    #[test]
    fn test_assemblyscript_faulty_compilation() {
        let compiler = Compiler::new();
        let compiler_options =
            simple_compiler_option_for("this is not valid assemblyscript code".into());
        let CompilationError(reason) = compiler.compile(&compiler_options).unwrap_err();
        assert!(reason.contains("ERROR"));
    }

    #[test]
    fn test_to_npx() {
        let mut options = CompilerOptions {
            source_code: "source".into(),
            optimization_strategy: OptimizationStrategy::O1,
            enable_bulk_memory: true,
            enable_sign_extension: true,
            enable_nontrapping_f2i: true,
            enable_export_memory: true,
            flag_use: HashMap::new(),
            trap_on_abort: true,
            runtime: super::RuntimeStrategy::Incremental,
        };

        assert_eq!(
            options.to_npx_command("path/to/source", "path/to/output"),
            concat!(
                "node ./node_modules/assemblyscript/bin/asc.js path/to/source ",
                "-o path/to/output ",
                "-O1 --runtime incremental ",
                "--lib . --use abort=custom_abort ",
            )
        );

        options = CompilerOptions {
            source_code: "source".into(),
            optimization_strategy: OptimizationStrategy::O2,
            enable_bulk_memory: false,
            enable_sign_extension: false,
            enable_nontrapping_f2i: false,
            enable_export_memory: false,
            flag_use: HashMap::new(),
            trap_on_abort: false,
            runtime: super::RuntimeStrategy::Incremental,
        };

        assert_eq!(
            options.to_npx_command("path/to/source", "path/to/output"),
            concat!(
                "node ./node_modules/assemblyscript/bin/asc.js path/to/source ",
                "-o path/to/output ",
                "-O2 ",
                "--disable bulk-memory ",
                "--disable sign-extension ",
                "--disable nontrapping-f2i ",
                "--runtime incremental ",
                "--noExportMemory ",
            )
        );
    }
}
