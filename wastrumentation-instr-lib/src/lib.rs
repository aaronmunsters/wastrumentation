use std::{collections::HashSet, fmt::Display};

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
}

impl WasmType {
    fn runtime_enum_value(&self) -> usize {
        match self {
            WasmType::I32 => 0,
            WasmType::F32 => 1,
            WasmType::I64 => 2,
            WasmType::F64 => 3,
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
        };
        write!(f, "{as_string}")
    }
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Signature {
    pub return_types: Vec<WasmType>,
    pub argument_types: Vec<WasmType>,
}

fn generics_offset(position: usize, rets_count: usize, args_offset: usize) -> String {
    if position == 0 {
        return "0".into();
    };
    let ret_offsets: Vec<String> = (0..rets_count).map(|n| format!("sizeof<R{n}>()")).collect();
    let arg_offsets: Vec<String> = (0..args_offset)
        .map(|n| format!("sizeof<T{n}>()"))
        .collect();

    ret_offsets
        .into_iter()
        .chain(arg_offsets.into_iter())
        .take(position)
        .collect::<Vec<String>>()
        .join(" + ")
}

fn arg_offset(arg_pos: usize, rets_count: usize, args_count: usize) -> String {
    generics_offset(rets_count + arg_pos, rets_count, args_count)
}

fn ret_offset(ret_pos: usize, rets_count: usize, args_count: usize) -> String {
    generics_offset(ret_pos, rets_count, args_count)
}

fn generate_allocate_generic(rets_count: usize, args_count: usize) -> String {
    // eg: rets_count := 2, args_count := 2

    // eg: [0, 1]
    let arg_ints = 0..args_count;
    // eg: [`T0`, `T1`]
    let arg_typs = arg_ints.clone().into_iter().map(|n| format!("T{n}"));

    // eg: [0, 1]
    let ret_ints = 0..rets_count;
    // eg: [`R0`, `R1`]
    let ret_typs = ret_ints.into_iter().map(|n| format!("R{n}"));

    // eg: [`R0`, `R1`, `T0`, `T1`]
    let array_of_all_generics = ret_typs.into_iter().chain(arg_typs.into_iter());
    // eg: `R0, R1, T0, T1`
    let string_all_generics = array_of_all_generics
        .clone()
        .collect::<Vec<String>>()
        .join(", ");

    // eg: `a0: T0, a1: T1`
    let signature = arg_ints
        .clone()
        .into_iter()
        .map(|n| format!("a{n}: T{n}"))
        .collect::<Vec<String>>()
        .join(", ");

    // eg: `sizeof<R0>() +  sizeof<R1>() +  sizeof<T0>() +  sizeof<T1>()`
    let total_allocation = array_of_all_generics
        .map(|ty| format!("sizeof<{ty}>()"))
        .collect::<Vec<String>>()
        .join(" + ");
    let all_stores = arg_ints
        .into_iter()
        .map(|n| {
            let offset = arg_offset(n, rets_count, args_count);
            format!(
                "// store a{n}
    const a{n}_offset = {offset}; // constant folded
    store<T{n}>(stack_begin, a{n}, a{n}_offset); // inlined"
            )
        })
        .collect::<Vec<String>>()
        .join("\n    ");

    format!(
        "
@inline
function allocate_ret_{rets_count}_arg_{args_count}<{string_all_generics}>({signature}): usize {{
    const to_allocate = {total_allocation}; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    {all_stores}
    return stack_begin;
}}"
    )
}

fn generate_allocate_specialized(signature: &Signature) -> String {
    // eg: Signature { return_type: [`i64`, `i32`], argument_types: [`f64`, `f32`] }
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: 2
    let signature_rets_count = signature_rets.len();
    // eg: `i64_i32`
    let signature_rets_ident = signature_rets
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: 2
    let signature_args_count = signature_args.len();
    // eg: `a0, a1`
    let args = signature_args
        .into_iter()
        .enumerate()
        .map(|(index, _ty)| format!("a{index}"))
        .collect::<Vec<String>>()
        .join(", ");
    // eg: `f64, f32`
    let signature_args_ident = signature_args
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: `a0: f64, a1: f32`
    let signature_args_typs_ident = signature_args
        .into_iter()
        .enumerate()
        .map(|(index, ty)| format!("a{index}: {ty}"))
        .collect::<Vec<String>>()
        .join(", ");
    // eg: `i64, i32, f64, f32`
    let signature_typs_ident = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ");
    return format!("
export function allocate_ret_{signature_rets_ident}_arg_{signature_args_ident}({signature_args_typs_ident}): usize {{
    return allocate_ret_{signature_rets_count}_arg_{signature_args_count}<{signature_typs_ident}>({args});
}};
");
}

fn generate_allocate_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    // eg: rets_count := 2, args_count := 2
    // eg: 4
    let total_allocation_slots = rets_count + args_count;
    // eg: `sizeof<i32>() * 4;`
    let total_allocation = format!("sizeof<i32>() * {total_allocation_slots};");

    return format!(
        "
@inline
function allocate_signature_types_buffer_ret_{rets_count}_arg_{args_count}(): usize {{
    const to_allocate = {total_allocation}; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}}"
    );
}

fn generate_allocate_types_buffer_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: 2
    let signature_rets_count = signature_rets.len();
    // eg: `i64, i32`
    let signature_rets_ident = signature_rets
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: 2
    let signature_args_count = signature_args.len();
    // eg: `f64, f32`
    let signature_args_ident = signature_args
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: `i64, i32, f64, f32`
    let all_stores = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(WasmType::runtime_enum_value)
        .enumerate()
        .map(|(index, enum_value)| {
            format!("store<i32>(types_buffer + (sizeof<i32>()*{index}), {enum_value}, NO_OFFSET);")
        })
        .collect::<Vec<String>>()
        .join("\n    ");

