use super::TransformationStrategy;
use crate::parse_nesting::{
    BodyInner, HighLevelBody, HighLevelInstr as Instr, TypedHighLevelInstr,
};
use wasabi_wasm::{Function, GlobalOp, Idx, Module, Mutability, RefType, Val, ValType};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    RefFunc(Idx<Function>),
    RefNull(Idx<Function>),
    RefIsNull(Idx<Function>),
    TableGet(Idx<Function>),
    TableSet(Idx<Function>),
    TableSize(Idx<Function>),
    TableCopy {
        trap_idx: Idx<Function>,
        get_src_idx: Idx<Function>,
        get_dst_idx: Idx<Function>,
        get_size_idx: Idx<Function>,
    },
    TableGrow(Idx<Function>),
    TableFill(Idx<Function>),
    TableInit {
        trap_idx: Idx<Function>,
        get_elem_source_idx: Idx<Function>,
        get_table_destination_idx: Idx<Function>,
        get_size_idx: Idx<Function>,
    },
    ElemDrop(Idx<Function>),
}

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody, module: &mut Module) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = transform(body, *self, module);
        HighLevelBody(transformed_body)
    }
}

fn transform(body: &BodyInner, target: Target, module: &mut Module) -> BodyInner {
    let mut result = Vec::new();
    let global_ref_store = module.add_global(
        ValType::Ref(RefType::FuncRef),
        Mutability::Mut,
        vec![
            wasabi_wasm::Instr::RefNull(RefType::FuncRef),
            wasabi_wasm::Instr::End,
        ],
    );
    let global_i32_store = module.add_global(
        ValType::I32,
        Mutability::Mut,
        vec![
            wasabi_wasm::Instr::Const(Val::I32(0)),
            wasabi_wasm::Instr::End,
        ],
    );

    for typed_instr @ TypedHighLevelInstr { instr, .. } in body {
        if typed_instr.is_uninstrumented() {
            match (target, instr) {
                // ref.func x: [] -> [ref]
                (Target::RefFunc(trap_idx), Instr::RefFunc(func_idx)) => {
                    result.extend_from_slice(&[
                        // Stack: []
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(func_idx.to_u32()).unwrap(),
                        ))),
                        // Stack: [func_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: []
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: [ref]
                    continue;
                }

                // ref.null x: [] -> [ref]
                (Target::RefNull(trap_idx), Instr::RefNull(_ref_type)) => {
                    // Stack: []
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: []
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: [ref]
                    continue;
                }

                // ref.is_null: [ref] -> [i]
                (Target::RefIsNull(trap_idx), Instr::RefIsNull) => {
                    // Stack: [ref]
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: [res:I32]
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: [res_new:I32]
                    continue;
                }

                // table.get x: [i] -> [ref]
                (Target::TableGet(trap_idx), Instr::TableGet(table_idx)) => {
                    result.extend_from_slice(&[
                        // Stack: [i:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(table_idx.to_u32()).unwrap(),
                        ))),
                        // Stack: [i:I32, table_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    result.extend_from_slice(&[typed_instr.place_original(instr.clone())]);
                    // Stack: [ref]
                    continue;
                }

                // table.set x: [i, ref] -> []
                (Target::TableSet(trap_idx), Instr::TableSet(table_idx)) => {
                    result.extend_from_slice(&[
                        // Stack: [i:I32, ref]
                        // temporarily store ref in global ref store
                        typed_instr.instrument_with(Instr::Global(GlobalOp::Set, global_ref_store)),
                        // Stack: [i:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(table_idx.to_u32()).unwrap(),
                        ))),
                        // Stack: [i:I32, table_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: [i:I32]
                    // retrieve ref from global ref store
                    result.push(
                        typed_instr.instrument_with(Instr::Global(GlobalOp::Get, global_ref_store)),
                    );
                    // Stack: [i:I32, ref]
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: []
                    continue;
                }

                // table.size x: [] -> [size]
                (Target::TableSize(trap_idx), Instr::TableSize(table_idx)) => {
                    result.extend_from_slice(&[
                        // Stack: []
                        typed_instr.place_original(instr.clone()),
                        // Stack: [size:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(table_idx.to_u32()).unwrap(),
                        ))),
                        // Stack: [size:I32, table_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: [new_size:I32]
                    continue;
                }

                // table.grow x: [ref, n] -> [prev_size_or_-1]
                (Target::TableGrow(trap_idx), Instr::TableGrow(table_idx)) => {
                    result.extend_from_slice(&[
                        // Stack: [ref, n:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(table_idx.to_u32()).unwrap(),
                        ))),
                        // Stack: [ref, n:I32, table_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: [ref, n_new:I32]
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: [prev_size_or_-1:I32]
                    continue;
                }

                // table.fill x: [i, val, n] -> []
                (Target::TableFill(trap_idx), Instr::TableFill(table_idx)) => {
                    result.extend_from_slice(&[
                        // Stack: [i:I32, val, n:I32]
                        typed_instr.instrument_with(Instr::Global(GlobalOp::Set, global_i32_store)),
                        // Stack: [i:I32, val]
                        typed_instr.instrument_with(Instr::Global(GlobalOp::Set, global_ref_store)),
                        // Stack: [i:I32]
                        typed_instr.instrument_with(Instr::Global(GlobalOp::Get, global_i32_store)),
                        // Stack: [i:I32, n:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(table_idx.to_u32()).unwrap(),
                        ))),
                        // Stack: [i:I32, n:I32, table_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: [i_new:I32]
                    result.extend_from_slice(&[
                        typed_instr.instrument_with(Instr::Global(GlobalOp::Get, global_ref_store)),
                        // Stack: [i_new:I32, val]
                        typed_instr.instrument_with(Instr::Global(GlobalOp::Get, global_i32_store)),
                        // Stack: [i_new:I32, val, n:I32]
                    ]);
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: []
                    continue;
                }

                // table.copy x1 x2: [d, s, n] -> []
                (
                    Target::TableCopy {
                        trap_idx,
                        get_src_idx,
                        get_dst_idx,
                        get_size_idx,
                    },
                    Instr::TableCopy(dst, src),
                ) => {
                    result.extend_from_slice(&[
                        // Stack: [d:I32, s:I32, n:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(dst.to_u32()).unwrap(),
                        ))),
                        // Stack: [d:I32, s:I32, n:I32, dst_idx:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(src.to_u32()).unwrap(),
                        ))),
                        // Stack: [d:I32, s:I32, n:I32, dst_idx:I32, src_idx:I32]
                    ]);

                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: []

                    // load d_new, s_new, n_new from analysis exports
                    result.extend_from_slice(&[
                        // Stack: []
                        typed_instr.instrument_with(Instr::Call(get_dst_idx)),
                        // Stack: [d_new:I32]
                        typed_instr.instrument_with(Instr::Call(get_src_idx)),
                        // Stack: [d_new:I32, s_new:I32]
                        typed_instr.instrument_with(Instr::Call(get_size_idx)),
                        // Stack: [d_new:I32, s_new:I32, n_new:I32]
                    ]);

                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: []
                    continue;
                }

                // table.init x y: [i, j, n] -> []
                (
                    Target::TableInit {
                        trap_idx,
                        get_elem_source_idx,
                        get_table_destination_idx,
                        get_size_idx,
                    },
                    Instr::TableInit(table, elem),
                ) => {
                    result.extend_from_slice(&[
                        // Stack: [i:I32, j:I32, n:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(table.to_u32()).unwrap(),
                        ))),
                        // Stack: [i:I32, j:I32, n:I32, table_idx:I32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(elem.to_u32()).unwrap(),
                        ))),
                        // Stack: [i:I32, j:I32, n:I32, table_idx:I32, elem_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: []

                    // load i_new, j_new, n_new from analysis exports
                    result.extend_from_slice(&[
                        // Stack: []
                        typed_instr.instrument_with(Instr::Call(get_elem_source_idx)),
                        // Stack: [i_new:I32]
                        typed_instr.instrument_with(Instr::Call(get_table_destination_idx)),
                        // Stack: [i_new:I32, j_new:I32]
                        typed_instr.instrument_with(Instr::Call(get_size_idx)),
                        // Stack: [i_new:I32, j_new:I32, n_new:I32]
                    ]);
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: []
                    continue;
                }

                // elem.drop x: [] -> []
                (Target::ElemDrop(trap_idx), Instr::ElemDrop(elem_idx)) => {
                    println!("Instrumenting elem.drop for elem segment index");
                    result.extend_from_slice(&[
                        // Stack: []
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(elem_idx.to_u32()).unwrap(),
                        ))),
                        // Stack: [elem_idx:I32]
                    ]);
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // Stack: []
                    result.push(typed_instr.place_original(instr.clone()));
                    // Stack: []
                    continue;
                }
                _ => {}
            }
        }

        // Default traversal
        match (target, instr) {
            (target, Instr::If(type_, then, None)) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target, module),
                    None,
                )));
            }
            (target, Instr::If(type_, then, Some(else_))) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target, module),
                    Some(transform(else_, target, module)),
                )))
            }
            (target, Instr::Loop(type_, body)) => {
                result.push(
                    typed_instr
                        .place_untouched(Instr::Loop(*type_, transform(body, target, module))),
                );
            }
            (target, Instr::Block(type_, body)) => {
                result.push(
                    typed_instr
                        .place_untouched(Instr::Block(*type_, transform(body, target, module))),
                );
            }
            (_, instr) => result.push(typed_instr.place_untouched(instr.clone())),
        }
    }
    result
}
