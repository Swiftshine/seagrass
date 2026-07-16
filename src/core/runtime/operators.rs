use crate::core::runtime::{RuntimeError, RuntimeResult, RuntimeValue};

impl RuntimeValue {
    pub fn add(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a + b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a + b)),

            // string concat
            (RuntimeValue::String(a), RuntimeValue::String(b)) => Ok(RuntimeValue::String(a + &b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "+", rhs_type,
            )),
        }
    }

    pub fn subtract(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a - b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a - b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "-", rhs_type,
            )),
        }
    }

    pub fn multiply(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a * b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a * b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "*", rhs_type,
            )),
        }
    }

    pub fn divide(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a / b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a / b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "/", rhs_type,
            )),
        }
    }
}
