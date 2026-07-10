use pest::Parser;
use seagrass::core::lang::ast::{Program, build_program};
use seagrass::core::lang::parser::{Rule, SGParser};

mod ast {
    mod expressions;
    mod statements;
}

pub fn parse_program(input: &str) -> Program {
    let pair = SGParser::parse(Rule::Program, input)
        .unwrap()
        .next()
        .unwrap();

    build_program(pair).unwrap()
}
