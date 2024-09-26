use anyhow::{anyhow, bail};
use cargo::{
    core::{
        compiler::{CompileKind, CompileTarget},
        Workspace,
    },
    ops::{compile, CompileOptions},
    GlobalContext,
};
use std::{
    fs::{self, create_dir, File},
    io::Write,
    path::Path,
};

pub struct RustToWasmCompiler {
    gctx: GlobalContext,
}

#[derive(Clone, Copy, Debug)]
pub enum Profile {
    Release,
    Dev,
    Test,
    Bench,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WasiSupport {
    Enabled,
    Disabled,
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
    pub fn compile(
        &self,
        wasi_support: WasiSupport,
        manifest_path: &Path,
        profile: Profile,
    ) -> anyhow::Result<Vec<u8>> {
        // Create new workspace, inheriting from global context
        let workspace = Workspace::new(manifest_path, &self.gctx)?;

        // Start with default compile options, orienting for a build mode
        let mut compile_options =
            CompileOptions::new(&self.gctx, cargo::core::compiler::CompileMode::Build)?;

        // Set target to wasm32-unknown-unknown
        let target = CompileTarget::new(match wasi_support {
            WasiSupport::Enabled => "wasm32-wasip1",
            WasiSupport::Disabled => "wasm32-unknown-unknown",
        })?;
        let compile_kind = CompileKind::Target(target);
        compile_options.build_config.requested_kinds = vec![compile_kind];
        compile_options.build_config.requested_profile = match profile {
            Profile::Release => "release",
            Profile::Dev => "dev",
            Profile::Test => "test",
            Profile::Bench => "bench",
        }
        .into();

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

    /// # Errors
    /// Whenever creation of files or compilation fails
    pub fn compile_source(
        &self,
        wasi_support: WasiSupport,
        manifest: &str,
        lib: &str,
        profile: Profile,
    ) -> anyhow::Result<Vec<u8>> {
        // -/
        let working_dir = tempfile::TempDir::new()?;

        // -/Cargo.toml
        let manifest_path = working_dir.path().join("Cargo.toml");
        let mut manifest_file = File::create_new(&manifest_path)?;
        manifest_file.write_all(manifest.as_bytes())?;

        // -/src/lib.rs
        let src_path = working_dir.path().join("src");
        create_dir(&src_path)?;
        let mut lib_file = File::create_new(src_path.join("lib.rs"))?;
        lib_file.write_all(lib.as_bytes())?;

        self.compile(wasi_support, &manifest_path, profile)
    }
}
