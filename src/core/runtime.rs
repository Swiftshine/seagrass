mod accessors;
mod operators;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::lang::ast::{
    Assignment, AssignmentTarget, Attribute, BinaryOperator, Block, ControlStatement, DataType, Expression, FunctionDefinition, MethodDefinition, Parameter, Program, Return, Statement, StructDefinition, StructImpl, StructInitialization, Value,
};

pub type RuntimeReference = Rc<RefCell<RuntimeVariable>>;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeVariable {
    pub value: RuntimeValue,
}

impl RuntimeVariable {
    pub fn from_value(value: RuntimeValue) -> Self {
        Self { value }
    }

    pub fn value(&self) -> RuntimeValue {
        self.value.clone()
    }

    pub fn set_value(&mut self, value: RuntimeValue) {
        self.value = value;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    None,
    U32(u32),
    S32(i32),
    String(String),
    Bool(bool),
    Struct {
        definition: Rc<StructDefinition>,
        fields: HashMap<String, RuntimeValue>,
    },
    Reference(RuntimeReference),
}

impl RuntimeValue {
    pub fn data_type(&self) -> RuntimeResult<DataType> {
        match self {
            Self::None => Err(RuntimeError::NoDataTypeAttached),
            Self::U32(_) => Ok(DataType::U32),
            Self::S32(_) => Ok(DataType::S32),
            Self::String(_) => Ok(DataType::String),
            Self::Bool(_) => Ok(DataType::Bool),
            Self::Struct { definition, .. } => {
                Ok(DataType::UserDefined(definition.identifier.clone()))
            }
            Self::Reference(variable) => Ok(DataType::Reference(Box::new(
                variable.borrow().value().data_type()?,
            ))),
        }
    }

    pub fn struct_access(&self, identifier: &str) -> RuntimeResult<RuntimeValue> {
        match self {
            Self::Struct { definition, fields } => {
                fields
                    .get(identifier)
                    .cloned()
                    .ok_or(RuntimeError::InvalidStructFieldAccess {
                        field_name: identifier.to_string(),
                        struct_name: definition.identifier.clone(),
                    })
            }

            Self::Reference(_) => self.dereference(),

            _ => Err(RuntimeError::InvalidStructFieldAccessTarget {
                field: identifier.to_string(),
                data_type: self.data_type()?.to_string(),
            }),
        }
    }

    pub fn resolve(&self) -> RuntimeValue {
        // this function should only be called for struct
        match self {
            Self::Reference(reference) => reference.borrow().value().resolve(),
            _ => self.clone(),
        }
    }

    // pub fn reference(self) -> RuntimeValue {
    //     RuntimeValue::Reference(
    //         Rc::new(RefCell::new(self))
    //     )
    // }

    pub fn dereference(&self) -> RuntimeResult<RuntimeValue> {
        match self {
            RuntimeValue::Reference(variable) => Ok(variable.borrow().value().clone()),
            _ => Err(RuntimeError::CannotDereferenceNonReference),
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference(_))
    }

    pub fn assert_reference(&self) -> RuntimeResult<()> {
        if !self.is_reference() {
            Err(RuntimeError::ExpectedReference(
                self.data_type()?.to_string(),
            ))
        } else {
            Ok(())
        }
    }

