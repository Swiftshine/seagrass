pub(crate) mod sg {
    use crate::core::{
        native::NativeFunctionContext,
        runtime::{RuntimeError, RuntimeResult, RuntimeValue},
    };

    pub fn create_serialization_target(
        context: NativeFunctionContext,
    ) -> RuntimeResult<RuntimeValue> {
        context.runtime.create_serialization_target();
        Ok(RuntimeValue::None)
    }

    pub fn destroy_serialization_target(
        context: NativeFunctionContext,
    ) -> RuntimeResult<RuntimeValue> {
        context.runtime.destroy_serialization_target();
        Ok(RuntimeValue::None)
    }

    pub fn serialize_to_target(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        context.assert_arguments("sg::serialize_to_target", &["data: auto T"])?;

        let runtime = context.runtime;
        let arguments = context.arguments;
        let value = &arguments[0];

        if !runtime.has_serialization_target() {
            return Err(RuntimeError::NoSerializationTarget);
        }

        let bytes = runtime.serialize_into(value)?;
        runtime.get_serialization_target_mut()?.data = bytes;

        Ok(RuntimeValue::None)
    }
}
