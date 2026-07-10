use crate::parse_program;
use seagrass::core::lang::ast::{Assignment, Expression, Program, Statement, Value};

#[test]
pub fn let_assignment() {
    let program = parse_program("let x = 42;");

    assert_eq!(
        program,
        Program {
            statements: vec![Statement::Assignment(Assignment {
                declarative: true,
                data_type: None,
                identifier: "x".into(),
                expression: Expression::Value(Value::S32(42)),
            })]
        }
    );
}

#[test]
pub fn reassignment() {
    let program = parse_program("x = 10;");

    match &program.statements[0] {
        Statement::Assignment(assign) => {
            assert!(!assign.declarative);
            assert_eq!(assign.identifier, "x");
        }

        _ => panic!("expected assignment"),
    }
}
