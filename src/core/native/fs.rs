pub(crate) mod sg {
    use crate::core::{
        native::NativeFunctionContext,
        runtime::{RuntimeError, RuntimeResult, value::RuntimeValue},
    };

    pub fn write(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        let runtime = context.runtime;
        let arguments = context.arguments;

        if arguments.len() != 2 {
            todo!("argument count error");
        }

        let value = &arguments[1];

        let filename = match &arguments[0] {
            RuntimeValue::String(value) => value,
            other => {
                return Err(RuntimeError::AnnotationError {
                    expected: "string".to_string(),
                    found: other.data_type()?.to_string(),
                });
            }
        };

        let mut bytes = Vec::new();

        runtime.serialize_into(value, &mut bytes)?;

        std::fs::write(filename, bytes).map_err(|_| todo!("filesystem error"))?;

        Ok(RuntimeValue::None)
    }

    pub fn read(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        assert_eq!(context.generics.len(), 1);

        let filename = match &context.arguments[0] {
            RuntimeValue::String(value) => value,
            other => {
                return Err(RuntimeError::AnnotationError {
                    expected: "string".to_string(),
                    found: other.data_type()?.to_string(),
                });
            }
        };

        let bytes = std::fs::read(filename).unwrap_or_else(|_| panic!("could not find file {filename}"));

        let value = context.runtime.deserialize(&context.generics[0], &bytes)?;

        Ok(value)
    }
}
