const NO_OFFSET: i32 = 0;

// ENUMS for types
const TYPE_I32: i32 = 0;
const TYPE_F32: i32 = 1;
const TYPE_I64: i32 = 2;
const TYPE_F64: i32 = 3;

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
        for(let type_index = 0; type_index < argc; type_index++) {
            this.ressOffsetTo.push(offset);
            switch(load<i32>(sigtypv + (type_index*sizeof<i32>()))) {
                case TYPE_I32:
                    offset += sizeof<i32>();
                    break;
                case TYPE_F32:
                    offset += sizeof<f32>();
                    break;
                case TYPE_I64:
                    offset += sizeof<i64>();
                    break;
                case TYPE_F64:
                    offset += sizeof<f64>();
                    break;
                default:
                    unreachable();
            }
        }
        const offsetToArgs = offset;
        offset = 0;
        for(let type_index = 0; type_index < resc; type_index++) {
            this.argsOffsetTo.push(offsetToArgs + offset);
            switch(load<i32>(sigtypv + ((argc + type_index)*sizeof<i32>()))) {
                case TYPE_I32:
                    offset += sizeof<i32>();
                    break;
                case TYPE_F32:
                    offset += sizeof<f32>();
                    break;
                case TYPE_I64:
                    offset += sizeof<i64>();
                    break;
                case TYPE_F64:
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
        return load<T>(this.sigv + this.argsOffsetTo[index], NO_OFFSET);
    }

    setArg<T>(index: i32, value: T): void {
        this.checkBounds(this.argc, index);
        store<T>(this.sigv + this.argsOffsetTo[index], value, NO_OFFSET);
    }

    getRes<T>(index: i32): T {
        this.checkBounds(this.resc, index);
        return load<T>(this.sigv + this.ressOffsetTo[index], NO_OFFSET);
    }

    setRes<T>(index: i32, value: T): void {
        this.checkBounds(this.resc, index);
        store<T>(this.sigv + this.ressOffsetTo[index], value, NO_OFFSET);
    }
}

class MutDynArgs {
    mutDynArgsResults: MutDynArgsResults;

    constructor(mutDynArgsResults: MutDynArgsResults) {
        this.mutDynArgsResults = mutDynArgsResults;
    }

    get<T>(index: i32): T  {
        return this.mutDynArgsResults.getArg<T>(index)
    }

    set<T>(index: i32, value: T): void  {
        this.mutDynArgsResults.setArg<T>(index, value);
    }
}

class MutDynRess {
    mutDynArgsResults: MutDynArgsResults;

    constructor(mutDynArgsResults: MutDynArgsResults) {
        this.mutDynArgsResults = mutDynArgsResults;
    }

    get<T>(index: i32): T  {
        return this.mutDynArgsResults.getRes<T>(index)
    }

    set<T>(index: i32, value: T): void  {
        this.mutDynArgsResults.setRes<T>(index, value);
    }
}

@external("instrumented_input", "call_base")
declare function call_base(f_apply: i32, sigv: i32): void

class WasmFunction {
    f_apply: i32;
    sigv: i32;

    constructor(f_apply: i32, sigv: i32) {
        this.f_apply = f_apply;
        this.sigv = sigv;
    }

    apply(): void {
        call_base(this.f_apply, this.sigv);
    }
}
