use pest::Span;
use pest_ast::FromPest;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "wasp.pest"]
pub struct WaspParser;

fn span_into_string(span: Span<'_>) -> &str {
    span.as_str()
}

#[derive(Debug, PartialEq, Eq)]
pub enum CallQualifier {
    Pre,
    Post,
}

fn span_into_qualifier(span: Span) -> CallQualifier {
    match span.as_str() {
        "pre" => CallQualifier::Pre,
        "post" => CallQualifier::Post,
        &_ => panic!("Could not parse `pre` or `post`"),
    }
}

fn drop_guest_delimiter(guest_code: &str) -> &str {
    guest_code
        .strip_prefix(">>>GUEST>>>")
        .unwrap()
        .strip_suffix("<<<GUEST<<<")
        .unwrap()
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::wasp_input))]
pub struct WaspInput {
    pub records: Wasp,
    _eoi: EndOfInput,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::wasp))]
pub struct Wasp(pub Vec<AdviceDefinition>);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::advice_definition))]
pub enum AdviceDefinition {
    AdviceGlobal(AdviceGlobal),
    AdviceTrap(AdviceTrap),
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::advice_global))]
pub struct AdviceGlobal(
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub  String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::advice_trap))]
pub struct AdviceTrap(pub TrapSignature);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_signature))]
pub enum TrapSignature {
    TrapApply(TrapApply),
    TrapCall(TrapCall),
    TrapBlockPre(TrapBlockPre),
    TrapBlockPost(TrapBlockPost),
    TrapLoopPre(TrapLoopPre),
    TrapLoopPost(TrapLoopPost),
    TrapSelect(TrapSelect),
    TrapCallIndirectPre(TrapCallIndirectPre),
    TrapCallIndirectPost(TrapCallIndirectPost),
    TrapIfThen(TrapIfThen),
    TrapIfThenElse(TrapIfThenElse),
    TrapBrIf(TrapBrIf),
    TrapBrTable(TrapBrTable),
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_apply))]
pub struct TrapApply {
    pub apply_hook_signature: ApplyHookSignature,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_call))]
pub struct TrapCall {
    #[pest_ast(inner(with(span_into_qualifier)))]
    pub call_qualifier: CallQualifier,
    pub formal_target: FormalTarget,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::formal_target))]
pub struct FormalTarget(#[pest_ast(inner(with(span_into_string), with(String::from)))] pub String);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_block_pre))]
pub struct TrapBlockPre {
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_block_post))]
pub struct TrapBlockPost {
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_loop_pre))]
pub struct TrapLoopPre {
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_loop_post))]
pub struct TrapLoopPost {
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_select))]
pub struct TrapSelect {
    pub select_formal_condition: SelectFormalCondition,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::select_formal_condition))]
pub struct SelectFormalCondition(
    #[pest_ast(inner(with(span_into_string), with(String::from)))] pub String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_call_indirect_pre))]
pub struct TrapCallIndirectPre {
    pub formal_table: FormalTable,
    pub formal_index: FormalIndex,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_call_indirect_post))]
pub struct TrapCallIndirectPost {
    pub formal_table: FormalTable,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::formal_table))]
pub struct FormalTable(#[pest_ast(inner(with(span_into_string), with(String::from)))] pub String);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::formal_index))]
pub struct FormalIndex(#[pest_ast(inner(with(span_into_string), with(String::from)))] pub String);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_hook_signature))]
pub enum ApplyHookSignature {
    Gen(ApplyGen),
    SpeInter(ApplySpeInter),
    SpeIntro(ApplySpeIntro),
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_gen))]
pub struct ApplyGen {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub apply_formal_argument: ApplyFormalArgument,
    pub apply_formal_result: ApplyFormalResult,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_spe_inter))]
pub struct ApplySpeInter {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub formal_arguments_arguments: Vec<ApplyFormalArgument>,
    pub formal_arguments_results: Vec<ApplyFormalResult>,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_spe_intro))]
pub struct ApplySpeIntro {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub formal_arguments_arguments: Vec<ApplyFormalArgument>,
    pub formal_arguments_results: Vec<ApplyFormalResult>,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_formal_wasm_f))]
pub struct ApplyFormalWasmF(
    #[pest_ast(inner(with(span_into_string), with(String::from)))] pub String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_formal_argument))]
pub struct ApplyFormalArgument(pub TypedArgument);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_formal_result))]
pub struct ApplyFormalResult(pub TypedArgument);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_if_then))]
pub struct TrapIfThen {
    pub branch_formal_condition: BranchFormalCondition,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_if_then_else))]
pub struct TrapIfThenElse {
    pub branch_formal_condition: BranchFormalCondition,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_br_if))]
pub struct TrapBrIf {
    pub branch_formal_condition: BranchFormalCondition,
    pub branch_formal_label: BranchFormalLabel,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::branch_formal_condition))]
