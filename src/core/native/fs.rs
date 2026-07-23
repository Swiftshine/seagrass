pub(crate) mod sg {
    use crate::core::runtime::{Runtime, RuntimeError, RuntimeResult, value::RuntimeValue};

    pub fn write(
        runtime: &mut Runtime,
        values: Vec<RuntimeValue>,
    ) -> RuntimeResult<RuntimeValue> {
        if values.len() != 2 {
            todo!("argument count error");
        }
    
        let value = &values[1];
    
        let filename = match &values[0] {
            RuntimeValue::String(value) => value,
            other => {
                return Err(RuntimeError::AnnotationError {
                    expected: "string".to_string(),
                    found: other.data_type()?.to_string(),
                });
            }
        };
    
        let mut bytes = Vec::new();
    
        runtime.serialize_into(
            value,
            &mut bytes,
        )?;
    
        std::fs::write(filename, bytes)
            .map_err(|_| {
                todo!("filesystem error")
            })?;
    
        Ok(RuntimeValue::None)
    }
}
