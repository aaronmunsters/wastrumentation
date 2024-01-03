use std::{
    fs::File,
    io::{Read, Write},
    process::Command,
};
use tempfile::{tempdir, NamedTempFile};

#[derive(Debug)]
pub struct CompileError(pub String);

#[derive(Debug)]
pub struct CompileResult {
    pub std_lib_module: Vec<u8>,
}

#[derive(Debug)]
pub struct CompileOptions {
    pub std_lib: String,
}

pub fn compile(compile_options: &CompileOptions) -> Result<CompileResult, CompileError> {
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

    let std_lib_file_path = working_dir.path().join("std_lib_file.ts");
    let mut std_lib_file =
        File::create(&std_lib_file_path).expect("Could not create temp input file");
    std_lib_file
        .write_all(compile_options.std_lib.as_bytes())
        .expect("Could not write std_lib to temp file");
    let std_lib_file_path = std_lib_file_path.to_string_lossy().to_string();

    let mut output_file = NamedTempFile::new().expect("Could not create temp output file");
    let output_file_path = output_file.path().to_string_lossy().to_string();

    let mut command_compile_lib = Command::new("bash");
    command_compile_lib.current_dir(&working_dir_path);

    let npx_command = format!(
        concat!(
            // Pass input file & output file to command
            "node ./node_modules/assemblyscript/bin/asc.js {std_lib_file_path} -o {output_file_path} ",
            // Pass wasi shim configuration to command
            "--config ./node_modules/@assemblyscript/wasi-shim/asconfig.json ",
            // Pas additional options to command
            "-O3 ",
            "--disable bulk-memory ",
            "--disable sign-extension ",
            "--disable nontrapping-f2i ",
            "--runtime minimal ",
            "--noExportMemory ",
        ),
        std_lib_file_path = &std_lib_file_path,
        output_file_path = &output_file_path,
    );

    command_compile_lib.args(["-c", &npx_command]);

    // Kick off command, i.e. merge
    let result = command_compile_lib
        .output()
        .expect("Could not execute compilation command");

    if !(result.stderr.is_empty() && result.stdout.is_empty()) {
        return Err(CompileError(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    };

    let mut result = Vec::new();
    output_file
        .read_to_end(&mut result)
        .expect("Could not read result from written output");
    Ok(CompileResult {
        std_lib_module: result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemblyscript_compilation() {
        let compile_options = CompileOptions {
            std_lib: String::from(
                r#"
        export function add_three(a: i32, b: i32, c: i32): i32 {
            return a + b + c;
        }
        "#,
            ),
        };

        let wasm_magic_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D];
        let header_actual = &compile(&compile_options).unwrap().std_lib_module[0..4];

        assert_eq!(header_actual, wasm_magic_bytes);
    }

    #[test]
    fn test_debug() {
        let compile_error = CompileError("what went wrong".into());

        let compile_result = CompileResult {
            std_lib_module: vec![],
        };

        let compile_options = CompileOptions {
            std_lib: "/* here would come code */".into(),
        };

        assert_eq!(
            format!("{compile_error:?} - {compile_result:?} - {compile_options:?}"),
            r#"CompileError("what went wrong") - CompileResult { std_lib_module: [] } - CompileOptions { std_lib: "/* here would come code */" }"#
        );
    }

    #[test]
    fn test_assemblyscript_faulty_compilation() {
        let compile_options = CompileOptions {
            std_lib: "this is not valid assemblyscript code".into(),
        };

        let CompileError(reason) = compile(&compile_options).unwrap_err();
        assert!(reason.contains("ERROR"));
    }
}
