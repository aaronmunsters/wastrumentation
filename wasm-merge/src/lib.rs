use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub mod error;
use error::{Error, MergeFailReason};

/*
# DOCUMENTATION
#
#  unit | npx options used *     | reason why
#  -----|------------------------|----------------------------------
#  both | (e) --config wasi-shim | ensure host is Wasi, not JavaScript [3]
#  both | (e) --runtime minimal  | prevent default GC crash [2]
#  xor  | (e) --noExportMemory   | used memory is not relevant to the outside
#  xor  | (i) --ExportMemory     | memory must be exposed for WASI to work [1]
#
# * (e) = explicit, (i) = implicit
#
# SRC:
# [1] https://github.com/bytecodealliance/wasmtime/issues/4985
# [2] Not sure why, but binaryen merge crashes the default runtime with multi-memory
# [3] https://github.com/AssemblyScript/wasi-shim
#
# TODO: Shift to self-implemented merge
*/

#[derive(Debug)]
pub struct MergeOptions {
    pub no_validate: bool,
    pub rename_export_conflicts: bool,
    pub enable_multi_memory: bool,
    /// Optionally a primary module is declared.
    /// The primary module receives index 0, which
    /// may be important e.g. for the WASI interface
    /// that reads IO buffers from memory 0.
    pub primary: Option<InputModule>,
    pub input_modules: Vec<InputModule>,
}

#[derive(Debug)]
pub struct InputModule {
    pub module: Vec<u8>,
    pub namespace: String,
}

