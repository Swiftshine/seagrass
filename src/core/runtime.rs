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
    #[error("Function {0} not in call stack")]
    FunctionNotInCallStack(String),
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

#[derive(Debug, Clone)]
pub enum RuntimeScopeType {
    Global,
    Block,
    Function,
}

#[derive(Debug, Clone)]
pub struct RuntimeScope {
    _scope_type: RuntimeScopeType,
    variables: HashMap<String, RuntimeValue>,
}

impl RuntimeScope {
    pub fn new(scope_type: RuntimeScopeType) -> Self {
        Self {
            _scope_type: scope_type,
            variables: HashMap::new(),
        }
    }

    pub fn get_variable(&self, identifier: &str) -> RuntimeResult<&RuntimeValue> {
        self.variables
            .get(identifier)
            .ok_or(RuntimeError::VariableNotFound(identifier.to_string()))
    }

    pub fn get_variable_mut(&mut self, identifier: &str) -> RuntimeResult<&mut RuntimeValue> {
        self.variables
            .get_mut(identifier)
            .ok_or(RuntimeError::VariableNotFound(identifier.to_string()))
    }

    pub fn variables(&self) -> &HashMap<String, RuntimeValue> {
        &self.variables
    }

    pub fn variables_mut(&mut self) -> &mut HashMap<String, RuntimeValue> {
        &mut self.variables
    }
}

#[derive(Clone, Copy)]
pub enum RuntimeConfigOption {
    PreserveExpiredFrames(bool),
}

#[derive(Debug, Default)]
pub struct RuntimeConfig {
    preserve_expired_frames: bool,
}

#[derive(Debug)]
pub struct FunctionFrame {
    pub name: String,
    scopes: Vec<RuntimeScope>,
}

impl FunctionFrame {
    pub fn new(name: String) -> Self {
        Self {
            name,
            scopes: vec![RuntimeScope::new(RuntimeScopeType::Function)],
        }
    }

    pub fn current_scope(&self) -> &RuntimeScope {
        self.scopes.last().unwrap()
    }

    pub fn current_scope_mut(&mut self) -> &mut RuntimeScope {
        self.scopes.last_mut().unwrap()
    }

    pub fn scopes_mut(&mut self) -> &mut Vec<RuntimeScope> {
        &mut self.scopes
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(RuntimeScope::new(RuntimeScopeType::Block));
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_variable_mut(&mut self, identifier: &str) -> Option<&mut RuntimeValue> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.variables_mut().get_mut(identifier))
    }

    pub fn get_variable(&self, identifier: &str) -> Option<&RuntimeValue> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.variables().get(identifier))
    }
}

#[derive(Debug)]
pub struct Runtime {
    global_scope: RuntimeScope,
    call_stack: Vec<FunctionFrame>,
    dead_frames: Vec<FunctionFrame>,
    functions: HashMap<String, RuntimeFunction>,
    config: RuntimeConfig,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            global_scope: RuntimeScope::new(RuntimeScopeType::Global),
            call_stack: Vec::new(),
            dead_frames: Vec::new(),
            functions: HashMap::new(),
            config: RuntimeConfig::default(),
        }
    }

    fn configure(&mut self, option: RuntimeConfigOption) {
        match option {
            RuntimeConfigOption::PreserveExpiredFrames(should_preserve) => {
                self.config.preserve_expired_frames = should_preserve;
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

    pub fn push_frame(&mut self, identifier: String) {
        self.call_stack.push(FunctionFrame::new(identifier));
    }

    pub fn pop_frame(&mut self) {
        if let Some(frame) = self.call_stack.pop() {
            if self.config.preserve_expired_frames {
                self.dead_frames.push(frame);
            }
        }
    }

    pub fn current_frame(&self) -> &FunctionFrame {
        self.call_stack.last().unwrap()
    }

    pub fn current_frame_mut(&mut self) -> &mut FunctionFrame {
        self.call_stack.last_mut().unwrap()
    }

    pub fn dead_frames(&self) -> &[FunctionFrame] {
        &self.dead_frames
    }

    pub fn get_frame(&self, identifier: &str) -> RuntimeResult<&FunctionFrame> {
        self.call_stack
            .iter()
            .find(|frame| frame.name == identifier)
            .ok_or(RuntimeError::FunctionNotInCallStack(identifier.to_string()))
    }

    pub fn get_dead_frame(&self, identifier: &str) -> RuntimeResult<&FunctionFrame> {
        self.dead_frames
            .iter()
            .find(|frame| frame.name == identifier)
            .ok_or(RuntimeError::FunctionNotInCallStack(identifier.to_string()))
    }

    pub fn get_global_variable(&self, identifier: &str) -> RuntimeResult<&RuntimeValue> {
        self.global_scope
            .variables()
            .get(identifier)
            .ok_or(RuntimeError::VariableNotFound(identifier.to_owned()))
    }

    pub fn get_global_variable_mut(
        &mut self,
        identifier: &str,
    ) -> RuntimeResult<&mut RuntimeValue> {
        self.global_scope
            .variables_mut()
            .get_mut(identifier)
            .ok_or(RuntimeError::VariableNotFound(identifier.to_owned()))
    }

    /// Within the current scope.
    pub fn get_variable(&self, identifier: &str) -> RuntimeResult<&RuntimeValue> {
        if let Some(frame) = self.call_stack.last() {
            if let Some(value) = frame.get_variable(identifier) {
                return Ok(value);
            }
        }

        self.get_global_variable(identifier)
    }

    /// Within the current scope.
    pub fn get_variable_mut(&mut self, identifier: &str) -> RuntimeResult<&mut RuntimeValue> {
        let exists_locally = self
            .call_stack
            .last()
            .and_then(|frame| frame.get_variable(identifier))
            .is_some();

        if exists_locally {
            let frame = self.call_stack.last_mut().unwrap();
            return Ok(frame.get_variable_mut(identifier).unwrap());
        }

        self.get_global_variable_mut(identifier)
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
                self.evaluate_expression(expression)?;
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
            self.assign_variable(ident, value);
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

    fn execute_function_body(&mut self, block: &Block) -> StatementResult {
        for statement in &block.statements {
            match self.execute_statement(statement)? {
                ControlFlow::Continue => {}
                flow @ ControlFlow::Return(_) => return Ok(flow),
            }
        }

        Ok(ControlFlow::Continue)
    }

    // keeping this for now for control statements
    // fn execute_block(&mut self, block: &Block) -> StatementResult {
    //     self.current_frame_mut().push_scope();

    //     let result = (|| {
    //         for statement in &block.statements {
    //             match self.execute_statement(statement)? {
    //                 ControlFlow::Continue => {}
    //                 flow @ ControlFlow::Return(_) => return Ok(flow),
    //             }
    //         }

    //         Ok(ControlFlow::Continue)
    //     })();

    //     self.current_frame_mut().pop_scope();

    //     result
    // }

    fn assign_variable(&mut self, identifier: String, value: RuntimeValue) {
        if self.call_stack.is_empty() {
            self.global_scope.variables_mut().insert(identifier, value);
        } else {
            self.current_frame_mut()
                .current_scope_mut()
                .variables_mut()
                .insert(identifier, value);
        }
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
                self.push_frame(identifier.to_string());
                let result = self.execute_function_body(&func.body);
                self.pop_frame();

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
