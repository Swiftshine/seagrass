use std::collections::HashMap;

use crate::core::lang::ast::{Assignment, Expression, Program, Statement, Value};

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Integer(i64),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Could not find variable {0}")]
    VariableNotFound(String),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug)]
pub struct Runtime {
    variables: HashMap<String, RuntimeValue>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn variables(&self) -> &HashMap<String, RuntimeValue> {
        &self.variables
    }

    pub fn get_variable(&self, identifier: &str) -> RuntimeResult<&RuntimeValue> {
        self.variables()
            .get(identifier)
            .ok_or(RuntimeError::VariableNotFound(identifier.to_string()))
    }

    pub fn execute(&mut self, program: &Program) {
        for statement in &program.statements {
            let result = self.execute_statement(statement);

            if let Err(e) = result {
                eprintln!("Runtime error: {:?}", e);
            }
        }
    }

    fn execute_statement(&mut self, statement: &Statement) -> RuntimeResult<()> {
        match statement {
            Statement::Assignment(assignment) => self.execute_assignment(assignment),

            Statement::Expression(expression) => {
                let _ = self.evaluate_expression(expression);
                Ok(())
            }
        }
    }

    fn execute_assignment(&mut self, assignment: &Assignment) -> RuntimeResult<()> {
        let value = self.evaluate_expression(&assignment.expression)?;

        let ident = assignment.identifier.clone();

        if assignment.declarative {
            // create a new variable
            self.variables.insert(ident, value);
        } else {
            // assign value to existing variable
            let val = self
                .variables
                .get_mut(&ident)
                .ok_or(RuntimeError::VariableNotFound(ident))?;

            *val = value;
        }

        Ok(())
    }

    fn evaluate_expression(&self, expression: &Expression) -> RuntimeResult<RuntimeValue> {
        match expression {
            Expression::Value(value) => self.resolve_value(value),
        }
    }

    fn resolve_value(&self, value: &Value) -> RuntimeResult<RuntimeValue> {
        match value {
            Value::Integer(i) => Ok(RuntimeValue::Integer(*i)),

            Value::Identifier(name) => self
                .variables
                .get(name)
                .cloned()
                .ok_or(RuntimeError::VariableNotFound(name.clone())),
        }
    }
}
