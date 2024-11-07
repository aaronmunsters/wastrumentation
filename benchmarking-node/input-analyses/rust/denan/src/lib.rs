use wastrumentation_rs_stdlib::*;

trait Denan {
    fn denan(self) -> Self;
}

impl Denan for WasmValue {
    fn denan(self) -> Self {
        match self.type_() {
            WasmType::I32 | WasmType::I64 => self,
            WasmType::F32 => self.as_f32().is_nan().then(|| 0_f32.into()).unwrap_or(self),
            WasmType::F64 => self.as_f64().is_nan().then(|| 0_f64.into()).unwrap_or(self),
        }
    }
}

// Target program events: all that [operate on | return a] WasmValue
advice! {
    apply (func: WasmFunction, args: MutDynArgs, ress: MutDynResults) {
         args.update_each_arg(|_index, v| v.denan());
         func.apply();
         ress.update_each_res(|_index, v| v.denan());
    }
    unary (opt: UnaryOperator, opnd: WasmValue, _l: Location) { opt.apply(opnd.denan()).denan() }
    binary (opt: BinaryOperator, l_opnd: WasmValue, r_opnd: WasmValue, _l: Location) { opt.apply(l_opnd.denan(), r_opnd.denan()).denan() }
    const_ (v: WasmValue, _l: Location) { v.denan() }
    local (v: WasmValue, _i: LocalIndex, _l: LocalOp, _l: Location) { v.denan() }
    global (v: WasmValue, _i: GlobalIndex, _g: GlobalOp, _l: Location) { v.denan() }
    load (i: LoadIndex, o: LoadOffset, op: LoadOperation, _l: Location) { op.perform(&i, &o).denan() }
    store (i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation, _l: Location) { op.perform(&i, &v.denan(), &o); }
}

// Source of original pass:
// binaryen/src/passes/DeNaN.cpp
// --> https://github.com/WebAssembly/binaryen/blob/39bf87eb39543ca14198a16533f262a147816793/src/passes/DeNaN.cpp
