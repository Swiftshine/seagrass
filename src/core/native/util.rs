pub(crate) mod sg {
    use crate::core::runtime::{RuntimeResult, RuntimeValue};

    pub fn print(values: Vec<RuntimeValue>) -> RuntimeResult<RuntimeValue> {
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }

            match value {
                RuntimeValue::U32(i) => print!("u32: {i}"),
                RuntimeValue::S32(i) => print!("s32: {i}"),
                RuntimeValue::String(string) => print!("string: {string}"),
                RuntimeValue::Bool(b) => print!("bool: {b}"),
                RuntimeValue::None => {}
                RuntimeValue::Struct { definition, .. } => { // remove this later
                    if definition.is_declared_pod() {
                        print!("struct {} (declared as POD)", definition.identifier);
                    } else {
                        print!("struct {}", definition.identifier);
                    }
                }

                _ => print!("cannot print for type: {}", value.data_type()?.to_string()),
            }
        }

        println!();

        Ok(RuntimeValue::None)
    }
}
