type WasmType = "i32" | "i64" | "f32" | "f64";

class Signature {
    return_types: WasmType[];
    argument_types: WasmType[]
    constructor(return_types: WasmType[], argument_types: WasmType[]) {
        this.return_types = return_types;
        this.argument_types = argument_types;
    };

    toString(): string {
        return [...this.return_types, ...this.argument_types].join()
    }   
}

function generics_offset(position: number, rets_count: number, args_offset: number): string {
    if (position === 0) { return `0`; }
    const array_of_ret_ints = Array.from(Array(rets_count).keys());
    const array_of_ret_sizes = array_of_ret_ints.map((n) => `sizeof<R${n}>()`);
    const array_of_arg_ints = Array.from(Array(args_offset).keys());
    const array_of_arg_sizes = array_of_arg_ints.map((n) => `sizeof<T${n}>()`);
    return [...array_of_ret_sizes, ...array_of_arg_sizes].slice(0, position).join(" + ");
}

function arg_offset(arg_pos: number, rets_count: number, args_count: number): string {
    return generics_offset(rets_count + arg_pos, rets_count, args_count);
}

function ret_offset(ret_pos: number, rets_count: number, args_count: number): string {
    return generics_offset(ret_pos, rets_count, args_count);
}

function generate_allocate_generic(rets_count: number, args_count: number): string {
    //                                                                                            // eg: rets_count := 2, args_count := 2
    const array_of_args_ints = Array.from(Array(args_count).keys());                              // eg: [0, 1]
    const array_of_args_typs = array_of_args_ints.map((n) => `T${n}`);                            // eg: [`T0`, `T1`]

    const array_of_rets_ints = Array.from(Array(rets_count).keys());                              // eg: [0, 1]
    const array_of_rets_typs = array_of_rets_ints.map((n) => `R${n}`);                            // eg: [`R0`, `R1`]

    const array_of_all_generics = [...array_of_rets_typs, ...array_of_args_typs];                 // eg: [`R0`, `R1`, `T0`, `T1`]
    const string_all_generics = array_of_all_generics.join(", ");                                 // eg: `R0, R1, T0, T1`

    const signature = array_of_args_ints.map((n) => `a${n}: T${n}`).join(", ");                   // eg: `a0: T0, a1: T1`
    const total_allocation = array_of_all_generics.map((T) => `sizeof<${T}>()`).join(" + ");      // eg: `sizeof<R0>() +  sizeof<R1>() +  sizeof<T0>() +  sizeof<T1>()`
    const all_stores = array_of_args_ints.map((n) => `// store a${n}
    const a${n}_offset = ${arg_offset(n, rets_count, args_count)}; // constant folded
    store<T${n}>(stack_begin, a${n}, a${n}_offset); // inlined`).join(`
    `);

    return `
@inline
function allocate_ret_${rets_count}_arg_${args_count}<${string_all_generics}>(${signature}): usize {
    const to_allocate = ${total_allocation}; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    ${all_stores}
    return stack_begin;
}`;
}

// eg: Signature { return_type: [`i64`, `i32`], argument_types: [`f64`, `f32`] }
function generate_allocate_specialized(signature: Signature): string {
    // eg: [`i64`, `i32`]
    const signature_rets = signature.return_types;
    // eg: 2
    const signature_rets_count = signature_rets.length;
    // eg: `i64, i32`
    const signature_rets_ident = signature_rets.join(`_`);
    // eg: [`f64`, `f32`]
    const signature_args = signature.argument_types;
    // eg: 2
    const signature_args_count = signature_args.length;
    // eg: `a0, a1`
    const args = signature_args.map((_type, index) => `a${index}`).join(`, `)
    // eg: `f64, f32`
    const signature_args_ident = signature_args.join(`_`);
    // eg: `a0: f64, a1: f32`
    const signature_args_typs_ident = signature_args.map((type, index) => `a${index}: ${type}`).join(`, `);
    // eg: `i64, i32, f64, f32`
    const signature_typs_ident = [...signature_rets, ...signature_args].join(`, `);
    return `
export function allocate_ret_${signature_rets_ident}_arg_${signature_args_ident}(${signature_args_typs_ident}): usize {
    return allocate_ret_${signature_rets_count}_arg_${signature_args_count}<${signature_typs_ident}>(${args});
};
`;
}

