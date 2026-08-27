use std::collections::HashMap;

use crate::core::{
    lang::ast::DataType,
    native::builtin::sg,
    runtime::{
        Runtime, RuntimeError, RuntimeResult, RuntimeValue,
        functions::{NativeFunction, RuntimeFunction},
    },
};

pub mod aque;
pub mod builtin;
pub mod fs;
pub mod nativeobject;
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

    pub fn assert_arguments(
        &self,
        function_name: &'static str,
        arg_names: &[&'static str],
    ) -> RuntimeResult<()> {
        if self.arguments.len() < arg_names.len() {
            return Err(RuntimeError::MissingNativeArguments {
                function_name,
                expected: arg_names.join(", "),
                found: self.arguments.len(),
            });
        }

        Ok(())
    }

    pub fn assert_generics(
        &self,
        function_name: &'static str,
        generic_names: &[&'static str],
    ) -> RuntimeResult<()> {
        if self.generics.len() < generic_names.len() {
            return Err(RuntimeError::MissingNativeGenerics {
                function_name,
                expected: generic_names.join(", "),
                found: self.generics.len(),
            });
        }

        Ok(())
    }
}

#[derive(Hash, Debug, Eq, PartialEq)]
pub enum BuiltinMethodTarget {
    Array,
    NativeObject(String),
}

impl Runtime {
    // todo: at some point i want to import specific ones from modules

    fn register_native(&mut self, identifier: &str, func: NativeFunction) {
        self.functions_mut()
            .insert(identifier.to_string(), RuntimeFunction::Native(func));
    }

    pub fn register_native_functions(&mut self) {
        let pairs: [(&str, NativeFunction); _] = [
            ("sg::print", util::sg::print),
            ("sg::set_byte_order", util::sg::set_byte_order),
            ("sg::write", fs::sg::write),
            ("sg::read", fs::sg::read),
            ("sg::open_file", fs::sg::open_file),
            (
                "sg::create_serialization_target",
                aque::sg::create_serialization_target,
            ),
            (
                "sg::destroy_serialization_target",
                aque::sg::destroy_serialization_target,
            ),
            ("sg::serialize_to_target", aque::sg::serialize_to_target),
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
        let types = [
            BuiltinMethodTarget::Array,
            BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
        ];

        for item in types {
            self.builtin_methods_mut().insert(item, HashMap::new());
        }

        let tuples: [(BuiltinMethodTarget, &str, NativeFunction); _] = [
            (
                BuiltinMethodTarget::Array,
                "iter",
                sg::arrays::array_iterator,
            ),
            /* sg::FileHandle */
            (
                BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
                "close",
                fs::sg::file_handle::close,
            ),
            (
                BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
                "delete",
                fs::sg::file_handle::delete,
            ),
            (
                BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
                "new",
                fs::sg::file_handle::new,
            ),
            (
                BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
                "open",
                fs::sg::file_handle::open,
            ),
            (
                BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
                "read",
                fs::sg::file_handle::read,
            ),
            (
                BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
                "read_value",
                fs::sg::file_handle::read_value,
            ),
            (
                BuiltinMethodTarget::NativeObject("sg::FileHandle".to_string()),
                "rename",
                fs::sg::file_handle::rename,
            ),
        ];

        for (target, identifier, func) in tuples {
            let map = self.builtin_methods_mut().get_mut(&target).unwrap();

            Self::register_builtin(map, identifier, func);
        }
    }

    pub fn invoke_method_for_native_object(
        &mut self,
        data_type: DataType,
        method_identifier: &String,
        object_reference: RuntimeValue,
        mut args: Vec<RuntimeValue>,
        generics: &Vec<DataType>,
    ) -> RuntimeResult<RuntimeValue> {
        object_reference.assert_reference()?;

        let identifier = match &data_type {
            DataType::NativeObject(name) => name.clone(),

            _ => {
                return Err(RuntimeError::CannotInvokeMethodOnType(
                    data_type.to_string(),
                ));
            }
        };

        let func = *self
            .builtin_methods()
            .get(&BuiltinMethodTarget::NativeObject(identifier.clone()))
            .and_then(|map| map.get(method_identifier))
            .ok_or_else(|| RuntimeError::NotABuiltInMethod {
                method: method_identifier.clone(),
                data_type: identifier,
            })?;

        args.insert(0, object_reference);
        let context = NativeFunctionContext::new(self, args, generics);

        func(context)
    }
}