    // pub fn assert_struct(&self) -> RuntimeResult<()> {
    //     match self {
    //         RuntimeValue::Struct { .. } => Ok(()),
    //         RuntimeValue::Reference(reference) => {
    //             // the first dereference and the first dereference only must resolve to a struct
    //             match reference.borrow().value() {
    //                 RuntimeValue::Struct { .. } => Ok(()),
    //                 _ => Err(RuntimeError::ExpectedStruct),
    //             }
    //         }
    //         _ => Err(RuntimeError::ExpectedStruct),
    //     }
    // }
}

pub type NativeFunction = fn(&mut Runtime, Vec<RuntimeValue>) -> RuntimeResult<RuntimeValue>;

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
    #[error("Method '{method_identifier}' not defined for '{struct_identifier}'")]
    MethodNotFound {
        method_identifier: String,
        struct_identifier: String,
    },
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
        struct_name: String,
    },
    #[error("Struct definition '{0}' not found")]
    StructDefinitionNotFound(String),
    #[error("Struct impl '{0}' not found")]
    StructImplNotFound(String),
    #[error("Expected reference, found '{0}'")]
    ExpectedReference(String),

    // Mismatches
    #[error("Unsupported binary operation for '[{lhs_type}] {operation} [{rhs_type}]'")]
    UnsupportedBinaryOperation {
        lhs_type: String,
        operation: &'static str,
        rhs_type: String,
    },
    #[error(
        "Failed to apply type annotation (expected annotated type '{expected}', but the assigned value resolved to '{found}')"
    )]
    AnnotationError { expected: String, found: String },
    #[error("Identifier '{0}' already defined as a {1}")]
    AlreadyDefined(String, &'static str),
    #[error("Invalid reference target")]
    InvalidReferenceTarget,
    #[error("Cannot compare structs of type '{0}' and '{1}'")]
    InvalidStructComparison(String, String),
    #[error("The expression given does not resolve to a boolean value")]
    ExpectedBoolean,
    #[error("The expression given does not resolve to a struct or struct reference")]
    ExpectedStruct,
    #[error("Cannot dereference a type that is not a reference")]
    CannotDereferenceNonReference,


    // Semantic errors
    #[error("Incomplete struct initialization for '{0}'")]
    IncompleteStructInitialization(String),
    #[error(
        "Invalid initialization type for field '{field_name}' of struct '{struct_name}' (expected '{expected}', found '{found}')"
    )]
    InvalidStructFieldInitialization {
        field_name: String,
        struct_name: String,
        expected: String,
        found: String,
    },
    #[error("Cannot access field '{field}' of '{data_type}' because it is not a struct")]
    InvalidStructFieldAccessTarget { field: String, data_type: String },
    #[error("Field of type '{0}' is not POD")]
    NonPODType(String),
    #[error("Attribute '{attribute}' expects {expected}, but found '{found}'")]
    InvalidAttributeArgument {
        attribute: String,
        expected: String,
        found: String,
    },

    #[error("Attribute '{attribute}' expects {expected} arguments, but found {found}")]
    InvalidAttributeArgumentCount {
        attribute: String,
        expected: usize,
        found: usize,
    },
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
    Break,
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
    variables: HashMap<String, RuntimeReference>,
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
    pub fn new(name: String, args: Vec<(String, RuntimeValue)>) -> Self {
        let mut scope = RuntimeScope::new(RuntimeScopeType::Function);

        for (identifier, value) in args {
            let var_ref = Rc::new(RefCell::new(RuntimeVariable::from_value(value)));
            scope.variables.insert(identifier, var_ref);
        }

        Self {
            name,
            scopes: vec![scope],
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

    pub fn get_variable(&self, identifier: &str) -> Option<RuntimeReference> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.variables().get(identifier).cloned())
    }

    pub fn get_variable_mut(&mut self, identifier: &str) -> Option<RuntimeReference> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.variables().get(identifier).cloned())
    }
}

impl StructImpl {
    pub fn get_method_definition(&self, identifier: &str) -> RuntimeResult<&MethodDefinition> {
        self.method_definitions
            .iter()
            .find(|m| m.identifier == identifier)
            .ok_or(RuntimeError::MethodNotFound {
                method_identifier: identifier.to_string(),
                struct_identifier: self.struct_identifier.clone(),
            })
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteOrder {
    Little,
    Big,
}


impl StructDefinition {
    pub fn has_attribute(&self, identifier: &str) -> bool {
        self.attributes
            .iter()
            .any(|a| a.identifier == identifier)
    }

    pub fn get_attribute(&self, identifier: &str) -> Option<&Attribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.identifier == identifier)
    }

    // "is declared" because the user could say it's pod when it's really not
    pub fn is_declared_pod(&self) -> bool {
        self.has_attribute("pod")
    }

