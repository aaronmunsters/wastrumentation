use std::{collections::HashSet, fmt::Display};

#[derive(Hash, PartialEq, Eq)]
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

#[derive(Hash, PartialEq, Eq)]
pub struct Signature {
    pub return_types: Vec<WasmType>,
    pub argument_types: Vec<WasmType>,
}

impl Signature {
    fn mangled_name(&self) -> String {
        let signature_rets = &self.return_types;
        let signature_rets_ident = signature_rets
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("_");
        let signature_args = &self.argument_types;
        let signature_args_ident = signature_args
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("_");
        format!("ret_{signature_rets_ident}_arg_{signature_args_ident}")
    }

    fn mangled_name_by_count(&self) -> String {
        let signature_rets_count = self.return_types.len();
        let signature_args_count = self.argument_types.len();
        format!("ret_{signature_rets_count}_arg_{signature_args_count}")
    }

    fn generic_name(signature_rets_count: usize, signature_args_count: usize) -> String {
        format!("ret_{signature_rets_count}_arg_{signature_args_count}")
    }

    fn comma_separated_types(&self) -> String {
        self.return_types
            .iter()
            .chain(self.argument_types.iter())
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn comma_separated_generics(rets_count: usize, args_count: usize) -> String {
        Self::generics(rets_count, args_count).join(", ")
    }

    fn generics(rets_count: usize, args_count: usize) -> Vec<String> {
        let arg_typs = (0..args_count).map(|n| format!("T{n}"));
        let ret_typs = (0..rets_count).map(|n| format!("R{n}"));
        ret_typs.chain(arg_typs).collect::<Vec<String>>()
    }

    fn generic_typed_arguments(args_count: usize) -> Vec<String> {
        (0..args_count)
            .clone()
            .map(|n| format!("a{n}: T{n}"))
            .collect::<Vec<String>>()
    }

    fn generic_comma_separated_typed_arguments(args_count: usize) -> String {
        Self::generic_typed_arguments(args_count).join(", ")
    }
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
        .chain(arg_offsets)
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
    let string_all_generics = Signature::comma_separated_generics(rets_count, args_count);
    let signature = Signature::generic_comma_separated_typed_arguments(args_count);
    let generic_name = Signature::generic_name(rets_count, args_count);

    // eg: `sizeof<R0>() +  sizeof<R1>() +  sizeof<T0>() +  sizeof<T1>()`
    let total_allocation = Signature::generics(rets_count, args_count)
        .iter()
        .map(|ty| format!("sizeof<{ty}>()"))
        .collect::<Vec<String>>()
        .join(" + ");
    let all_stores = (0..args_count)
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
function allocate_{generic_name}<{string_all_generics}>({signature}): usize {{
    const to_allocate = {total_allocation}; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    {all_stores}
    return stack_begin;
}}"
    )
}

fn generate_allocate_specialized(signature: &Signature) -> String {
    // eg: Signature { return_type: [`i64`, `i32`], argument_types: [`f64`, `f32`] }
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: `a0, a1`
    let args = signature_args
        .iter()
        .enumerate()
        .map(|(index, _ty)| format!("a{index}"))
        .collect::<Vec<String>>()
        .join(", ");
    // eg: `a0: f64, a1: f32`
    let signature_args_typs_ident = signature_args
        .iter()
        .enumerate()
        .map(|(index, ty)| format!("a{index}: {ty}"))
        .collect::<Vec<String>>()
        .join(", ");

    let comma_separated_types = signature.comma_separated_types();
    let mangled_name = signature.mangled_name();
    let mangled_by_count_name = signature.mangled_name_by_count();

    format!(
        "
export function allocate_{mangled_name}({signature_args_typs_ident}): usize {{
    return allocate_{mangled_by_count_name}<{comma_separated_types}>({args});
}};
"
    )
}

fn generate_allocate_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let total_allocation = format!("sizeof<i32>() * {};", rets_count + args_count);
    let generic_name = Signature::generic_name(rets_count, args_count);
    format!(
        "
@inline
function allocate_signature_types_buffer_{generic_name}(): usize {{
    const to_allocate = {total_allocation}; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}}"
    )
}

