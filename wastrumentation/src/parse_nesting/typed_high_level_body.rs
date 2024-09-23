use crate::parse_nesting::typed_indexed_instr::{type_inference_function, TypedIndexedInstr};
use wasabi_wasm::types::InferredInstructionType;
use wasabi_wasm::{
    BinaryOp, Code, Data, Element, Function, FunctionType, Global, GlobalOp, Idx, Label, LoadOp,
    Local, LocalOp, Memarg, Memory, Module, RefType, StoreOp, Table, UnaryOp, Val, ValType,
};
use LowToHighError::{
    BodyNonEndTermination, EndWithoutParent, ExcessiveEnd, IfDidNotPrecedeElse, TrivialCastAttempt,
    TypeInference,
};

use super::typed_high_level_body_error::LowToHighError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Body(pub Vec<TypedHighLevelInstr>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypedHighLevelInstr {
    index: usize,
    type_: InferredInstructionType,
    instr: Instr,
}

/// Equal to `wasabi_wasm::Instr` minus `Else` and `End` instruction
/// Which occur in `Block`, `Loop` and `If`
/// Cfr. [Control Instructions](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Instr {
    Unreachable,
    Nop,

    Block(FunctionType, Body),
    Loop(FunctionType, Body),
    If(FunctionType, Body, Option<Body>),

    Br(Label),
    BrIf(Label),
    BrTable { table: Box<[Label]>, default: Label },

    Return,
    Call(Idx<Function>),
    CallIndirect(FunctionType, Idx<Table>),

    RefNull(RefType),
    RefIsNull,
    RefFunc(Idx<Function>),

    Drop,
    Select,
    TypedSelect(ValType),

    Local(LocalOp, Idx<Local>),
    Global(GlobalOp, Idx<Global>),

    TableGet(Idx<Table>),
    TableSet(Idx<Table>),
    TableSize(Idx<Table>),
    TableGrow(Idx<Table>),
    TableFill(Idx<Table>),
    TableCopy(Idx<Table>, Idx<Table>),
    TableInit(Idx<Table>, Idx<Element>),
    ElemDrop(Idx<Element>),

    Load(LoadOp, Memarg),
    Store(StoreOp, Memarg),

    MemorySize(Idx<Memory>),
    MemoryGrow(Idx<Memory>),
    MemoryFill,
    MemoryCopy,
    MemoryInit(Idx<Data>),
    DataDrop(Idx<Data>),

    Const(Val),
    Unary(UnaryOp),
    Binary(BinaryOp),
}

impl Instr {
    #[must_use]
    pub fn if_then(type_: FunctionType, then: Body) -> Self {
        Instr::If(type_, then, None)
    }

    #[must_use]
    pub fn if_then_else(type_: FunctionType, then: Body, else_: Body) -> Self {
        Instr::If(type_, then, Some(else_))
    }
}

impl TryFrom<wasabi_wasm::Instr> for Instr {
    type Error = LowToHighError;