function generate_allocate_types_buffer_generic(rets_count: number, args_count: number): string {
    //                                                                                            // eg: rets_count := 2, args_count := 2
    const total_allocation = `sizeof<i32>() * ${rets_count + args_count};`;                       // eg: `sizeof<i32>() * 4;`
    
    return `
@inline
function allocate_signature_types_buffer_ret_${rets_count}_arg_${args_count}(): usize {
    const to_allocate = ${total_allocation}; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}`;
}

function generate_allocate_types_buffer_specialized(signature: Signature): string {
    // eg: [`i64`, `i32`]
    const signature_rets = signature.return_types;
    // eg: 2
    const signature_rets_count = signature_rets.length;
    // eg: `i64, i32`
    const signature_rets_ident = signature_rets.join(`_`);
    // eg: [`f64`, `f32`]
    const signature_args = signature.argument_types;
    // eg: 2
    const signature_args_count = signature_args.length;
    // eg: `a0, a1`
    const args = signature_args.map((_type, index) => `a${index}`).join(`, `)
    // eg: `f64, f32`
    const signature_args_ident = signature_args.join(`_`);
    // eg: `i64, i32, f64, f32`
    const all_stores = [...signature_rets, ...signature_args].map((type) => {
        // TODO: tie to known set of types
        switch(type) {
            case "i32": return 0;
            case "f32": return 1;
            case "i64": return 2;
            case "f64": return 3;
        }
    }).map((value, index) => `
    store<i32>(types_buffer + (sizeof<i32>()*${index}), ${value}, NO_OFFSET);`).join(``);
    
    return `
export function allocate_types_ret_${signature_rets_ident}_arg_${signature_args_ident}(): usize {
    const NO_OFFSET = 0;
    const types_buffer = allocate_signature_types_buffer_ret_${signature_rets_count}_arg_${signature_args_count}();
    ${all_stores}
    return types_buffer;
}`;
}

function generate_load_generic(rets_count: number, args_count: number): string {
    //                                                                                            // eg: rets_count := 2, args_count := 2
    const array_of_args_ints = Array.from(Array(args_count).keys());                              // eg: [0, 1]
    const array_of_args_typs = array_of_args_ints.map((n) => `T${n}`);                            // eg: [`T0`, `T1`]

    const array_of_rets_ints = Array.from(Array(rets_count).keys());                              // eg: [0, 1]
    const array_of_rets_typs = array_of_rets_ints.map((n) => `R${n}`);                            // eg: [`R0`, `R1`]

    const array_of_all_generics = [...array_of_rets_typs, ...array_of_args_typs];                 // eg: [`R0`, `R1`, `T0`, `T1`]
    const string_all_generics = array_of_all_generics.join(", ");                                 // eg: `R0, R1, T0, T1`

    const all_arg_loads = array_of_args_ints.map((n) => `
@inline
function load_arg${n}_ret_${rets_count}_arg_${args_count}<${string_all_generics}>(stack_ptr: usize): T${n} {
    const a${n}_offset = ${arg_offset(n, rets_count, args_count)}; // constant folded
    return load<T${n}>(stack_ptr, a${n}_offset); // inlined
}`);
    const all_ret_loads = array_of_rets_ints.map((n) => `
@inline
function load_ret${n}_ret_${rets_count}_arg_${args_count}<${string_all_generics}>(stack_ptr: usize): R${n} {
    const r${n}_offset = ${ret_offset(n, rets_count, args_count)}; // constant folded
    return load<R${n}>(stack_ptr, r${n}_offset); // inlined
}`);
    return [...all_arg_loads, ...all_ret_loads].join(`\n`);
}