fn generate_allocate_types_buffer_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
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

    let mangled_name = signature.mangled_name();
    let mangled_by_count_name = signature.mangled_name_by_count();

    format!(
        "
export function allocate_types_{mangled_name}(): usize {{
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_{mangled_by_count_name}();
    {all_stores}
    return types_buffer;
}}"
    )
}

fn generate_load_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = Signature::comma_separated_generics(rets_count, args_count);
    let generic_name = Signature::generic_name(rets_count, args_count);

    let all_arg_loads = (0..args_count).map(|n| {
        let an_offset = arg_offset(n, rets_count, args_count);
        format!(
            "
@inline
function load_arg{n}_{generic_name}<{string_all_generics}>(stack_ptr: usize): T{n} {{
    const a{n}_offset = {an_offset}; // constant folded
    return load<T{n}>(stack_ptr, a{n}_offset); // inlined
}}"
        )
    });
    let all_ret_loads = (0..rets_count).map(|n| {
        let ar_offset = ret_offset(n, rets_count, args_count);
        format!(
            "
@inline
function load_ret{n}_{generic_name}<{string_all_generics}>(stack_ptr: usize): R{n} {{
    const r{n}_offset = {ar_offset}; // constant folded
    return load<R{n}>(stack_ptr, r{n}_offset); // inlined
}}"
        )
    });
    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_load_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;

    let comma_separated_types = signature.comma_separated_types();
    let mangled_name = signature.mangled_name();
    let mangled_by_count_name = signature.mangled_name_by_count();

    let all_arg_loads = signature_args
        .iter()
        .enumerate()
        .map(|(index, arg_i_ret_type)| {
            format!(
                "
export function load_arg{index}_{mangled_name}(stack_ptr: usize): {arg_i_ret_type} {{
    return load_arg{index}_{mangled_by_count_name}<{comma_separated_types}>(stack_ptr);
}};"
            )
        });
    let all_ret_loads = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ret_i_ret_type)| {
            format!(
                "
export function load_ret{index}_{mangled_name}(stack_ptr: usize): {ret_i_ret_type} {{
    return load_ret{index}_{mangled_by_count_name}<{comma_separated_types}>(stack_ptr);
}};"
            )
        });

    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = Signature::comma_separated_generics(rets_count, args_count);
    let generic_name = Signature::generic_name(rets_count, args_count);

    let all_arg_stores = (0..args_count).map(|n| {
        let an_offset = arg_offset(n, rets_count, args_count);
        format!(
            "
@inline
function store_arg{n}_{generic_name}<{string_all_generics}>(stack_ptr: usize, a{n}: T{n}): void {{
    const a{n}_offset = {an_offset}; // constant folded
    return store<T{n}>(stack_ptr + a{n}_offset, a{n}); // inlined
}}"
        )
    });
    let all_ret_stores = (0..rets_count).map(|n| {
        let ar_offset = ret_offset(n, rets_count, args_count);
        format!(
            "
@inline
function store_ret{n}_{generic_name}<{string_all_generics}>(stack_ptr: usize, r{n}: R{n}): void {{
    const r{n}_offset = {ar_offset}; // constant folded
    return store<T{n}>(stack_ptr + r{n}_offset, r{n}); // inlined
}}"
        )
    });
    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;

    let comma_separated_types = signature.comma_separated_types();
    let mangled_name = signature.mangled_name();
    let mangled_by_count_name = signature.mangled_name_by_count();

    let all_arg_stores = signature_args.iter().enumerate().map(|(index, arg_i_ret_type)| format!("
export function store_arg{index}_{mangled_name}(stack_ptr: usize, a{index}: {arg_i_ret_type}): void {{
    return store_arg{index}_{mangled_by_count_name}<{comma_separated_types}>(stack_ptr, a{index});
}};"));
    let all_ret_stores = signature_rets.iter().enumerate().map(|(index, ret_i_ret_type)| format!("
export function store_ret{index}_{mangled_name}(stack_ptr: usize, a{index}: {ret_i_ret_type}): void {{
    return store_ret{index}_{mangled_by_count_name}<{comma_separated_types}>(stack_ptr, a{index});
}};"));
    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_free_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = Signature::comma_separated_generics(rets_count, args_count);
    let generic_name = Signature::generic_name(rets_count, args_count);

    // eg: `sizeof<R0>() +  sizeof<R1>() +  sizeof<T0>() +  sizeof<T1>()`
    let total_allocation = Signature::generics(rets_count, args_count)
        .iter()
        .map(|ty| format!("sizeof<{ty}>()"))
        .collect::<Vec<String>>()
        .join(" + ");

    format!(
        "
@inline
function free_{generic_name}<{string_all_generics}>(): void {{
    const to_deallocate = {total_allocation}; // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}}"
    )
}

fn generate_free_specialized(signature: &Signature) -> String {
    format!(
        "
export function free_{}(): void {{
    return free_{}<{}>();
}};",
        signature.mangled_name(),
        signature.mangled_name_by_count(),
        signature.comma_separated_types(),
    )
}

fn generate_store_rets_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = Signature::comma_separated_generics(rets_count, args_count);
    let generic_name = Signature::generic_name(rets_count, args_count);

    // eg: [`a0: R0`, `a1: R1`]
    let array_of_rets_signature = (0..rets_count).map(|n| format!("a{n}: R{n}"));
    let total_signature = (vec![String::from("stack_ptr: usize")])
        .into_iter()
        .chain(array_of_rets_signature)
        .collect::<Vec<String>>()
        .join(", ");
    let all_stores = (0..rets_count)
        .map(|n| {
            format!(
                "// store a{n}
    store_ret{n}_{generic_name}<{string_all_generics}>(stack_ptr, a{n});"
            )
        })
        .collect::<Vec<String>>()
        .join("\n    ");

    format!(
        "
@inline
function store_rets_{generic_name}<{string_all_generics}>({total_signature}): void {{
    {all_stores}
    return;
}}"
    )
}

fn generate_store_rets_specialized(signature: &Signature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`a0`, `a1`]
    let rets_arguments = signature_rets
        .iter()
        .enumerate()
        .map(|(index, _type)| format!("a{index}"));
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

    let comma_separated_types = signature.comma_separated_types();
    let mangled_name = signature.mangled_name();
    let mangled_by_count_name = signature.mangled_name_by_count();

    format!(
        "
export function store_rets_{mangled_name}({total_signature}): void {{
    return store_rets_{mangled_by_count_name}<{comma_separated_types}>({total_arguments});
}};"
    )
}

const LIB_BOILERPLATE: &str = include_str!("lib_boilerplate.ts");

pub fn generate_lib(signatures: &[Signature]) -> String {
    let mut lib = String::from(LIB_BOILERPLATE);
    lib.push_str(&generate_lib_for(signatures));
    lib
}

fn generate_lib_for(signatures: &[Signature]) -> String {
    let mut processed_signature_counts: HashSet<(usize, usize)> = HashSet::new();
    let mut processed_signatures: HashSet<&Signature> = HashSet::new();
    let mut program = String::new();
    for signature in signatures {
        let signature_ret_count = signature.return_types.len();
        let signature_arg_count = signature.argument_types.len();
        let signature_length = (signature.return_types.len(), signature.argument_types.len());
        if !processed_signature_counts.contains(&signature_length) {
            processed_signature_counts.insert(signature_length);
            for generator in [
                generate_allocate_generic,
                generate_load_generic,
                generate_store_generic,
                generate_free_generic,
                generate_store_rets_generic,
                generate_allocate_types_buffer_generic,
            ] {
                program.push_str(generator(signature_ret_count, signature_arg_count).as_str())
            }
        }
        if !processed_signatures.contains(signature) {
            processed_signatures.insert(signature);
            for generator in [
                generate_allocate_specialized,
                generate_load_specialized,
                generate_store_specialized,
                generate_free_specialized,
                generate_store_rets_specialized,
                generate_allocate_types_buffer_specialized,
            ] {
                program.push_str(&generator(signature));
            }
        }
    }
    program
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
function store_rets_ret_0_arg_1<T0>(stack_ptr: usize): void {
    
    return;
}",
        );
        assert_eq!(generate_store_rets_generic(5, 5), "
@inline
function store_rets_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: R0, a1: R1, a2: R2, a3: R3, a4: R4): void {
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
    return store_rets_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0, a1);
};"
        );

        assert_eq!(generate_store_rets_specialized(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()), "
export function store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64): void {
    return store_rets_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
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
}
