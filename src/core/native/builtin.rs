pub(crate) mod sg {
    pub(crate) mod arrays {
        use crate::core::{
            native::NativeFunctionContext,
            runtime::{RuntimeResult, RuntimeValue},
        };

        pub fn array_iterator(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            dbg!(&context.arguments);
            todo!()
        }
    }
}