    pub fn byte_order(&self) -> RuntimeResult<ByteOrder> {
        let Some(attribute) = self.get_attribute("byte_order") else {
            return Ok(ByteOrder::Little);
        };
    
        if attribute.arguments.len() != 1 {
            return Err(RuntimeError::InvalidAttributeArgumentCount {
                attribute: "byte_order".to_string(),
                expected: 1,
                found: attribute.arguments.len(),
            });
        }
    
        match &attribute.arguments[0] {
            Expression::Value(Value::String(value)) => match value.as_str() {
                "little" => Ok(ByteOrder::Little),
                "big" => Ok(ByteOrder::Big),
    
                other => Err(RuntimeError::InvalidAttributeArgument {
                    attribute: "byte_order".to_string(),
                    expected: "\"little\" or \"big\"".to_string(),
                    found: other.to_string(),
                }),
            },
    
            other => Err(RuntimeError::InvalidAttributeArgument {
                attribute: "byte_order".to_string(),
                expected: "string literal".to_string(),
                found: format!("{:?}", other),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Runtime {
    global_scope: RuntimeScope,
    global_sub_scopes: Vec<RuntimeScope>,
    call_stack: Vec<FunctionFrame>,
    dead_frames: Vec<FunctionFrame>,
    functions: HashMap<String, RuntimeFunction>,
    structs: HashMap<String, Rc<StructDefinition>>,
    struct_impls: HashMap<String, Rc<StructImpl>>,
    config: RuntimeConfig,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            global_scope: RuntimeScope::new(RuntimeScopeType::Global),
            global_sub_scopes: Vec::new(),
            call_stack: Vec::new(),
            dead_frames: Vec::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            struct_impls: HashMap::new(),
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

    pub fn push_frame(&mut self, identifier: String, args: Vec<(String, RuntimeValue)>) {
        self.call_stack.push(FunctionFrame::new(identifier, args));
    }

    pub fn pop_frame(&mut self) {
        if let Some(frame) = self.call_stack.pop()
            && self.config.preserve_expired_frames
        {
            self.dead_frames.push(frame);
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
                Statement::StructImpl(struct_impl) => {
                    self.impl_struct(struct_impl)?;
                }
                _ => {}
            }
        }

        // execute normal statements
        for statement in &program.statements {
            match statement {
                Statement::FunctionDefinition(_)
                | Statement::StructDefinition(_)
                | Statement::StructImpl(_) => {}
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

            Statement::ControlStatement(control_statement) => {
                self.execute_control_statement(control_statement)
            }

            Statement::Break => Ok(ControlFlow::Break),

            _ => unreachable!("{:?}", statement),
        }
    }

    fn execute_control_statement(
        &mut self,
        control_statement: &ControlStatement,
    ) -> StatementResult {
        match control_statement {
            ControlStatement::If {
                expression,
                block,
                children,
            } => self.execute_if(expression, block, children),

            ControlStatement::While { expression, block } => self.execute_while(expression, block),

            ControlStatement::Loop { block } => self.execute_loop(block),

            _ => unreachable!("{:?}", control_statement),
        }
    }

    fn execute_loop(&mut self, block: &Block) -> StatementResult {
        loop {
            let flow = self.execute_block(block)?;

            if matches!(flow, ControlFlow::Break) {
                break;
            }
        }

        Ok(ControlFlow::Continue)
    }

    fn execute_while(&mut self, expression: &Expression, block: &Block) -> StatementResult {
        while self.evaluate_boolean_expression(expression)? {
            let flow = self.execute_block(block)?;

            if matches!(flow, ControlFlow::Break) {
                break;
            }
        }

        Ok(ControlFlow::Continue)
    }

    fn execute_if(
        &mut self,
        expression: &Expression,
        block: &Block,
        children: &Vec<ControlStatement>,
    ) -> StatementResult {
        // execute our statement first

        if self.evaluate_boolean_expression(expression)? {
            return self.execute_block(block);
        }

        // check children

        for child in children {
            match child {
                ControlStatement::ElseIf { expression, block } => {
                    if self.evaluate_boolean_expression(expression)? {
                        return self.execute_block(block);
                    }
                }

                ControlStatement::Else { block } => {
                    return self.execute_block(block);
                }

                _ => {
                    unreachable!("{:?}", child);
                }
            }
        }

        // continue anyway
        Ok(ControlFlow::Continue)
    }

    fn evaluate_boolean_expression(&mut self, expression: &Expression) -> RuntimeResult<bool> {
        let value = self.evaluate_expression(expression)?;

        if let RuntimeValue::Bool(boolean) = value {
            Ok(boolean)
        } else {
            Err(RuntimeError::ExpectedBoolean)
        }
    }

    fn validate_pod(&self, struct_definition: &StructDefinition) -> RuntimeResult<()> {
        for field in &struct_definition.fields {
            self.validate_pod_type(&field.data_type)?;
        }

        Ok(())
    }

    fn validate_pod_type(&self, data_type: &DataType) -> RuntimeResult<()> {
        match data_type {
            DataType::U32
            | DataType::S32
            | DataType::Bool => Ok(()),
    
            DataType::String | DataType::Reference(_) => {
                Err(RuntimeError::NonPODType(data_type.to_string()))
            }
    
            DataType::UserDefined(name) => {
                let definition = self.get_struct_definition(name)?;
    
                if !definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(name.clone()));
                }
    
                self.validate_pod(definition)
            }
        }
    }

    fn execute_assignment(&mut self, assignment: &Assignment) -> StatementResult {
        let mut value = self.evaluate_expression(&assignment.expression)?;

        if let Some(expected_type) = &assignment.data_type {
            value = self.apply_type_annotation(value, expected_type)?;
        }

        if assignment.declarative {
            match &assignment.target {
                AssignmentTarget::Identifier(ident) => {
                    self.assign_variable(ident.clone(), value);
                }

                _ => unreachable!("invalid declarative assignment target"),
            }
        } else {
            self.assign_to_target(&assignment.target, value)?;
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

            (RuntimeValue::Bool(b), DataType::Bool) => Ok(RuntimeValue::Bool(b)),

            (value, DataType::UserDefined(expected))
                if value.data_type()?.to_string() == *expected =>
            {
                Ok(value)
            }

            // todo: handle type annotations of struct initialization
            _ => Err(RuntimeError::AnnotationError {
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
                _ => unreachable!("{:?}", statement),
            }
        }

        Ok(ControlFlow::Continue)
    }

    fn execute_block(&mut self, block: &Block) -> StatementResult {
        self.push_scope();

        let result = (|| {
            for statement in &block.statements {
                let flow = self.execute_statement(statement)?;

                match flow {
                    ControlFlow::Continue => {}
                    ControlFlow::Return(_) | ControlFlow::Break => return Ok(flow),
                }
            }

            Ok(ControlFlow::Continue)
        })();

        self.pop_scope();

        result
    }

    fn assign_variable(&mut self, identifier: String, value: RuntimeValue) {
        let variable = Rc::new(RefCell::new(RuntimeVariable::from_value(value)));

        if self.call_stack.is_empty() {
            if !self.global_sub_scopes.is_empty() {
                self.global_sub_scopes
                    .last_mut()
                    .unwrap()
                    .variables_mut()
                    .insert(identifier, variable);
            } else {
                self.global_scope
                    .variables_mut()
                    .insert(identifier, variable);
            }
        } else {
            self.current_frame_mut()
                .current_scope_mut()
                .variables_mut()
                .insert(identifier, variable);
        }
    }

    fn push_scope(&mut self) {
        if self.call_stack.is_empty() {
            self.global_sub_scopes
                .push(RuntimeScope::new(RuntimeScopeType::Block));
        } else {
            self.current_frame_mut().push_scope();
        }
    }

    fn pop_scope(&mut self) {
        if self.call_stack.is_empty() {
            self.global_sub_scopes.pop();
        } else {
            self.current_frame_mut().pop_scope();
        }
    }

    fn assign_to_target(
        &mut self,
        target: &AssignmentTarget,
        value: RuntimeValue,
    ) -> RuntimeResult<()> {
        match target {
            AssignmentTarget::Identifier(name) => {
                let variable = self.get_variable(name)?;
                variable.borrow_mut().set_value(value);
            }

            AssignmentTarget::Dereference(expression) => {
                let reference = self.evaluate_expression(expression)?;

                match reference {
                    RuntimeValue::Reference(reference) => {
                        *reference.borrow_mut() = RuntimeVariable::from_value(value);
                    }

                    other => {
                        return Err(RuntimeError::ExpectedReference(
                            other.data_type()?.to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
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

        if struct_definition.is_declared_pod() {
            self.validate_pod(struct_definition)?;
        }

        self.structs
            .insert(identifier, Rc::new(struct_definition.clone()));

        Ok(ControlFlow::Continue)
    }

    fn impl_struct(&mut self, struct_impl: &StructImpl) -> StatementResult {
        let identifier = struct_impl.struct_identifier.clone();
        // self.validate_identifier(&identifier)?;

        self.struct_impls
            .insert(identifier, Rc::new(struct_impl.clone()));

        Ok(ControlFlow::Continue)
    }

    fn invoke_method(
        &mut self,
        struct_identifier: &str,
        method_identifier: &str,
        struct_reference: RuntimeValue,
        args: Vec<RuntimeValue>,
    ) -> RuntimeResult<RuntimeValue> {
        // make sure we have the struct definition and the function the method call
        // is asking for
        self.get_struct_definition(&struct_identifier)?;

        let parameters = &self
            .get_struct_impl(&struct_identifier)?
            .get_method_definition(method_identifier)?
            .parameters;

        let mut args = Self::collect_runtime_function_arguments(parameters, args)?;

        let scope_resolved_name = struct_identifier.to_string() + "::" + method_identifier;

        struct_reference.assert_reference()?;
        args.insert(0, ("self".to_string(), struct_reference));

        self.push_frame(scope_resolved_name, args);

        let block = &self
            .get_struct_impl(&struct_identifier)?
            .get_method_definition(method_identifier)?
            .body
            .clone();

        let result = self.execute_function_body(&block);
        self.pop_frame();

        match result? {
            ControlFlow::Continue => Ok(RuntimeValue::None),
            ControlFlow::Return(value) => Ok(value),
            _ => unreachable!("expected ControlFlow::Continue or ControlFlow::Return"),
        }
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
            RuntimeFunction::Native(native) => native(self, args),

            RuntimeFunction::User(func) => {
                let args = Self::collect_runtime_function_arguments(&func.parameters, args)?;

                self.push_frame(identifier.to_string(), args);
                let result = self.execute_function_body(&func.body);
                self.pop_frame();

                match result? {
                    ControlFlow::Continue => Ok(RuntimeValue::None),
                    ControlFlow::Return(value) => Ok(value),
                    _ => unreachable!("expected ControlFlow::Continue or ControlFlow::Return"),
                }
            }
        }
    }

    // todo: maybe use a struct for this
    fn collect_runtime_function_arguments(
        ast_parameters: &[Parameter],
        runtime_values: Vec<RuntimeValue>,
    ) -> RuntimeResult<Vec<(String, RuntimeValue)>> {
        assert_eq!(ast_parameters.len(), runtime_values.len());

        let mut args = Vec::new();

        for (index, value) in runtime_values.into_iter().enumerate() {
            let identifier = ast_parameters[index].identifier.clone();
            args.push((identifier, value));
        }

        Ok(args)
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

            Expression::StructFieldAccess {
                expression,
                field_identifier: field,
            } => {
                let value = self.evaluate_expression(expression)?.resolve();

                match value {
                    RuntimeValue::Struct { definition, fields } => fields
                        .get(field)
                        .cloned()
                        .ok_or(RuntimeError::InvalidStructFieldAccess {
                            field_name: field.clone(),
                            struct_name: definition.identifier.clone(),
                        }),

                    _ => Err(RuntimeError::InvalidStructFieldAccessTarget {
                        field: field.clone(),
                        data_type: value.data_type()?.to_string(),
                    }),
                }
            }

            Expression::Reference(expression) => match expression.as_ref() {
                Expression::Value(Value::Identifier(identifier)) => {
                    let variable = self.get_variable(identifier)?;

                    Ok(RuntimeValue::Reference(variable))
                }

                _ => Err(RuntimeError::InvalidReferenceTarget),
            },

            Expression::Dereference(expression) => {
                let value = self.evaluate_expression(expression)?;
                value.dereference()
            }

            Expression::MethodCall {
                expression,
                method_identifier,
                arguments,
            } => {
                // later on i plan on implementing custom functions for native types
                // but for now, structs only

                let value = self.evaluate_method_receiver(expression)?;
                value.assert_reference()?;

                // find name of the struct definition
                let struct_identifier = if value.is_reference() {
                    value.dereference()?.data_type()?
                } else {
                    value.data_type()?
                }
                .to_string();

                let args = arguments
                    .iter()
                    .flat_map(|expr| self.evaluate_expression(expr))
                    .collect();

                self.invoke_method(&struct_identifier, method_identifier, value, args)
            }
        }
    }

    fn evaluate_method_receiver(&mut self, expression: &Expression) -> RuntimeResult<RuntimeValue> {
        match expression {
            Expression::Reference(_) => {
                self.evaluate_expression(expression)
            }
        
            Expression::Value(Value::Identifier(name)) => {
                Ok(RuntimeValue::Reference(
                    self.get_variable(name)?
                ))
            }
        
            _ => {
                Err(RuntimeError::InvalidReferenceTarget)
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
                .find(|f| f.identifier == field_definition.identifier)
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
                    .find(|f| f.identifier == field_definition.identifier)
                    .unwrap();

                let value = self.evaluate_expression(&initialized_field.expression)?;

                let value = match self.apply_type_annotation(value, &field_definition.data_type) {
                    Ok(value) => value,
                    Err(RuntimeError::AnnotationError { expected, found }) => {
                        return Err(RuntimeError::InvalidStructFieldInitialization {
                            field_name: field_definition.identifier.clone(),
                            struct_name: definition.identifier.clone(),
                            expected,
                            found,
                        });
                    }

                    Err(err) => return Err(err),
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
            BinaryOperator::EqualTo => lhs.compare_eq(rhs),
            BinaryOperator::NotEqualTo => lhs.compare_neq(rhs),
        }
    }

    fn resolve_value(&self, value: &Value) -> RuntimeResult<RuntimeValue> {
        match value {
            Value::S32(i) => Ok(RuntimeValue::S32(*i)),

            Value::U32(i) => Ok(RuntimeValue::U32(*i)),

            Value::String(string) => Ok(RuntimeValue::String(string.clone())),

            Value::Bool(b) => Ok(RuntimeValue::Bool(*b)),

            Value::Identifier(name) => {
                let var = self.get_variable(name)?;
                Ok(var.borrow().value())
            }
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

    pub fn serialize_into(
        &self,
        value: &RuntimeValue,
        output: &mut Vec<u8>,
    ) -> RuntimeResult<()> {
        let byte_order = ByteOrder::Little; // todo! do self.byte_order instead and make this a runtime configuration

        match value {
            RuntimeValue::Reference(reference) => {
                let value = reference.borrow();
    
                self.serialize_value(
                    &value.value(),
                    output,
                    byte_order
                )
            }
    
            value => {
                self.serialize_value(
                    value,
                    output,
                    byte_order
                )
            }
        }
    }

    fn serialize_value(
        &self,
        value: &RuntimeValue,
        output: &mut Vec<u8>,
        byte_order: ByteOrder,
    ) -> RuntimeResult<()> {
        match value {
            RuntimeValue::U32(value) => {
                match byte_order {
                    ByteOrder::Little => {
                        output.extend(value.to_le_bytes())
                    }

                    ByteOrder::Big => {
                        output.extend(value.to_be_bytes())
                    }
                }
            }

            RuntimeValue::S32(value) => {
                match byte_order {
                    ByteOrder::Little => {
                        output.extend(value.to_le_bytes())
                    }

                    ByteOrder::Big => {
                        output.extend(value.to_be_bytes())
                    }
                }
            }

            RuntimeValue::Bool(value) => {
                output.push(if *value { 1 } else { 0 });
            }

            RuntimeValue::Struct {
                definition,
                fields,
            } => {
                if !definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(
                        definition.identifier.clone()
                    ));
                }

                let struct_byte_order = definition.byte_order()?;

                for field in &definition.fields {
                    let value = fields
                        .get(&field.identifier)
                        .unwrap();

                    self.serialize_value(
                        value,
                        output,
                        struct_byte_order,
                    )?;
                }
            }

            _ => {
                return Err(RuntimeError::NonPODType(
                    value.data_type()?.to_string()
                ));
            }
        }

        Ok(())
    }
}