    return format!("
export function allocate_types_ret_{signature_rets_ident}_arg_{signature_args_ident}(): usize {{
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_ret_{signature_rets_count}_arg_{signature_args_count}();
    {all_stores}
    return types_buffer;
}}");
}

fn generate_load_generic(rets_count: usize, args_count: usize) -> String {
    // eg: rets_count := 2, args_count := 2
    // eg: [0, 1]
    let array_of_args_ints = 0..args_count;
    // eg: [`T0`, `T1`]
    let array_of_args_typs = array_of_args_ints.clone().map(|n| format!("T{n}"));

    // eg: [0, 1]
    let array_of_rets_ints = 0..rets_count;
    // eg: [`R0`, `R1`]
    let array_of_rets_typs = array_of_rets_ints.clone().map(|n| format!("R{n}"));

    // eg: [`R0`, `R1`, `T0`, `T1`]
    let array_of_all_generics = array_of_rets_typs
        .into_iter()
        .chain(array_of_args_typs.into_iter());
    // eg: `R0, R1, T0, T1`
    let string_all_generics = array_of_all_generics.collect::<Vec<String>>().join(", ");

    let all_arg_loads = array_of_args_ints.map(|n| {
        let an_offset = arg_offset(n, rets_count, args_count);
        format!("
@inline
function load_arg{n}_ret_{rets_count}_arg_{args_count}<{string_all_generics}>(stack_ptr: usize): T{n} {{
    const a{n}_offset = {an_offset}; // constant folded
    return load<T{n}>(stack_ptr, a{n}_offset); // inlined
}}")});
    let all_ret_loads = array_of_rets_ints.map(|n| {
        let ar_offset = ret_offset(n, rets_count, args_count);
        format!("
@inline
function load_ret{n}_ret_{rets_count}_arg_{args_count}<{string_all_generics}>(stack_ptr: usize): R{n} {{
    const r{n}_offset = {ar_offset}; // constant folded
    return load<R{n}>(stack_ptr, r{n}_offset); // inlined
}}")});
    return all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n");
}

fn generate_load_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: 2
    let signature_rets_count = signature_rets.len();
    // eg: `i64_i32`
    let signature_rets_ident = signature_rets
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: 2
    let signature_args_count = signature_args.len();
    // eg: `f64_f32`
    let signature_args_ident = signature_args
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: `i64, i32, f64, f32`
    let signature_typs_ident = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ");
    let all_arg_loads = signature_args.iter().enumerate().map(|(index, arg_i_ret_type)| format!("
export function load_arg{index}_ret_{signature_rets_ident}_arg_{signature_args_ident}(stack_ptr: usize): {arg_i_ret_type} {{
    return load_arg{index}_ret_{signature_rets_count}_arg_{signature_args_count}<{signature_typs_ident}>(stack_ptr);
}};"));
    let all_ret_loads = signature_rets.iter().enumerate().map(|(index, ret_i_ret_type )| format!("
export function load_ret{index}_ret_{signature_rets_ident}_arg_{signature_args_ident}(stack_ptr: usize): {ret_i_ret_type} {{
    return load_ret{index}_ret_{signature_rets_count}_arg_{signature_args_count}<{signature_typs_ident}>(stack_ptr);
}};"));

    return all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n");
}

fn generate_store_generic(rets_count: usize, args_count: usize) -> String {
    // eg: rets_count := 2, args_count := 2
    // eg: [0, 1]
    let array_of_args_ints = 0..args_count;
    // eg: [`T0`, `T1`]
    let array_of_args_typs = array_of_args_ints.clone().map(|n| format!("T{n}"));

    // eg: [0, 1]
    let array_of_rets_ints = 0..rets_count;
    // eg: [`R0`, `R1`]
    let array_of_rets_typs = array_of_rets_ints.clone().map(|n| format!("R{n}"));

    // eg: [`R0`, `R1`, `T0`, `T1`]
    let array_of_all_generics = array_of_rets_typs.chain(array_of_args_typs);

    // eg: `R0, R1, T0, T1`
    let string_all_generics = array_of_all_generics.collect::<Vec<String>>().join(", ");

    let all_arg_stores = array_of_args_ints.map(|n| {
        let an_offset = arg_offset(n, rets_count, args_count);
        format!("
@inline
function store_arg{n}_ret_{rets_count}_arg_{args_count}<{string_all_generics}>(stack_ptr: usize, a{n}: T{n}): void {{
    const a{n}_offset = {an_offset}; // constant folded
    return store<T{n}>(stack_ptr + a{n}_offset, a{n}); // inlined
}}")});
    let all_ret_stores = array_of_rets_ints.map(|n| {
        let ar_offset = ret_offset(n, rets_count, args_count);
        format!("
@inline
function store_ret{n}_ret_{rets_count}_arg_{args_count}<{string_all_generics}>(stack_ptr: usize, r{n}: R{n}): void {{
    const r{n}_offset = {ar_offset}; // constant folded
    return store<T{n}>(stack_ptr + r{n}_offset, r{n}); // inlined
}}")});
    return all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n");
}

fn generate_store_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: 2
    let signature_rets_count = signature_rets.len();
    // eg: `i64_i32`
    let signature_rets_ident = signature_rets
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: 2
    let signature_args_count = signature_args.len();
    // eg: `f64_f32`
    let signature_args_ident = signature_args
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: `i64, i32, f64, f32`
    let signature_typs_ident = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ");
    let all_arg_stores = signature_args.iter().enumerate().map(|(index, arg_i_ret_type)| format!("
export function store_arg{index}_ret_{signature_rets_ident}_arg_{signature_args_ident}(stack_ptr: usize, a{index}: {arg_i_ret_type}): void {{
    return store_arg{index}_ret_{signature_rets_count}_arg_{signature_args_count}<{signature_typs_ident}>(stack_ptr, a{index});
}};"));
    let all_ret_stores = signature_rets.iter().enumerate().map(|(index, ret_i_ret_type)| format!("
export function store_ret{index}_ret_{signature_rets_ident}_arg_{signature_args_ident}(stack_ptr: usize, a{index}: {ret_i_ret_type}): void {{
    return store_ret{index}_ret_{signature_rets_count}_arg_{signature_args_count}<{signature_typs_ident}>(stack_ptr, a{index});
}};"));
    return all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n");
}

fn generate_free_generic(rets_count: usize, args_count: usize) -> String {
    // eg: rets_count := 2, args_count := 2
    // eg: [0, 1]
    let array_of_args_ints = 0..args_count;
    // eg: [`T0`, `T1`]
    let array_of_args_typs = array_of_args_ints.map(|n| format!("T{n}"));

    // eg: [0, 1]
    let array_of_rets_ints = 0..rets_count;
    // eg: [`R0`, `R1`]
    let array_of_rets_typs = array_of_rets_ints.map(|n| format!("R{n}"));

    // eg: [`R0`, `R1`, `T0`, `T1`]
    let array_of_all_generics = array_of_rets_typs.chain(array_of_args_typs);
    // eg: `R0, R1, T0, T1`
    let string_all_generics = array_of_all_generics
        .clone()
        .collect::<Vec<String>>()
        .join(", ");

    // eg: `sizeof<R0>() +  sizeof<R1>() +  sizeof<T0>() +  sizeof<T1>()`
    let total_allocation = array_of_all_generics
        .map(|ty| format!("sizeof<{ty}>()"))
        .collect::<Vec<String>>()
        .join(" + ");

    return format!(
        "
@inline
function free_ret_{rets_count}_arg_{args_count}<{string_all_generics}>(): void {{
    const to_deallocate = {total_allocation}; // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}}"
    );
}

fn generate_free_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: 2
    let signature_rets_count = signature_rets.len();
    // eg: `i64_i32`
    let signature_rets_ident = signature_rets
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: 2
    let signature_args_count = signature_args.len();
    // eg: `f64_f32`
    let signature_args_ident = signature_args
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: `i64, i32, f64, f32`
    let signature_typs_ident = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ");
    return format!(
        "
export function free_ret_{signature_rets_ident}_arg_{signature_args_ident}(): void {{
    return free_ret_{signature_rets_count}_arg_{signature_args_count}<{signature_typs_ident}>();
}};"
    );
}

fn generate_store_rets_generic(rets_count: usize, args_count: usize) -> String {
    // eg: rets_count := 2, args_count := 2
    // eg: [0, 1]
    let array_of_args_ints = 0..args_count;
    // eg: [`T0`, `T1`]
    let array_of_args_typs = array_of_args_ints.map(|n| format!("T{n}"));

    // eg: [0, 1]
    let array_of_rets_ints = 0..rets_count;
    // eg: [`R0`, `R1`]
    let array_of_rets_typs = array_of_rets_ints.clone().map(|n| format!("R{n}"));

    // eg: [`R0`, `R1`, `T0`, `T1`]
    let array_of_all_generics = array_of_rets_typs.chain(array_of_args_typs);
    // eg: `R0, R1, T0, T1`
    let string_all_generics = array_of_all_generics.collect::<Vec<String>>().join(", ");

    // eg: [`a0: R0`, `a1: R1`]
    let array_of_rets_signature = array_of_rets_ints.clone().map(|n| format!("a{n}: R{n}"));
    let total_signature = (vec![String::from("stack_ptr: usize")])
        .into_iter()
        .chain(array_of_rets_signature)
        .collect::<Vec<String>>()
        .join(", ");
    let all_stores = array_of_rets_ints
        .map(|n| {
            format!(
                "// store a{n}
    store_ret{n}_ret_{rets_count}_arg_{args_count}<{string_all_generics}>(stack_ptr, a{n});"
            )
        })
        .collect::<Vec<String>>()
        .join("\n    ");

    return format!(
        "
@inline
function store_rets_{rets_count}_arg_{args_count}<{string_all_generics}>({total_signature}): void {{
    {all_stores}
    return;
}}"
    );
}

fn generate_store_rets_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: 2
    let signature_rets_count = signature_rets.len();
    // eg: `i64_i32`
    let signature_rets_ident = signature_rets
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: 2
    let signature_args_count = signature_args.len();
    // eg: [`a0`, `a1`]
    let rets_arguments = signature_rets
        .iter()
        .enumerate()
        .map(|(index, _type)| format!("a{index}"));
    // eg: `f64_f32`
    let signature_args_ident = signature_args
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("_");
    // eg: `i64, i32, f64, f32`
    let signature_typs_ident = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ");
    // eg: [`a0: R0`, `a1: R1`]
    let rets_signature = signature_rets
        .iter()
        .enumerate()
        .map(|(index, _ty)| format!("a{index}: {_ty}"));
    // eg: `stack_ptr: usize, a0: R0, a1: R1`
    let total_signature = (vec![String::from("stack_ptr: usize")])
        .into_iter()
        .chain(rets_signature)
        .collect::<Vec<String>>()
        .join(", ");
    // eg: `stack_ptr, a0, a1`
    let total_arguments = (vec![String::from("stack_ptr")])
        .into_iter()
        .chain(rets_arguments)
        .collect::<Vec<String>>()
        .join(", ");

    return format!("
export function store_rets_ret_{signature_rets_ident}_arg_{signature_args_ident}({total_signature}): void {{
    return store_rets_{signature_rets_count}_arg_{signature_args_count}<{signature_typs_ident}>({total_arguments});
}};");
}

const LIB_BOILERPLATE: &str = "
let $STACK_FREE_PTR: usize = 0;
let $TOTAL_MEMORY: usize = 0;
const $PAGE_BYTE_SIZE: usize = 65536;
const $MEMORY_GROWTH_SIZE: usize = 1;

@inline
function grow_memory(): void {
    // Grow by constant size
    if (memory.grow(($MEMORY_GROWTH_SIZE as i32)) == -1) {
        // If growth failed, trap
        unreachable()
    } else {
        // If growth did not fail, adjust house keeping
        $TOTAL_MEMORY += $MEMORY_GROWTH_SIZE;
    }
}

@inline
function stack_allocate(bytes: usize): usize {
    const stack_free_ptr_before_alloc = $STACK_FREE_PTR;
    const stack_free_ptr_after_alloc = $STACK_FREE_PTR + bytes;
    let total_memory_in_bytes = $TOTAL_MEMORY * $PAGE_BYTE_SIZE;

    while (stack_free_ptr_after_alloc > total_memory_in_bytes) {
        grow_memory();
        total_memory_in_bytes = $TOTAL_MEMORY * $PAGE_BYTE_SIZE;
    }

    $STACK_FREE_PTR = stack_free_ptr_after_alloc;
    return stack_free_ptr_before_alloc;
}

@inline
function stack_deallocate(bytes: usize): void {
    $STACK_FREE_PTR =- bytes;
}

export function wastrumentation_stack_load_i32(ptr: i32): i32 {
    return load<i32>(ptr);
};
export function wastrumentation_stack_load_f32(ptr: i32): f32 {
    return load<f32>(ptr);
};
export function wastrumentation_stack_load_i64(ptr: i32): i64 {
    return load<i64>(ptr);
};
export function wastrumentation_stack_load_f64(ptr: i32): f64 {
    return load<f64>(ptr);
};

export function wastrumentation_stack_store_i32(ptr: i32, value: i32): void {
    return store<i32>(ptr, value);
};
export function wastrumentation_stack_store_f32(ptr: i32, value: f32): void {
    return store<f32>(ptr, value);
};
export function wastrumentation_stack_store_i64(ptr: i32, value: i64): void {
    return store<i64>(ptr, value);
};
export function wastrumentation_stack_store_f64(ptr: i32, value: f64): void {
    return store<f64>(ptr, value);
};

";

pub fn generate_lib(signatures: &[Signature]) -> String {
    let mut lib = String::from(LIB_BOILERPLATE);
    lib.push_str(&generate_lib_for(signatures));
    lib
}

fn generate_lib_for(signatures: &[Signature]) -> String {
    let mut processed_signature_counts: HashSet<String> = HashSet::new();
    let mut processed_signatures: HashSet<&Signature> = HashSet::new();
    let mut program = String::new();
    for signature in signatures {
        let signature_ret_count = signature.return_types.len();
        let signature_arg_count = signature.argument_types.len();
        let signature_length: String = format!("{signature_ret_count},{signature_arg_count}");
        if !processed_signature_counts.contains(&signature_length) {
            program.push_str(&generate_allocate_generic(
                signature_ret_count,
                signature_arg_count,
            ));
            program.push_str(&generate_load_generic(
                signature_ret_count,
                signature_arg_count,
            ));
            program.push_str(&generate_store_generic(
                signature_ret_count,
                signature_arg_count,
            ));
            program.push_str(&generate_free_generic(
                signature_ret_count,
                signature_arg_count,
            ));
            program.push_str(&generate_store_rets_generic(
                signature_ret_count,
                signature_arg_count,
            ));
            program.push_str(&generate_allocate_types_buffer_generic(
                signature_ret_count,
                signature_arg_count,
            ));
            processed_signature_counts.insert(signature_length);
        }
        if !processed_signatures.contains(signature) {
            program.push_str(&generate_allocate_specialized(signature));
            program.push_str(&generate_load_specialized(signature));
            program.push_str(&generate_store_specialized(signature));
            program.push_str(&generate_free_specialized(signature));
            program.push_str(&generate_store_rets_specialized(signature));
            program.push_str(&generate_allocate_types_buffer_specialized(signature));
            processed_signatures.insert(signature);
        }
    }
    return program;
}

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
    fn compute_memory_offset_generically_works_correctly() {
        let pos_frst = 0;
        let pos_scnd = 1;
        let pos_thrd = 2;
        let pos_frth = 3;
        for ((position, ret_count, arg_offset), expected) in [
            ((pos_frst, 0, 1), "0"),
            ((pos_frst, 0, 2), "0"),
            ((pos_scnd, 0, 2), "sizeof<T0>()"),
            ((pos_frst, 1, 0), "0"),
            ((pos_frst, 1, 1), "0"),
            ((pos_scnd, 1, 1), "sizeof<R0>()"),
            ((pos_frst, 1, 2), "0"),
            ((pos_scnd, 1, 2), "sizeof<R0>()"),
            ((pos_thrd, 1, 2), "sizeof<R0>() + sizeof<T0>()"),
            ((pos_frst, 2, 0), "0"),
            ((pos_scnd, 2, 0), "sizeof<R0>()"),
            ((pos_frst, 2, 1), "0"),
            ((pos_scnd, 2, 1), "sizeof<R0>()"),
            ((pos_thrd, 2, 1), "sizeof<R0>() + sizeof<R1>()"),
            ((pos_frst, 2, 2), "0"),
            ((pos_scnd, 2, 2), "sizeof<R0>()"),
            ((pos_thrd, 2, 2), "sizeof<R0>() + sizeof<R1>()"),
            (
                (pos_frth, 2, 2),
                "sizeof<R0>() + sizeof<R1>() + sizeof<T0>()",
            ),
        ] {
            assert_eq!(generics_offset(position, ret_count, arg_offset), expected);
        }
    }

    #[test]
    fn generating_allocate_generic_instructions() {
        assert_eq!(
            generate_allocate_generic(0, 1),
            "
@inline
function allocate_ret_0_arg_1<T0>(a0: T0): usize {
    const to_allocate = sizeof<T0>(); // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    // store a0
    const a0_offset = 0; // constant folded
    store<T0>(stack_begin, a0, a0_offset); // inlined
    return stack_begin;
}"
        );

        assert_eq!(
        generate_allocate_generic(5, 5),
        "
@inline
function allocate_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(a0: T0, a1: T1, a2: T2, a3: T3, a4: T4): usize {
    const to_allocate = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>() + sizeof<T4>(); // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    // store a0
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>(); // constant folded
    store<T0>(stack_begin, a0, a0_offset); // inlined
    // store a1
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>(); // constant folded
    store<T1>(stack_begin, a1, a1_offset); // inlined
    // store a2
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    store<T2>(stack_begin, a2, a2_offset); // inlined
    // store a3
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    store<T3>(stack_begin, a3, a3_offset); // inlined
    // store a4
    const a4_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>(); // constant folded
    store<T4>(stack_begin, a4, a4_offset); // inlined
    return stack_begin;
}"
    );
    }

    #[test]
    fn generating_allocate_specialized_instructions() {
        assert_eq!(
            generate_allocate_specialized(&get_ret_f64_f32_arg_i32_i64()),
            "
export function allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64): usize {
    return allocate_ret_2_arg_2<f64, f32, i32, i64>(a0, a1);
};
"
        );

        assert_eq!(generate_allocate_specialized(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()), "
export function allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64): usize {
    return allocate_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(a0, a1, a2, a3);
};
");
    }

    #[test]
    fn generating_load_generic_instructions() {
        assert_eq!(
            generate_load_generic(0, 1),
            "
@inline
function load_arg0_ret_0_arg_1<T0>(stack_ptr: usize): T0 {
    const a0_offset = 0; // constant folded
    return load<T0>(stack_ptr, a0_offset); // inlined
}",
        );
        assert_eq!(generate_load_generic(5, 5), "
@inline
function load_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T0 {
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>(); // constant folded
    return load<T0>(stack_ptr, a0_offset); // inlined
}

@inline
function load_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T1 {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>(); // constant folded
    return load<T1>(stack_ptr, a1_offset); // inlined
}

@inline
function load_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T2 {
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    return load<T2>(stack_ptr, a2_offset); // inlined
}

@inline
function load_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T3 {
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    return load<T3>(stack_ptr, a3_offset); // inlined
}

@inline
function load_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T4 {
    const a4_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>(); // constant folded
    return load<T4>(stack_ptr, a4_offset); // inlined
}

@inline
function load_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R0 {
    const r0_offset = 0; // constant folded
    return load<R0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return load<R1>(stack_ptr, r1_offset); // inlined
}

@inline
function load_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R2 {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return load<R2>(stack_ptr, r2_offset); // inlined
}

@inline
function load_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R3 {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return load<R3>(stack_ptr, r3_offset); // inlined
}

@inline
function load_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R4 {
    const r4_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    return load<R4>(stack_ptr, r4_offset); // inlined
}");
    }

    #[test]
    fn generating_load_specialized_instructions() {
        assert_eq!(
            generate_load_specialized(&get_ret_f64_f32_arg_i32_i64()),
            "
export function load_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i32 {
    return load_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i64 {
    return load_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f64 {
    return load_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f32 {
    return load_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};"
        );

        assert_eq!(
            generate_load_specialized(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()),
            "
export function load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};"
        );
    }

    #[test]
    fn generating_store_generic_instructions() {
        assert_eq!(
            generate_store_generic(0, 1),
            "
@inline
function store_arg0_ret_0_arg_1<T0>(stack_ptr: usize, a0: T0): void {
    const a0_offset = 0; // constant folded
    return store<T0>(stack_ptr + a0_offset, a0); // inlined
}",
        );
        assert_eq!(generate_store_generic(5, 5), "
@inline
function store_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: T0): void {
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>(); // constant folded
    return store<T0>(stack_ptr + a0_offset, a0); // inlined
}

@inline
function store_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a1: T1): void {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>(); // constant folded
    return store<T1>(stack_ptr + a1_offset, a1); // inlined
}

@inline
function store_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a2: T2): void {
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    return store<T2>(stack_ptr + a2_offset, a2); // inlined
}

@inline
function store_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a3: T3): void {
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    return store<T3>(stack_ptr + a3_offset, a3); // inlined
}

@inline
function store_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a4: T4): void {
    const a4_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>(); // constant folded
    return store<T4>(stack_ptr + a4_offset, a4); // inlined
}

@inline
function store_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r0: R0): void {
    const r0_offset = 0; // constant folded
    return store<T0>(stack_ptr + r0_offset, r0); // inlined
}

@inline
function store_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r1: R1): void {
    const r1_offset = sizeof<R0>(); // constant folded
    return store<T1>(stack_ptr + r1_offset, r1); // inlined
}

@inline
function store_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r2: R2): void {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return store<T2>(stack_ptr + r2_offset, r2); // inlined
}

@inline
function store_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r3: R3): void {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return store<T3>(stack_ptr + r3_offset, r3); // inlined
}

@inline
function store_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r4: R4): void {
    const r4_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    return store<T4>(stack_ptr + r4_offset, r4); // inlined
}");
    }

    #[test]
    fn generating_store_specialized_instructions() {
        assert_eq!(
            generate_store_specialized(&get_ret_f64_f32_arg_i32_i64()),
            "
export function store_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: i32): void {
    return store_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: i64): void {
    return store_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};

export function store_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};",
        );

        assert_eq!(generate_store_specialized(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()), "
export function store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: i64): void {
    return store_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: i32): void {
    return store_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: f32): void {
    return store_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: f64): void {
    return store_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};

export function store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: i32): void {
    return store_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: i64): void {
    return store_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};");
    }

    #[test]
    fn generating_free_generic_instruction() {
        assert_eq!(
            generate_free_generic(0, 1),
            "
@inline
function free_ret_0_arg_1<T0>(): void {
    const to_deallocate = sizeof<T0>(); // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}"
        );
        assert_eq!(generate_free_generic(5, 5), "
@inline
function free_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(): void {
    const to_deallocate = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>() + sizeof<T4>(); // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}");
    }

    #[test]
    fn generating_free_specialized_instruction() {
        assert_eq!(
            generate_free_specialized(&get_ret_f64_f32_arg_i32_i64()),
            "
export function free_ret_f64_f32_arg_i32_i64(): void {
    return free_ret_2_arg_2<f64, f32, i32, i64>();
};",
        );

        assert_eq!(
            generate_free_specialized(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()),
            "
export function free_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(): void {
    return free_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>();
};",
        );
    }

    #[test]
    fn generating_store_rets_generic_instruction() {
        assert_eq!(
            generate_store_rets_generic(0, 1),
            "
@inline
function store_rets_0_arg_1<T0>(stack_ptr: usize): void {
    
    return;
}",
        );
        assert_eq!(generate_store_rets_generic(5, 5), "
@inline
function store_rets_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: R0, a1: R1, a2: R2, a3: R3, a4: R4): void {
    // store a0
    store_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a0);
    // store a1
    store_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a1);
    // store a2
    store_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a2);
    // store a3
    store_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a3);
    // store a4
    store_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a4);
    return;
}");
    }

    #[test]
    fn generating_store_rets_specialized_instruction() {
        assert_eq!(
            generate_store_rets_specialized(&get_ret_f64_f32_arg_i32_i64()),
            "
export function store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32): void {
    return store_rets_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0, a1);
};"
        );

        assert_eq!(generate_store_rets_specialized(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()), "
export function store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64): void {
    return store_rets_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
};");
    }

    #[test]
    fn generating_allocate_types_generic_specialized() {
        assert_eq!(
            generate_allocate_types_buffer_generic(0, 1),
            "
@inline
function allocate_signature_types_buffer_ret_0_arg_1(): usize {
    const to_allocate = sizeof<i32>() * 1;; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}"
        );

        assert_eq!(
            generate_allocate_types_buffer_generic(5, 5),
            "
@inline
function allocate_signature_types_buffer_ret_5_arg_5(): usize {
    const to_allocate = sizeof<i32>() * 10;; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}"
        )
    }

    #[test]
    fn generating_allocate_types_buffer_specialized_instruction() {
        let signature_0 = Signature {
            return_types: vec![WasmType::F64, WasmType::F32],
            argument_types: vec![WasmType::I32, WasmType::I64],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&signature_0),
            "
export function allocate_types_ret_f64_f32_arg_i32_i64(): usize {
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    store<i32>(types_buffer + (sizeof<i32>()*0), 3, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*1), 1, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*2), 0, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*3), 2, NO_OFFSET);
    return types_buffer;
}"
        );
        let signature_1 = Signature {
            return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
            argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&signature_1),
            "
export function allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(): usize {
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_ret_4_arg_4();
    store<i32>(types_buffer + (sizeof<i32>()*0), 3, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*1), 1, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*2), 0, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*3), 2, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*4), 2, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*5), 0, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*6), 1, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*7), 3, NO_OFFSET);
    return types_buffer;
}"
        );
    }

    #[test]
    fn generating_library_for_signatures() {
        let get_ret_f32_f64_arg_i32_i64 = || Signature {
            return_types: vec![WasmType::F32, WasmType::F64],
            argument_types: vec![WasmType::I32, WasmType::I64],
        };

        let signatures: Vec<Signature> = vec![
            get_ret_f32_f64_arg_i32_i64(),                 // dupe [A]
            get_ret_f64_f32_arg_i32_i64(),                 // dupe [B]
            get_ret_f32_f64_arg_i32_i64(),                 // dupe [A]
            get_ret_f64_f32_arg_i32_i64(),                 // dupe [B]
            get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(), // unique
        ];

        let lib = generate_lib_for(&signatures);
        assert_eq!(lib, "
@inline
function allocate_ret_2_arg_2<R0, R1, T0, T1>(a0: T0, a1: T1): usize {
    const to_allocate = sizeof<R0>() + sizeof<R1>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    // store a0
    const a0_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    store<T0>(stack_begin, a0, a0_offset); // inlined
    // store a1
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<T0>(); // constant folded
    store<T1>(stack_begin, a1, a1_offset); // inlined
    return stack_begin;
}
@inline
function load_arg0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): T0 {
    const a0_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return load<T0>(stack_ptr, a0_offset); // inlined
}

@inline
function load_arg1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): T1 {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<T0>(); // constant folded
    return load<T1>(stack_ptr, a1_offset); // inlined
}

