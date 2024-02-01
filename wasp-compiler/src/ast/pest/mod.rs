use pest::Span;
use pest_ast::FromPest;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "wasp.pest"]
pub struct WaspParser;

fn span_into_string(span: Span) -> String {
    span.as_str().to_string()
}

fn drop_guest_delimiter(guest_code: String) -> String {
    guest_code
        .strip_prefix(">>>GUEST>>>")
        .unwrap()
        .strip_suffix("<<<GUEST<<<")
        .unwrap()
        .to_string()
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
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter)))] pub String,
);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::advice_trap))]
pub struct AdviceTrap(pub TrapSignature);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_signature))]
pub enum TrapSignature {
    TrapApply(TrapApply),
    TrapIfThenElse(TrapIfThenElse),
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_apply))]
pub struct TrapApply {
    pub apply_hook_signature: ApplyHookSignature,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter)))]
    pub body: String,
}

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
pub struct ApplyFormalWasmF(#[pest_ast(inner(with(span_into_string)))] pub String);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_formal_argument))]
pub struct ApplyFormalArgument(pub TypedArgument);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::apply_formal_result))]
pub struct ApplyFormalResult(pub TypedArgument);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::trap_if_then_else))]
pub struct TrapIfThenElse {
    pub if_then_else_hook_signature: IfThenElseHookSignature,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter)))]
    pub body: String,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::if_then_else_hook_signature))]
pub struct IfThenElseHookSignature {
    pub if_then_else_formal_condition: IfThenElseFormalCondition,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::if_then_else_formal_condition))]
pub struct IfThenElseFormalCondition(#[pest_ast(inner(with(span_into_string)))] pub String);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::typed_argument))]
pub struct TypedArgument {
    #[pest_ast(inner(with(span_into_string)))]
    pub identifier: String,
    #[pest_ast(inner(with(span_into_string)))]
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

    #[test]
    fn test_wasp_parser() {
        let tester: PestTester<Rule, WaspParser> = {
            let path_to_source_dir = cargo_manifest_dir().join("src").join("ast").join("pest");

            PestTester::new(
                path_to_source_dir,
                "wasp-test",
                Rule::wasp_input,
                Default::default(),
            )
        };

        // TODO: find these *.wasp-test files automatically
        tester.evaluate_strict("test-empty").unwrap();
        tester.evaluate_strict("test-global-only").unwrap();
        tester.evaluate_strict("test-trap-apply-hook").unwrap();
        tester.evaluate_strict("test-trap-apply-spe-inter").unwrap();
        tester.evaluate_strict("test-trap-apply-spe-intro").unwrap();
        tester.evaluate_strict("test-trap-applies").unwrap();
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
      (global >>>GUEST>>>🟣<<<GUEST<<<))"#;
        let mut parse_tree = WaspParser::parse(Rule::wasp_input, program_source).unwrap();
        let wasp_input = WaspInput::from_pest(&mut parse_tree).unwrap();
        let formatted = format!("{wasp_input:?}");
        for guest_code in ["🔴", "🟠", "🟡", "🟢", "🔵", "🟣"] {
            assert!(formatted.contains(guest_code))
        }
    }
}
