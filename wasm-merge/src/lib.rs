//! Wasm-Merge, allows one to invoke the 'wasm-merge'
//! that is provided by binaryen from Rust.
//!
//! It does so by calling out to `wasm-merge` on the
//! machine CLI. As such, it could be made more
//! efficient using FFI.
#![deny(clippy::pedantic)]

pub mod error;
pub mod options;

use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

#[cfg(test)]
mod tests;

use error::Error;
use options::AsOption;

#[derive(Debug)]
pub struct InputModule<'a> {
    pub module: &'a [u8],
    pub namespace: String,
}

macro_rules! as_bash_args {
    ($self:ident, $($field:ident),*) => {
        format!(
            concat!(
                $(concat!(" {", stringify!($field), "} ")),*
            ),
            $($field = $self.$field.as_option()),*
        )
    };
}

#[derive(Debug, Default)]
pub struct MergeOptions<'a> {
    /// Optionally a primary module is declared.
    /// The primary module receives index 0, which
    /// may be important e.g. for the WASI interface
    /// that reads IO buffers from memory 0.
    pub primary: Option<InputModule<'a>>,
    pub input_modules: Vec<InputModule<'a>>,

    // Options:
    pub no_validation: options::NoValidate,
    pub rename_export_conflicts: options::RenameExportConflicts,

    // Features:
    pub bulk_memory: options::BulkMemory,
    pub bulk_memory_opt: options::BulkMemoryOpt,
    pub call_indirect_overlong: options::CallIndirectOverlong,
    pub extended_const: options::ExtendedConst,
    pub exception_handling: options::ExceptionHandling,
    pub fp16: options::Fp16,
    pub gc: options::Gc,
    pub memory64: options::Memory64,
    pub multimemory: options::Multimemory,
    pub multivalue: options::Multivalue,
    pub mutable_globals: options::MutableGlobals,
    pub nontrapping_float_to_int: options::NontrappingFloatToInt,
    pub reference_types: options::ReferenceTypes,
    pub relaxed_simd: options::RelaxedSimd,
    pub shared_everything: options::SharedEverything,
    pub sign_ext: options::SignExt,
    pub simd: options::Simd,
    pub strings: options::Strings,
    pub tail_call: options::TailCall,
    pub threads: options::Threads,
    pub typed_continuations: options::TypedContinuations,
}

impl MergeOptions<'_> {
    /// # Errors
    /// When merging fails according to wasm-merge.
    ///
    /// # Panics
    /// When accessing resources are failing to be acquired.
    #[inline]
    pub fn merge(&self) -> Result<Vec<u8>, Error> {
        let MergeOptions {
            primary,
            input_modules,
            ..
        } = self;

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

        let merge_command = format!(
            concat!(
                "wasm-merge",
                " {bash_arguments} ",
                "{merge_name_combinations} -o {output_file_path}",
            ),
            bash_arguments = self.as_bash_arguments(),
            merge_name_combinations = merge_name_combinations,
            output_file_path = output_file_path,
        );

        // FIXME: move this to separate file, splitting up functionality
        // FIXME: this implementation shares constructs with wastrumentation-instr-lib (code-dupe)

        let mut command_merge = Command::new("bash");
        command_merge.args(["-c", &merge_command]);

        // Kick off command, i.e. merge
        let command_output = command_merge
            .output()
            .map_err(Error::MergeExecutionFailed)?;

        if !command_output.stderr.is_empty() {
            let std_err_string = String::from_utf8_lossy(&command_output.stderr).to_string();
            return Err(Error::MergeExecutionFailedReason(std_err_string));
        }

        let mut result = Vec::new();
        output_file
            .read_to_end(&mut result)
            .map_err(Error::ReadFromOutputFileFailed)?;

        Ok(result)
    }

    fn as_bash_arguments(&self) -> String {
        as_bash_args!(
            self,
            rename_export_conflicts,
            sign_ext,
            threads,
            mutable_globals,
            nontrapping_float_to_int,
            simd,
            bulk_memory,
            bulk_memory_opt,
            call_indirect_overlong,
            exception_handling,
            tail_call,
            reference_types,
            multivalue,
            gc,
            memory64,
            relaxed_simd,
            extended_const,
            strings,
            multimemory,
            typed_continuations,
            shared_everything,
            fp16
        )
    }
}
