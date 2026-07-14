use std::{collections::HashMap, rc::Rc};

use crate::core::lang::ast::{
    Assignment, Block, DataType, Expression, FunctionDefinition, Program, Return, Statement, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    None,
    U32(u32),
    S32(i32),
}

pub type NativeFunction = fn(Vec<RuntimeValue>) -> RuntimeResult<RuntimeValue>;

#[derive(Debug, Clone)]
pub enum RuntimeFunction {
    Native(NativeFunction),
    User(Rc<FunctionDefinition>),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Variable {0} not defined")]
    VariableNotFound(String),
    #[error("Function {0} not defined")]
    FunctionNotFound(String),
    #[error("Function {0} is not a user-defined function")]
    NotAUserDefinedFunction(String),
    #[error("Function {0} is not a native function")]
    NotANativeFunction(String),
    #[error("Type mismatch")]
    TypeMismatch,
}

#[derive(Debug, PartialEq)]
pub enum ControlFlow {
    Continue,
    Return(RuntimeValue),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
pub type StatementResult = Result<ControlFlow, RuntimeError>;

#[derive(Debug, Clone, Default)]
pub enum RuntimeScopeType {
    #[default]
    Global,
    Block,
    Function,
}

#[derive(Debug, Clone)]
pub struct RuntimeScope {
    scope_type: RuntimeScopeType,
    variables: HashMap<String, RuntimeValue>,
}

impl RuntimeScope {
    pub fn new(scope_type: RuntimeScopeType) -> Self {
        Self {
            scope_type,
            variables: HashMap::new(),
        }
    }

