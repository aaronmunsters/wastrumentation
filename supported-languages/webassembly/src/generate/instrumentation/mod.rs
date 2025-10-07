use indoc::{formatdoc, indoc};
use std::collections::HashSet;
use wastrumentation::compiler::{LibGeneratable, Library};
use wastrumentation::wasm_constructs::{Signature, WasmType};

use crate::compile::{WebAssembly, options::WebAssemblySource};

impl LibGeneratable for WebAssembly {
    fn generate_lib(signatures: &[Signature]) -> Library<Self> {
        Library {
            content: WebAssemblySource::Wat(generate_lib(signatures)),
            language: std::marker::PhantomData,
        }
    }
}

trait AsWasmType {
    fn wat_type(&self) -> &'static str;
}

impl AsWasmType for WasmType {
    fn wat_type(&self) -> &'static str {
        match self {
            WasmType::I32 => "i32",
            WasmType::F32 => "f32",
            WasmType::I64 => "i64",
            WasmType::F64 => "f64",
            WasmType::Ref(wastrumentation::wasm_constructs::RefType::FuncRef) => "funcref",
            WasmType::Ref(wastrumentation::wasm_constructs::RefType::ExternRef) => "externref",
        }
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct WatSignature<'a>(&'a Signature);

impl std::ops::Deref for WatSignature<'_> {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        let Self(target) = self;
        target
    }
}

impl WatSignature<'_> {
    fn mangled_wat_name(&self) -> String {
        let ret_types = self
            .return_types
            .iter()
            .map(WasmType::wat_type)
            .collect::<Vec<_>>()
            .join("_");
        let arg_types = self
            .argument_types
            .iter()
            .map(WasmType::wat_type)
            .collect::<Vec<_>>()
            .join("_");

        match (ret_types.is_empty(), arg_types.is_empty()) {
            (true, true) => "ret_arg".to_string(),
            (true, false) => format!("ret_arg_{arg_types}"),
            (false, true) => format!("ret_{ret_types}_arg"),
            (false, false) => format!("ret_{ret_types}_arg_{arg_types}"),
        }
    }

    fn mangled_name_by_count(&self) -> String {
        let ret_len = self.return_types.len();
        let arg_len = self.argument_types.len();
        format!("ret_{ret_len}_arg_{arg_len}")
    }

    fn type_name(&self) -> String {
        format!("$type_{}", self.mangled_wat_name())
    }

    fn wat_type(&self) -> String {
        let params = self
            .argument_types
            .iter()
            .map(|t| format!("(param {})", t.wat_type()))
            .collect::<Vec<_>>()
            .join(" ");

        let results = if self.return_types.is_empty() {
            String::new()
        } else {
            format!(
                " (result {})",
                self.return_types
                    .iter()
                    .map(|t| t.wat_type())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };

        format!("(type {} (func {params}{results}))", self.type_name())
    }
}

pub fn generate_lib(signatures: &[Signature]) -> String {
    let mut wat = String::from(WAT_BOILERPLATE);
    wat.push_str(&generate_wat_functions(signatures));
    wat.push_str(")\n"); // Close module
    wat
}

fn generate_wat_functions(signatures: &[Signature]) -> String {
    let mut processed_signature_counts: HashSet<(usize, usize)> = HashSet::new();
    let mut processed_signatures: HashSet<WatSignature> = HashSet::new();
    let mut program = String::new();

    program.push('\n'); // Add spacing before types

    // Generate function types first
    for signature in signatures {
        let wat_sig = WatSignature(signature);
        if !processed_signatures.contains(&wat_sig) {
            program.push_str(&format!("{DEF}{}\n", wat_sig.wat_type()));
            processed_signatures.insert(wat_sig);
        }
    }

    program.push('\n'); // Add spacing after types

    processed_signatures.clear();

    // Generate generic functions by count
    for signature in signatures {
        let signature_length = (signature.return_types.len(), signature.argument_types.len());

        if !processed_signature_counts.contains(&signature_length) {
            processed_signature_counts.insert(signature_length);
            program.push_str(&generate_allocate_generic_wat(
                signature.return_types.len(),
                signature.argument_types.len(),
            ));
            program.push_str(&generate_load_store_generic_wat(
                signature.return_types.len(),
                signature.argument_types.len(),
            ));
            program.push_str(&generate_free_generic_wat(
                signature.return_types.len(),
                signature.argument_types.len(),
            ));
        }
    }

    // Generate specialized functions
    for signature in signatures {
        let wat_sig = WatSignature(signature);
        if !processed_signatures.contains(&wat_sig) {
            program.push_str(&generate_specialized_functions_wat(&wat_sig));
            processed_signatures.insert(wat_sig);
        }
    }

    program
}

