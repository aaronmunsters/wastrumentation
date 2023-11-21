use crate::Rule;
use pest::Span;
use pest_ast::FromPest;

fn span_into_string(span: Span) -> String {
    span.as_str().to_string()
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::wasp_input))]
pub struct WaspInput {
    pub records: Wasp,
    _eoi: EOI,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::wasp))]
pub struct Wasp {
    pub records: Vec<AdviceDefinition>,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::advice_definition))]
pub enum AdviceDefinition {
    AdviceGlobal(AdviceGlobal),
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::advice_global))]
pub struct AdviceGlobal(#[pest_ast(inner(with(span_into_string)))] pub String);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::advice_trap))]
pub struct AdviceTrap {}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::EOI))]
struct EOI;

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use from_pest::FromPest;
    use pest::Parser;

    use crate::WaspParser;

    use super::*;

    #[test]
    fn aspect_global_empty() -> anyhow::Result<()> {
        let mut parse_tree = WaspParser::parse(Rule::wasp_input, "(aspect)")?;
        let WaspInput { .. } = WaspInput::from_pest(&mut parse_tree)?;
        Ok(())
    }

    #[test]
    fn aspect_global_only() -> anyhow::Result<()> {
        let mut parse_tree = WaspParser::parse(
            Rule::wasp_input,
            r#"
        (aspect
            (global >>>GUEST>>> console.log("Hello world!") <<<GUEST<<<))"#,
        )?;
        let WaspInput { .. } = WaspInput::from_pest(&mut parse_tree)?;
        Ok(())
    }
}
