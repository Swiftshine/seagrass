mod accessors;
mod operators;

use std::{collections::HashMap, rc::Rc};

use crate::core::lang::ast::{
    Assignment, BinaryOperator, Block, DataType, Expression, FunctionDefinition, Program, Return,
    Statement, StructDefinition, StructInitialization, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    None,
    U32(u32),
    S32(i32),
    String(String),
    Struct {
        definition: Rc<StructDefinition>,
        fields: HashMap<String, RuntimeValue>,
    },
}

impl RuntimeValue {
    // pub fn data_type(&self) -> String {
    //     match self {
    //         Self::None => "None",
    //         Self::U32(_) => "u32",
    //         Self::S32(_) => "s32",
    //         Self::String(_) => "string",
    //         Self::Struct { definition, .. } => &definition.identifier,
    //     }
    //     .to_string()
    // }

    pub fn data_type(&self) -> RuntimeResult<DataType> {
        match self {
            Self::None => Err(RuntimeError::NoDataTypeAttached),
            Self::U32(_) => Ok(DataType::U32),
            Self::S32(_) => Ok(DataType::S32),
            Self::String(_) => Ok(DataType::String),
            Self::Struct { definition, ..} => Ok(DataType::UserDefined(definition.identifier.clone()))
        }
    }
}

pub type NativeFunction = fn(Vec<RuntimeValue>) -> RuntimeResult<RuntimeValue>;

#[derive(Debug, Clone)]
pub enum RuntimeFunction {
    Native(NativeFunction),
    User(Rc<FunctionDefinition>),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    // Missing values
    #[error("Variable '{0}' not defined")]
    VariableNotFound(String),
    #[error("Function '{0}' not defined")]
    FunctionNotFound(String),
    #[error("Function '{0}' is not a user-defined function")]
    NotAUserDefinedFunction(String),
    #[error("Function '{0}' is not a native function")]
    NotANativeFunction(String),
    #[error("Function '{0}' not in call stack")]
    FunctionNotInCallStack(String),
    #[error("Expected data type, but found None")]
    NoDataTypeAttached,
    #[error("Field '{field_name}' does not exist in struct '{struct_name}'")]
    InvalidStructFieldAccess {
        field_name: String,
        struct_name: String
    },
    #[error("Struct definition '{0}' not found")]
    StructDefinitionNotFound(String),

