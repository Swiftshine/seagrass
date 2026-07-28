use crate::core::runtime::{RuntimeError, RuntimeResult, RuntimeValue};

impl RuntimeValue {
    /* arithmetic */

    pub fn add(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (RuntimeValue::S32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::S32(a + b)),

            (RuntimeValue::U32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a + b)),

            (RuntimeValue::U32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::U32(a + b as u32)),

            (RuntimeValue::S32(a), RuntimeValue::U32(b)) => Ok(RuntimeValue::U32(a as u32 + b)),

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

            (RuntimeValue::U32(a), RuntimeValue::S32(b)) => Ok(RuntimeValue::U32(a * b as u32)),
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

    pub fn compare_eq(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result: bool| Ok(Self::Bool(result));

        match (self, rhs) {
            // coerece types
            (Self::S32(a), Self::U32(b)) => compare(a == b as i32),
            (Self::U32(a), Self::S32(b)) => compare(a == b as u32),

            // primitives
            (Self::S32(a), Self::S32(b)) => compare(a == b),
            (Self::U32(a), Self::U32(b)) => compare(a == b),
            (Self::Bool(a), Self::Bool(b)) => compare(a == b),
            (Self::String(a), Self::String(b)) => compare(a == b),
            (
                Self::Struct {
                    definition: def_a,
                    fields: fields_a,
                },
                Self::Struct {
                    definition: def_b,
                    fields: fields_b,
                },
            ) => {
                // check if they're even the same struct
                if def_a != def_b {
                    return Err(RuntimeError::InvalidStructComparison(
                        def_a.identifier.clone(),
                        def_b.identifier.clone(),
                    ));
                }

                // go through the fields and see if they're equal

                compare(fields_a == fields_b)
            }

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "==", rhs_type,
            )),
        }
    }

    pub fn compare_neq(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result: bool| Ok(Self::Bool(result));

        match (self, rhs) {
            // coerece types
            (Self::S32(a), Self::U32(b)) => compare(a != b as i32),
            (Self::U32(a), Self::S32(b)) => compare(a != b as u32),

            // primitives
            (Self::S32(a), Self::S32(b)) => compare(a != b),
            (Self::U32(a), Self::U32(b)) => compare(a != b),
            (Self::Bool(a), Self::Bool(b)) => compare(a != b),
            (Self::String(a), Self::String(b)) => compare(a != b),
            (
                Self::Struct {
                    definition: def_a,
                    fields: fields_a,
                },
                Self::Struct {
                    definition: def_b,
                    fields: fields_b,
                },
            ) => {
                // check if they're even the same struct
                if def_a != def_b {
                    return Err(RuntimeError::InvalidStructComparison(
                        def_a.identifier.clone(),
                        def_b.identifier.clone(),
                    ));
                }

                // go through the fields and see if they're equal

                compare(fields_a != fields_b)
            }

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "!=", rhs_type,
            )),
        }
    }

    pub fn modulo(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a % b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a % b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "%", rhs_type,
            )),
        }
    }

    /* comparisons */
    pub fn compare_lt(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result| Ok(Self::Bool(result));

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => compare(a < b),
            (Self::U32(a), Self::U32(b)) => compare(a < b),

            (Self::S32(a), Self::U32(b)) => compare(a < b as i32),
            (Self::U32(a), Self::S32(b)) => compare((a as i32) < b),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "<", rhs_type,
            )),
        }
    }

    pub fn compare_lte(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result| Ok(Self::Bool(result));

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => compare(a <= b),
            (Self::U32(a), Self::U32(b)) => compare(a <= b),

            (Self::S32(a), Self::U32(b)) => compare(a <= b as i32),
            (Self::U32(a), Self::S32(b)) => compare((a as i32) <= b),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "<=", rhs_type,
            )),
        }
    }

    pub fn compare_gt(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result| Ok(Self::Bool(result));

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => compare(a > b),
            (Self::U32(a), Self::U32(b)) => compare(a > b),

            (Self::S32(a), Self::U32(b)) => compare(a > b as i32),
            (Self::U32(a), Self::S32(b)) => compare((a as i32) > b),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, ">", rhs_type,
            )),
        }
    }

    pub fn compare_gte(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result| Ok(Self::Bool(result));

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => compare(a >= b),
            (Self::U32(a), Self::U32(b)) => compare(a >= b),

            (Self::S32(a), Self::U32(b)) => compare(a >= b as i32),
            (Self::U32(a), Self::S32(b)) => compare((a as i32) >= b),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, ">=", rhs_type,
            )),
        }
    }

    /* bitwise */
    pub fn bitwise_and(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a & b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a & b)),

            (Self::S32(a), Self::U32(b)) => Ok(Self::S32(a & b as i32)),
            (Self::U32(a), Self::S32(b)) => Ok(Self::S32((a as i32) & b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "&", rhs_type,
            )),
        }
    }

    pub fn bitwise_or(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a | b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a | b)),

            (Self::S32(a), Self::U32(b)) => Ok(Self::S32(a | b as i32)),
            (Self::U32(a), Self::S32(b)) => Ok(Self::S32((a as i32) | b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "|", rhs_type,
            )),
        }
    }

    pub fn bitwise_xor(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a ^ b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a ^ b)),

            (Self::S32(a), Self::U32(b)) => Ok(Self::S32(a ^ b as i32)),
            (Self::U32(a), Self::S32(b)) => Ok(Self::S32((a as i32) ^ b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "^", rhs_type,
            )),
        }
    }

    /* shifts */
    pub fn shift_left(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a << b)),
            (Self::S32(a), Self::U32(b)) => Ok(Self::S32(a << b)),

            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a << b)),
            (Self::U32(a), Self::S32(b)) => Ok(Self::U32(a << b as u32)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "<<", rhs_type,
            )),
        }
    }

    pub fn shift_right(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a >> b)),
            (Self::S32(a), Self::U32(b)) => Ok(Self::S32(a >> b)),

            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a >> b)),
            (Self::U32(a), Self::S32(b)) => Ok(Self::U32(a >> b as u32)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, ">>", rhs_type,
            )),
        }
    }

    /* logical */
    pub fn logical_and(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => Ok(Self::Bool(a && b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "&&", rhs_type,
            )),
        }
    }

    pub fn logical_or(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => Ok(Self::Bool(a || b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "||", rhs_type,
            )),
        }
    }
}
