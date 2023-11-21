use pest::Parser;
use pest_derive::Parser;

mod advice_ast;

#[derive(Parser)]
#[grammar = "wasp.pest"]
pub struct WaspParser;

#[cfg(test)]
mod tests {
    use super::*;

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
}