fn generate_allocate_generic_wat(ret_count: usize, arg_count: usize) -> String {
    let total_slots = ret_count + arg_count;
    let generic_name = format!("ret_{ret_count}_arg_{arg_count}");

    if total_slots == 0 {
        return format!("{DEF}(func $allocate_{generic_name} (result i32) (i32.const 0))\n");
    }

    let mut body = format!("{DEF}(func $allocate_{generic_name}",);

    // Add parameters for arguments
    for i in 0..arg_count {
        body.push_str(&format!(" (param $a{i} i64)"));
    }

    body.push_str(" (result i32)\n");

    // Declare local
    body.push_str(&format!("{BDY____}(local $stack_ptr i32)"));

    // Allocate memory
    body.push_str(&formatdoc!(
        "
        {BDY____};; Allocate {total_slots} slots
        {BDY____}(local.set $stack_ptr (call $stack_allocate_values (i32.const {total_slots})))
        "
    ));

    // Store arguments
    for i in 0..arg_count {
        let slot_index = ret_count + i;
        body.push_str(&formatdoc!(
            "
            {BDY____};; Store argument {i}
            {BDY____}(local.get $stack_ptr)
            {BDY____}(local.get $a{i})
            {BDY____}(i32.const {slot_index})
            {BDY____}(call $wastrumentation_memory_store)
            "
        ));
    }

    body.push_str(&format!("{BDY____}(local.get $stack_ptr)"));

    // Close body
    body.push_str(")\n");
    body
}

fn generate_load_store_generic_wat(ret_count: usize, arg_count: usize) -> String {
    let generic_name = format!("ret_{}_arg_{}", ret_count, arg_count);
    let mut functions = String::new();

    // Generate load functions for arguments
    for i in 0..arg_count {
        let slot_index = ret_count + i;
        functions.push_str(&formatdoc!(
            "
            {DEF}(func $load_arg{i}_{generic_name} (param $stack_ptr i32) (result i64)
            {BDY____}(local.get $stack_ptr)
            {BDY____}(i32.const {slot_index})
            {BDY____}(call $wastrumentation_memory_load))
            "
        ));
    }

    // Generate load functions for returns
    for i in 0..ret_count {
        functions.push_str(&formatdoc!(
            "
            {DEF}(func $load_ret{i}_{generic_name} (param $stack_ptr i32) (result i64)
            {BDY____}(local.get $stack_ptr)
            {BDY____}(i32.const {i})
            {BDY____}(call $wastrumentation_memory_load))
            "
        ));
    }

    // Generate store functions for arguments
    for i in 0..arg_count {
        let slot_index = ret_count + i;
        functions.push_str(&formatdoc!(
            "
            {DEF}(func $store_arg{i}_{generic_name} (param $stack_ptr i32) (param $value i64)
            {BDY____}(local.get $stack_ptr)
            {BDY____}(local.get $value)
            {BDY____}(i32.const {slot_index})
            {BDY____}(call $wastrumentation_memory_store))
            "
        ));
    }

    // Generate store functions for returns
    for i in 0..ret_count {
        functions.push_str(&formatdoc!(
            "
            {DEF}(func $store_ret{i}_{generic_name} (param $stack_ptr i32) (param $value i64)
            {BDY____}(local.get $stack_ptr)
            {BDY____}(local.get $value)
            {BDY____}(i32.const {i})
            {BDY____}(call $wastrumentation_memory_store))
            "
        ));
    }

    functions
}