@inline
function load_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): R0 {
    const r0_offset = 0; // constant folded
    return load<R0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return load<R1>(stack_ptr, r1_offset); // inlined
}
@inline
function store_arg0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a0: T0): void {
    const a0_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return store<T0>(stack_ptr + a0_offset, a0); // inlined
}

@inline
function store_arg1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a1: T1): void {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<T0>(); // constant folded
    return store<T1>(stack_ptr + a1_offset, a1); // inlined
}

@inline
function store_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, r0: R0): void {
    const r0_offset = 0; // constant folded
    return store<T0>(stack_ptr + r0_offset, r0); // inlined
}

@inline
function store_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, r1: R1): void {
    const r1_offset = sizeof<R0>(); // constant folded
    return store<T1>(stack_ptr + r1_offset, r1); // inlined
}
@inline
function free_ret_2_arg_2<R0, R1, T0, T1>(): void {
    const to_deallocate = sizeof<R0>() + sizeof<R1>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}
@inline
function store_rets_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a0: R0, a1: R1): void {
    // store a0
    store_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr, a0);
    // store a1
    store_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr, a1);
    return;
}
@inline
function allocate_signature_types_buffer_ret_2_arg_2(): usize {
    const to_allocate = sizeof<i32>() * 4;; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}
export function allocate_ret_f32_f64_arg_i32_i64(a0: i32, a1: i64): usize {
    return allocate_ret_2_arg_2<f32, f64, i32, i64>(a0, a1);
};

