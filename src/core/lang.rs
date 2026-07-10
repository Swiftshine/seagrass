pub mod ast;
pub mod parser;

pub fn build_program(source: &str) -> anyhow::Result<ast::Program> {
    use parser::*;
    use pest::Parser;

    let pairs = SGParser::parse(Rule::Program, source)?;
    ast::build_program(pairs.into_iter().next().unwrap())
}
