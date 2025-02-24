use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::Instant;

use csv::WriterBuilder;
use wasmtime::{Config, Engine, Instance, Module, Store};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The instrumentation platform
    #[arg(short, long)]
    platform: String,

    /// The dynamic analysis
    #[arg(short, long)]
    analysis: String,

    /// The input program
    #[arg(short, long)]
    input_program: String,

    /// The input program path
    #[arg(short, long)]
    input_program_path: PathBuf,

    /// Number of runs
    #[arg(short, long)]
    runs: usize,
}

#[derive(Debug, serde::Serialize)]
struct Record {
    runtime: String,
    platform: String,
    analysis: String,
    input_program: String,
    completion_time: u128,
    time_unit: String,
    timeout: bool,
    timeout_amount: u128,
}

fn main() {
    let Args {
        ref platform,
        ref analysis,
        ref input_program,
        input_program_path: program_path,
        runs,
    } = Args::parse();

    // Setup Engine
    let mut config = Config::default();
    config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);

    // Setup CSV to report results
    // Open the CSV file, create a CSV writer with headers
    let file = OpenOptions::new()
        .create(true) // Create the file if it doesn't exist
        .append(true) // Open in append mode
        .write(true) // Open for writing
        .open(Path::new("results.csv"))
        .unwrap();

    let mut writer = WriterBuilder::new()
        .has_headers(false) // Do not include headers
        .from_writer(file);

    for _run in 0..runs {
        let engine = Engine::new(&config).unwrap();
        let module = Module::from_file(&engine, &program_path).unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        println!(
            "Running on {platform:?} (analysis: {analysis:?}) benchmark for {input_program:?}!"
        );
        let start_function = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .unwrap();

        let now = Instant::now();
        start_function.call(&mut store, ()).unwrap();
        let elapsed_during_benchmark = now.elapsed();

        println!("{elapsed_during_benchmark:?}");

        let record: Record = Record {
            runtime: "Wasmtime".into(),
            platform: platform.to_string(),
            analysis: analysis.to_string(),
            input_program: input_program.to_string(),
            completion_time: elapsed_during_benchmark.as_nanos(),
            time_unit: "ns".into(),
            timeout: false,
            timeout_amount: 0,
        };

        writer.serialize(record).unwrap();
        writer.flush().unwrap();
    }
}
