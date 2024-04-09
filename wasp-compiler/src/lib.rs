use ast::assemblyscript::AssemblyScriptProgram;
use ast::pest::{Rule, WaspInput, WaspParser};
use ast::wasp::WaspRoot;
use from_pest::FromPest;
use joinpoints::JoinPoints;
use pest::Parser;
use wasp_interface::WaspInterface;

pub mod ast;
pub mod joinpoints;
mod util;
pub mod wasp_interface;

#[derive(Debug, PartialEq, Eq)]
pub struct CompilationResult {
    pub analysis_source_code: AssemblyScriptProgram,
    pub join_points: JoinPoints,
    pub wasp_interface: WaspInterface,
}

pub fn compile(wasp: &str) -> anyhow::Result<CompilationResult> {
    let mut pest_parse = WaspParser::parse(Rule::wasp_input, wasp)?;
    let wasp_input = WaspInput::from_pest(&mut pest_parse).expect("pest to input");
    let wasp_root = WaspRoot::try_from(wasp_input)?;
    let wasp_interface = WaspInterface::from(&wasp_root);
    let join_points = wasp_root.join_points();
    let assemblyscript_program = AssemblyScriptProgram::from(wasp_root);

    Ok(CompilationResult {
        analysis_source_code: assemblyscript_program,
        join_points,
        wasp_interface,
    })
}

impl<'a> TryFrom<&'a str> for AssemblyScriptProgram {
    type Error = anyhow::Error;

    fn try_from(program: &'a str) -> Result<Self, Self::Error> {
        let mut pest_parse = WaspParser::parse(Rule::wasp_input, program)?;
        let wasp_input = WaspInput::from_pest(&mut pest_parse).expect("pest to input");
        let wasp_root = WaspRoot::try_from(wasp_input)?;
        let assemblyscript_program = AssemblyScriptProgram::from(wasp_root);
        Ok(assemblyscript_program)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pest::Parser;

    fn assert_parse_ok(s: &str) {
        assert!(WaspParser::parse(Rule::wasp_input, s).is_ok())
    }

    #[test]
    fn whitespaces() {
        assert_parse_ok("(aspect)        ");
        assert_parse_ok("(aspect)        ");
        assert_parse_ok("        (aspect)");
        assert_parse_ok("(aspect        )");
        assert_parse_ok("(        aspect)");
        assert_parse_ok("(    aspect    )");
        assert_parse_ok("(    aspect    )");
    }

    #[test]
    fn parse_profiling() {
        assert_parse_ok(
            r#"
            (aspect
                (global
                    >>>GUEST>>>
                    // Keep global function counter
                    let global_function_count = 0;
                    <<<GUEST<<<)
            
                (advice apply (func    WasmFunction)
                              (args    Args)
                              (results Results)
                    >>>GUEST>>>
                    global_function_count++;
                    <<<GUEST<<<)
            )"#,
        )
    }

    #[test]
    fn parse_fail_pest() {
        assert!(AssemblyScriptProgram::try_from("")
            .unwrap_err()
            .to_string()
            .as_str()
            .contains("expected wasp"))
    }

    #[test]
    fn assemblyscript_conversion_fail() {
        assert_eq!(
            AssemblyScriptProgram::try_from(
                "
                (aspect
                    (advice apply (a WasmFunction)
                                  (a Args)
                                  (a Results) >>>GUEST>>>
                        1;
                    <<<GUEST<<<))
                "
            )
            .unwrap_err()
            .to_string()
            .as_str(),
            "Parameters must be unique, got: a, a, a."
        )
    }

    #[test]
    fn test_compile() {
        assert_eq!(
            compile("(aspect)").unwrap(),
            CompilationResult {
                analysis_source_code: AssemblyScriptProgram { content: "".into() },
                join_points: JoinPoints::default(),
                wasp_interface: WaspInterface::default()
            }
        );

        assert!(compile("malformed")
            .unwrap_err()
            .to_string()
            .as_str()
            .contains("expected wasp"));

        assert_eq!(
            compile(
                "
            (aspect
                (advice apply (a WasmFunction)
                              (a Args)
                              (a Results) >>>GUEST>>>
                    1;
                <<<GUEST<<<))
            "
            )
            .unwrap_err()
            .to_string()
            .as_str(),
            "Parameters must be unique, got: a, a, a."
        );
    }

    #[test]
    fn test_debug() {
        let compilation_result = CompilationResult {
            analysis_source_code: AssemblyScriptProgram { content: "".into() },
            join_points: JoinPoints::default(),
            wasp_interface: WaspInterface::default(),
        };
        assert_eq!(
            format!("{compilation_result:#?}"),
            indoc! {r#"CompilationResult {
                analysis_source_code: AssemblyScriptProgram {
                    content: "",
                },
                join_points: JoinPoints {
                    generic: false,
                    specialized: {},
                    if_then: false,
                    if_then_else: false,
                    br_if: false,
                    br_table: false,
                    call_pre: false,
                    call_post: false,
                    call_indirect_pre: false,
                    call_indirect_post: false,
                },
                wasp_interface: WaspInterface {
                    inputs: [],
                    outputs: [],
                    generic_interface: None,
                    if_then_trap: None,
                    if_then_else_trap: None,
                    br_if_trap: None,
                    br_table_trap: None,
                    pre_trap_call: None,
                    pre_trap_call_indirect: None,
                    post_trap_call: None,
                    post_trap_call_indirect: None,
                },
            }"#
            }
        );
    }
}
