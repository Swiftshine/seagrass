use pest::Parser;
use pest::iterators::Pair;
use seagrass::core::lang::parser::{Rule, SGParser};

pub mod grammar {
    pub mod expressions;
    pub mod statements;
}

pub fn parse(rule: Rule, input: &str) -> Pair<'_, Rule> {
    SGParser::parse(rule, input).unwrap().next().unwrap()
}

pub fn parse_ok(rule: Rule, input: &str) {
    assert!(
        SGParser::parse(rule, input.trim()).is_ok(),
        "Expected {:?} to parse:\n{}",
        rule,
        input
    );
}

pub fn parse_err(rule: Rule, input: &str) {
    assert!(
        SGParser::parse(rule, input.trim()).is_err(),
        "Expected {:?} to fail parsing:\n{}",
        rule,
        input
    );
}
