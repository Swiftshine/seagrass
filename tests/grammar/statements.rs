use crate::parse_ok;
use seagrass::core::lang::parser::Rule;

#[test]
fn parse_assignment() {
    parse_ok(Rule::Assignment, "let my_ident = 123;");
}

#[test]
fn parse_assignment_as_statement() {
    parse_ok(Rule::Statement, "let my_ident = 123;");
}
