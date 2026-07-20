use std::{collections::HashMap, rc::Rc};

use crate::core::{
    lang::ast::{FunctionDefinition, StructDefinition},
    runtime::{
        FunctionFrame, Runtime, RuntimeError, RuntimeFunction, RuntimeReference, RuntimeResult,
        RuntimeScope, RuntimeScopeType, RuntimeValue,
    },
};

impl RuntimeScope {
    pub fn new(scope_type: RuntimeScopeType) -> Self {
        Self {
            _scope_type: scope_type,
            variables: HashMap::new(),
        }
    }

    pub fn get_variable(&self, identifier: &str) -> RuntimeResult<RuntimeReference> {
        self.variables
            .get(identifier)
            .cloned()
            .ok_or(RuntimeError::VariableNotFound(identifier.to_string()))
    }

    pub fn variables(&self) -> &HashMap<String, RuntimeReference> {
        &self.variables
    }

    pub fn variables_mut(&mut self) -> &mut HashMap<String, RuntimeReference> {
        &mut self.variables
    }
}

impl Runtime {
    pub fn get_struct_definition(&self, identifier: &str) -> RuntimeResult<&Rc<StructDefinition>> {
        self.structs
            .get(identifier)
            .ok_or(RuntimeError::StructDefinitionNotFound(
                identifier.to_string(),
            ))
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

    pub fn get_global_variable(&self, identifier: &str) -> RuntimeResult<RuntimeReference> {
        self.global_scope.get_variable(identifier)
    }

    /// Within the current scope.
    pub fn get_variable(&self, identifier: &str) -> RuntimeResult<RuntimeReference> {
        if let Some(frame) = self.call_stack.last()
            && let Some(variable) = frame.get_variable(identifier) {
                return Ok(variable);
            }

        self.get_global_variable(identifier)
    }

    pub fn functions(&self) -> &HashMap<String, RuntimeFunction> {
        &self.functions
    }

    pub fn functions_mut(&mut self) -> &mut HashMap<String, RuntimeFunction> {
        &mut self.functions
    }

    pub fn get_function(&self, identifier: &str) -> RuntimeResult<&RuntimeFunction> {
        self.functions
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
}
