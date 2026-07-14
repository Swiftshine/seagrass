pub mod ast;
pub mod parser;

pub fn build_program(source: &str) -> anyhow::Result<ast::Program> {
    use parser::*;
    use pest::Parser;

    let pairs = SGParser::parse(Rule::Program, source)?;
    ast::build_program(pairs.into_iter().next().unwrap())
}

pub fn dump_program(source: &str) {
    use parser::*;
    use pest::Parser;

    let pairs = SGParser::parse(Rule::Program, source).expect("failed to create program");
    ast::dump(pairs.into_iter().next().unwrap(), 0);
}
