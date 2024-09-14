use serde::Deserialize;
use std::path::PathBuf;
use wastrumentation_instr_lib::std_lib_compile::rust::Hook;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TestConfiguration {
    pub input_program: InputProgram,
    pub uninstrumented_assertion: UninstrumentedAssertion,
    pub instrumented_assertions: Vec<InstrumentedAssertion>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InputProgram {
    pub path: PathBuf,
    pub r#type: ProgramType,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum ProgramType {
    Wat,
    AssemblyScript,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UninstrumentedAssertion {
    pub input_entry_point: String,
    pub arguments: Vec<WasmValue>,
    pub results: Vec<WasmValue>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum InputProgramType {
    Wat,
    AssemblyScript,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstrumentedAssertion {
    pub analysis: Analysis,
    pub input_program_assertion: InputProgramAssertion,
    pub post_execution_assertions: Vec<PostExecutionAssertion>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Analysis {
    Wasp(PathBuf),
    Rust(AnalysisRust),
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRust {
    pub manifest: PathBuf,
    pub hooks: Vec<Hook>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum InputProgramAssertion {
    EqualToUninstrumented,
    DifferentReturnValue(Vec<WasmValue>),
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub enum WasmValue {
    I32(i32),
    F32(u32),
    I64(i64),
    F64(u64),
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum PostExecutionAssertion {
    CallYields(CallYields),
    GlobalValueEquals(GlobalValueEquals),
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CallYields {
    pub call: String,
    pub arguments: Vec<WasmValue>,
    pub results: Vec<WasmValue>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GlobalValueEquals {
    pub identifier: String,
    pub result: WasmValue,
}
