use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// FIXME: change to enum in which `wasi_enabled` is colocated with `InputProgramType`'s where wasi is an option

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TestConfiguration {
    pub input_program: PathBuf,
    pub input_program_type: InputProgramType,
    pub wasi_enabled: bool,
    pub uninstrumented_assertion: UninstrumentedAssertion,
    pub instrumented_assertions: Vec<InstrumentedAssertion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UninstrumentedAssertion {
    pub input_entry_point: String,
    pub arguments: Vec<WasmValue>,
    pub results: Vec<WasmValue>,
    pub post_execution_assertions: Vec<InstrumentedAssertion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum InputProgramType {
    Wat,
    AssemblyScript,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstrumentedAssertion {
    pub analysis: PathBuf,
    pub uninstrumented_assertion: UninstrumentedInstrumentedAssertion,
    pub post_execution_assertions: Vec<PostExecutionAssertion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum UninstrumentedInstrumentedAssertion {
    EqualToUninstrumentedAssertion,
    DifferentReturnValue(Vec<WasmValue>),
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
pub enum PostExecutionAssertion {
    CallYields(CallYields),
    GlobalValueEquals(GlobalValueEquals),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CallYields {
    pub call: String,
    pub arguments: Vec<WasmValue>,
    pub results: Vec<WasmValue>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GlobalValueEquals {
    pub identifier: String,
    pub result: WasmValue,
}