function generate_load_specialized(signature: Signature): string {
    // eg: [`i64`, `i32`]
    const signature_rets = signature.return_types;
    // eg: 2
    const signature_rets_count = signature_rets.length;
    // eg: `i64_i32`
    const signature_rets_ident = signature_rets.join(`_`);
    // eg: [`f64`, `f32`]
    const signature_args = signature.argument_types;
    // eg: 2
    const signature_args_count = signature_args.length;
    // eg: `f64_f32`
    const signature_args_ident = signature_args.join(`_`);
    // eg: `i64, i32, f64, f32`
    const signature_typs_ident = [...signature_rets, ...signature_args].join(`, `);
    const all_arg_loads = signature_args.map((arg_i_ret_type, index) => `
export function load_arg${index}_ret_${signature_rets_ident}_arg_${signature_args_ident}(stack_ptr: usize): ${arg_i_ret_type} {
    return load_arg${index}_ret_${signature_rets_count}_arg_${signature_args_count}<${signature_typs_ident}>(stack_ptr);
};`);
    const all_ret_loads = signature_rets.map((ret_i_ret_type, index) => `
export function load_ret${index}_ret_${signature_rets_ident}_arg_${signature_args_ident}(stack_ptr: usize): ${ret_i_ret_type} {
    return load_ret${index}_ret_${signature_rets_count}_arg_${signature_args_count}<${signature_typs_ident}>(stack_ptr);
};`);
    
    return [...all_arg_loads, ...all_ret_loads].join(`\n`);
}

function generate_store_generic(rets_count: number, args_count: number): string {
    //                                                                                            // eg: rets_count := 2, args_count := 2
    const array_of_args_ints = Array.from(Array(args_count).keys());                              // eg: [0, 1]
    const array_of_args_typs = array_of_args_ints.map((n) => `T${n}`);                            // eg: [`T0`, `T1`]

    const array_of_rets_ints = Array.from(Array(rets_count).keys());                              // eg: [0, 1]
    const array_of_rets_typs = array_of_rets_ints.map((n) => `R${n}`);                            // eg: [`R0`, `R1`]

    const array_of_all_generics = [...array_of_rets_typs, ...array_of_args_typs];                 // eg: [`R0`, `R1`, `T0`, `T1`]
    const string_all_generics = array_of_all_generics.join(", ");                                 // eg: `R0, R1, T0, T1`

    const all_arg_stores = array_of_args_ints.map((n) => `
@inline
function store_arg${n}_ret_${rets_count}_arg_${args_count}<${string_all_generics}>(stack_ptr: usize, a${n}: T${n}): void {
    const a${n}_offset = ${arg_offset(n, rets_count, args_count)}; // constant folded
    return store<T${n}>(stack_ptr + a${n}_offset, a${n}); // inlined
}`);
    const all_ret_stores = array_of_rets_ints.map((n) => `
@inline
function store_ret${n}_ret_${rets_count}_arg_${args_count}<${string_all_generics}>(stack_ptr: usize, r${n}: R${n}): void {
    const r${n}_offset = ${ret_offset(n, rets_count, args_count)}; // constant folded
    return store<T${n}>(stack_ptr + r${n}_offset, r${n}); // inlined
}`);
    return [...all_arg_stores, ...all_ret_stores].join(`\n`);
}

function generate_store_specialized(signature: Signature): string {
    // eg: [`i64`, `i32`]
    const signature_rets = signature.return_types;
    // eg: 2
    const signature_rets_count = signature_rets.length;
    // eg: `i64_i32`
    const signature_rets_ident = signature_rets.join(`_`);
    // eg: [`f64`, `f32`]
    const signature_args = signature.argument_types;
    // eg: 2
    const signature_args_count = signature_args.length;
    // eg: `f64_f32`
    const signature_args_ident = signature_args.join(`_`);
    // eg: `i64, i32, f64, f32`
    const signature_typs_ident = [...signature_rets, ...signature_args].join(`, `);
    const all_arg_stores = signature_args.map((arg_i_ret_type, index) => `
export function store_arg${index}_ret_${signature_rets_ident}_arg_${signature_args_ident}(stack_ptr: usize, a${index}: ${arg_i_ret_type}): void {
    return store_arg${index}_ret_${signature_rets_count}_arg_${signature_args_count}<${signature_typs_ident}>(stack_ptr, a${index});
};`);
    const all_ret_stores = signature_rets.map((ret_i_ret_type, index) => `
export function store_ret${index}_ret_${signature_rets_ident}_arg_${signature_args_ident}(stack_ptr: usize, a${index}: ${ret_i_ret_type}): void {
    return store_ret${index}_ret_${signature_rets_count}_arg_${signature_args_count}<${signature_typs_ident}>(stack_ptr, a${index});
};`);
    
    return [...all_arg_stores, ...all_ret_stores].join(`\n`);
}

