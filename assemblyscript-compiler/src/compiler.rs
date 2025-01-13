use std::io::Write;
use std::process::Command;
use std::{fs::File, io::Read};

use crate::error::{CompilationError, CompilerSetupError};
use tempfile::{tempdir, TempDir};

use crate::options::CompilerOptions;

pub struct Compiler {
    working_dir: TempDir,
}

impl Compiler {
    /// # Errors
    /// When setup of a temporary directory fails.
    pub fn new() -> Result<Self, CompilerSetupError> {
        let working_dir = tempdir().expect("Could not create temp dir");

        let custom_abort_path = &working_dir.path().join("custom_abort_source_file.ts");
        let mut custom_abort_file =
            File::create(custom_abort_path).map_err(CompilerSetupError::CustomAbortFileCreation)?;
        custom_abort_file
            .write_all(include_str!("./custom_abort_lib.ts").as_bytes())
            .map_err(CompilerSetupError::CustomAbortFileWrite)?;

        Command::new("npm")
            .args(["init", "-y"])
            .current_dir(&working_dir)
            .output()
            .map_err(CompilerSetupError::NpmInitFailed)?;

        Command::new("npm")
            .args(["install", "assemblyscript"])
            .current_dir(&working_dir)
            .output()
            .map_err(CompilerSetupError::NpmInstallFailed)?;

        Ok(Self { working_dir })
    }

    /// # Errors
    /// When the compilation failes.
    pub fn compile(&self, compiler_options: &CompilerOptions) -> Result<Vec<u8>, CompilationError> {
        let mut source_file = tempfile::Builder::new()
            .prefix("source_file")
            .suffix(".ts")
            .tempfile()
            .map_err(CompilationError::CreateTempInputFile)?;

        source_file
            .write_all(compiler_options.source.as_bytes())
            .map_err(CompilationError::WriteSourceCodeToTempInputFile)?;
        source_file
            .flush()
            .map_err(CompilationError::FlushSourceCodeToTempInputFile)?;

        let mut output_file = tempfile::Builder::new()
            .prefix("output_file")
            .suffix(".ts")
            .tempfile()
            .map_err(CompilationError::CreateTempOutputFile)?;

        let source_file_path = source_file.path();
        let output_file_path = output_file.path();
        let npx_command = compiler_options.to_npx_command(source_file_path, output_file_path);

        let mut command_compile_lib = Command::new("bash");
        command_compile_lib
            .args(["-c", &npx_command])
            .current_dir(&self.working_dir);

        // Kick off command, i.e. compile
        let result = command_compile_lib
            .output()
            .map_err(CompilationError::ExecuteCompilationCommand)?;

        result.status.success().then_some(true).ok_or(
            CompilationError::AssemblyScriptCompilationFailed(
                String::from_utf8_lossy(&result.stderr).to_string(),
            ),
        )?;

        drop(source_file);

        let mut result = Vec::new();
        output_file
            .read_to_end(&mut result)
            .map_err(CompilationError::ReadResultFromCompiledOutput)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::CompilerRuntime;
    use strum::IntoEnumIterator;
    use wasmtime::{Engine, Instance, Module, Store};

    #[test]
    fn test_assemblyscript_compilation_binary() {
        let source_code = r#"
        export function add_three(a: i32, b: i32, c: i32): i32 {
            return a + b + c;
        }
        "#;

        let compiler = Compiler::new().unwrap();
        let compile_options = CompilerOptions::default_for(source_code);
        let wasm_module = compiler.compile(&compile_options).unwrap();

        let wasm_magic_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D];
        assert_eq!(&wasm_module[0..4], wasm_magic_bytes);
    }

    #[test]
    fn test_assemblyscript_compilation_working_binary() {
        let source_code = r#"
        function fac(n: i32): i32 {
            return n === 1 ? 1 : n * fac(n-1);
        }

        export function add_to_fac(a: i32, b: i32, c: i32): i32 {
            return a + b + fac(c);
        }
        "#;

        let compiler = Compiler::new().unwrap();
        let mut compile_options = CompilerOptions::default_for(source_code);
        for compiler_runtime in CompilerRuntime::iter() {
            compile_options.compiler_runtime = compiler_runtime;
            let wasm_module = compiler.compile(&compile_options).unwrap();

            let engine = Engine::default();
            let module = Module::from_binary(&engine, &wasm_module).unwrap();
            let mut store = Store::new(&engine, ());

            let instance = Instance::new(&mut store, &module, &[]).unwrap();
            let run = instance
                .get_typed_func::<(i32, i32, i32), i32>(&mut store, "add_to_fac")
                .unwrap();

            // And last but not least we can call it!
            assert_eq!(run.call(&mut store, (1, 2, 3)).unwrap(), 9);
        }
    }

    #[test]
    fn test_assemblyscript_faulty_compilation() {
        let compiler = Compiler::new().unwrap();
        for compiler_runtime in CompilerRuntime::iter() {
            let mut compiler_options =
                CompilerOptions::default_for("this is not valid assemblyscript code");
            compiler_options.compiler_runtime = compiler_runtime;

            assert!(compiler
                .compile(&compiler_options)
                .unwrap_err()
                .to_string()
                .contains("ERROR"));
        }
    }
}
