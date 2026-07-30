use crate::core::{
    lang::ast::DataType,
    runtime::{RuntimeError, RuntimeResult, RuntimeValue},
};

impl RuntimeValue {
    fn numeric_arithmetic<F>(
        lhs: RuntimeValue,
        rhs: RuntimeValue,
        op: F,
    ) -> RuntimeResult<RuntimeValue>
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let (lhs, rhs) = Self::promote_numeric_pair(lhs, rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(op(a as i32, b as i32) as i8)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(op(a as i32, b as i32) as u8)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(op(a as i32, b as i32) as i16)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(op(a as i32, b as i32) as u16)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(op(a, b))),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(op(a as i32, b as i32) as u32)),

            _ => unreachable!(),
        }
    }

    fn promote_numeric_pair(
        lhs: RuntimeValue,
        rhs: RuntimeValue,
    ) -> RuntimeResult<(RuntimeValue, RuntimeValue)> {
        let lhs_type = lhs.data_type()?;
        let rhs_type = rhs.data_type()?;

        let target = match (&lhs_type, &rhs_type) {
            (DataType::S32, _) | (_, DataType::S32) => DataType::S32,
            (DataType::U32, _) | (_, DataType::U32) => DataType::U32,

            (DataType::S16, DataType::S16)
            | (DataType::S16, DataType::S8)
            | (DataType::S8, DataType::S16) => DataType::S16,

            (DataType::U16, DataType::U16)
            | (DataType::U16, DataType::U8)
            | (DataType::U8, DataType::U16) => DataType::U16,

            (DataType::S8, DataType::S8) => DataType::S8,
            (DataType::U8, DataType::U8) => DataType::U8,

            // mixed signed/unsigned
            (DataType::U8, DataType::S8) | (DataType::S8, DataType::U8) => DataType::S16,

            (DataType::U16, DataType::S16) | (DataType::S16, DataType::U16) => DataType::S32,

            _ => {
                return Err(RuntimeError::unsupported_binary_operation(
                    lhs_type.to_string(),
                    "numeric promotion",
                    rhs_type.to_string(),
                ));
            }
        };

        Ok((
            Self::coerce_numeric(lhs, &target)?,
            Self::coerce_numeric(rhs, &target)?,
        ))
    }

    fn coerce_numeric(value: RuntimeValue, target: &DataType) -> RuntimeResult<RuntimeValue> {
        match (value.copy_value(), target) {
            (RuntimeValue::S8(v), DataType::S16) => Ok(Self::S16(v as i16)),
            (RuntimeValue::S8(v), DataType::S32) => Ok(Self::S32(v as i32)),

            (RuntimeValue::U8(v), DataType::U16) => Ok(Self::U16(v as u16)),
            (RuntimeValue::U8(v), DataType::S16) => Ok(Self::S16(v as i16)),
            (RuntimeValue::U8(v), DataType::U32) => Ok(Self::U32(v as u32)),

            (RuntimeValue::S16(v), DataType::S32) => Ok(Self::S32(v as i32)),

            (RuntimeValue::U16(v), DataType::U32) => Ok(Self::U32(v as u32)),
            (RuntimeValue::U16(v), DataType::S32) => Ok(Self::S32(v as i32)),

            // into usize
            (RuntimeValue::U8(v), DataType::Usize) => Ok(Self::Usize(v as usize)),
            (RuntimeValue::U16(v), DataType::Usize) => Ok(Self::Usize(v as usize)),
            (RuntimeValue::U32(v), DataType::Usize) => Ok(Self::Usize(v as usize)),

            (RuntimeValue::S8(v), DataType::Usize) if v >= 0 => Ok(Self::Usize(v as usize)),

            (RuntimeValue::S16(v), DataType::Usize) if v >= 0 => Ok(Self::Usize(v as usize)),

            (RuntimeValue::S32(v), DataType::Usize) if v >= 0 => Ok(Self::Usize(v as usize)),

            (value, ty) if value.data_type()? == *ty => Ok(value),

            _ => unreachable!(
                "value is {:#?}, target data type is {}",
                value,
                target.to_string()
            ),
        }
    }

    /* arithmetic */

    pub fn add(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(a + b)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(a + b)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(a + b)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(a + b)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a + b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a + b)),

            _ => unreachable!(),
        }
    }

    pub fn subtract(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(a - b)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(a - b)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(a - b)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(a - b)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a - b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a - b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "-", rhs_type,
            )),
        }
    }

    pub fn multiply(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(a * b)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(a * b)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(a * b)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(a * b)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a * b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a * b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "*", rhs_type,
            )),
        }
    }

    pub fn divide(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(a / b)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(a / b)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(a / b)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(a / b)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a / b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a / b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "/", rhs_type,
            )),
        }
    }

    pub fn modulo(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(a % b)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(a % b)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(a % b)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(a % b)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a % b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a % b)),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "%", rhs_type,
            )),
        }
    }

    /* comparisons */

    pub fn compare_eq(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result: bool| Ok(Self::Bool(result));

        // numeric equality
        if self.data_type()?.is_numeric() && rhs.data_type()?.is_numeric() {
            let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

            return match (lhs.copy_value(), rhs.copy_value()) {
                (Self::S8(a), Self::S8(b)) => compare(a == b),
                (Self::U8(a), Self::U8(b)) => compare(a == b),

                (Self::S16(a), Self::S16(b)) => compare(a == b),
                (Self::U16(a), Self::U16(b)) => compare(a == b),

                (Self::S32(a), Self::S32(b)) => compare(a == b),
                (Self::U32(a), Self::U32(b)) => compare(a == b),

                _ => unreachable!(
                    "numeric promotion produced non-numeric values of lhs: '{:#?}' and rhs: '{:#?}'",
                    lhs, rhs
                ),
            };
        }

        match (self, rhs) {
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
                if def_a != def_b {
                    return Err(RuntimeError::InvalidStructComparison(
                        def_a.identifier.clone(),
                        def_b.identifier.clone(),
                    ));
                }

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

        // numeric inequality
        if self.data_type()?.is_numeric() && rhs.data_type()?.is_numeric() {
            let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

            return match (lhs, rhs) {
                (Self::S8(a), Self::S8(b)) => compare(a != b),
                (Self::U8(a), Self::U8(b)) => compare(a != b),

                (Self::S16(a), Self::S16(b)) => compare(a != b),
                (Self::U16(a), Self::U16(b)) => compare(a != b),

                (Self::S32(a), Self::S32(b)) => compare(a != b),
                (Self::U32(a), Self::U32(b)) => compare(a != b),

                _ => unreachable!("numeric promotion produced non-numeric values"),
            };
        }

        match (self, rhs) {
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
                if def_a != def_b {
                    return Err(RuntimeError::InvalidStructComparison(
                        def_a.identifier.clone(),
                        def_b.identifier.clone(),
                    ));
                }

                compare(fields_a != fields_b)
            }

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "!=", rhs_type,
            )),
        }
    }

    pub fn compare_lt(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        let compare = |result| Ok(Self::Bool(result));

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => compare(a < b),
            (Self::U8(a), Self::U8(b)) => compare(a < b),

            (Self::S16(a), Self::S16(b)) => compare(a < b),
            (Self::U16(a), Self::U16(b)) => compare(a < b),

            (Self::S32(a), Self::S32(b)) => compare(a < b),
            (Self::U32(a), Self::U32(b)) => compare(a < b),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "<", rhs_type,
            )),
        }
    }

    pub fn compare_lte(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        let compare = |result| Ok(Self::Bool(result));

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => compare(a <= b),
            (Self::U8(a), Self::U8(b)) => compare(a <= b),

            (Self::S16(a), Self::S16(b)) => compare(a <= b),
            (Self::U16(a), Self::U16(b)) => compare(a <= b),

            (Self::S32(a), Self::S32(b)) => compare(a <= b),
            (Self::U32(a), Self::U32(b)) => compare(a <= b),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, "<=", rhs_type,
            )),
        }
    }

    pub fn compare_gt(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        let compare = |result| Ok(Self::Bool(result));

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => compare(a > b),
            (Self::U8(a), Self::U8(b)) => compare(a > b),

            (Self::S16(a), Self::S16(b)) => compare(a > b),
            (Self::U16(a), Self::U16(b)) => compare(a > b),

            (Self::S32(a), Self::S32(b)) => compare(a > b),
            (Self::U32(a), Self::U32(b)) => compare(a > b),

            _ => Err(RuntimeError::unsupported_binary_operation(
                lhs_type, ">", rhs_type,
            )),
        }
    }

    pub fn compare_gte(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let (lhs, rhs) = Self::promote_numeric_pair(self, rhs)?;

        let compare = |result| Ok(Self::Bool(result));

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => compare(a >= b),
            (Self::U8(a), Self::U8(b)) => compare(a >= b),

            (Self::S16(a), Self::S16(b)) => compare(a >= b),
            (Self::U16(a), Self::U16(b)) => compare(a >= b),

            (Self::S32(a), Self::S32(b)) => compare(a >= b),
            (Self::U32(a), Self::U32(b)) => compare(a >= b),

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
