use super::compilation_result::{CompilationError, CompilationResult};
use crate::std_lib_compile::{
    CompilerOptions as CompilerOptionsTrait, CompilerResult as CompilerResultTrait,
};

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    process::Command,
};
use tempfile::{tempdir, NamedTempFile};

#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct CompilerOptions {
    pub source_code: String,
    pub optimization_strategy: OptimizationStrategy,
    pub enable_bulk_memory: bool,
    pub enable_sign_extension: bool,
    pub enable_nontrapping_f2i: bool,
    pub enable_export_memory: bool,
    pub enable_wasi_shim: bool,
    pub flag_use: Option<HashMap<String, String>>,
    pub runtime: RuntimeStrategy,
}

impl CompilerOptionsTrait for CompilerOptions {
    fn source_code(&self) -> Vec<u8> {
        Vec::from(self.source_code.as_bytes())
    }

    fn compile(&self) -> Box<dyn CompilerResultTrait> {
        Box::new(Self::compile(self))
    }
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
    pub fn no_wasi(source_code: String) -> Self {
        Self {
            source_code,
            optimization_strategy: OptimizationStrategy::O3,
            enable_bulk_memory: false,
            enable_sign_extension: false,
            enable_nontrapping_f2i: false,
            enable_export_memory: false,
            enable_wasi_shim: false,
            flag_use: Some(HashMap::from_iter(vec![(
                "abort".into(),
                "custom_abort".into(),
            )])),
            runtime: RuntimeStrategy::Incremental,
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

        let flag_wasi = if self.enable_wasi_shim {
            "--config ./node_modules/@assemblyscript/wasi-shim/asconfig.json "
        } else {
            ""
        };

        let flag_use = if let Some(uses) = &self.flag_use {
            if uses.is_empty() {
                String::new()
            } else {
                let ch = uses.iter().map(|(key, value)| format!("{key}={value}"));
                format!("--lib . --use {} ", ch.collect::<Vec<String>>().join(" "))
            }
        } else {
            String::new()
        };

        format!(
            concat!(
                // Pass input file & output file to command
                "node ./node_modules/assemblyscript/bin/asc.js {source_path} -o {output_path} ",
                // Pass wasi shim configuration to command
                "{flag_wasi}",
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
            flag_wasi = flag_wasi,
            flag_use = flag_use,
        )
    }

    /// # Errors
    /// When the compilation failes.
    ///
    /// # Panics
    /// When system resources such as files cannot be acquired.
    pub fn compile(compile_options: &CompilerOptions) -> CompilationResult {
        let working_dir = tempdir().expect("Could not create temp dir");
        let working_dir_path = working_dir.path().to_string_lossy().to_string();

        Command::new("npm")
            .args(["init", "-y"])
            .current_dir(&working_dir_path)
            .output()
            .expect("Npm init failed");
        Command::new("npm")
            .args(["install", "assemblyscript", "@assemblyscript/wasi-shim"])
            .current_dir(&working_dir_path)
            .output()
            .expect("Npm install failed");

        let assemblyscript_source_file_path =
            working_dir.path().join("assemblyscript_source_file.ts");
        let mut assemblyscript_source_file = File::create(&assemblyscript_source_file_path)
            .expect("Could not create temp input file");
        assemblyscript_source_file
            .write_all(compile_options.source_code.as_bytes())
            .expect("Could not write std_lib to temp file");
        let assemblyscript_source_file_path = assemblyscript_source_file_path
            .to_string_lossy()
            .to_string();

        // TODO: this custom abort is hardcoded here, but weirdly 'provided' at call-site ... refactor!
        let custom_abort_source_file_path = working_dir.path().join("custom_abort_source_file.ts");
        let mut custom_abort_source_file =
            File::create(custom_abort_source_file_path).expect("Could not create temp input file");
        let custom_abort_lib = include_str!("./custom_abort_lib.ts");
        custom_abort_source_file
            .write_all(custom_abort_lib.as_bytes())
            .expect("Could not write std_lib to temp file");

        let mut output_file = NamedTempFile::new().expect("Could not create temp output file");
        let output_file_path = output_file.path().to_string_lossy().to_string();

        let mut command_compile_lib = Command::new("bash");
        command_compile_lib.current_dir(&working_dir_path);

        let npx_command =
            compile_options.to_npx_command(&assemblyscript_source_file_path, &output_file_path);

        command_compile_lib.args(["-c", &npx_command]);

        // Kick off command, i.e. compile
        let result = command_compile_lib
            .output()
            .expect("Could not execute compilation command");

        if !(result.stderr.is_empty() && result.stdout.is_empty()) {
            return Err(CompilationError(
                String::from_utf8_lossy(&result.stderr).to_string(),
            ));
        };

        let mut result = Vec::new();
        output_file
            .read_to_end(&mut result)
            .expect("Could not read result from written output");

        Ok(result) // FIXME: do not unwrap, make known what went wrong
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_compiler_option_for(source_code: String) -> CompilerOptions {
        CompilerOptions {
            source_code,
            enable_bulk_memory: false,
            enable_export_memory: false,
            enable_nontrapping_f2i: false,
            enable_sign_extension: false,
            enable_wasi_shim: false,
            flag_use: None,
            optimization_strategy: OptimizationStrategy::O3,
            runtime: RuntimeStrategy::Incremental,
        }
    }

    #[test]
    fn test_no_wasi() {
        let conf = CompilerOptions::no_wasi("/* source code here */".into());
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
    fn test_source_code_retrieval() {
        let option = simple_compiler_option_for("/* source-code here */".into());
        assert_eq!(
            String::from_utf8(option.source_code()).unwrap(),
            "/* source-code here */"
        )
    }

    #[test]
    fn test_assemblyscript_faulty_compilation() {
        let compiler_options =
            simple_compiler_option_for("this is not valid assemblyscript code".into());

        let reason = compiler_options.compile().module().unwrap_err();
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
            enable_wasi_shim: true,
            flag_use: Some(HashMap::from_iter(vec![(
                "abort".into(),
                "custom_abort".into(),
            )])),
            runtime: super::RuntimeStrategy::Incremental,
        };

        assert_eq!(
            options.to_npx_command("path/to/source", "path/to/output"),
            concat!(
                "node ./node_modules/assemblyscript/bin/asc.js path/to/source ",
                "-o path/to/output ",
                "--config ./node_modules/@assemblyscript/wasi-shim/asconfig.json ",
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
            enable_wasi_shim: false,
            flag_use: None,
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
