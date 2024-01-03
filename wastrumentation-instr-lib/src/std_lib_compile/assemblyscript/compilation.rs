use std::{
    fs::File,
    io::{Read, Write},
    process::Command,
};

use tempfile::{tempdir, NamedTempFile};

use super::{
    compilation_result::{CompilationError, CompilationResult, WasmModule},
    compiler_options::{CompilationOutputType, CompilerOptions},
};

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

    let assemblyscript_source_file_path = working_dir.path().join("assemblyscript_source_file.ts");
    let mut assemblyscript_source_file =
        File::create(&assemblyscript_source_file_path).expect("Could not create temp input file");
    assemblyscript_source_file
        .write_all(compile_options.source_code.as_bytes())
        .expect("Could not write std_lib to temp file");
    let assemblyscript_source_file_path = assemblyscript_source_file_path
        .to_string_lossy()
        .to_string();

    let mut output_file = NamedTempFile::new().expect("Could not create temp output file");
    let output_file_path = output_file.path().to_string_lossy().to_string();

    let mut command_compile_lib = Command::new("bash");
    command_compile_lib.current_dir(&working_dir_path);

    let npx_command =
        compile_options.to_npx_command(assemblyscript_source_file_path, output_file_path);

    command_compile_lib.args(["-c", &npx_command]);

    // Kick off command, i.e. merge
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
    let result = match compile_options.compilation_output_type {
        CompilationOutputType::Text => {
            WasmModule::Text(String::from_utf8(result).expect("Could not parse webassembly text"))
        }
        CompilationOutputType::Binary => WasmModule::Binary(result),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::std_lib_compile::assemblyscript::{
        compilation_result::CompilationError,
        compiler_options::{
            CompilationOutputType, CompilerOptions, OptimizationStrategy, RuntimeStrategy,
        },
    };

    use super::compile;

    fn simple_compiler_option_for(
        source_code: String,
        compilation_output_type: CompilationOutputType,
    ) -> CompilerOptions {
        CompilerOptions {
            source_code,
            compilation_output_type,
            enable_bulk_memory: false,
            enable_export_memory: false,
            enable_nontrapping_f2i: false,
            enable_sign_extension: false,
            enable_wasi_shim: false,
            optimization_strategy: OptimizationStrategy::O3,
            runtime: RuntimeStrategy::Minimal,
        }
    }

    #[test]
    fn test_assemblyscript_compilation_binary() {
        let compile_options = simple_compiler_option_for(
            r#"
        export function add_three(a: i32, b: i32, c: i32): i32 {
            return a + b + c;
        }
        "#
            .into(),
            CompilationOutputType::Binary,
        );

        let wasm_module = compile(&compile_options).unwrap();
        let binary_module = wasm_module.unwrap_binary();

        let wasm_magic_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D];
        assert_eq!(&binary_module[0..4], wasm_magic_bytes);
    }

    #[test]
    fn test_assemblyscript_compilation_text() {
        let compile_options = simple_compiler_option_for(
            r#"
        export function add_three(a: i32, b: i32, c: i32): i32 {
            return a + b + c;
        }
        "#
            .into(),
            CompilationOutputType::Text,
        );

        let wasm_module = compile(&compile_options).unwrap();
        let binary_module = wasm_module.unwrap_text();
        assert!(binary_module.contains(r#"(export "add_three" "#));
    }

    // TODO: remove
    // #[test]
    // fn test_debug() {
    //     let compile_error = CompileError("what went wrong".into());

    //     let compile_result = CompileResult {
    //         std_lib_module: vec![],
    //     };

    //     let compile_options = CompileOptions {
    //         std_lib: "/* here would come code */".into(),
    //     };

    //     assert_eq!(
    //         format!("{compile_error:?} - {compile_result:?} - {compile_options:?}"),
    //         r#"CompileError("what went wrong") - CompileResult { std_lib_module: [] } - CompileOptions { std_lib: "/* here would come code */" }"#
    //     );
    // }

    #[test]
    fn test_assemblyscript_faulty_compilation() {
        let compiler_options = simple_compiler_option_for(
            "this is not valid assemblyscript code".into(),
            CompilationOutputType::Binary,
        );

        let CompilationError(reason) = compile(&compiler_options).unwrap_err();
        assert!(reason.contains("ERROR"));
    }
}