function generate_free_generic(rets_count: number, args_count: number): string {
    //                                                                                            // eg: rets_count := 2, args_count := 2
    const array_of_args_ints = Array.from(Array(args_count).keys());                              // eg: [0, 1]
    const array_of_args_typs = array_of_args_ints.map((n) => `T${n}`);                            // eg: [`T0`, `T1`]

    const array_of_rets_ints = Array.from(Array(rets_count).keys());                              // eg: [0, 1]
    const array_of_rets_typs = array_of_rets_ints.map((n) => `R${n}`);                            // eg: [`R0`, `R1`]

    const array_of_all_generics = [...array_of_rets_typs, ...array_of_args_typs];                 // eg: [`R0`, `R1`, `T0`, `T1`]
    const string_all_generics = array_of_all_generics.join(", ");                                 // eg: `R0, R1, T0, T1`

    const signature = array_of_args_ints.map((n) => `a${n}: T${n}`).join(", ");                   // eg: `a0: T0, a1: T1`
    const total_allocation = array_of_all_generics.map((T) => `sizeof<${T}>()`).join(" + ");      // eg: `sizeof<R0>() +  sizeof<R1>() +  sizeof<T0>() +  sizeof<T1>()`
    const all_stores = array_of_args_ints.map((n) => `// store a${n}
    const a${n}_offset = ${arg_offset(n, rets_count, args_count)}; // constant folded
    store<T${n}>(stack_begin, a${n}, a${n}_offset); // inlined`).join(`
    `);

    return `
@inline
function free_ret_${rets_count}_arg_${args_count}<${string_all_generics}>(): void {
    const to_deallocate = ${total_allocation}; // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}`;
}

function generate_free_specialized(signature: Signature): string {
    // eg: [`i64`, `i32`]
    const signature_rets = signature.return_types;
    // eg: 2
    const signature_rets_count = signature_rets.length;
    // eg: `i64_i32`
    const signature_rets_ident = signature_rets.join(`_`);
    // eg: [`f64`, `f32`]
    const signature_args = signature.argument_types;
    // eg: 2
    const signature_args_count = signature_args.length;
    // eg: `a0, a1`
    const args = signature_args.map((_type, index) => `a${index}`).join(`, `)
    // eg: `f64_f32`
    const signature_args_ident = signature_args.join(`_`);
    // eg: `i64, i32, f64, f32`
    const signature_typs_ident = [...signature_rets, ...signature_args].join(`, `);
    return `
export function free_ret_${signature_rets_ident}_arg_${signature_args_ident}(): void {
    return free_ret_${signature_rets_count}_arg_${signature_args_count}<${signature_typs_ident}>();
};`;
}

function generate_store_rets_generic(rets_count: number, args_count: number): string {
    //                                                                                            // eg: rets_count := 2, args_count := 2
    const array_of_args_ints = Array.from(Array(args_count).keys());                              // eg: [0, 1]
    const array_of_args_typs = array_of_args_ints.map((n) => `T${n}`);                            // eg: [`T0`, `T1`]

    const array_of_rets_ints = Array.from(Array(rets_count).keys());                              // eg: [0, 1]
    const array_of_rets_typs = array_of_rets_ints.map((n) => `R${n}`);                            // eg: [`R0`, `R1`]

    const array_of_all_generics = [...array_of_rets_typs, ...array_of_args_typs];                 // eg: [`R0`, `R1`, `T0`, `T1`]
    const string_all_generics = array_of_all_generics.join(", ");                                 // eg: `R0, R1, T0, T1`

    const array_of_rets_signature = array_of_rets_ints.map((n) => `a${n}: R${n}`);                // eg: [`a0: R0`, `a1: R1`]
    const total_signature = [`stack_ptr: usize`, ...array_of_rets_signature].join(`, `);
    const all_stores = array_of_rets_ints.map((n) => `// store a${n}
    store_ret${n}_ret_${rets_count}_arg_${args_count}<${string_all_generics}>(stack_ptr, a${n});`).join(`
    `);

    return `
@inline
function store_rets_${rets_count}_arg_${args_count}<${string_all_generics}>(${total_signature}): void {
    ${all_stores}
    return;
}`;
}

