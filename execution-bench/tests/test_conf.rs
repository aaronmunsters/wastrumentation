use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TestConfiguration {
    pub input_program: PathBuf,
    pub input_entry_point: String,
    pub arguments: Vec<WasmValue>,
    pub results: Vec<WasmValue>,
    pub wasi_enabled: bool,
    pub instrumentation_configurations: Vec<InstrumentationConfiguration>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstrumentationConfiguration {
    pub analysis: PathBuf,
    pub instrumentation_results: Vec<InstrumentationResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub enum WasmValue {
    I32(i32),
    F32(u32),
    I64(i64),
    F64(u64),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum InstrumentationResult {
    CallYields(CallYields),
    GlobalValueEquals(GlobalValueEquals),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CallYields {
    pub call: String,
    pub arguments: Vec<WasmValue>,
    pub result: Vec<WasmValue>, //  TODO: rename to results
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GlobalValueEquals {
    pub identifier: String,
    pub result: WasmValue,
}