pub struct BranchFormalCondition(
    #[pest_ast(inner(with(span_into_string), with(String::from)))] pub String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::branch_formal_label))]
pub struct BranchFormalLabel(
    #[pest_ast(inner(with(span_into_string), with(String::from)))] pub String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_br_table))]
pub struct TrapBrTable {
    pub branch_formal_target: BranchFormalTarget,
    pub branch_formal_default: BranchFormalDefault,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter), with(String::from)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::branch_formal_target))]
pub struct BranchFormalTarget(
    #[pest_ast(inner(with(span_into_string), with(String::from)))] pub String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::branch_formal_default))]
pub struct BranchFormalDefault(
    #[pest_ast(inner(with(span_into_string), with(String::from)))] pub String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::typed_argument))]
pub struct TypedArgument {
    #[pest_ast(inner(with(span_into_string), with(String::from)))]
    pub identifier: String,
    #[pest_ast(inner(with(span_into_string), with(String::from)))]
    pub type_identifier: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::EOI))]
struct EndOfInput;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WaspParser;
    use from_pest::FromPest;
    use pest::Parser;

    use pest_test::{cargo_manifest_dir, PestTester};
    use std::{fs::DirEntry, io::Result};

    #[test]
    fn test_wasp_parser() {
        const TEST_EXTENSION: &str = "wasp-test";
        let tests_dir = cargo_manifest_dir().join("src").join("ast").join("pest");
        let tester: PestTester<Rule, WaspParser> = {
            PestTester::new(
                &tests_dir,
                TEST_EXTENSION,
                Rule::wasp_input,
                Default::default(),
            )
        };

        for dir_entry in tests_dir
            .read_dir()
            .unwrap()
            .collect::<Result<Vec<DirEntry>>>()
            .unwrap()
        {
            let file_name = dir_entry.file_name();
            if file_name == "mod.rs" {
                continue;
            }

            let name = file_name.to_string_lossy();
            let (test_name, test_extension) = name.split_once('.').unwrap();
            assert_eq!(test_extension, TEST_EXTENSION);
            tester.evaluate_strict(test_name).unwrap();
        }
    }

    #[test]
    fn test_debug() {
        let program_source = r#"
     (aspect
      (advice apply (func    WasmFunction)
                    (args    Args)
                    (results Results)
          >>>GUEST>>>🔴<<<GUEST<<<)
      (advice apply (func    WasmFunction)
                    (args    DynArgs)
                    (results DynResults)
          >>>GUEST>>>🟠<<<GUEST<<<)
      (advice apply (func    WasmFunction)
                    (args    MutDynArgs)
                    (results MutDynResults)
          >>>GUEST>>>🟡<<<GUEST<<<)
      (advice apply (func    WasmFunction)
                    ((a I32) (b F32))
                    ((c I64) (d F64))
          >>>GUEST>>>🟢<<<GUEST<<<)
      (advice apply (func    WasmFunction)
                    (Mut (a I32) (b F32))
                    (Mut (c I64) (d F64))
          >>>GUEST>>>🔵<<<GUEST<<<)
      (global >>>GUEST>>>🟣<<<GUEST<<<)
      (advice if_then      (cond Condition) >>>GUEST>>>[🌶]<<<GUEST<<<)
      (advice if_then_else (cond Condition) >>>GUEST>>>[🧂]<<<GUEST<<<)
      (advice br_if        (cond Condition)
                           (label Label)
          >>>GUEST>>>🌿<<<GUEST<<<)
      (advice br_table (target  Target)
                       (default Default)
          >>>GUEST>>>🏓<<<GUEST<<<)
      (advice select (cond Condition)
          >>>GUEST>>>🦂<<<GUEST<<<)
      (advice call pre
              (f FunctionIndex)
          >>>GUEST>>>🧐🏃<<<GUEST<<<)
      (advice call post
              (f FunctionIndex)
          >>>GUEST>>>👀🏃<<<GUEST<<<)
      (advice call_indirect pre
              (table FunctionTable)
              (index FunctionTableIndex)
          >>>GUEST>>>🧐🏄<<<GUEST<<<)
      (advice call_indirect post
              (table FunctionTable)
          >>>GUEST>>>👀🏄<<<GUEST<<<))"#;
        let mut parse_tree = WaspParser::parse(Rule::wasp_input, program_source).unwrap();
        let wasp_input = WaspInput::from_pest(&mut parse_tree).unwrap();
        let formatted = format!("{wasp_input:?}");
        for guest_code in [
            "🔴", "🟠", "🟡", "🟢", "🔵", "🟣", "🌶", "🧂", "🌿", "🏓", "🦂", "🧐🏃", "👀🏃",
            "🧐🏄", "👀🏄",
        ] {
            assert!(formatted.contains(guest_code))
        }
    }
}
