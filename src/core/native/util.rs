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
                RuntimeValue::None => {}
            }
        }

        println!();

        Ok(RuntimeValue::None)
    }
}
