use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TestConfiguration {
    pub input_program: InputProgram,
    pub uninstrumented_assertion: UninstrumentedAssertion,
    pub instrumented_assertions: Vec<InstrumentedAssertion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InputProgram {
    pub path: PathBuf,
    pub r#type: ProgramType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum ProgramType {
    Wat,
    AssemblyScript(AssemblyScript),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AssemblyScript {
    pub wasi_enabled: bool,
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
    pub wasi_enabled: bool,
    pub input_program_assertion: InputProgramAssertion,
    pub post_execution_assertions: Vec<PostExecutionAssertion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum InputProgramAssertion {
    EqualToUninstrumented,
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
