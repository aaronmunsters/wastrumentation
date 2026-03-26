use std::collections::HashMap;
use wasabi_wasm::{Function, FunctionType, Global, Idx, LocalOp, Memory, Module, Val, ValType};

fn fnv1a(s: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for &byte in s.as_bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

trait Exportable {
    fn get_name(&self) -> Option<&String>;
}
impl Exportable for Function {
    fn get_name(&self) -> Option<&String> {
        self.name.as_ref().or_else(|| self.export.first())
    }
}
impl Exportable for Global {
    fn get_name(&self) -> Option<&String> {
        self.export.first()
    }
}
impl Exportable for Memory {
    fn get_name(&self) -> Option<&String> {
        self.export.first()
    }
}
fn extract_exported_names<'a, T, I>(
    collection: impl Iterator<Item = (Idx<I>, &'a T)>,
) -> HashMap<String, u32>
where
    T: Exportable + 'a,
{
    let mut map: HashMap<String, u32> = HashMap::new();
    for (index, item) in collection {
        if let Some(name) = item.get_name() {
            map.insert(name.clone(), index.to_u32());
        }
    }
    map
}

fn build_lookup_function(
    module: &mut Module,
    name_to_idx: &HashMap<String, u32>,
    export_name: &str,
) {
    use wasabi_wasm::Instr::{Binary, Const, End, Return};

    let function_type = FunctionType::new(&[ValType::I32], &[ValType::I32]);
    let mut body: Vec<wasabi_wasm::Instr> = Vec::new();

    for (name, &idx) in name_to_idx {
        let hash = fnv1a(name) as i32;
        body.push(wasabi_wasm::Instr::Block(FunctionType::empty()));
        body.push(wasabi_wasm::Instr::Local(LocalOp::Get, 0_u32.into()));
        body.push(Const(Val::I32(hash)));
        body.push(Binary(wasabi_wasm::BinaryOp::I32Ne));
        body.push(wasabi_wasm::Instr::BrIf(0_u32.into()));
        body.push(Const(Val::I32(idx as i32)));
        body.push(Return);
        body.push(End);
    }

    body.push(Const(Val::I32(-1)));
    body.push(End);

    let fn_idx = module.add_function(function_type, vec![], body);
    module
        .function_mut(fn_idx)
        .export
        .push(export_name.to_string());
}

pub fn inject_function_name_mapping(module: &mut Module) {
    if module.memories.is_empty() {
        return;
    }

    let functions = extract_exported_names(module.functions());
    let globals = extract_exported_names(module.globals());
    let memories = extract_exported_names(module.memories());

    build_lookup_function(module, &functions, "get_function_idx_by_name");
    build_lookup_function(module, &globals, "get_global_idx_by_name");
    build_lookup_function(module, &memories, "get_memory_idx_by_name");
}
