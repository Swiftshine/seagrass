use crate::core::{
    lang::ast::DataType,
    runtime::{
        Runtime, RuntimeValue,
        functions::{NativeFunction, RuntimeFunction},
    },
};

pub mod fs;
pub mod util;

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

impl Runtime {
    // todo: at some point i want to import specific ones from modules

    fn register_native(&mut self, identifier: &str, func: NativeFunction) {
        self.functions_mut()
            .insert(identifier.to_string(), RuntimeFunction::Native(func));
    }

    pub fn register_native_functions(&mut self) {
        self.register_native("sg::print", util::sg::print);
        self.register_native("sg::set_byte_order", util::sg::set_byte_order);
        self.register_native("sg::write", fs::sg::write);
        self.register_native("sg::read", fs::sg::read);
    }
}