fn generate_free_generic_wat(ret_count: usize, arg_count: usize) -> String {
    let total_slots = ret_count + arg_count;
    let generic_name = format!("ret_{ret_count}_arg_{arg_count}");

    if total_slots == 0 {
        return formatdoc!(
            "
            {DEF}(func $free_values_{generic_name} (param $ptr i32)
            {BDY____}(;No deallocation needed for empty signatures;))
            "
        );
    }

    formatdoc!(
        "
        {DEF}(func $free_values_{generic_name} (param $ptr i32)
        {BDY____}(local.get $ptr)
        {BDY____}(i32.const {total_slots})
        {BDY____}(call $stack_deallocate_values))
        "
    )
}

fn wasm_type_to_i64_conversion(wasm_type: &WasmType) -> &'static str {
    match wasm_type {
        WasmType::I32 => "(i64.extend_i32_u)",
        WasmType::F32 => "(i32.reinterpret_f32) (;then;) (i64.extend_i32_u)",
        WasmType::I64 => "", // No conversion needed
        WasmType::F64 => "(i64.reinterpret_f64)",
        WasmType::Ref(_) => "(i64.extend_i32_u)", // Refs are i32
    }
}

fn i64_to_wasm_type_conversion(wasm_type: &WasmType) -> &'static str {
    match wasm_type {
        WasmType::I32 => "(i32.wrap_i64)",
        WasmType::F32 => "(i32.wrap_i64) (;then;) (f32.reinterpret_i32)",
        WasmType::I64 => "", // No conversion needed
        WasmType::F64 => "(f64.reinterpret_i64)",
        WasmType::Ref(_) => "(i32.wrap_i64)", // Refs are i32
    }
}

// Add this function to generate the type storage operations
fn generate_type_stores(signature: &WatSignature) -> String {
    let mut stores = String::new();

    // Store return types first
    for (index, ret_type) in signature.return_types.iter().enumerate() {
        let type_enum = ret_type.runtime_enum_value();
        stores.push_str(&formatdoc!(
            "
            {BDY____};; Store return type {index}
            {BDY____}(local.get $types_buffer)
            {BDY____}(i32.const {index})
            {BDY____}(i32.const {type_enum})
            {BDY____}(call $wastrumentation_stack_store_type )
            "
        ));
    }

    // Store argument types after return types
    for (index, arg_type) in signature.argument_types.iter().enumerate() {
        let type_enum = arg_type.runtime_enum_value();
        let total_index = signature.return_types.len() + index;
        stores.push_str(&formatdoc!(
            "
            {BDY____};; Store argument type {index}
            {BDY____}(local.get $types_buffer)
            {BDY____}(i32.const {total_index})
            {BDY____}(i32.const {type_enum})
            {BDY____}(call $wastrumentation_stack_store_type)
            "
        ));
    }

    stores
}