function generate_store_rets_specialized(signature: Signature): string {
    // eg: [`i64`, `i32`]
    const signature_rets = signature.return_types;
    // eg: 2
    const signature_rets_count = signature_rets.length;
    // eg: `i64_i32`
    const signature_rets_ident = signature_rets.join(`_`);
    // eg: [`f64`, `f32`]
    const signature_args = signature.argument_types;
    // eg: 2
    const signature_args_count = signature_args.length;
    // eg: [`a0`, `a1`]
    const rets_arguments = signature_rets.map((_type, index) => `a${index}`);
    // eg: `f64_f32`
    const signature_args_ident = signature_args.join(`_`);
    // eg: `i64, i32, f64, f32`
    const signature_typs_ident = [...signature_rets, ...signature_args].join(`, `);
    // eg: [`a0: R0`, `a1: R1`]
    const rets_signature = signature_rets.map((type, index) => `a${index}: ${type}`);
    // eg: `stack_ptr: usize, a0: R0, a1: R1`
    const total_signature = [`stack_ptr: usize`, ...rets_signature].join(`, `);
    // eg: `stack_ptr, a0, a1`
    const total_arguments = [`stack_ptr`, ...rets_arguments].join(`, `);

    return `
export function store_rets_ret_${signature_rets_ident}_arg_${signature_args_ident}(${total_signature}): void {
    return store_rets_${signature_rets_count}_arg_${signature_args_count}<${signature_typs_ident}>(${total_arguments});
};`;
}

const lib_boilerplate = `
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

`;

function generate_lib(signatures: Signature[]): string {
    return [lib_boilerplate, generate_lib_for(signatures)].join(`\n`);
}

function generate_lib_for(signatures: Signature[]): string {
    const processed_signature_counts = new Set<string>();
    const processed_signatures = new Set<string>();
    let program = ``;
    for (const signature of signatures) {
        const signature_ret_count = signature.return_types.length;
        const signature_arg_count = signature.argument_types.length;
        const signature_length: string = `${signature_ret_count},${signature_arg_count}`;
        if (!processed_signature_counts.has(signature_length)) {
            program += generate_allocate_generic(signature_ret_count, signature_arg_count);
            program += generate_load_generic(signature_ret_count, signature_arg_count);
            program += generate_store_generic(signature_ret_count, signature_arg_count);
            program += generate_free_generic(signature_ret_count, signature_arg_count);
            program += generate_store_rets_generic(signature_ret_count, signature_arg_count);
            program += generate_allocate_types_buffer_generic(signature_ret_count, signature_arg_count);
            processed_signature_counts.add(signature_length);
        }
        if (!processed_signatures.has(signature.toString())) {
            program += generate_allocate_specialized(signature);
            program += generate_load_specialized(signature);
            program += generate_store_specialized(signature);
            program += generate_free_specialized(signature);
            program += generate_store_rets_specialized(signature);
            program += generate_allocate_types_buffer_specialized(signature);
            processed_signatures.add(signature.toString());
        }
    }
    return program;
}

// ================================= //
// ================================= //
// ========= PROGRAM ENTRY ========= //
// ================================= //
// ================================= //

const lib = generate_lib([
    new Signature(["i32"], ["i32", "i32"]),
]);
await Deno.writeTextFile("src_generated/lib.ts", lib);
    
// ================================= //
// ================================= //
// ============= TESTS ============= //
// ================================= //
// ================================= //

import { assertEquals } from "https://deno.land/std@0.202.0/assert/mod.ts";

