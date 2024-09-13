use super::compiler_options::CompilerOptions;
use crate::std_lib_compile::{CompilationError, CompilationResult, Compiles};
use crate::AssemblyScript;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

use anyhow::anyhow;
use tempfile::{tempdir, TempDir};

pub struct Compiler {
    working_dir: TempDir,
}

impl Compiles<AssemblyScript> for Compiler {
    type CompilerOptions = CompilerOptions;

    fn setup_compiler() -> anyhow::Result<Self> {
        let working_dir = tempdir().expect("Could not create temp dir");

        let custom_abort_source_file_path = working_dir.path().join("custom_abort_source_file.ts");
        let mut custom_abort_source_file = File::create(custom_abort_source_file_path)
            .map_err(|e| anyhow!("Could not create temp input file: {e:?}"))?;
        let custom_abort_lib = include_str!("./custom_abort_lib.ts");
        custom_abort_source_file
            .write_all(custom_abort_lib.as_bytes())
            .map_err(|e| anyhow!("Could not write std_lib to temp file: {e:?}"))?;

        Command::new("npm")
            .args(["init", "-y"])
            .current_dir(&working_dir)
            .output()
            .map_err(|e| anyhow!("Npm init failed: {e:?}"))?;

        Command::new("npm")
            .args(["install", "assemblyscript"])
            .current_dir(&working_dir)
            .output()
            .map_err(|e| anyhow!("Npm install failed: {e:?}"))?;

        Ok(Self { working_dir })
    }

    /// # Errors
    /// When the compilation failes.
    ///
    /// # Panics
    /// When system resources such as files cannot be acquired.
    fn compile(
        &self,
        compiler_options: &Self::CompilerOptions,
    ) -> CompilationResult<AssemblyScript> {
        let mut source_file = tempfile::Builder::new()
            .prefix("source_file")
            .suffix(".ts")
            .tempfile()
            .map_err(|e| e.to_string())
            .map_err(|e| {
                CompilationError::because(format!("Could not create temp input file: {e}"))
            })?;

        source_file
            .write_all(compiler_options.source.as_bytes())
            .map_err(|e| e.to_string())
            .map_err(|e| {
                CompilationError::because(format!("Could not source code to temp file: {e}"))
            })?;
        source_file
            .flush()
            .map_err(|e| e.to_string())
            .map_err(|e| {
                CompilationError::because(format!("Could not source code to temp file: {e}"))
            })?;

        let mut output_file = tempfile::Builder::new()
            .prefix("output_file")
            .suffix(".ts")
            .tempfile()
            .map_err(|e| e.to_string())
            .map_err(|e| {
                CompilationError::because(format!("Could not create temp output file: {e}"))
            })?;

        let source_file_path = source_file.path().to_string_lossy().to_string();
        let output_file_path = output_file.path().to_string_lossy().to_string();
        let npx_command = compiler_options.to_npx_command(&source_file_path, &output_file_path);

        let mut command_compile_lib = Command::new("bash");
        command_compile_lib
            .args(["-c", &npx_command])
            .current_dir(&self.working_dir);

        // Kick off command, i.e. compile
        let result = command_compile_lib
            .output()
            .map_err(|e| e.to_string())
            .map_err(|e| {
                CompilationError::because(format!("Could not execute compilation command: {e}"))
            })?;

        result
            .status
            .success()
            .then_some(true)
            .ok_or(CompilationError::because(format!(
                "{:?}",
                String::from_utf8_lossy(&result.stderr)
            )))?;

        drop(source_file);

        let mut result = Vec::new();
        output_file
            .read_to_end(&mut result)
            .map_err(|e| e.to_string())
            .map_err(|e| {
                CompilationError::because(format!("Could not read result from written output: {e}"))
            })?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::std_lib_compile::assemblyscript::compiler_options::CompilerOptions;
    use crate::std_lib_compile::{assemblyscript::compiler::Compiler, DefaultCompilerOptions};

    use super::*;
    use wasmtime::{Engine, Instance, Module, Store};

    #[test]
    fn test_assemblyscript_compilation_binary() {
        let source_code = r#"
        export function add_three(a: i32, b: i32, c: i32): i32 {
            return a + b + c;
        }
        "#
        .to_string();

        let compiler = Compiler::setup_compiler().unwrap();
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
        "#
        .to_string();

        let compiler = Compiler::setup_compiler().unwrap();
        let compile_options = CompilerOptions::default_for(source_code);
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

    #[test]
    fn test_assemblyscript_faulty_compilation() {
        let source_code = "this is not valid assemblyscript code".to_string();

        let compiler = Compiler::setup_compiler().unwrap();
        let compile_options = CompilerOptions::default_for(source_code);
        let outcome = compiler.compile(&compile_options);

        assert!(outcome.is_err_and(
            |CompilationError {
                 reason,
                 language: _,
             }| reason.contains("ERROR")
        ));
    }
}
