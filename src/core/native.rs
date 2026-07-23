use crate::core::runtime::{NativeFunction, Runtime, RuntimeFunction};

pub mod util;
pub mod fs;

impl Runtime {
    // todo: at some point i want to import specific ones from modules

    fn register_native(&mut self, identifier: &str, func: NativeFunction) {
        self.functions_mut()
            .insert(identifier.to_string(), RuntimeFunction::Native(func));
    }

    pub fn register_native_functions(&mut self) {
        self.register_native("sg::print", util::sg::print);
        self.register_native("sg::write", fs::sg::write);
    }
}