    // Mismatches
    #[error("Unsupported binary operation for '[{lhs_type}] {operation} [{rhs_type}]'")]
    UnsupportedBinaryOperation {
        lhs_type: String,
        operation: &'static str,
        rhs_type: String,
    },
    #[error("Type mismatch (expected '{expected}', found '{found}')")]
    TypeMismatch {
        expected: String,
        found: String,
    },
    #[error("Identifier '{0}' already defined as a {1}")]
    AlreadyDefined(String, &'static str),

    
    // Semantic errors
    #[error("Incomplete struct initialization for '{0}'")]
    IncompleteStructInitialization(String),
    #[error("Invalid initialization type for field '{field_name}' of struct '{struct_name}' (expected '{expected}', found '{found}')")]
    InvalidStructFieldInitialization {
        field_name: String,
        struct_name: String,
        expected: String,
        found: String
    },
    #[error("Cannot access field '{field}' of '{data_type}' because it is not a struct")]
    InvalidStructFieldAccessTarget {
        field: String,
        data_type: String
    }
}

impl RuntimeError {
    pub fn unsupported_binary_operation(
        lhs_type: String,
        operation: &'static str,
        rhs_type: String,
    ) -> Self {
        Self::UnsupportedBinaryOperation {
            lhs_type,
            operation,
            rhs_type,
        }
    }
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

#[derive(Clone, Copy)]
pub enum RuntimeConfigOption {
    PreserveExpiredFrames(bool),
    ErrorOnIncompleteFieldInitialization(bool),
}

#[derive(Debug)]
pub struct RuntimeConfig {
    /// (Development) Allows expired function frames and scopes to be preserved to inspect its end-of-life state.
    preserve_expired_frames: bool,
    /// (Interpreter) Raises an error if struct initialization does not list every variable.
    error_on_incomplete_struct_initialization: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            preserve_expired_frames: false,
            error_on_incomplete_struct_initialization: true,
        }
    }
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
    structs: HashMap<String, Rc<StructDefinition>>,
    config: RuntimeConfig,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            global_scope: RuntimeScope::new(RuntimeScopeType::Global),
            call_stack: Vec::new(),
            dead_frames: Vec::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            config: RuntimeConfig::default(),
        }
    }

    fn configure(&mut self, option: RuntimeConfigOption) {
        match option {
            RuntimeConfigOption::PreserveExpiredFrames(should_preserve) => {
                self.config.preserve_expired_frames = should_preserve;
            }

            RuntimeConfigOption::ErrorOnIncompleteFieldInitialization(should_error) => {
                self.config.error_on_incomplete_struct_initialization = should_error;
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

    pub fn execute(&mut self, program: &Program) -> RuntimeResult<()> {
        // collect sg:: functions
        self.register_native_functions();

        // collect struct and function definitions
        for statement in &program.statements {
            match statement {
                Statement::FunctionDefinition(func) => {
                    self.define_function(func)?;
                }
                Statement::StructDefinition(struct_definition) => {
                    self.define_struct(struct_definition)?;
                }
                _ => {}
            }
        }

        // execute normal statements
        for statement in &program.statements {
            match statement {
                Statement::FunctionDefinition(_) | Statement::StructDefinition(_) => {}
                _ => {
                    self.execute_statement(statement)?;
                }
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

            Statement::Return(ret) => self.execute_return(ret),

            _ => unreachable!("{:?}", statement),
        }
    }

    fn execute_assignment(&mut self, assignment: &Assignment) -> StatementResult {
        let mut value = self.evaluate_expression(&assignment.expression)?;

        if let Some(expected_type) = &assignment.data_type {
            value = self.apply_type_annotation(value, expected_type)?;
        }

        let ident = assignment.identifier.clone();

        self.validate_identifier(&ident)?;

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
        let value_data_type_string = value.data_type()?.to_string();

        match (value, expected) {
            (RuntimeValue::S32(i), DataType::S32) => Ok(RuntimeValue::S32(i)),

            (RuntimeValue::U32(i), DataType::U32) => Ok(RuntimeValue::U32(i)),

            (RuntimeValue::S32(i), DataType::U32) if i >= 0 => Ok(RuntimeValue::U32(i as u32)),

            (value, DataType::UserDefined(expected))
                if value.data_type()?.to_string() == *expected =>
            {
                Ok(value)
            }

            // todo: handle type annotations of struct initialization
            _ => Err(RuntimeError::TypeMismatch {
                expected: expected.to_string(),
                found: value_data_type_string,
            }),
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
        let identifier = func.identifier.clone();
        self.validate_identifier(&identifier)?;

        self.functions
            .insert(identifier, RuntimeFunction::User(Rc::new(func.clone())));

        Ok(ControlFlow::Continue)
    }

    fn define_struct(&mut self, struct_definition: &StructDefinition) -> StatementResult {
        let identifier = struct_definition.identifier.clone();
        self.validate_identifier(&identifier)?;

        self.structs
            .insert(identifier, Rc::new(struct_definition.clone()));

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

            Expression::Binary { lhs, rhs, operator } => {
                let lhs = self.evaluate_expression(lhs)?;
                let rhs = self.evaluate_expression(rhs)?;

                self.evaluate_binary(*operator, lhs, rhs)
            }

            Expression::StructInitialization(init) => self.initialize_struct(init),

            Expression::StructFieldAccess { expression, field } => {
                let value = self.evaluate_expression(expression)?;

                match value {
                    RuntimeValue::Struct { definition, fields } => {
                        fields.get(field).cloned().ok_or(RuntimeError::InvalidStructFieldAccess {
                            field_name: field.clone(),
                            struct_name: definition.identifier.clone()
                        })
                    }

                    _ => Err(RuntimeError::InvalidStructFieldAccessTarget {
                        field: field.clone(),
                        data_type: value.data_type()?.to_string()
                    })
                }
            }
        }
    }

    fn initialize_struct(&mut self, init: &StructInitialization) -> RuntimeResult<RuntimeValue> {
        let definition = self.get_struct_definition(&init.identifier)?.clone();

        let mut runtime_struct = RuntimeValue::Struct {
            definition: definition.clone(),
            fields: HashMap::new(),
        };

        let mut struct_fields = HashMap::new();

        for field_definition in &definition.fields {
            if init
                .initialized_fields
                .iter()
                .find(|f| &f.identifier == &field_definition.identifier)
                .is_none()
            {
                if self.config.error_on_incomplete_struct_initialization {
                    return Err(RuntimeError::IncompleteStructInitialization(
                        definition.identifier.clone(),
                    ));
                } else {
                    todo!("implement default values");
                }
            } else {
                let initialized_field = init
                    .initialized_fields
                    .iter()
                    .find(|f| &f.identifier == &field_definition.identifier)
                    .unwrap();

                let value = self.evaluate_expression(&initialized_field.expression)?;

                let value = match self.apply_type_annotation(value, &field_definition.data_type) {
                    Ok(value) => value,
                    Err(RuntimeError::TypeMismatch { expected, found} ) => {
                        return Err(RuntimeError::InvalidStructFieldInitialization {
                            field_name: field_definition.identifier.clone(),
                            struct_name: definition.identifier.clone(),
                            expected,
                            found
                        })
                    }

                    Err(err) => return Err(err)
                };
                let value = self.apply_type_annotation(value, &field_definition.data_type)?;
                

                struct_fields.insert(initialized_field.identifier.clone(), value);
            }
        }

        if let RuntimeValue::Struct { fields, .. } = &mut runtime_struct {
            *fields = struct_fields;
        }

        Ok(runtime_struct)
    }

    fn evaluate_binary(
        &self,
        operator: BinaryOperator,
        lhs: RuntimeValue,
        rhs: RuntimeValue,
    ) -> RuntimeResult<RuntimeValue> {
        match operator {
            BinaryOperator::Add => lhs.add(rhs),
            BinaryOperator::Subtract => lhs.subtract(rhs),
            BinaryOperator::Multiply => lhs.multiply(rhs),
            BinaryOperator::Divide => lhs.divide(rhs),
        }
    }

    fn resolve_value(&self, value: &Value) -> RuntimeResult<RuntimeValue> {
        match value {
            Value::S32(i) => Ok(RuntimeValue::S32(*i)),

            Value::U32(i) => Ok(RuntimeValue::U32(*i)),

            Value::String(string) => Ok(RuntimeValue::String(string.clone())),

            Value::Identifier(name) => self.get_variable(name).cloned(),
        }
    }

    fn validate_identifier(&self, identifier: &str) -> RuntimeResult<()> {
        // check against data types and functions
        if self.structs.get(identifier).is_some() {
            Err(RuntimeError::AlreadyDefined(
                identifier.to_string(),
                "struct",
            ))
        } else if self.functions.get(identifier).is_some() {
            Err(RuntimeError::AlreadyDefined(
                identifier.to_string(),
                "function",
            ))
        } else {
            Ok(())
        }
    }
}