Deno.test("Compute memory offset generically works correctly", () => {
    const POS_FRST = 0;
    const POS_SCND = 1;
    const POS_THRD = 2;
    const POS_FRTH = 3;
    assertEquals(generics_offset(POS_FRST, 0, 1), `0`);
    assertEquals(generics_offset(POS_FRST, 0, 2), `0`);
    assertEquals(generics_offset(POS_SCND, 0, 2), `sizeof<T0>()`);
    assertEquals(generics_offset(POS_FRST, 1, 0), `0`);
    assertEquals(generics_offset(POS_FRST, 1, 1), `0`);
    assertEquals(generics_offset(POS_SCND, 1, 1), `sizeof<R0>()`);
    assertEquals(generics_offset(POS_FRST, 1, 2), `0`);
    assertEquals(generics_offset(POS_SCND, 1, 2), `sizeof<R0>()`);
    assertEquals(generics_offset(POS_THRD, 1, 2), `sizeof<R0>() + sizeof<T0>()`);
    assertEquals(generics_offset(POS_FRST, 2, 0), `0`);
    assertEquals(generics_offset(POS_SCND, 2, 0), `sizeof<R0>()`);
    assertEquals(generics_offset(POS_FRST, 2, 1), `0`);
    assertEquals(generics_offset(POS_SCND, 2, 1), `sizeof<R0>()`);
    assertEquals(generics_offset(POS_THRD, 2, 1), `sizeof<R0>() + sizeof<R1>()`);
    assertEquals(generics_offset(POS_FRST, 2, 2), `0`);
    assertEquals(generics_offset(POS_SCND, 2, 2), `sizeof<R0>()`);
    assertEquals(generics_offset(POS_THRD, 2, 2), `sizeof<R0>() + sizeof<R1>()`);
    assertEquals(generics_offset(POS_FRTH, 2, 2), `sizeof<R0>() + sizeof<R1>() + sizeof<T0>()`);
})

Deno.test("Generating allocate generic instructions", () => {
    assertEquals(
        generate_allocate_generic(0, 1),
        `
@inline
function allocate_ret_0_arg_1<T0>(a0: T0): usize {
    const to_allocate = sizeof<T0>(); // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    // store a0
    const a0_offset = 0; // constant folded
    store<T0>(stack_begin, a0, a0_offset); // inlined
    return stack_begin;
}`,
    );

    assertEquals(
        generate_allocate_generic(5, 5),
        `
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
}`,
    );
})

Deno.test("Generating allocate specialized instructions", () => {
    const signature_0 = new Signature([`f64`, `f32`], [`i32`, `i64`]);
    assertEquals(generate_allocate_specialized(signature_0), `
export function allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64): usize {
    return allocate_ret_2_arg_2<f64, f32, i32, i64>(a0, a1);
};
`);

    const signature_1 = new Signature([`f64`, `f32`, `i32`, `i64`], [`i64`, `i32`, `f32`, `f64`]);
    assertEquals(generate_allocate_specialized(signature_1), `
export function allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64): usize {
    return allocate_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(a0, a1, a2, a3);
};
`);
});

Deno.test("Signature's behavior toString", () => {
    const signature = new Signature(["f32", "f64"], ["i32", "i64"]);
    assertEquals(signature.toString(), `f32,f64,i32,i64`);
});

Deno.test("Generating a library for signatures", () => {
    // a few signatures, some different, some duplicate
    const duped_signature = new Signature([`f64`, `f32`], [`i32`, `i64`]);
    const signatures: Signature[] = [
        new Signature(["f32", "f64"], ["i32", "i64"]), // duped 'by content'
        duped_signature,                               // duped 'by pointer'
        new Signature(["f32", "f64"], ["i32", "i64"]), // duped 'by content'
        duped_signature,
        new Signature([`f64`, `f32`, `i32`, `i64`], [`i64`, `i32`, `f32`, `f64`]),
    ];
    const lib = generate_lib_for(signatures);
    assertEquals(lib, `
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
    return load<T0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return load<T1>(stack_ptr, r1_offset); // inlined
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
function allocate_signature_4(): usize {
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
    return load<T0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return load<T1>(stack_ptr, r1_offset); // inlined
}

@inline
function load_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R2 {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return load<T2>(stack_ptr, r2_offset); // inlined
}

@inline
function load_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R3 {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return load<T3>(stack_ptr, r3_offset); // inlined
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
function allocate_signature_8(): usize {
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
};`);
});

