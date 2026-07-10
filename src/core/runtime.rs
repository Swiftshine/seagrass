use std::collections::HashMap;

use crate::core::lang::ast::{
    Assignment, Block, Expression, FunctionDefinition, Program, Return, Statement, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    None,
    Integer(i64),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Variable {0} not defined")]
    VariableNotFound(String),
    #[error("Function {0} not defined")]
    FunctionNotFound(String),
}

#[derive(Debug, PartialEq)]
pub enum ControlFlow {
    Continue,
    Return(RuntimeValue),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
pub type StatementResult = Result<ControlFlow, RuntimeError>;

#[derive(Debug)]
pub struct Runtime {
    variables: HashMap<String, RuntimeValue>,
    functions: HashMap<String, FunctionDefinition>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn variables(&self) -> &HashMap<String, RuntimeValue> {
        &self.variables
    }

    pub fn functions(&self) -> &HashMap<String, FunctionDefinition> {
        &self.functions
    }

    pub fn get_variable(&self, identifier: &str) -> RuntimeResult<&RuntimeValue> {
        self.variables()
            .get(identifier)
            .ok_or(RuntimeError::VariableNotFound(identifier.to_string()))
    }

    pub fn get_function(&self, identifier: &str) -> RuntimeResult<&FunctionDefinition> {
        self.functions()
            .get(identifier)
            .ok_or(RuntimeError::FunctionNotFound(identifier.to_string()))
    }

    pub fn execute(&mut self, program: &Program) -> RuntimeResult<()> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }

        if self.get_function("main").is_ok() {
            self.call_function("main")?;
        }

        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> StatementResult {
        match statement {
            Statement::Assignment(assignment) => self.execute_assignment(assignment),

            Statement::Expression(expression) => {
                let _ = self.evaluate_expression(expression);
                Ok(ControlFlow::Continue)
            }

            Statement::FunctionDefinition(function) => self.define_function(function),

            Statement::Return(ret) => self.execute_return(ret),
        }
    }

    fn execute_assignment(&mut self, assignment: &Assignment) -> StatementResult {
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

        Ok(ControlFlow::Continue)
    }

    fn execute_return(&mut self, ret: &Return) -> StatementResult {
        let value = self.evaluate_expression(&ret.expression)?;
        Ok(ControlFlow::Return(value))
    }

    fn execute_block(&mut self, block: &Block) -> StatementResult {
        for statement in &block.statements {
            match self.execute_statement(statement)? {
                ControlFlow::Continue => {}
                flow @ ControlFlow::Return(_) => return Ok(flow),
            }
        }
        Ok(ControlFlow::Continue)
    }

    fn define_function(&mut self, func: &FunctionDefinition) -> StatementResult {
        self.functions.insert(func.identifier.clone(), func.clone());
        Ok(ControlFlow::Continue)
    }

    fn call_function(&mut self, identifier: &str) -> RuntimeResult<RuntimeValue> {
        let func = self
            .functions
            .get(identifier)
            .cloned()
            .ok_or(RuntimeError::FunctionNotFound(identifier.to_string()))?;

        match self.execute_block(&func.body)? {
            ControlFlow::Continue => Ok(RuntimeValue::None),

            ControlFlow::Return(value) => Ok(value),
        }
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> RuntimeResult<RuntimeValue> {
        match expression {
            Expression::Value(value) => self.resolve_value(value),
            Expression::FunctionCall(function) => self.call_function(&function.identifier),
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
