use anyhow::{anyhow, bail};
use cargo::{
    core::{
        compiler::{CompileKind, CompileTarget},
        Workspace,
    },
    ops::{compile, CompileOptions},
    GlobalContext,
};
use std::{fs, path::Path};

pub struct RustToWasmCompiler {
    gctx: GlobalContext,
}

impl RustToWasmCompiler {
    /// # Errors
    /// When creating a default global context fails
    pub fn new() -> anyhow::Result<Self> {
        let gctx = GlobalContext::default()?;
        Ok(Self { gctx })
    }

    /// # Errors
    /// Whenever compilation fails
    pub fn compile(&self, manifest_path: &Path) -> anyhow::Result<Vec<u8>> {
        // Create new workspace, inheriting from global context
        let workspace = Workspace::new(manifest_path, &self.gctx)?;

        // Start with default compile options, orienting for a build mode
        let mut compile_options =
            CompileOptions::new(&self.gctx, cargo::core::compiler::CompileMode::Build)?;

        // Set target to wasm32-unknown-unknown
        let wasm32_unknown_unkown = CompileTarget::new("wasm32-unknown-unknown")?;
        let compile_kind = CompileKind::Target(wasm32_unknown_unkown);
        compile_options.build_config.requested_kinds = vec![compile_kind];

        // Perform the compilation
        let compilation_result = compile(&workspace, &compile_options)?;

        if compilation_result.cdylibs.len() != 1 {
            bail!("Compilation result after compiling has no first value")
        }

        let Some(compiled_wasm) = compilation_result.cdylibs.first() else {
            bail!("Compilation result after compiling has no first value")
        };

        fs::read(compiled_wasm.path.clone()).map_err(|err| anyhow!(err))
    }
}
