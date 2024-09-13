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

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            WasmType::I32 => "i32",
            WasmType::F32 => "f32",
            WasmType::I64 => "i64",
            WasmType::F64 => "f64",
            WasmType::Ref(RefType::FuncRef) => "ref_func",
            WasmType::Ref(RefType::ExternRef) => "ref_extern",
        }
    }
}

impl Display for WasmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct Signature {
    pub return_types: Vec<WasmType>,
    pub argument_types: Vec<WasmType>,
}

impl Signature {
    pub(crate) fn is_empty(&self) -> bool {
        self.argument_types.is_empty() && self.return_types.is_empty()
    }
}

impl Signature {
    pub(crate) fn generate_name(&self, name: &str) -> String {
        let Signature {
            return_types,
            argument_types,
        } = self;
        let rets = vec!["ret"]
            .into_iter()
            .chain(return_types.iter().map(|ft| ft.as_str()))
            .collect::<Vec<&str>>()
            .join("_");
        let args: String = vec!["arg"]
            .into_iter()
            .chain(argument_types.iter().map(|ft| ft.as_str()))
            .collect::<Vec<&str>>()
            .join("_");
        format!("{name}_{rets}_{args}")
    }

    pub fn generate_allocate_values_buffer_name(&self) -> String {
        self.generate_name("allocate")
    }

    pub fn generate_allocate_types_buffer_name(&self) -> String {
        self.generate_name("allocate_types")
    }

    pub fn generate_free_values_buffer_name(&self) -> String {
        self.generate_name("free_values")
    }

    pub fn generate_free_types_buffer_name(&self) -> String {
        self.generate_name("free_types")
    }

    fn generate_indexed_name(
        &self,
        name: &str,
        signature_side: SignatureSide,
        index: usize,
    ) -> String {
        let prefix = format!("{name}_{signature_side}{index}");
        self.generate_name(&prefix)
    }

    pub fn generate_load_name(&self, signature_side: SignatureSide, index: usize) -> String {
        self.generate_indexed_name("load", signature_side, index)
    }

    pub fn generate_store_name(&self, signature_side: SignatureSide, index: usize) -> String {
        self.generate_indexed_name("store", signature_side, index)
    }

    // FIXME: this function is not used.
    // The 'store all args' functionality is implemented in allocate_values.
    // Should it be removed from here?
    pub fn generate_store_args_name(&self) -> String {
        self.generate_name("store_args")
    }

    pub fn generate_store_rets_name(&self) -> String {
        self.generate_name("store_rets")
    }
}

#[derive(Clone, Copy)]
pub enum SignatureSide {
    Return,
    Argument,
}

impl Display for SignatureSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Return => "ret",
            Self::Argument => "arg",
        };
        write!(f, "{str}")
    }
}
