use std::{cell::RefCell, rc::Rc};

use crate::core::{
    lang::ast::{Block, DataType, FunctionDefinition},
    native::NativeFunctionContext,
    runtime::{
        ControlFlow, Runtime, RuntimeError, RuntimeResult, RuntimeScope, RuntimeScopeType,
        RuntimeValue, StatementResult, value::RuntimeReference,
    },
};

#[derive(Debug)]
pub struct FunctionFrame {
    pub name: String,
    scopes: Vec<RuntimeScope>,
}

impl FunctionFrame {
    pub fn new(name: String, args: Vec<(String, RuntimeValue)>) -> Self {
        let mut scope = RuntimeScope::new(RuntimeScopeType::Function);

        for (identifier, value) in args {
            let var_ref = Rc::new(RefCell::new(value));
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

pub type NativeFunction = fn(NativeFunctionContext) -> RuntimeResult<RuntimeValue>;

#[derive(Debug, Clone)]
pub enum RuntimeFunction {
    Native(NativeFunction),
    User(Rc<FunctionDefinition>),
}

impl Runtime {
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

    pub fn execute_function_body(&mut self, block: &Block) -> StatementResult {
        for statement in &block.statements {
            match self.execute_statement(statement)? {
                ControlFlow::Continue => {}
                flow @ ControlFlow::Return(_) => return Ok(flow),
                _ => unreachable!("{:?}", statement),
            }
        }

        Ok(ControlFlow::Continue)
    }

    pub fn call_function(
        &mut self,
        identifier: &str,
        args: Vec<RuntimeValue>,
        generics: &Vec<DataType>,
    ) -> RuntimeResult<RuntimeValue> {
        let func = self
            .functions
            .get(identifier)
            .cloned()
            .ok_or(RuntimeError::FunctionNotFound(identifier.to_string()))?;

        match func {
            RuntimeFunction::Native(native) => {
                native(NativeFunctionContext::new(self, args, generics))
            }

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
}
