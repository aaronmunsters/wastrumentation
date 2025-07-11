const NO_OFFSET: i32 = 0;

enum WasmType {
    i32 = 0,
    f32 = 1,
    i64 = 2,
    f64 = 3,
}

@external("wastrumentation_stack", "wastrumentation_stack_load_i32")
declare function wastrumentation_stack_load_i32(ptr: i32, offset: i32): i32;
@external("wastrumentation_stack", "wastrumentation_stack_load_f32")
declare function wastrumentation_stack_load_f32(ptr: i32, offset: i32): f32;
@external("wastrumentation_stack", "wastrumentation_stack_load_i64")
declare function wastrumentation_stack_load_i64(ptr: i32, offset: i32): i64;
@external("wastrumentation_stack", "wastrumentation_stack_load_f64")
declare function wastrumentation_stack_load_f64(ptr: i32, offset: i32): f64;

@external("wastrumentation_stack", "wastrumentation_stack_store_i32")
declare function wastrumentation_stack_store_i32(ptr: i32, value: i32, offset: i32): void;
@external("wastrumentation_stack", "wastrumentation_stack_store_f32")
declare function wastrumentation_stack_store_f32(ptr: i32, value: f32, offset: i32): void;
@external("wastrumentation_stack", "wastrumentation_stack_store_i64")
declare function wastrumentation_stack_store_i64(ptr: i32, value: i64, offset: i32): void;
@external("wastrumentation_stack", "wastrumentation_stack_store_f64")
declare function wastrumentation_stack_store_f64(ptr: i32, value: f64, offset: i32): void;

function wastrumentation_memory_load<T>(ptr: i32, offset: i32): T {
    if (false) { unreachable(); }
    else if (sizeof<T>() == 4 && isInteger<T>())
        return wastrumentation_stack_load_i32(ptr, offset);
    else if (sizeof<T>() == 4 && isFloat<T>())
        return wastrumentation_stack_load_f32(ptr, offset);
    else if (sizeof<T>() == 8 && isInteger<T>())
        return wastrumentation_stack_load_i64(ptr, offset);
    else if (sizeof<T>() == 8 && isFloat<T>())
        return wastrumentation_stack_load_f64(ptr, offset);
    unreachable();
}

function wastrumentation_memory_store<T>(ptr: i32, value: T, offset: i32): void {
    if (false) { unreachable(); }
    else if (sizeof<T>() == 4 && isInteger<T>())
        return wastrumentation_stack_store_i32(ptr, value, offset);
    else if (sizeof<T>() == 4 && isFloat<T>())
        return wastrumentation_stack_store_f32(ptr, value, offset);
    else if (sizeof<T>() == 8 && isInteger<T>())
        return wastrumentation_stack_store_i64(ptr, value, offset);
    else if (sizeof<T>() == 8 && isFloat<T>())
        return wastrumentation_stack_store_f64(ptr, value, offset);
    unreachable();
}

class MutDynArgsResults {
    argc: i32;
    resc: i32;
    sigv: i32;
    sigtypv: i32;
    ressOffsetTo: i32[];
    argsOffsetTo: i32[];

    constructor(argc: i32, resc: i32, sigv: i32, sigtypv: i32) {
        this.argc = argc;
        this.resc = resc;
        this.sigv = sigv;
        this.sigtypv = sigtypv;

        /**
         *   <4>   <4>     <8>     <4>
         * |-i32-|-i32-|---f64---|-i32-|
         *   ___res___   _____arg_____
         * ressOffsetTo == [ 0,  4] => accessing res0 requires no offset, res1 requires offset 4
         * argsOffsetTo == [ 8, 16] => accessing arg0 requires offset 8, arg1 requires offset 16
         */
        let offset = 0;
        this.ressOffsetTo = [];
        this.argsOffsetTo = [];
        for(let type_index = 0; type_index < resc; type_index++) {
            this.ressOffsetTo.push(offset);
            switch(wastrumentation_memory_load<i32>(sigtypv, (0 + type_index)*sizeof<i32>())) {
                case WasmType.i32:
                    offset += sizeof<i32>();
                    break;
                case WasmType.f32:
                    offset += sizeof<f32>();
                    break;
                case WasmType.i64:
                    offset += sizeof<i64>();
                    break;
                case WasmType.f64:
                    offset += sizeof<f64>();
                    break;
                default:
                    unreachable();
            }
        }
        const offsetToArgs = offset;
        offset = 0;
        for(let type_index = 0; type_index < argc; type_index++) {
            this.argsOffsetTo.push(offsetToArgs + offset);
            switch(wastrumentation_memory_load<i32>(sigtypv, (resc + type_index)*sizeof<i32>())) {
                case WasmType.i32:
                    offset += sizeof<i32>();
                    break;
                case WasmType.f32:
                    offset += sizeof<f32>();
                    break;
                case WasmType.i64:
                    offset += sizeof<i64>();
                    break;
                case WasmType.f64:
                    offset += sizeof<f64>();
                    break;
                default:
                    unreachable();
            }
        }
    }