fn generate_specialized_functions_wat(signature: &WatSignature) -> String {
    let mut functions = String::new();
    let mangled_name = signature.mangled_wat_name();
    let generic_name = signature.mangled_name_by_count();

    // Allocate function
    functions.push_str(&format!("{DEF}(func (export \"allocate_{mangled_name}\") "));

    for (i, arg_type) in signature.argument_types.iter().enumerate() {
        functions.push_str(&format!("(param $a{i} {}) ", arg_type.wat_type()));
    }
    functions.push_str("(result i32)\n");

    if signature.argument_types.is_empty() {
        functions.push_str(&format!("{BDY____}(call $allocate_{generic_name}))\n"));
    } else {
        for (i, arg_type) in signature.argument_types.iter().enumerate() {
            functions.push_str(&format!("{BDY____}(local.get $a{i})\n"));
            let conversion = wasm_type_to_i64_conversion(arg_type);
            if !conversion.is_empty() {
                functions.push_str(&format!("{BDY____}{conversion}\n"));
            }
        }
        functions.push_str(&format!("{BDY____}(call $allocate_{generic_name}))\n"));
    }

    // Load functions for arguments
    for (i, arg_type) in signature.argument_types.iter().enumerate() {
        let conversion = i64_to_wasm_type_conversion(arg_type);
        let arg_ty_wasm = arg_type.wat_type();

        functions.push_str(&formatdoc!(
            "
            {DEF}(func (export \"load_arg{i}_{mangled_name}\") (param $stack_ptr i32) (result {arg_ty_wasm})
            {BDY____}local.get $stack_ptr
            {BDY____}(call $load_arg{i}_{generic_name})
            "
        ));

        functions.push_str(&format!("{BDY____}{conversion})\n"));
    }

    // Load functions for returns
    for (i, ret_type) in signature.return_types.iter().enumerate() {
        let conversion = i64_to_wasm_type_conversion(ret_type);
        let ret_type_wasm = ret_type.wat_type();

        functions.push_str(&formatdoc!(
            "
            {DEF}(func (export \"load_ret{i}_{mangled_name}\") (param $stack_ptr i32) (result {ret_type_wasm})
            {BDY____}local.get $stack_ptr
            {BDY____}(call $load_ret{i}_{generic_name})
            "
        ));

        functions.push_str(&format!("{BDY____}{conversion})\n"));
    }

    // Store functions for arguments
    for (i, arg_type) in signature.argument_types.iter().enumerate() {
        let conversion = wasm_type_to_i64_conversion(arg_type);
        let arg_type_wasm = arg_type.wat_type();

        functions.push_str(&formatdoc!(
            "
            {DEF}(func (export \"store_arg{i}_{mangled_name}\") (param $stack_ptr i32) (param $value {arg_type_wasm})
            {BDY____}local.get $stack_ptr
            {BDY____}local.get $value
            "
        ));

        if !conversion.is_empty() {
            functions.push_str(&format!("{BDY____}{conversion}\n"));
        }

        functions.push_str(&format!("{BDY____}(call $store_arg{i}_{generic_name}))\n"));
    }

    // Store functions for returns
    for (i, ret_type) in signature.return_types.iter().enumerate() {
        let conversion = wasm_type_to_i64_conversion(ret_type);
        let ret_type_wasm = ret_type.wat_type();

        functions.push_str(&formatdoc!(
            "
            {DEF}(func (export \"store_ret{i}_{mangled_name}\") (param $stack_ptr i32) (param $value {ret_type_wasm})
            {BDY____}local.get $stack_ptr
            {BDY____}local.get $value
            "
        ));

        if !conversion.is_empty() {
            functions.push_str(&format!("{BDY____}{conversion}\n"));
        }

        functions.push_str(&format!("{BDY____}(call $store_ret{i}_{generic_name}))\n"));
    }

    // Store all returns function
    functions.push_str(&format!(
        "{DEF}(func (export \"store_rets_{mangled_name}\") (param $stack_ptr i32)"
    ));

    if signature.return_types.is_empty() {
        functions.push_str(&format!("\n{BDY____}(;No return values to store;))\n"));
    } else {
        // Add parameters for each return type
        for (i, ret_type) in signature.return_types.iter().enumerate() {
            functions.push_str(&format!(" (param $ret{i} {})", ret_type.wat_type()));
        }
        functions.push('\n');

        // Store each return value
        for (i, ret_type) in signature.return_types.iter().enumerate() {
            functions.push_str(&formatdoc!(
                "
                {BDY____};; Store return value {i}
                {BDY____}local.get $stack_ptr
                {BDY____}local.get $ret{i}
                "
            ));

            let conversion = wasm_type_to_i64_conversion(ret_type);
            if !conversion.is_empty() {
                functions.push_str(&format!("{BDY____}{conversion}\n"));
            }

            functions.push_str(&format!("{BDY____}(call $store_ret{i}_{generic_name})\n"));
        }
        functions.push_str(&format!("{DEF})\n"));
    }

    let total_types = signature.return_types.len() + signature.argument_types.len();
    if total_types > 0 {
        let type_stores = generate_type_stores(signature);

        functions.push_str(&formatdoc!(
            "
        {DEF}(func (export \"allocate_types_{mangled_name}\") (result i32)
        {BDY____}(local $types_buffer i32)
        {BDY____};; Allocate types buffer
        {BDY____}(local.set $types_buffer (call $stack_allocate_types (i32.const {total_types})))
        {BDY____};; Fill in the types
        {type_stores}
        {BDY____};; Return the buffer
        {BDY____}(local.get $types_buffer))
        "
        ));

        // Free types function
        functions.push_str(&formatdoc!(
            "
            {DEF}(func (export \"free_types_{mangled_name}\") (param $ptr i32)
            {BDY____}(call $stack_deallocate_types (local.get $ptr) (i32.const {total_types})))
            "
        ));
    } else {
        // Handle empty signature case - unchanged
        functions.push_str(&formatdoc!(
            "
            {DEF}(func (export \"allocate_types_{mangled_name}\") (result i32)
            {BDY____}(i32.const 0))
            "
        ));

        functions.push_str(&formatdoc!(
            "
            {DEF}(func (export \"free_types_{mangled_name}\") (param $ptr i32)
            {BDY____}(;No deallocation needed for empty signatures;))
            "
        ));
    }

    // Free function
    functions.push_str(&formatdoc!(
        "
        {DEF}(func (export \"free_values_{mangled_name}\") (param $ptr i32)
        {BDY____}(call $free_values_{generic_name} (local.get $ptr)))
        "
    ));

    functions
}

const DEF: &str = "    ";
const BDY____: &str = "        ";
const WAT_BOILERPLATE: &str = indoc! { r#"
(module
    ;; Memory management
    (memory 1)
    (global $stack_free_ptr (mut i32) (i32.const 0))
    (global $total_memory   (mut i32) (i32.const 1))

    ;; Constants
    (global $memory_growth_size i32 (i32.const     1))
    (global $page_byte_size     i32 (i32.const 65536))
    (global $size_wasm_value    i32 (i32.const     8)) ;; sizeof i64
    (global $size_wasm_type     i32 (i32.const     4)) ;; sizeof i32

    ;; Grow memory
    (func $grow_memory
        (if (i32.eq (memory.grow (global.get $memory_growth_size)) (i32.const -1))
            (then unreachable)
            (else
                (global.set
                    $total_memory
                    (i32.add
                        (global.get $total_memory)
                        (global.get $memory_growth_size))))))

    ;; Calculate total used memory in bytes
    (func $total_used_memory_in_bytes (result i32)
        (i32.mul (global.get $total_memory) (global.get $page_byte_size)))

    ;; Stack allocate raw bytes
    (func $stack_allocate (param $bytes i32) (result i32)
        (local $stack_free_ptr_before i32)
        (local $stack_free_ptr_after i32)

        (local.set $stack_free_ptr_before (global.get $stack_free_ptr))
        (local.set
            $stack_free_ptr_after
            (i32.add (global.get $stack_free_ptr) (local.get $bytes)))

        ;; Grow memory if needed
        (loop $grow_loop
            (if (i32.gt_u (local.get $stack_free_ptr_after) (call $total_used_memory_in_bytes))
                (then
                    (call $grow_memory)
                    (br $grow_loop))))

        (global.set $stack_free_ptr (local.get $stack_free_ptr_after))
        (local.get $stack_free_ptr_before))

    ;; Stack deallocate
    (func $stack_deallocate (param $ptr i32) (param $bytes i32)
        (global.set
            $stack_free_ptr
            (i32.sub (global.get $stack_free_ptr) (local.get $bytes))))

    ;; Stack allocate for WASM values (i64 slots)
    (func $stack_allocate_values (param $count i32) (result i32)
        (call $stack_allocate
            (i32.mul (local.get $count) (global.get $size_wasm_value))))

    ;; Stack deallocate for WASM values
    (func $stack_deallocate_values (param $ptr i32) (param $count i32)
        (call $stack_deallocate (local.get $ptr)
            (i32.mul (local.get $count) (global.get $size_wasm_value))))

    ;; Stack allocate for WASM types (i32 slots)
    (func $stack_allocate_types (param $count i32) (result i32)
        (call $stack_allocate
            (i32.mul (local.get $count) (global.get $size_wasm_type))))

    ;; Stack deallocate for WASM types
    (func $stack_deallocate_types (param $ptr i32) (param $count i32)
        (call $stack_deallocate (local.get $ptr)
            (i32.mul (local.get $count) (global.get $size_wasm_type))))

    ;; Memory load/store operations
    (func $wastrumentation_memory_load (param $ptr i32) (param $offset i32) (result i64)
        (i64.load
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_value)))))

    (func $wastrumentation_memory_store (param $ptr i32) (param $value i64) (param $offset i32)
        (i64.store
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_value)))
            (local.get $value)))

    ;; Type operations
    (func (export "wastrumentation_stack_load_type") (param $ptr i32) (param $offset i32) (result i32)
        (i32.load
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_type)))))

    (func $wastrumentation_stack_store_type (export "wastrumentation_stack_store_type") (param $ptr i32) (param $offset i32) (param $ty i32)
        (i32.store
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_type)))
            (local.get $ty)))

    ;; Typed load/store operations for export
    (func (export "wastrumentation_stack_load_i32") (param $ptr i32) (param $offset i32) (result i32)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset))
        i32.wrap_i64)

    (func (export "wastrumentation_stack_load_f32") (param $ptr i32) (param $offset i32) (result f32)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset))
        i32.wrap_i64
        f32.reinterpret_i32)

    (func (export "wastrumentation_stack_load_i64") (param $ptr i32) (param $offset i32) (result i64)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset)))

    (func (export "wastrumentation_stack_load_f64") (param $ptr i32) (param $offset i32) (result f64)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset))
        f64.reinterpret_i64)

    (func (export "wastrumentation_stack_store_i32") (param $ptr i32) (param $value i32) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr)
        (i64.extend_i32_u (local.get $value)) (local.get $offset)))

    (func (export "wastrumentation_stack_store_f32") (param $ptr i32) (param $value f32) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr)
        (i64.extend_i32_u (i32.reinterpret_f32 (local.get $value))) (local.get $offset)))

    (func (export "wastrumentation_stack_store_i64") (param $ptr i32) (param $value i64) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr) (local.get $value) (local.get $offset)))

    (func (export "wastrumentation_stack_store_f64") (param $ptr i32) (param $value f64) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr)
        (i64.reinterpret_f64 (local.get $value)) (local.get $offset)))
