use wastrumentation_rs_stdlib::*;
use crate::{ ShadowMeta, ShadowState, ShadowValue };

pub struct LocationContext<'a> {
    pub location: &'a Location,
}

pub struct UnaryTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub op: &'a UnaryOperator,
    pub operand: &'a ShadowValue<M>,
    pub result: &'a mut ShadowValue<M>,
    pub location: &'a Location,
}

pub struct BinaryTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub op: &'a BinaryOperator,
    pub lhs: &'a ShadowValue<M>,
    pub rhs: &'a ShadowValue<M>,
    pub result: &'a mut ShadowValue<M>,
    pub location: &'a Location,
}

pub struct ConstTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub value: &'a mut ShadowValue<M>,
    pub location: &'a Location,
}

pub struct LoadTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub index: &'a LoadIndex,
    pub offset: &'a LoadOffset,
    pub operation: &'a LoadOperation,
    pub result: &'a mut ShadowValue<M>,
    pub location: &'a Location,
}

pub struct StoreTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub index: &'a StoreIndex,
    pub value: &'a mut ShadowValue<M>,
    pub pointer: &'a mut ShadowValue<M>,
    pub offset: &'a StoreOffset,
    pub operation: &'a StoreOperation,
    pub location: &'a Location,
}

pub struct MemorySizeTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub size: &'a mut ShadowValue<M>,
    pub index: &'a MemoryIndex,
    pub location: &'a Location,
}

pub struct MemoryGrowTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub amount: &'a mut ShadowValue<M>,
    pub index: &'a MemoryIndex,
    pub location: &'a Location,
}

pub struct MemoryInitTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct MemoryCopyTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct MemoryFillTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct LocalTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub value: &'a mut ShadowValue<M>,
    pub index: &'a LocalIndex,
    pub op: &'a LocalOp,
    pub location: &'a Location,
}

pub struct GlobalTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub value: &'a mut ShadowValue<M>,
    pub index: &'a GlobalIndex,
    pub op: &'a GlobalOp,
    pub location: &'a Location,
}

pub struct DropTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct SelectTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub path: &'a PathContinuation,
    pub location: &'a Location,
}

pub struct ApplyBeforeTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub function: &'a WasmFunction,
    pub args: &'a MutDynArgs,
    pub results: &'a MutDynResults,
}

pub struct ApplyAfterTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub function: &'a WasmFunction,
    pub args: &'a MutDynArgs,
    pub results: &'a MutDynResults,
}

pub struct CallToImportedBeforeTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub arg_count: usize,
}

pub struct CallToImportedAfterTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub result_count: usize,
}

pub struct CallPreTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub target: &'a FunctionIndex,
    pub location: &'a Location,
}

pub struct CallPostTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub target: &'a FunctionIndex,
    pub location: &'a Location,
}

pub struct CallIndirectPreTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub target: &'a FunctionTableIndex,
    pub table: &'a FunctionTable,
    pub location: &'a Location,
}

pub struct CallIndirectPostTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub target: &'a FunctionTable,
    pub location: &'a Location,
}

pub struct BrTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub target: &'a BranchTargetLabel,
    pub location: &'a Location,
}

pub struct BrIfTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub condition: &'a ParameterBrIfCondition,
    pub target: &'a ParameterBrIfLabel,
    pub location: &'a Location,
}

pub struct BrTableTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub target: &'a BranchTableTarget,
    pub effective: &'a BranchTableEffective,
    pub default: &'a BranchTableDefault,
    pub location: &'a Location,
}

pub struct ReturnTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct BlockPreTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub input_count: &'a BlockInputCount,
    pub arity: &'a BlockArity,
    pub location: &'a Location,
}

pub struct BlockPostTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct LoopPreTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub input_count: &'a LoopInputCount,
    pub arity: &'a LoopArity,
    pub location: &'a Location,
}

pub struct LoopPostTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct IfThenElseTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub path: &'a PathContinuation,
    pub input_count: &'a IfThenElseInputCount,
    pub arity: &'a IfThenElseArity,
    pub location: &'a Location,
}

pub struct IfThenElsePostTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}

pub struct IfThenTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub path: &'a PathContinuation,
    pub input_count: &'a IfThenInputCount,
    pub arity: &'a IfThenArity,
    pub location: &'a Location,
}

pub struct IfThenPostTrapContext<'a, M: ShadowMeta> {
    pub state: &'a mut ShadowState<M>,
    pub location: &'a Location,
}
