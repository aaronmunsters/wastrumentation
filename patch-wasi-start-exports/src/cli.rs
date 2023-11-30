use anyhow::anyhow;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "wastrumentation",
    about = "A Wasm instrumentation platform",
    long_about = None
)]
pub struct Cli {
    /// The entry program, in Wasm
    #[arg(short, long)]
    pub entry: PathBuf,
    /// The other programs to include, in Wasm
    #[clap(short, long, num_args = 1.., value_delimiter = ' ')]
    pub joins: Vec<String>,
}

pub struct ParsedCli {
    pub entry: PathBuf,
    pub joins: Vec<ModuleDeclaration>,
}

impl TryFrom<Cli> for ParsedCli {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        let parses: Vec<ModuleDeclaration> = cli
            .joins
            .into_iter()
            .map(|module_path| {
                if let Some((name, path)) = module_path.split_once('=') {
                    Ok(ModuleDeclaration {
                        name: name.into(),
                        path: path.into(),
                    })
                } else {
                    Err(anyhow!(
                        "argument should be in the format 'name=path', got: {}",
                        module_path
                    ))
                }
            })
            .collect::<Result<Vec<ModuleDeclaration>, Self::Error>>()?;
        Ok(ParsedCli {
            entry: cli.entry,
            joins: parses,
        })
    }
}

pub struct ModuleDeclaration {
    pub name: String,
    pub path: String,
}
