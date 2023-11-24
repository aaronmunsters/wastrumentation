use ast::assemblyscript::TypeScriptProgram;
use ast::pest::{Rule, WaspParser, WaspRoot};
use ast::wasp::WaspInput;
use from_pest::FromPest;
use pest::Parser;

mod ast;

impl<'a> TryFrom<&'a str> for TypeScriptProgram {
    type Error = anyhow::Error;

    fn try_from(program: &'a str) -> Result<Self, Self::Error> {
        let mut pest_parse = WaspParser::parse(Rule::wasp_input, program)?;
        let wasp_input = WaspInput::from_pest(&mut pest_parse).expect("pest to input");
        let wasp_root = WaspRoot::try_from(wasp_input)?;
        let typescript_program = TypeScriptProgram::from(wasp_root);
        Ok(typescript_program)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(TypeScriptProgram::try_from("")
            .unwrap_err()
            .to_string()
            .as_str()
            .contains("expected wasp"))
    }

    #[test]
    fn typescript_conversion_fail() {
        assert_eq!(
            TypeScriptProgram::try_from(
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
}
