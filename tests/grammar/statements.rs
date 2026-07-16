use crate::parse_ok;
use seagrass::core::lang::parser::Rule;

#[test]
pub fn parse_assignment() {
    parse_ok(Rule::Assignment, "let my_ident = 123;");
}

#[test]
pub fn parse_assignment_as_statement() {
    parse_ok(Rule::Statement, "let my_ident = 123;");
}

#[test]
pub fn parse_function_definition() {
    parse_ok(Rule::FunctionDefinition, "fn my_func() { }");
}

#[test]
pub fn parse_function_call() {
    parse_ok(Rule::FunctionCall, "my_func();");
}

#[test]
pub fn parse_struct_definition() {
    let input = "
        struct MyStruct {
            my_field: u32
        }
    ";
    parse_ok(Rule::StructDefinition, input);
}
