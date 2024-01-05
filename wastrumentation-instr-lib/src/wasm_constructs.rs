use std::fmt::Display;

#[derive(Hash, PartialEq, Eq)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
    Ref(RefType),
}

#[derive(Hash, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl WasmType {
    pub(crate) fn runtime_enum_value(&self) -> usize {
        match self {
            WasmType::I32 => 0,
            WasmType::F32 => 1,
            WasmType::I64 => 2,
            WasmType::F64 => 3,
            WasmType::Ref(RefType::FuncRef) => 4,
            WasmType::Ref(RefType::ExternRef) => 5,
        }
    }
}

impl Display for WasmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_string = match self {
            WasmType::I32 => "i32",
            WasmType::F32 => "f32",
            WasmType::I64 => "i64",
            WasmType::F64 => "f64",
            WasmType::Ref(RefType::FuncRef) => "FuncRef",
            WasmType::Ref(RefType::ExternRef) => "ExternRef",
        };
        write!(f, "{as_string}")
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct Signature {
    pub return_types: Vec<WasmType>,
    pub argument_types: Vec<WasmType>,
}
