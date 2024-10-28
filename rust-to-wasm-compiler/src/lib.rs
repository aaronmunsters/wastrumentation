use cargo::core::compiler::{CompileKind, CompileTarget};
use cargo::core::Workspace;
use cargo::ops::{compile, CompileOptions};
use cargo::GlobalContext;

use dirs::cache_dir;
use sha256::digest;
use tempfile::TempDir;

use std::fs::{self, create_dir, File};
use std::path::PathBuf;
use std::{io::Write, path::Path};

mod error;
pub use error::{CompilationError, CompilerSetupError};

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
    pub fn new() -> Result<Self, CompilerSetupError> {
        let gctx = GlobalContext::default().map_err(CompilerSetupError::from)?;
        Ok(Self { gctx })
    }

    /// # Errors
    /// Whenever compilation fails
    pub fn compile(
        &self,
        wasi_support: WasiSupport,
        manifest_path: &Path,
        profile: Profile,
    ) -> Result<Vec<u8>, CompilationError> {
        // Create new workspace, inheriting from global context
        let workspace = Workspace::new(manifest_path, &self.gctx)
            .map_err(Into::into)
            .map_err(CompilationError::WorkspaceCreationFailed)?;

        // Start with default compile options, orienting for a build mode
        let mut compile_options =
            CompileOptions::new(&self.gctx, cargo::core::compiler::CompileMode::Build)
                .map_err(Into::into)
                .map_err(CompilationError::CompileOptionsCreationFailed)?;

        // Set target to wasm32-unknown-unknown
        let target = CompileTarget::new(match wasi_support {
            WasiSupport::Enabled => "wasm32-wasip1",
            WasiSupport::Disabled => "wasm32-unknown-unknown",
        })
        .map_err(Into::into)
        .map_err(CompilationError::TargetCreationFailed)?;

        compile_options.target_rustc_args = Some(vec![
            "-C".into(),
            "target-feature=+nontrapping-fptoint".into(),
        ]);

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
        let compilation_result = compile(&workspace, &compile_options)
            .map_err(Into::into)
            .map_err(CompilationError::CompilationFailed)?;

        compilation_result
            .cdylibs
            .len()
            .le(&1)
            .then_some(())
            .ok_or(CompilationError::ExpectedSingleCompilationResult)?;

        let compiled_wasm = compilation_result
            .cdylibs
            .first()
            .ok_or(CompilationError::SingleCompilationResultNotPresent)?;

        fs::read(compiled_wasm.path.clone())
            .map_err(CompilationError::CompilationResultOutputNotAccessible)
    }

    /// # Errors
    /// Whenever creation of files or compilation fails
    pub fn compile_source(
        &self,
        wasi_support: WasiSupport,
        manifest: &str,
        lib: &str,
        profile: Profile,
    ) -> Result<Vec<u8>, CompilationError> {
        let cache_dir: PathBuf = cache_dir().map(Ok).unwrap_or_else(|| {
            tempfile::TempDir::new()
                .map(TempDir::into_path)
                .map_err(CompilationError::CreateTempWorkingDir)
        })?;

        let working_dir = cache_dir
            .join("rust_to_wasm_compiler")
            .join(digest(manifest))
            .join(digest(lib));

        dbg!(&working_dir);

        std::fs::create_dir_all(&working_dir).map_err(CompilationError::CreateTempWorkingDir)?;

        // temp/Cargo.toml
        let manifest_path = working_dir.join("Cargo.toml");
        // temp/src/lib.rs
        let src_path = working_dir.join("src");

        match (manifest_path.exists(), src_path.exists()) {
            (false, false) => {
                let mut manifest_file = File::create_new(&manifest_path)
                    .map_err(CompilationError::CreateTempCargoManifest)?;
                manifest_file
                    .write_all(manifest.as_bytes())
                    .map_err(CompilationError::WriteTempCargoManifest)?;

                create_dir(&src_path).map_err(CompilationError::CreateTempSourceDirectory)?;
                let mut lib_file = File::create_new(src_path.join("lib.rs"))
                    .map_err(CompilationError::CreateTempLibraryFile)?;
                lib_file
                    .write_all(lib.as_bytes())
                    .map_err(CompilationError::WriteTempLibraryFile)?;
            }
            (true, true) => (),
            _ => return Err(CompilationError::CacheFilesTamperedWith(working_dir)),
        }

        self.compile(wasi_support, &manifest_path, profile)
    }
}
