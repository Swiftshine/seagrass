use std::collections::HashMap;

use crate::core::{
    lang::ast::DataType,
    native::builtin::sg,
    runtime::{
        Runtime, RuntimeValue,
        functions::{NativeFunction, RuntimeFunction},
    },
};

pub mod builtin;
pub mod fs;
pub mod util;

#[derive(Debug)]
pub struct NativeFunctionContext<'a> {
    pub runtime: &'a mut Runtime,
    pub arguments: Vec<RuntimeValue>,
    pub generics: &'a Vec<DataType>,
}

impl<'a> NativeFunctionContext<'a> {
    pub fn new(
        runtime: &'a mut Runtime,
        arguments: Vec<RuntimeValue>,
        generics: &'a Vec<DataType>,
    ) -> Self {
        Self {
            runtime,
            arguments,
            generics,
        }
    }
}

#[derive(Hash, Debug, Eq, PartialEq)]
pub enum BuiltinMethodTarget {
    Array,
}

impl Runtime {
    // todo: at some point i want to import specific ones from modules

    fn register_native(&mut self, identifier: &str, func: NativeFunction) {
        self.functions_mut()
            .insert(identifier.to_string(), RuntimeFunction::Native(func));
    }

    pub fn register_native_functions(&mut self) {
        let pairs: [(&str, NativeFunction); 4] = [
            ("sg::print", util::sg::print),
            ("sg::set_byte_order", util::sg::set_byte_order),
            ("sg::write", fs::sg::write),
            ("sg::read", fs::sg::read),
        ];

        for (identifier, func) in pairs {
            self.register_native(identifier, func);
        }
    }

    fn register_builtin(
        map: &mut HashMap<String, NativeFunction>,
        identifier: &str,
        method: NativeFunction,
    ) {
        map.insert(identifier.to_string(), method);
    }

    pub fn register_builtin_methods(&mut self) {
        self.builtin_methods_mut()
            .insert(BuiltinMethodTarget::Array, HashMap::new());

        let pairs: [(&str, NativeFunction); 1] = [("iter", sg::arrays::array_iterator)];

        for (identifier, func) in pairs {
            Self::register_builtin(
                self.builtin_methods_mut()
                    .get_mut(&BuiltinMethodTarget::Array)
                    .unwrap(),
                identifier,
                func,
            );
        }
    }
}