"# };

#[cfg(test)]
mod tests {
    use super::*;

    // Some sample signatures for testing purposes
    fn get_ret_f64_f32_arg_i32_i64() -> Signature {
        Signature {
            return_types: vec![WasmType::F64, WasmType::F32],
            argument_types: vec![WasmType::I32, WasmType::I64],
        }
    }

    fn get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64() -> Signature {
        Signature {
            return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
            argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
        }
    }

    #[test]
    fn generating_library_for_signatures() {
        let get_ret_f32_f64_arg_i32_i64 = || Signature {
            return_types: vec![WasmType::F32, WasmType::F64],
            argument_types: vec![WasmType::I32, WasmType::I64],
        };

        let signatures: Vec<Signature> = vec![
            Signature {
                return_types: vec![],
                argument_types: vec![],
            },
            get_ret_f32_f64_arg_i32_i64(),                 // dupe [A]
            get_ret_f64_f32_arg_i32_i64(),                 // dupe [B]
            get_ret_f32_f64_arg_i32_i64(),                 // dupe [A]
            get_ret_f64_f32_arg_i32_i64(),                 // dupe [B]
            get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(), // unique
        ];

        let generated_lib = super::generate_lib(&signatures);
        let expected = include_str!("./expected_lib.wat");

        assert_eq!(generated_lib, expected);
    }
}
