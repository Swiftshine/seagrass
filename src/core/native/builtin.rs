pub(crate) mod sg {
    pub(crate) mod arrays {
        use crate::core::{
            native::NativeFunctionContext,
            runtime::{RuntimeError, RuntimeResult, RuntimeValue},
        };

        pub fn array_iterator(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            let array = context.arguments.first().unwrap();

            match array {
                RuntimeValue::Array {
                    inner_data_type,
                    contents,
                } => Ok(RuntimeValue::Iterator {
                    inner_data_type: inner_data_type.clone(),
                    contents: contents.clone(),
                }),

                RuntimeValue::Reference(reference) => {
                    let borrowed = reference.borrow();

                    match &*borrowed {
                        RuntimeValue::Array {
                            inner_data_type,
                            contents,
                        } => Ok(RuntimeValue::Iterator {
                            inner_data_type: inner_data_type.clone(),
                            contents: contents.clone(),
                        }),

                        other => Err(RuntimeError::CannotInvokeMethodOnType(
                            other.data_type()?.to_string(),
                        )),
                    }
                }

                other => Err(RuntimeError::CannotInvokeMethodOnType(
                    other.data_type()?.to_string(),
                )),
            }
        }
    }
}
