use ast::pest::{Rule, WaspInput, WaspParser};
use ast::wasp::Root;
use from_pest::FromPest;
use joinpoints::JoinPoints;
use pest::Parser;

pub mod ast;
pub mod joinpoints;
pub mod wasp_interface;

#[derive(Debug, PartialEq, Eq)]
pub struct CompilationResult {
    pub wasp_root: Root,
    pub join_points: JoinPoints,
}

/// # Errors
/// Whenever compilation would fail due to parsing or compiling the code.
pub fn compile(wasp: &str) -> anyhow::Result<CompilationResult> {
    let mut pest_parse = WaspParser::parse(Rule::wasp_input, wasp)?;
    let wasp_input = WaspInput::from_pest(&mut pest_parse)?;
    let wasp_root = Root::try_from(wasp_input)?;
    let join_points: JoinPoints = wasp_root.join_points();

    Ok(CompilationResult {
        wasp_root,
        join_points,
    })
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
    fn test_compile() {
        assert_eq!(
            compile("(aspect)").unwrap(),
            CompilationResult {
                wasp_root: Root(vec![]),
                join_points: JoinPoints::default(),
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
            wasp_root: Root(vec![]),
            join_points: JoinPoints::default(),
        };
        assert_eq!(
            format!("{compilation_result:#?}"),
            indoc! {r#"CompilationResult {
                wasp_root: Root(
                    [],
                ),
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
                    block_pre: false,
                    block_post: false,
                    loop_pre: false,
                    loop_post: false,
                    select: false,
                },
            }"#
            }
        );
    }
}