export function load_arg0_ret_f32_f64_arg_i32_i64(stack_ptr: usize): i32 {
    return load_arg0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};

export function load_arg1_ret_f32_f64_arg_i32_i64(stack_ptr: usize): i64 {
    return load_arg1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};

export function load_ret0_ret_f32_f64_arg_i32_i64(stack_ptr: usize): f32 {
    return load_ret0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};

export function load_ret1_ret_f32_f64_arg_i32_i64(stack_ptr: usize): f64 {
    return load_ret1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};
export function store_arg0_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a0: i32): void {
    return store_arg0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a0);
};

export function store_arg1_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a1: i64): void {
    return store_arg1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a1);
};

export function store_ret0_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a0: f32): void {
    return store_ret0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a0);
};

export function store_ret1_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a1: f64): void {
    return store_ret1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a1);
};
export function free_ret_f32_f64_arg_i32_i64(): void {
    return free_ret_2_arg_2<f32, f64, i32, i64>();
};
export function store_rets_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a0: f32, a1: f64): void {
    return store_rets_2_arg_2<f32, f64, i32, i64>(stack_ptr, a0, a1);
};
export function allocate_types_ret_f32_f64_arg_i32_i64(): usize {
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    store<i32>(types_buffer + (sizeof<i32>()*0), 1, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*1), 3, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*2), 0, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*3), 2, NO_OFFSET);
    return types_buffer;
}
export function allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64): usize {
    return allocate_ret_2_arg_2<f64, f32, i32, i64>(a0, a1);
};

