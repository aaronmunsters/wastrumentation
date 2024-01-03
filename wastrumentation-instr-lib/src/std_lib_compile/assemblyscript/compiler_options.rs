pub struct CompilerOptions {
    pub source_code: String,
    pub compilation_output_type: CompilationOutputType,
    pub optimization_strategy: OptimizationStrategy,
    pub enable_bulk_memory: bool,
    pub enable_sign_extension: bool,
    pub enable_nontrapping_f2i: bool,
    pub enable_export_memory: bool,
    pub enable_wasi_shim: bool,
    pub runtime: RuntimeStrategy,
}

pub enum OptimizationStrategy {
    O1,
    O2,
    O3,
}

pub enum RuntimeStrategy {
    Minimal,
}

pub enum CompilationOutputType {
    Text,
    Binary,
}

impl CompilerOptions {
    pub(crate) fn to_npx_command(&self, source_path: String, output_path: String) -> String {
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

        let flag_output_type = match self.compilation_output_type {
            CompilationOutputType::Binary => " -o ",
            CompilationOutputType::Text => " --textFile ",
        };

        format!(
            concat!(
                // Pass input file & output file to command
                "node ./node_modules/assemblyscript/bin/asc.js {source_path} {flag_output_type} {output_path} ",
                // Pass wasi shim configuration to command
                "{flag_wasi}",
                // Pas additional options to command
                "{flag_optimization}",
                "{flag_bulk_memory}",
                "{flag_sign_extension}",
                "{flag_non_trapping_f2i}",
                "{flag_runtime}",
                "{flag_export_memory}",
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
            flag_output_type = flag_output_type,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_npx() {
        let mut options = CompilerOptions {
            source_code: "source".into(),
            compilation_output_type: CompilationOutputType::Binary,
            optimization_strategy: OptimizationStrategy::O1,
            enable_bulk_memory: true,
            enable_sign_extension: true,
            enable_nontrapping_f2i: true,
            enable_export_memory: true,
            enable_wasi_shim: true,
            runtime: super::RuntimeStrategy::Minimal,
        };

        assert_eq!(
            options.to_npx_command("path/to/source".into(), "path/to/output".into()),
            concat!(
                "node ./node_modules/assemblyscript/bin/asc.js path/to/source  ",
                "-o  path/to/output ",
                "--config ./node_modules/@assemblyscript/wasi-shim/asconfig.json ",
                "-O1 --runtime minimal ",
            )
        );

        options = CompilerOptions {
            source_code: "source".into(),
            compilation_output_type: CompilationOutputType::Text,
            optimization_strategy: OptimizationStrategy::O2,
            enable_bulk_memory: false,
            enable_sign_extension: false,
            enable_nontrapping_f2i: false,
            enable_export_memory: false,
            enable_wasi_shim: false,
            runtime: super::RuntimeStrategy::Minimal,
        };

        assert_eq!(
            options.to_npx_command("path/to/source".into(), "path/to/output".into()),
            concat!(
                "node ./node_modules/assemblyscript/bin/asc.js path/to/source  ",
                "--textFile  path/to/output ",
                "-O2 ",
                "--disable bulk-memory ",
                "--disable sign-extension ",
                "--disable nontrapping-f2i ",
                "--runtime minimal ",
                "--noExportMemory ",
            )
        );
    }
}
