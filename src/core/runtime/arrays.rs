use crate::core::{
    lang::ast::DataType,
    native::{BuiltinMethodTarget, NativeFunctionContext},
    runtime::{Runtime, RuntimeError, RuntimeResult, RuntimeValue},
};

impl Runtime {
    pub fn invoke_method_for_array(
        &mut self,
        data_type: DataType,
        method_identifier: &String,
        arr_reference: RuntimeValue,
        mut args: Vec<RuntimeValue>,
    ) -> RuntimeResult<RuntimeValue> {
        arr_reference.assert_reference()?;

        let func = *self
            .builtin_methods()
            .get(&BuiltinMethodTarget::Array)
            .and_then(|map| map.get(method_identifier))
            .ok_or_else(|| RuntimeError::NotABuiltInMethod {
                method: method_identifier.clone(),
                data_type: data_type.to_string(),
            })?;

        args.insert(0, arr_reference);

        let _generics = vec![];
        let context = NativeFunctionContext::new(self, args, &_generics);

        func(context)
    }
}
