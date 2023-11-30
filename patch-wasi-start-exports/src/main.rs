use clap::Parser;
use cli::{Cli, ParsedCli};
use patch_wasi_start_exports::{extend_start, rename_wasi_export, ModuleStart};
use wasabi_wasm::Module;

mod cli;

fn main() {
    let cli = ParsedCli::try_from(Cli::parse()).expect("Could not parse arguments");
    let mut relevant_joins = Vec::with_capacity(cli.joins.len());

    for join in cli.joins {
        let (mut module, _offsets, _parse_issue) =
            Module::from_file(&join.path).expect(&format!("Invalid wasm file: {}", &join.path));
        if let Some(name) = rename_wasi_export(&mut module, &join.name) {
            relevant_joins.push(ModuleStart {
                module_name: join.name,
                start_function: name,
            });
            module
                .to_file(&join.path)
                .expect(&format!("Could not save _start rewrite for {}", &join.path));
        }
    }

    let (mut module, _offsets, _parse_issue) = Module::from_file(&cli.entry).expect(&format!(
        "Invalid wasm file: {}",
        &cli.entry.to_string_lossy()
    ));

    extend_start(&mut module, relevant_joins).expect("Failed to extend entry _start function.");
    module.to_file(&cli.entry).expect(&format!(
        "Could not save _start rewrite for {}",
        &cli.entry.to_string_lossy()
    ));
}