export function load_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i32 {
    return load_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i64 {
    return load_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f64 {
    return load_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f32 {
    return load_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};
export function store_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: i32): void {
    return store_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: i64): void {
    return store_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};

export function store_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};
export function free_ret_f64_f32_arg_i32_i64(): void {
    return free_ret_2_arg_2<f64, f32, i32, i64>();
};
export function store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32): void {
    return store_rets_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0, a1);
};
export function allocate_types_ret_f64_f32_arg_i32_i64(): usize {
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    store<i32>(types_buffer + (sizeof<i32>()*0), 3, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*1), 1, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*2), 0, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*3), 2, NO_OFFSET);
    return types_buffer;
}
@inline
function allocate_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(a0: T0, a1: T1, a2: T2, a3: T3): usize {
    const to_allocate = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>(); // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    // store a0
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    store<T0>(stack_begin, a0, a0_offset); // inlined
    // store a1
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>(); // constant folded
    store<T1>(stack_begin, a1, a1_offset); // inlined
    // store a2
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    store<T2>(stack_begin, a2, a2_offset); // inlined
    // store a3
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    store<T3>(stack_begin, a3, a3_offset); // inlined
    return stack_begin;
}
@inline
function load_arg0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T0 {
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    return load<T0>(stack_ptr, a0_offset); // inlined
}

