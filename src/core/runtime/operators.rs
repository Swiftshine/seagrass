use crate::core::runtime::{RuntimeError, RuntimeResult, RuntimeValue};

impl RuntimeValue {
    pub fn add(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a + b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a + b)),

            _ => Err(RuntimeError::TypeMismatch),
        }
    }

    pub fn subtract(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a - b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a - b)),

            _ => Err(RuntimeError::TypeMismatch),
        }
    }

    pub fn multiply(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a * b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a * b)),

            _ => Err(RuntimeError::TypeMismatch),
        }
    }

    pub fn divide(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a / b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a / b)),

            _ => Err(RuntimeError::TypeMismatch),
        }
    }
}