/// # Errors
/// When merging fails according to wasm-merge.
///
/// # Panics
/// When accessing resources are failing to be acquired.
pub fn merge(merge_options: &MergeOptions) -> Result<Vec<u8>, Error> {
    let MergeOptions {
        primary,
        input_modules,
        ..
    } = merge_options;
    let merges: Vec<(&InputModule, String, NamedTempFile)> = primary
        .as_ref()
        .map_or(vec![], |p| vec![p])
        .iter()
        .chain(input_modules.iter().collect::<Vec<&InputModule>>().iter())
        .map(|im @ InputModule { module, .. }| {
            let mut input_module =
                NamedTempFile::new().map_err(Error::TempInputFileCreationFailed)?;
            input_module
                .write_all(module)
                .map_err(Error::TempInputFileWriteFailed)?;
            let input_module_path = input_module.path().to_string_lossy().to_string();
            Ok((*im, input_module_path, input_module))
        })
        .collect::<Result<Vec<(&InputModule, String, NamedTempFile)>, Error>>()?;

    let merge_name_combinations = merges
        .iter()
        .map(|(InputModule { namespace, .. }, input_module_path, ..)| {
            format!("{input_module_path} {namespace}")
        })
        .collect::<Vec<String>>()
        .join(" ");

    let mut output_file = NamedTempFile::new().map_err(Error::TempOutputFileCreationFailed)?;
    let output_file_path = output_file.path().to_string_lossy().to_string();

    // FIXME: move this to separate file, splitting up functionality
    // FIXME: this implementation shares constructs with wastrumentation-instr-lib (code-dupe)

    let flag_no_validate = if merge_options.no_validate {
        " -n "
    } else {
        ""
    };

    let flag_rename_export_conflicts = if merge_options.rename_export_conflicts {
        " --rename-export-conflicts "
    } else {
        ""
    };

    let flag_enable_multi_memory = if merge_options.enable_multi_memory {
        " --enable-multimemory "
    } else {
        ""
    };

    let merge_command = format!(
        concat!(
            "wasm-merge",
            "{flag_no_validate}",
            "{flag_rename_export_conflicts}",
            "{flag_enable_multi_memory}",
            "{merge_name_combinations} -o {output_file_path}",
        ),
        flag_no_validate = flag_no_validate,
        flag_rename_export_conflicts = flag_rename_export_conflicts,
        flag_enable_multi_memory = flag_enable_multi_memory,
        merge_name_combinations = merge_name_combinations,
        output_file_path = output_file_path,
    );

    let mut command_merge = Command::new("bash");
    command_merge.args(["-c", &merge_command]);

    // Kick off command, i.e. merge
    let result = command_merge
        .output()
        .map_err(Error::MergeExecutionFailed)?;

    (result.stderr.is_empty() && result.stdout.is_empty())
        .then_some(true)
        .ok_or_else(|| {
            let std_err_string = String::from_utf8_lossy(&result.stderr).to_string();
            if std_err_string
                .contains("parse exception: control flow inputs are not supported yet ")
            {
                return Error::MergeExecutionFailedReason(MergeFailReason::ControlFlowInputs);
            }
            Error::MergeExecutionFailedReason(MergeFailReason::CompilerReason(std_err_string))
        })?;

    let mut result = Vec::new();
    output_file
        .read_to_end(&mut result)
        .map_err(Error::ReadFromOutputFileFailed)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    use wabt::wat2wasm;
    use wasmtime::*;

    const WAT_ODD: &str = r#"
    (module
        (import "even" "even" (func $even (param i32) (result i32)))
        (export "odd" (func $odd))
        (func $odd (param $0 i32) (result i32)
         local.get $0
         i32.eqz
         if
          i32.const 0
          return
         end
         local.get $0
         i32.const 1
         i32.sub
         call $even))"#;

    const WAT_EVEN: &str = r#"
    (module
        (import "odd" "odd" (func $odd (param i32) (result i32)))
        (export "even" (func $even))
        (func $even (param $0 i32) (result i32)
         local.get $0
         i32.eqz
         if
          i32.const 1
          return
         end
         local.get $0
         i32.const 1
         i32.sub
         call $odd))"#;

    #[test]
    fn test_merge() {
        let merge_options = MergeOptions {
            enable_multi_memory: false,
            no_validate: true,
            rename_export_conflicts: true,
            primary: None,
            input_modules: vec![
                InputModule {
                    module: wat2wasm(WAT_EVEN).unwrap(),
                    namespace: String::from("even"),
                },
                InputModule {
                    module: wat2wasm(WAT_ODD).unwrap(),
                    namespace: String::from("odd"),
                },
            ],
        };

        // Merge even & odd
        let merged_wasm = merge(&merge_options).unwrap();

        // Interpret even & odd
        let mut store = Store::<()>::default();
        let module = Module::from_binary(store.engine(), &merged_wasm).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        // Fetch `even` and `odd` export
        let even = instance
            .get_typed_func::<i32, i32>(&mut store, "even")
            .unwrap();

        let odd = instance
            .get_typed_func::<i32, i32>(&mut store, "odd")
            .unwrap();

        assert_eq!(even.call(&mut store, 12345).unwrap(), 0);
        assert_eq!(even.call(&mut store, 12346).unwrap(), 1);
        assert_eq!(odd.call(&mut store, 12345).unwrap(), 1);
        assert_eq!(odd.call(&mut store, 12346).unwrap(), 0);
    }

    #[test]
    fn test_merge_fail() {
        let merge_options = MergeOptions {
            enable_multi_memory: true,
            no_validate: false,
            rename_export_conflicts: false,
            primary: None,
            input_modules: vec![
                InputModule {
                    module: vec![99, 88, 77, 66],
                    namespace: String::from("foo"),
                },
                InputModule {
                    module: vec![99, 88, 77, 66],
                    namespace: String::from("bar"),
                },
            ],
        };

        let merge_error = merge(&merge_options).unwrap_err();
        assert!(merge_error.to_string().contains("Fatal"));
    }

    #[test]
    fn test_debug() {
        let merge_options = MergeOptions {
            no_validate: false,
            rename_export_conflicts: false,
            enable_multi_memory: false,
            primary: None,
            input_modules: vec![],
        };
        let input_module = InputModule {
            module: vec![],
            namespace: "namespace".into(),
        };

        assert_eq!(
            format!("{merge_options:?}"),
            concat!(
                "MergeOptions { ",
                /**/ "no_validate: false, ",
                /**/ "rename_export_conflicts: false, ",
                /**/ "enable_multi_memory: false, ",
                /**/ "primary: None, ",
                /**/ "input_modules: [] ",
                "}"
            )
        );

        assert_eq!(
            format!("{input_module:?}"),
            concat!(
                "InputModule { ",
                /**/ "module: [], ",
                /**/ "namespace: \"namespace\" ",
                "}"
            )
        );
    }
}