    fn try_from(value: wasabi_wasm::Instr) -> Result<Self, Self::Error> {
        Ok(match value {
            // Happy path
            wasabi_wasm::Instr::Unreachable => Instr::Unreachable,
            wasabi_wasm::Instr::Nop => Instr::Nop,
            wasabi_wasm::Instr::Br(v) => Instr::Br(v),
            wasabi_wasm::Instr::BrIf(v) => Instr::BrIf(v),
            wasabi_wasm::Instr::BrTable { table, default } => Instr::BrTable { table, default },
            wasabi_wasm::Instr::Return => Instr::Return,
            wasabi_wasm::Instr::Call(v) => Instr::Call(v),
            wasabi_wasm::Instr::CallIndirect(v1, v2) => Instr::CallIndirect(v1, v2),
            wasabi_wasm::Instr::RefNull(v) => Instr::RefNull(v),
            wasabi_wasm::Instr::RefIsNull => Instr::RefIsNull,
            wasabi_wasm::Instr::RefFunc(v) => Instr::RefFunc(v),
            wasabi_wasm::Instr::Drop => Instr::Drop,
            wasabi_wasm::Instr::Select => Instr::Select,
            wasabi_wasm::Instr::TypedSelect(v) => Instr::TypedSelect(v),
            wasabi_wasm::Instr::Local(v1, v2) => Instr::Local(v1, v2),
            wasabi_wasm::Instr::Global(v1, v2) => Instr::Global(v1, v2),
            wasabi_wasm::Instr::TableGet(v) => Instr::TableGet(v),
            wasabi_wasm::Instr::TableSet(v) => Instr::TableSet(v),
            wasabi_wasm::Instr::TableSize(v) => Instr::TableSize(v),
            wasabi_wasm::Instr::TableGrow(v) => Instr::TableGrow(v),
            wasabi_wasm::Instr::TableFill(v) => Instr::TableFill(v),
            wasabi_wasm::Instr::TableCopy(v1, v2) => Instr::TableCopy(v1, v2),
            wasabi_wasm::Instr::TableInit(v1, v2) => Instr::TableInit(v1, v2),
            wasabi_wasm::Instr::ElemDrop(v) => Instr::ElemDrop(v),
            wasabi_wasm::Instr::Load(v1, v2) => Instr::Load(v1, v2),
            wasabi_wasm::Instr::Store(v1, v2) => Instr::Store(v1, v2),
            wasabi_wasm::Instr::MemorySize(v) => Instr::MemorySize(v),
            wasabi_wasm::Instr::MemoryGrow(v) => Instr::MemoryGrow(v),
            wasabi_wasm::Instr::MemoryFill => Instr::MemoryFill,
            wasabi_wasm::Instr::MemoryCopy => Instr::MemoryCopy,
            wasabi_wasm::Instr::MemoryInit(v) => Instr::MemoryInit(v),
            wasabi_wasm::Instr::DataDrop(v) => Instr::DataDrop(v),
            wasabi_wasm::Instr::Const(v) => Instr::Const(v),
            wasabi_wasm::Instr::Unary(v) => Instr::Unary(v),
            wasabi_wasm::Instr::Binary(v) => Instr::Binary(v),
            // Sad path
            wasabi_wasm::Instr::Block(_)
            | wasabi_wasm::Instr::Loop(_)
            | wasabi_wasm::Instr::If(_)
            | wasabi_wasm::Instr::Else
            | wasabi_wasm::Instr::End => return Err(TrivialCastAttempt),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LowLevelBody(pub Vec<wasabi_wasm::Instr>);

impl TryFrom<(&Module, &Function, &Code)> for Body {
    type Error = LowToHighError;

    fn try_from(module_function_code: (&Module, &Function, &Code)) -> Result<Self, Self::Error> {
        let (module, function, code) = module_function_code;
        let indexed_typed_body = type_inference_function(function, code, module)
            .map_err(|e| TypeInference { type_error: e })?;

        enum Entered {
            Block {
                index: usize,
                type_: FunctionType,
            },
            Loop {
                index: usize,
                type_: FunctionType,
            },
            IfStart {
                index: usize,
                type_: FunctionType,
            },
            IfThenElse {
                index: usize,
                type_: FunctionType,
                then_body: Body,
            },
        }

        let [instructions @ .., TypedIndexedInstr {
            index: _,
            type_: _,
            instr: wasabi_wasm::Instr::End,
        }] = &indexed_typed_body[..]
        else {
            return Err(BodyNonEndTermination);
        };

        let mut entered_stack: Vec<Entered> = Vec::new();
        let mut body_stack: Vec<Vec<TypedHighLevelInstr>> = Vec::new();
        let mut current_body: Vec<TypedHighLevelInstr> = Vec::new();

        for TypedIndexedInstr {
            index,
            type_,
            instr,
        } in instructions
        {
            match instr {
                wasabi_wasm::Instr::Block(type_) => {
                    entered_stack.push(Entered::Block {
                        index: *index,
                        type_: *type_,
                    });
                    body_stack.push(current_body.clone());
                    current_body = Vec::new();
                }
                wasabi_wasm::Instr::Loop(type_) => {
                    entered_stack.push(Entered::Loop {
                        index: *index,
                        type_: *type_,
                    });
                    body_stack.push(current_body.clone());
                    current_body = Vec::new();
                }
                wasabi_wasm::Instr::If(type_) => {
                    entered_stack.push(Entered::IfStart {
                        index: *index,
                        type_: *type_,
                    });
                    body_stack.push(current_body.clone());
                    current_body = Vec::new();
                }
                wasabi_wasm::Instr::Else => match entered_stack.pop() {
                    Some(Entered::IfStart { index, type_ }) => {
                        let then_body = Body(current_body.clone());
                        entered_stack.push(Entered::IfThenElse {
                            index,
                            type_,
                            then_body,
                        });
                        current_body = Vec::new();
                    }
                    _ => return Err(IfDidNotPrecedeElse),
                },
                wasabi_wasm::Instr::End => {
                    let ended_body = current_body.clone();
                    let (begin_idx, instruction) = match entered_stack.pop().ok_or(ExcessiveEnd)? {
                        Entered::Block { index, type_ } => {
                            (index, Instr::Block(type_, Body(ended_body)))
                        }
                        Entered::Loop { index, type_ } => {
                            (index, Instr::Loop(type_, Body(ended_body)))
                        }
                        Entered::IfStart { index, type_ } => {
                            (index, Instr::if_then(type_, Body(ended_body)))
                        }
                        Entered::IfThenElse {
                            index,
                            type_,
                            then_body,
                        } => (
                            index,
                            Instr::if_then_else(type_, then_body, Body(ended_body)),
                        ),
                    };
                    current_body = body_stack.pop().ok_or(EndWithoutParent)?;
                    current_body.push(TypedHighLevelInstr {
                        index: begin_idx, // prefer the begin-index over the end-index!
                        type_: type_.clone(),
                        instr: instruction,
                    });
                }
                instruction => current_body.push(TypedHighLevelInstr {
                    index: *index,
                    type_: type_.clone(),
                    instr: instruction.clone().try_into()?,
                }),
            };
        }

        assert!(entered_stack.is_empty());
        assert!(body_stack.is_empty());
        Ok(Body(current_body))
    }
}

impl From<Body> for LowLevelBody {
    fn from(high_level_body: Body) -> Self {
        let mut low_level_body = Self::from_recurse(high_level_body);
        low_level_body.push(wasabi_wasm::Instr::End);
        Self(low_level_body)
    }
}

// TODO macro this?
impl LowLevelBody {
    fn from_recurse(instructions: Body) -> Vec<wasabi_wasm::Instr> {
        let Body(instructions) = instructions;
        let mut result = Vec::with_capacity(instructions.len());
        for TypedHighLevelInstr {
            index: _,
            type_: _,
            instr,
        } in instructions
        {
            match instr {
                // interesting
                Instr::Block(type_, body_) => {
                    result.push(wasabi_wasm::Instr::Block(type_));
                    result.extend(Self::from_recurse(body_));
                    result.push(wasabi_wasm::Instr::End);
                }
                Instr::Loop(type_, body_) => {
                    result.push(wasabi_wasm::Instr::Loop(type_));
                    result.extend(Self::from_recurse(body_));
                    result.push(wasabi_wasm::Instr::End);
                }
                Instr::If(type_, then, Some(else_)) => {
                    result.push(wasabi_wasm::Instr::If(type_));
                    result.extend(Self::from_recurse(then));
                    result.push(wasabi_wasm::Instr::Else);
                    result.extend(Self::from_recurse(else_));
                    result.push(wasabi_wasm::Instr::End);
                }
                Instr::If(type_, then, None) => {
                    result.push(wasabi_wasm::Instr::If(type_));
                    result.extend(Self::from_recurse(then));
                    result.push(wasabi_wasm::Instr::End);
                }

                // rest is not interesting, just push in result
                Instr::Unreachable => result.push(wasabi_wasm::Instr::Unreachable),
                Instr::Nop => result.push(wasabi_wasm::Instr::Nop),
                Instr::Br(v) => result.push(wasabi_wasm::Instr::Br(v)),
                Instr::BrIf(v) => result.push(wasabi_wasm::Instr::BrIf(v)),
                Instr::BrTable { table, default } => {
                    result.push(wasabi_wasm::Instr::BrTable { table, default });
                }
                Instr::Return => result.push(wasabi_wasm::Instr::Return),
                Instr::Call(v) => result.push(wasabi_wasm::Instr::Call(v)),
                Instr::CallIndirect(v1, v2) => {
                    result.push(wasabi_wasm::Instr::CallIndirect(v1, v2));
                }
                Instr::RefNull(v) => result.push(wasabi_wasm::Instr::RefNull(v)),
                Instr::RefIsNull => result.push(wasabi_wasm::Instr::RefIsNull),
                Instr::RefFunc(v) => result.push(wasabi_wasm::Instr::RefFunc(v)),
                Instr::Drop => result.push(wasabi_wasm::Instr::Drop),
                Instr::Select => result.push(wasabi_wasm::Instr::Select),
                Instr::TypedSelect(v) => result.push(wasabi_wasm::Instr::TypedSelect(v)),
                Instr::Local(v1, v2) => result.push(wasabi_wasm::Instr::Local(v1, v2)),
                Instr::Global(v1, v2) => result.push(wasabi_wasm::Instr::Global(v1, v2)),
                Instr::TableGet(v) => result.push(wasabi_wasm::Instr::TableGet(v)),
                Instr::TableSet(v) => result.push(wasabi_wasm::Instr::TableSet(v)),
                Instr::TableSize(v) => result.push(wasabi_wasm::Instr::TableSize(v)),
                Instr::TableGrow(v) => result.push(wasabi_wasm::Instr::TableGrow(v)),
                Instr::TableFill(v) => result.push(wasabi_wasm::Instr::TableFill(v)),
                Instr::TableCopy(v1, v2) => result.push(wasabi_wasm::Instr::TableCopy(v1, v2)),
                Instr::TableInit(v1, v2) => result.push(wasabi_wasm::Instr::TableInit(v1, v2)),
                Instr::ElemDrop(v) => result.push(wasabi_wasm::Instr::ElemDrop(v)),
                Instr::Load(v1, v2) => result.push(wasabi_wasm::Instr::Load(v1, v2)),
                Instr::Store(v1, v2) => result.push(wasabi_wasm::Instr::Store(v1, v2)),
                Instr::MemorySize(v) => result.push(wasabi_wasm::Instr::MemorySize(v)),
                Instr::MemoryGrow(v) => result.push(wasabi_wasm::Instr::MemoryGrow(v)),
                Instr::MemoryFill => result.push(wasabi_wasm::Instr::MemoryFill),
                Instr::MemoryCopy => result.push(wasabi_wasm::Instr::MemoryCopy),
                Instr::MemoryInit(v) => result.push(wasabi_wasm::Instr::MemoryInit(v)),
                Instr::DataDrop(v) => result.push(wasabi_wasm::Instr::DataDrop(v)),
                Instr::Const(v) => result.push(wasabi_wasm::Instr::Const(v)),
                Instr::Unary(v) => result.push(wasabi_wasm::Instr::Unary(v)),
                Instr::Binary(v) => result.push(wasabi_wasm::Instr::Binary(v)),
            }
        }
        result
    }
}
