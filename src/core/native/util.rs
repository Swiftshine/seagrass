pub(crate) mod sg {
    use crate::core::{
        native::NativeFunctionContext,
        runtime::{RuntimeError, RuntimeResult, structs::ByteOrder, value::RuntimeValue},
    };

    pub fn print(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        let values = context.arguments;

        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }

            match value {
                RuntimeValue::S8(i) => print!("s8: {i}"),
                RuntimeValue::U8(i) => print!("u8: {i}"),
                RuntimeValue::S16(i) => print!("s16: {i}"),
                RuntimeValue::U16(i) => print!("u16: {i}"),
                RuntimeValue::S32(i) => print!("s32: {i}"),
                RuntimeValue::U32(i) => print!("u32: {i}"),
                RuntimeValue::String(string) => print!("string: {string}"),
                RuntimeValue::Bool(b) => print!("bool: {b}"),
                RuntimeValue::None => {}
                _ => print!("cannot print for type: {}", value.data_type()?),
            }
        }

        println!();

        Ok(RuntimeValue::None)
    }

    pub fn set_byte_order(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        match &context.arguments[0] {
            RuntimeValue::String(input) => match input.as_str() {
                "big" => {
                    context.runtime.set_byte_order(ByteOrder::Big);
                }

                "little" => {
                    context.runtime.set_byte_order(ByteOrder::Little);
                }

                _ => {
                    return Err(RuntimeError::InvalidFunctionArguments {
                        signature: "sg::set_byte_order(string)".to_string(),
                        note: "The the byte order must either be \"big\" or \"little\"".to_string(),
                    });
                }
            },
            _ => {
                return Err(RuntimeError::InvalidFunctionArguments {
                    signature: "sg::set_byte_order(string)".to_string(),
                    note: "Input needs to be [string]".to_string(),
                });
            }
        }

        Ok(RuntimeValue::None)
    }
}