    pub fn variables(&self) -> &HashMap<String, RuntimeValue> {
        &self.variables
    }
}

#[derive(Clone, Copy)]
pub enum RuntimeConfigOption {
    PreserveScope(bool),
}

#[derive(Debug, Default)]
pub struct RuntimeConfig {
    preserve_scopes: bool,
}

#[derive(Debug)]
pub struct Runtime {
    scopes: Vec<RuntimeScope>,
    dead_scopes: Vec<RuntimeScope>,
    functions: HashMap<String, RuntimeFunction>,
    config: RuntimeConfig,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            scopes: vec![RuntimeScope::new(RuntimeScopeType::Global)],
            dead_scopes: Vec::new(),
            functions: HashMap::new(),
            config: RuntimeConfig::default(),
        }
    }

    fn configure(&mut self, option: RuntimeConfigOption) {
        match option {
            RuntimeConfigOption::PreserveScope(should_preserve) => {
                self.config.preserve_scopes = should_preserve;
            }
        }
    }

    pub fn with_config(mut self, option: RuntimeConfigOption) -> Runtime {
        self.configure(option);
        self
    }

    pub fn with_configs(mut self, options: &[RuntimeConfigOption]) -> Runtime {
        for option in options {
            self.configure(*option);
        }

        self
    }

    pub fn dead_scopes(&self) -> &[RuntimeScope] {
        &self.dead_scopes
    }

    pub fn global_scope(&self) -> &RuntimeScope {
        &self.scopes[0]
    }

    pub fn global_dead_scope(&self) -> &RuntimeScope {
        &self.dead_scopes[0]
    }

    pub fn current_scope(&self) -> &RuntimeScope {
        self.scopes.last().unwrap()
    }

    pub fn current_dead_scope(&self) -> &RuntimeScope {
        self.dead_scopes.last().unwrap()
    }

    pub fn current_scope_mut(&mut self) -> &mut RuntimeScope {
        self.scopes.last_mut().unwrap()
    }

    pub fn push_scope(&mut self, scope_type: RuntimeScopeType) {
        self.scopes.push(RuntimeScope::new(scope_type));
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            let scope = self.scopes.pop().unwrap();

            if self.config.preserve_scopes {
                self.dead_scopes.push(scope);
            }
        }
    }

    pub fn get_variable(&self, identifier: &str) -> RuntimeResult<&RuntimeValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(identifier) {
                return Ok(value);
            }

            if matches!(scope.scope_type, RuntimeScopeType::Function) {
                // functions should not have access to the scope of other functions
                break;
            }
        }

        Err(RuntimeError::VariableNotFound(identifier.to_owned()))
    }

    pub fn get_dead_variable(&self, identifier: &str) -> RuntimeResult<&RuntimeValue> {
        for scope in self.dead_scopes.iter().rev() {
            if let Some(value) = scope.variables.get(identifier) {
                return Ok(value);
            }
        }

        Err(RuntimeError::VariableNotFound(identifier.to_owned()))
    }

    pub fn get_variable_mut(&mut self, identifier: &str) -> RuntimeResult<&mut RuntimeValue> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(value) = scope.variables.get_mut(identifier) {
                return Ok(value);
            }
        }

        Err(RuntimeError::VariableNotFound(identifier.to_owned()))
    }

    pub fn functions(&self) -> &HashMap<String, RuntimeFunction> {
        &self.functions
    }

    pub fn functions_mut(&mut self) -> &mut HashMap<String, RuntimeFunction> {
        &mut self.functions
    }

    pub fn get_function(&self, identifier: &str) -> RuntimeResult<&RuntimeFunction> {
        self.functions()
            .get(identifier)
            .ok_or(RuntimeError::FunctionNotFound(identifier.to_string()))
    }

    pub fn get_user_function(&self, identifier: &str) -> RuntimeResult<&FunctionDefinition> {
        match self.get_function(identifier)? {
            RuntimeFunction::User(func) => Ok(func),

            RuntimeFunction::Native(_) => Err(RuntimeError::NotAUserDefinedFunction(
                identifier.to_string(),
            )),
        }
    }

    pub fn get_native_function(
        &self,
        identifier: &str,
    ) -> RuntimeResult<fn(Vec<RuntimeValue>) -> RuntimeResult<RuntimeValue>> {
        match self.get_function(identifier)? {
            RuntimeFunction::Native(func) => Ok(*func),

            RuntimeFunction::User(_) => {
                Err(RuntimeError::NotANativeFunction(identifier.to_string()))
            }
        }
    }

    pub fn execute(&mut self, program: &Program) -> RuntimeResult<()> {
        // collect sg:: functions
        self.register_native_functions();

        // collect function definitions
        for statement in &program.statements {
            if let Statement::FunctionDefinition(func) = statement {
                self.define_function(func)?;
            }
        }

        // execute normal statements
        for statement in &program.statements {
            if !matches!(statement, Statement::FunctionDefinition(_)) {
                self.execute_statement(statement)?;
            }
        }

        // call main()
        if self.get_function("main").is_ok() {
            self.call_function("main", vec![])?;
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
        let mut value = self.evaluate_expression(&assignment.expression)?;

        if let Some(expected_type) = &assignment.data_type {
            value = self.apply_type_annotation(value, expected_type)?;
        }

        let ident = assignment.identifier.clone();

        if assignment.declarative {
            // create a new variable
            self.current_scope_mut().variables.insert(ident, value);
        } else {
            // assign value to existing variable
            *self.get_variable_mut(&ident)? = value;
        }

        Ok(ControlFlow::Continue)
    }

    fn apply_type_annotation(
        &self,
        value: RuntimeValue,
        expected: &DataType,
    ) -> RuntimeResult<RuntimeValue> {
        match (value, expected) {
            (RuntimeValue::S32(i), DataType::S32) => Ok(RuntimeValue::S32(i)),

            (RuntimeValue::U32(i), DataType::U32) => Ok(RuntimeValue::U32(i)),

            (RuntimeValue::S32(i), DataType::U32) if i >= 0 => Ok(RuntimeValue::U32(i as u32)),

            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    fn execute_return(&mut self, ret: &Return) -> StatementResult {
        let value = self.evaluate_expression(&ret.expression)?;
        Ok(ControlFlow::Return(value))
    }

    fn execute_block(&mut self, block: &Block) -> StatementResult {
        self.push_scope(RuntimeScopeType::Block);

        let result = (|| {
            for statement in &block.statements {
                match self.execute_statement(statement)? {
                    ControlFlow::Continue => {}
                    flow @ ControlFlow::Return(_) => return Ok(flow),
                }
            }

            Ok(ControlFlow::Continue)
        })();

        self.pop_scope();

        result
    }

    fn define_function(&mut self, func: &FunctionDefinition) -> StatementResult {
        self.functions.insert(
            func.identifier.clone(),
            RuntimeFunction::User(Rc::new(func.clone())),
        );

        Ok(ControlFlow::Continue)
    }

    fn call_function(
        &mut self,
        identifier: &str,
        args: Vec<RuntimeValue>,
    ) -> RuntimeResult<RuntimeValue> {
        let func = self
            .functions
            .get(identifier)
            .cloned()
            .ok_or(RuntimeError::FunctionNotFound(identifier.to_string()))?;

        match func {
            RuntimeFunction::Native(native) => native(args),

            RuntimeFunction::User(func) => {
                self.push_scope(RuntimeScopeType::Function);

                let result = self.execute_block(&func.body);

                self.pop_scope();

                match result? {
                    ControlFlow::Continue => Ok(RuntimeValue::None),
                    ControlFlow::Return(value) => Ok(value),
                }
            }
        }
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> RuntimeResult<RuntimeValue> {
        match expression {
            Expression::Value(value) => self.resolve_value(value),
            Expression::FunctionCall(call) => {
                let args = call
                    .arguments
                    .iter()
                    .map(|expr| self.evaluate_expression(expr))
                    .collect::<RuntimeResult<Vec<_>>>()?;

                self.call_function(&call.identifier, args)
            }
        }
    }

    fn resolve_value(&self, value: &Value) -> RuntimeResult<RuntimeValue> {
        match value {
            Value::S32(i) => Ok(RuntimeValue::S32(*i)),

            Value::U32(i) => Ok(RuntimeValue::U32(*i)),

            Value::Identifier(name) => self.get_variable(name).cloned(),
        }
    }
}
