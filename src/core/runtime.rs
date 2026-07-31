pub(crate) mod accessors;
pub(crate) mod arrays;
pub(crate) mod evaluation;
pub(crate) mod execution;
pub(crate) mod functions;
pub(crate) mod operators;
pub(crate) mod scopes;
pub(crate) mod serialization;
pub(crate) mod structs;
pub(crate) mod value;

pub use value::RuntimeValue;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::{
    lang::ast::{AssignmentTarget, FunctionDefinition, Parameter, StructDefinition, StructImpl},
    native::BuiltinMethodTarget,
    runtime::{
        functions::{FunctionFrame, NativeFunction, RuntimeFunction},
        scopes::{RuntimeScope, RuntimeScopeType},
        serialization::ByteOrder,
    },
};

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
    #[error("Method '{method}' is not a built-in method for '{data_type}'")]
    NotABuiltInMethod { method: String, data_type: String },
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
    #[error("Attribute '{0}' not found")]
    AttributeNotFound(&'static str),
    #[error("Cannot infer array type from empty array")]
    CannotInferEmptyArrayType,
    #[error("Array index {0} invalid")]
    InvalidArrayIndex(String),
    #[error("Expected integral type for ranges, found '{0}'")]
    ExpectedIntegerForRange(String),
    #[error("Expected integral type, found '{0}'")]
    ExpectedInteger(String),

    // Mismatches
    #[error("Unsupported binary operation for '[{lhs_type}] {operation} [{rhs_type}]'")]
    UnsupportedBinaryOperation {
        lhs_type: String,
        operation: &'static str,
        rhs_type: String,
    },
    #[error("Unsupported unary operation for '{0}[{1}]'")]
    UnsupportedUnaryOperation(&'static str, String),
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
    #[error("Cannot coerce a '{from}' into a '{to}'")]
    TypeCoercionFail { from: String, to: String },
    #[error("Cannot invoke a method on type '{0}'")]
    CannotInvokeMethodOnType(String),
    #[error("Cannot iterate on type '{0}'")]
    CannotIterateOnType(String),
    #[error("Cannot cast a '{0}' to a '{1}'")]
    InvalidCast(String, String),

    // Semantic errors
    #[error("Incomplete struct initialization for '{0}'")]
    IncompleteStructInitialization(String),
    #[error("Incomplete array initialization for '{0}'")]
    IncompleteArrayInitialization(String),
    #[error(
        "Attempted to initialize array of type '{array_type}' with too many elements (expected {expected}, found {found}"
    )]
    TooManyArrayElementsForInitialization {
        array_type: String,
        expected: usize,
        found: usize,
    },
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
    #[error("Attribute '{attribute}' was given '{found}', but it is not a valid argument")]
    UnexpectedAttributeArgument { attribute: String, found: String },
    #[error("Attribute '{attribute}' expects {expected} arguments, but found {found}")]
    InvalidAttributeArgumentCount {
        attribute: String,
        expected: usize,
        found: usize,
    },
    #[error("Invalid function arguments for [signature]. {note}")]
    InvalidFunctionArguments { signature: String, note: String },
    #[error("References cannot have default values")]
    CannotDefaultInitializeReference,
    #[error("Iterators cannot have default values")]
    CannotDefaultInitializeIterator,

    #[error("Array index {index} out of bounds when the length is {length}")]
    ArrayIndexOutOfBounds { index: usize, length: usize },
    #[error("Cannot index type {0}")]
    CannotIndexNonArrayType(String),

    // De/Serialization errors
    #[error("Attempted to serialize the following non-ascii string as ascii: {0}")]
    NonAsciiString(String),
    #[error("Unexpectedly reached EOF when deserializing file")]
    UnexpectedEOF,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Cannot deserialize a '{0}'")]
    CannotDeserialize(String),
    #[error("An array must have a count")]
    UncountedArray,
    #[error("Failed to perform I/O operation: {0}")]
    Io(#[from] std::io::Error),

    // Attribute errors
    #[error(
        "Struct field '{0}' not read before trying to use the \"counted_by\" attribute or is non-numeric"
    )]
    CountedByFail(String),
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

#[derive(Clone, Copy)]
pub enum RuntimeConfigOption {
    PreserveExpiredFrames(bool),
}

#[derive(Debug, Default)]
pub struct RuntimeConfig {
    /// (Development) Allows expired function frames and scopes to be preserved to inspect its end-of-life state.
    preserve_expired_frames: bool,
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
    builtin_methods: HashMap<BuiltinMethodTarget, HashMap<String, NativeFunction>>,
    config: RuntimeConfig,
    byte_order: ByteOrder,
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
            byte_order: ByteOrder::Little,
            builtin_methods: HashMap::new(),
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

    fn assign_variable(&mut self, identifier: String, value: RuntimeValue) {
        let variable = Rc::new(RefCell::new(value));

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

    fn assign_to_target(
        &mut self,
        target: &AssignmentTarget,
        mut value: RuntimeValue,
    ) -> RuntimeResult<()> {
        let lvalue = self.evaluate_lvalue(target)?;

        let existing_type = lvalue.read_value()?.data_type()?;

        value = self.coerce(value, &existing_type)?;

        lvalue.write_value(value)?;

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

    fn validate_identifier(&self, identifier: &str) -> RuntimeResult<()> {
        // check against data types and functions
        if self.structs.contains_key(identifier) {
            Err(RuntimeError::AlreadyDefined(
                identifier.to_string(),
                "struct",
            ))
        } else if self.functions.contains_key(identifier) {
            Err(RuntimeError::AlreadyDefined(
                identifier.to_string(),
                "function",
            ))
        } else {
            Ok(())
        }
    }

    pub fn set_byte_order(&mut self, byte_order: ByteOrder) {
        self.byte_order = byte_order;
    }

    pub fn builtin_methods_mut(
        &mut self,
    ) -> &mut HashMap<BuiltinMethodTarget, HashMap<String, NativeFunction>> {
        &mut self.builtin_methods
    }

    pub fn builtin_methods(
        &self,
    ) -> &HashMap<BuiltinMethodTarget, HashMap<String, NativeFunction>> {
        &self.builtin_methods
    }
}