@inline
function load_arg1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T1 {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>(); // constant folded
    return load<T1>(stack_ptr, a1_offset); // inlined
}

@inline
function load_arg2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T2 {
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    return load<T2>(stack_ptr, a2_offset); // inlined
}

@inline
function load_arg3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T3 {
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    return load<T3>(stack_ptr, a3_offset); // inlined
}

@inline
function load_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R0 {
    const r0_offset = 0; // constant folded
    return load<R0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return load<R1>(stack_ptr, r1_offset); // inlined
}

@inline
function load_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R2 {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return load<R2>(stack_ptr, r2_offset); // inlined
}

@inline
function load_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R3 {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return load<R3>(stack_ptr, r3_offset); // inlined
}
@inline
function store_arg0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a0: T0): void {
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    return store<T0>(stack_ptr + a0_offset, a0); // inlined
}

@inline
function store_arg1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a1: T1): void {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>(); // constant folded
    return store<T1>(stack_ptr + a1_offset, a1); // inlined
}

@inline
function store_arg2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a2: T2): void {
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    return store<T2>(stack_ptr + a2_offset, a2); // inlined
}

@inline
function store_arg3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a3: T3): void {
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    return store<T3>(stack_ptr + a3_offset, a3); // inlined
}

@inline
function store_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r0: R0): void {
    const r0_offset = 0; // constant folded
    return store<T0>(stack_ptr + r0_offset, r0); // inlined
}

