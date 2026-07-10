use crate::parse_program;
use seagrass::core::lang::ast::{Expression, Statement, Value};

#[test]
pub fn integer_expression() {
    let program = parse_program("123;");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(Expression::Value(Value::S32(value))) => {
            assert_eq!(*value, 123);
        }

        other => panic!("unexpected AST: {:?}", other),
    }
}
