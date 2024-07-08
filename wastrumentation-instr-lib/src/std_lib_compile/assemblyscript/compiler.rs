use super::compilation_result::{CompilationError, CompilationResult};
use super::compiler_options::CompilerOptions;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

use tempfile::{tempdir, TempDir};

pub struct Compiler {
    working_dir: TempDir,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let working_dir = tempdir().expect("Could not create temp dir");

        let custom_abort_source_file_path = working_dir.path().join("custom_abort_source_file.ts");
        let mut custom_abort_source_file =
            File::create(custom_abort_source_file_path).expect("Could not create temp input file");
        let custom_abort_lib = include_str!("./custom_abort_lib.ts");
        custom_abort_source_file
            .write_all(custom_abort_lib.as_bytes())
            .expect("Could not write std_lib to temp file");

        Command::new("npm")
            .args(["init", "-y"])
            .current_dir(&working_dir)
            .output()
            .expect("Npm init failed");

        Command::new("npm")
            .args(["install", "assemblyscript"])
            .current_dir(&working_dir)
            .output()
            .expect("Npm install failed");

        Compiler { working_dir }
    }

    /// # Errors
    /// When the compilation failes.
    ///
    /// # Panics
    /// When system resources such as files cannot be acquired.
    pub fn compile(&self, compile_options: &CompilerOptions) -> CompilationResult {
        let mut source_file = tempfile::Builder::new()
            .prefix("source_file")
            .suffix(".ts")
            .tempfile()
            .expect("Could not create temp input file");
        source_file
            .write_all(compile_options.source_code.as_bytes())
            .expect("Could not write source code to temp file");
        source_file
            .flush()
            .expect("Could not write source code to temp file");

        let mut output_file = tempfile::Builder::new()
            .prefix("output_file")
            .suffix(".ts")
            .tempfile()
            .expect("Could not create temp output file");

        let source_file_path = source_file.path().to_string_lossy().to_string();
        let output_file_path = output_file.path().to_string_lossy().to_string();
        let npx_command = compile_options.to_npx_command(&source_file_path, &output_file_path);

        let mut command_compile_lib = Command::new("bash");
        command_compile_lib
            .args(["-c", &npx_command])
            .current_dir(&self.working_dir);

        // Kick off command, i.e. compile
        let result = command_compile_lib
            .output()
            .expect("Could not execute compilation command");

        if !(result.stderr.is_empty() && result.stdout.is_empty()) {
            return Err(CompilationError(
                String::from_utf8_lossy(&result.stderr).to_string(),
            ));
        };

        drop(source_file);

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
    use std::collections::HashMap;
    use wasmtime::{Engine, Instance, Module, Store};

    #[test]
    fn test_assemblyscript_compilation_binary() {
        let source_code = r#"
        export function add_three(a: i32, b: i32, c: i32): i32 {
            return a + b + c;
        }
        "#
        .into();

        let compile_options = CompilerOptions {
            source_code,
            ..Default::default()
        };

        let compiler = Compiler::new();
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
        .into();

        let mut compile_options = CompilerOptions {
            source_code,
            ..Default::default()
        };

        // TODO: I code-dupe this hashmap in a lot of places ... it's only purpose seems to abort?
        compile_options.flag_use = Some(HashMap::from_iter(vec![(
            "abort".into(),
            "custom_abort".into(),
        )]));

        let compiler = Compiler::new();
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
        let compiler_options = CompilerOptions {
            source_code: "this is not valid assemblyscript code".into(),
            ..Default::default()
        };

        let compiler = Compiler::new();
        let outcome = compiler.compile(&compiler_options);
        assert!(outcome.is_err_and(|CompilationError(reason)| reason.contains("ERROR")));
    }
}
