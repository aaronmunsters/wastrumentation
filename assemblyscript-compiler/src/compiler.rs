use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use tempfile::{tempdir, TempDir};

use crate::options::CompilerOptions;

pub struct Compiler {
    working_dir: TempDir,
}

impl Compiler {
    /// # Errors
    /// When setup of a temporary directory fails.
    pub fn new() -> Result<Self> {
        let working_dir = tempdir().expect("Could not create temp dir");

        let custom_abort_path = &working_dir.path().join("custom_abort_source_file.ts");
        let mut custom_abort_file = File::create(custom_abort_path)
            .with_context(|| format!("Create custom abort file failed: {custom_abort_path:?}"))?;
        custom_abort_file
            .write_all(include_str!("./custom_abort_lib.ts").as_bytes())
            .with_context(|| format!("Write custom abort file failed: {custom_abort_path:?}"))?;

        Command::new("npm")
            .args(["init", "-y"])
            .current_dir(&working_dir)
            .output()
            .context("Npm init failed")?;

        Command::new("npm")
            .args(["install", "assemblyscript"])
            .current_dir(&working_dir)
            .output()
            .context("Npm install failed")?;

        Ok(Self { working_dir })
    }

    /// # Errors
    /// When the compilation failes.
    pub fn compile(&self, compiler_options: &CompilerOptions) -> Result<Vec<u8>> {
        let mut source_file = tempfile::Builder::new()
            .prefix("source_file")
            .suffix(".ts")
            .tempfile()
            .context("Could not create temp input file")?;

        source_file
            .write_all(compiler_options.source.as_bytes())
            .context("Could not write source code to temp imput file")?;
        source_file
            .flush()
            .context("Could not flush source code to temp imput file")?;

        let mut output_file = tempfile::Builder::new()
            .prefix("output_file")
            .suffix(".ts")
            .tempfile()
            .context("Could not create temp output file")?;

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
            .context("Could not execute compilation command")?;

        result.status.success().then_some(true).ok_or(anyhow!(
            "AssemblyScript compilation failed: {:?}",
            String::from_utf8_lossy(&result.stderr)
        ))?;

        drop(source_file);

        let mut result = Vec::new();
        output_file
            .read_to_end(&mut result)
            .context("Could not read result from compiled output")?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let compiler = Compiler::new().unwrap();
        let compiler_options =
            CompilerOptions::default_for("this is not valid assemblyscript code");

        println!("{}", compiler.compile(&compiler_options).unwrap_err());

        assert!(compiler
            .compile(&compiler_options)
            .unwrap_err()
            .to_string()
            .contains("ERROR"));
    }
}
