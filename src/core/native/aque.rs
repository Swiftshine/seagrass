pub(crate) mod sg {
    use crate::core::{
        native::NativeFunctionContext,
        runtime::{RuntimeError, RuntimeResult, RuntimeValue},
    };

    pub fn serialize_to_target(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        context.assert_arguments(
            "sg::serialize_to_target",
            &["target_name: string", "data: auto T"],
        )?;

        let runtime = context.runtime;
        let arguments = context.arguments;
        let target_name = match &arguments[0] {
            RuntimeValue::String(value) => value,
            other => {
                return Err(RuntimeError::AnnotationError {
                    expected: "string".to_string(),
                    found: other.data_type()?.to_string(),
                });
            }
        };
        let value = &arguments[1];

        let bytes = runtime.serialize_into(value)?;
        *runtime.get_serialization_target(target_name) = bytes;

        Ok(RuntimeValue::None)
    }
}