Deno.test("Generating load generic instructions", () => {
    assertEquals(generate_load_generic(0, 1), `
@inline
function load_arg0_ret_0_arg_1<T0>(stack_ptr: usize): T0 {
    const a0_offset = 0; // constant folded
    return load<T0>(stack_ptr, a0_offset); // inlined
}`);
    assertEquals(generate_load_generic(5, 5), `
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
    return load<T0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return load<T1>(stack_ptr, r1_offset); // inlined
}

@inline
function load_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R2 {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return load<T2>(stack_ptr, r2_offset); // inlined
}

@inline
function load_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R3 {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return load<T3>(stack_ptr, r3_offset); // inlined
}

@inline
function load_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R4 {
    const r4_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    return load<T4>(stack_ptr, r4_offset); // inlined
}`);
});

Deno.test("Generating load specialized instructions", () => {
    const signature_0 = new Signature([`f64`, `f32`], [`i32`, `i64`]);
    assertEquals(generate_load_specialized(signature_0), `
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
};`);
    const signature_1 = new Signature([`f64`, `f32`, `i32`, `i64`], [`i64`, `i32`, `f32`, `f64`]);
    assertEquals(generate_load_specialized(signature_1), `
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
};`);
});

Deno.test("Generating store generic instructions", () => {
    assertEquals(generate_store_generic(0, 1), `
@inline
function store_arg0_ret_0_arg_1<T0>(stack_ptr: usize, a0: T0): void {
    const a0_offset = 0; // constant folded
    return store<T0>(stack_ptr + a0_offset, a0); // inlined
}`);
    assertEquals(generate_store_generic(5, 5), `
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
}`);
});

Deno.test("Generating store specialized instructions", () => {
    const signature_0 = new Signature([`f64`, `f32`], [`i32`, `i64`]);
    assertEquals(generate_store_specialized(signature_0), `
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
};`);
    const signature_1 = new Signature([`f64`, `f32`, `i32`, `i64`], [`i64`, `i32`, `f32`, `f64`]);
    assertEquals(generate_store_specialized(signature_1), `
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
};`);
});

Deno.test("Generating free generic instruction", () => {
    assertEquals(generate_free_generic(0, 1), `
@inline
function free_ret_0_arg_1<T0>(): void {
    const to_deallocate = sizeof<T0>(); // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}`);
    assertEquals(generate_free_generic(5, 5), `
@inline
function free_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(): void {
    const to_deallocate = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<R4>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>() + sizeof<T4>(); // constant folded
    stack_deallocate(to_deallocate); // inlined
    return;
}`);
});

Deno.test("Generating free specialized instruction", () => {
    const signature_0 = new Signature([`f64`, `f32`], [`i32`, `i64`]);
    assertEquals(generate_free_specialized(signature_0), `
export function free_ret_f64_f32_arg_i32_i64(): void {
    return free_ret_2_arg_2<f64, f32, i32, i64>();
};`);
    const signature_1 = new Signature([`f64`, `f32`, `i32`, `i64`], [`i64`, `i32`, `f32`, `f64`]);
    assertEquals(generate_free_specialized(signature_1), `
export function free_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(): void {
    return free_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>();
};`);
});

Deno.test("Generating store rets generic instruction", () => {
    assertEquals(generate_store_rets_generic(0, 1), `
@inline
function store_rets_0_arg_1<T0>(stack_ptr: usize): void {
    
    return;
}`);
    assertEquals(generate_store_rets_generic(5, 5), `
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
}`);
});

Deno.test("Generating store rets specialized instruction", () => {
    const signature_0 = new Signature([`f64`, `f32`], [`i32`, `i64`]);
    assertEquals(generate_store_rets_specialized(signature_0), `
export function store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32): void {
    return store_rets_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0, a1);
};`);
    const signature_1 = new Signature([`f64`, `f32`, `i32`, `i64`], [`i64`, `i32`, `f32`, `f64`]);
    assertEquals(generate_store_rets_specialized(signature_1), `
export function store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64): void {
    return store_rets_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
};`);
});