    checkBounds(count: i32, index: i32): void {
        if(
            (!(index >= 0)) ||  // negative index
            (index >= count)    // index out of bounds
        ) unreachable();
    }

    getArg<T>(index: i32): T {
        this.checkBounds(this.argc, index);
        return wastrumentation_memory_load<T>(this.sigv, this.argsOffsetTo[index]);
    }

    setArg<T>(index: i32, value: T): void {
        this.checkBounds(this.argc, index);
        wastrumentation_memory_store<T>(this.sigv, value, this.argsOffsetTo[index]);
    }

    getRes<T>(index: i32): T {
        this.checkBounds(this.resc, index);
        return wastrumentation_memory_load<T>(this.sigv, this.ressOffsetTo[index]);
    }

    setRes<T>(index: i32, value: T): void {
        this.checkBounds(this.resc, index);
        wastrumentation_memory_store<T>(this.sigv, value, this.ressOffsetTo[index]);
    }

    getArgType(index: i32): WasmType {
        this.checkBounds(this.argc, index);
        const serialized_type: i32 = wastrumentation_memory_load<i32>(
            this.sigtypv,
            (this.resc + index)*sizeof<i32>(),
        );
        return serialized_type as i32;
    }

    getResType(index: i32): WasmType {
        this.checkBounds(this.resc, index);
        const serialized_type: i32 = wastrumentation_memory_load<i32>(
            this.sigtypv,
            (0 + index)*sizeof<i32>(),
        );
        return serialized_type as i32;
    }
}

abstract class DynValues {
    protected mutDynArgsResults: MutDynArgsResults;
    readonly length: i32;

    constructor(mutDynArgsResults: MutDynArgsResults) {
        this.mutDynArgsResults = mutDynArgsResults;
    }

    abstract get<T>(index: i32): T
    abstract getType(index: i32): WasmType
    abstract set<T>(index: i32, value: T): void
}

class MutDynArgs extends DynValues {
    readonly length: i32;

    constructor(mutDynArgsResults: MutDynArgsResults) {
        super(mutDynArgsResults);
        this.length = mutDynArgsResults.argc;
    }

    get<T>(index: i32): T  {
        return this.mutDynArgsResults.getArg<T>(index);
    }

    getType(index: i32): WasmType {
        return this.mutDynArgsResults.getArgType(index);
    }

    set<T>(index: i32, value: T): void  {
        this.mutDynArgsResults.setArg<T>(index, value);
    }
}

class MutDynRess extends DynValues {
    readonly length: i32;

    constructor(mutDynArgsResults: MutDynArgsResults) {
        super(mutDynArgsResults);
        this.length = mutDynArgsResults.resc;
    }

    get<T>(index: i32): T  {
        return this.mutDynArgsResults.getRes<T>(index);
    }

    getType(index: i32): WasmType {
        return this.mutDynArgsResults.getResType(index);
    }

    set<T>(index: i32, value: T): void  {
        this.mutDynArgsResults.setRes<T>(index, value);
    }
}

@external("instrumented_input", "call_base")
declare function call_base(f_apply: i32, sigv: i32): void

class WasmFunction {
    f_apply: i32;
    instr_f_idx: i32;
    sigv: i32;

    constructor(f_apply: i32, instr_f_idx: i32, sigv: i32) {
        this.f_apply = f_apply;
        this.instr_f_idx = instr_f_idx;
        this.sigv = sigv;
    }

    apply(): void {
        call_base(this.f_apply, this.sigv);
    }
}
