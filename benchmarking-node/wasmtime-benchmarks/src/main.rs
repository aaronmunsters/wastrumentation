use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::Instant;

use csv::Writer;
use wasmtime::{Config, Engine, Instance, Module, Store};

#[derive(Debug, serde::Serialize)]
struct Record {
    runtime: String,
    platform: String,
    analysis: Option<String>,
    input_program: String,
    completion_time: u128,
    time_unit: String,
}

static RUNS: u32 = 30;
static INPUT_PROGRAMS: &'static [&'static str] = &[
    // "commanderkeen",   // CRASH ON Apply Silicon M2
    // "sandspiel",       // CRASH ON Apply Silicon M2
    "jqkungfu",        // 12666 ns
    "game-of-life",    // 15417 ns
    "rtexpacker",      // 18166 ns
    "factorial",       // 23500 ns
    "rtexviewer",      // 20750 ns
    "ffmpeg",          // 31084 ns
    "figma-startpage", // 50334 ns
    "parquet",         // 90250 ns
    "hydro",           // 94083 ns
    "pacalc",          // 258750 ns
    "riconpacker",     // 434000 ns
    "sqlgui",          // 649125 ns
    "jsc",             // 2401333 ns
    "boa",             // 3646083 ns
    "guiicons",        // 17659125 ns
    "rguistyler",      // 16036084 ns
    "rguilayout",      // 18067291 ns
    "rfxgen",          // 18035166 ns
    "bullet",          // 26375500 ns
    "pathfinding",     // 386305041 ns
    "funky-kart",      // 39554500 ns
    "fib",             // 2386120708 ns
    "multiplyInt",     // 2683662583 ns
    "mandelbrot",      // 3032975916 ns
    "multiplyDouble",  // 7099443958 ns
];

fn main() {
    // Setup Engine
    let mut config = Config::default();
    config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);
    let engine = Engine::new(&config).unwrap();

    // Setup CSV to report results
    let mut writer = csv::Writer::from_path(Path::new("results.csv")).unwrap();

    // Setup benchmarks
    let benchmarks_containing_directory = PathBuf::from(".")
        .join("..")
        .join("wasmr3-python")
        .join("working-dir")
        .join("wasm-r3");

    let baseline = benchmarks_containing_directory.join("benchmarks");
    let forward_wastrumentation = benchmarks_containing_directory
        .join("benchmarks_wastrumentation")
        .join("forward");

    println!("{baseline:?}");
    println!("{forward_wastrumentation:?}");

    for (directory, platform, analysis) in [
        (baseline, "uninstrumented", None),
        (forward_wastrumentation, "Wastrumentation", Some("forward")),
    ] {
        for benchmark_name in INPUT_PROGRAMS {
            let mut benchmark_file_name = benchmark_name.to_string();
            benchmark_file_name.push_str(".wasm");

            let benchmark_path = directory.join(benchmark_name).join(benchmark_file_name);

            // assert benchmark path is valid
            assert!(fs::exists(&benchmark_path).unwrap());

            run_benchmark(
                platform,
                benchmark_name,
                &analysis,
                &engine,
                &benchmark_path,
                &mut writer,
            );
        }
    }
}

fn run_benchmark(
    platform: &str,
    program: &str,
    analysis: &Option<&str>,
    engine: &Engine,
    benchmark_path: &PathBuf,
    writer: &mut Writer<File>,
) {
    let module = Module::from_file(&engine, benchmark_path).unwrap();
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();
    println!("Running on {platform:?} benchmark for {benchmark_path:?}!");
    let start_function = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .unwrap();

    for _ in 0..RUNS {
        let now = Instant::now();
        start_function.call(&mut store, ()).unwrap();
        let elapsed_during_benchmark = now.elapsed();

        println!("{elapsed_during_benchmark:?}");

        let record: Record = Record {
            runtime: "Wasmtime".into(),
            platform: platform.into(),
            analysis: analysis.map(|v| v.to_string()).clone(),
            input_program: program.into(),
            completion_time: elapsed_during_benchmark.as_nanos(),
            time_unit: "ns".into(),
        };

        writer.serialize(record).unwrap();
        writer.flush().unwrap();
    }
}
