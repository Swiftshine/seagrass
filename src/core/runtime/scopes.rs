use std::collections::HashMap;

use crate::core::runtime::{Runtime, value::RuntimeReference};

#[derive(Debug, Clone)]
pub enum RuntimeScopeType {
    Global,
    Block,
    Function,
}

#[derive(Debug, Clone)]
pub struct RuntimeScope {
    pub _scope_type: RuntimeScopeType,
    pub variables: HashMap<String, RuntimeReference>,
}

impl Runtime {
    pub fn push_scope(&mut self) {
        if self.call_stack.is_empty() {
            self.global_sub_scopes
                .push(RuntimeScope::new(RuntimeScopeType::Block));
        } else {
            self.current_frame_mut().push_scope();
        }
    }
    
    pub fn pop_scope(&mut self) {
        if self.call_stack.is_empty() {
            self.global_sub_scopes.pop();
        } else {
            self.current_frame_mut().pop_scope();
        }
    }
}
