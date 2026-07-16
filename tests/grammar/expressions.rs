use crate::parse_ok;
use seagrass::core::lang::parser::Rule;

#[test]
fn parse_integer() {
    parse_ok(Rule::Integer, "123");
}

#[test]
fn parse_integer_as_value() {
    parse_ok(Rule::Value, "123");
}

#[test]
fn parse_integer_as_expression() {
    parse_ok(Rule::Expression, "123");
}

#[test]
fn parse_addition() {
    parse_ok(Rule::Addition, "1 + 2");
    parse_ok(Rule::Addition, "1 - 2");
}

#[test]
fn parse_multiplication() {
    parse_ok(Rule::Multiplication, "1 * 2");
    parse_ok(Rule::Multiplication, "1 / 2");
}