@inline
function store_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r1: R1): void {
    const r1_offset = sizeof<R0>(); // constant folded
    return store<T1>(stack_ptr + r1_offset, r1); // inlined
}

@inline
function store_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r2: R2): void {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return store<T2>(stack_ptr + r2_offset, r2); // inlined
}

@inline
function store_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r3: R3): void {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return store<T3>(stack_ptr + r3_offset, r3); // inlined
}
@inline
function free_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(): void {
    const to_deallocate = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>(); // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}
@inline
function store_rets_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a0: R0, a1: R1, a2: R2, a3: R3): void {
    // store a0
    store_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a0);
    // store a1
    store_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a1);
    // store a2
    store_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a2);
    // store a3
    store_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a3);
    return;
}
@inline
function allocate_signature_types_buffer_ret_4_arg_4(): usize {
    const to_allocate = sizeof<i32>() * 8;; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}
export function allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64): usize {
    return allocate_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(a0, a1, a2, a3);
};

export function load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};
export function store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: i64): void {
    return store_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: i32): void {
    return store_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: f32): void {
    return store_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: f64): void {
    return store_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};

export function store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: i32): void {
    return store_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: i64): void {
    return store_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};
export function free_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(): void {
    return free_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>();
};
export function store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64): void {
    return store_rets_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
};
export function allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(): usize {
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_ret_4_arg_4();
    store<i32>(types_buffer + (sizeof<i32>()*0), 3, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*1), 1, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*2), 0, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*3), 2, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*4), 2, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*5), 0, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*6), 1, NO_OFFSET);
    store<i32>(types_buffer + (sizeof<i32>()*7), 3, NO_OFFSET);
    return types_buffer;
}");
    }
}
