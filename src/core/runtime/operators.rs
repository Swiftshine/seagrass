use crate::core::{
    lang::ast::DataType,
    runtime::{RuntimeError, RuntimeResult, RuntimeValue},
};

macro_rules! numeric_compare {
    ($lhs:expr, $rhs:expr, $op:tt) => {{
        let (lhs, rhs) = Self::promote_numeric_pair($lhs, $rhs)?;

        Ok(Self::Bool(match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => a $op b,
            (Self::U8(a), Self::U8(b)) => a $op b,

            (Self::S16(a), Self::S16(b)) => a $op b,
            (Self::U16(a), Self::U16(b)) => a $op b,

            (Self::S32(a), Self::S32(b)) => a $op b,
            (Self::U32(a), Self::U32(b)) => a $op b,

            (Self::F32(a), Self::F32(b)) => a $op b,
            (Self::F64(a), Self::F64(b)) => a $op b,

            (Self::Usize(a), Self::Usize(b)) => a $op b,

            _ => unreachable!(),
        }))
    }};
}

macro_rules! numeric_equality {
    ($lhs:expr, $rhs:expr, $op:tt) => {{
        let (lhs, rhs) = Self::promote_numeric_pair($lhs, $rhs)?;

        Ok(Self::Bool(match (lhs.copy_value(), rhs.copy_value()) {
            (Self::S8(a), Self::S8(b)) => a $op b,
            (Self::U8(a), Self::U8(b)) => a $op b,

            (Self::S16(a), Self::S16(b)) => a $op b,
            (Self::U16(a), Self::U16(b)) => a $op b,

            (Self::S32(a), Self::S32(b)) => a $op b,
            (Self::U32(a), Self::U32(b)) => a $op b,

            (Self::F32(a), Self::F32(b)) => a $op b,
            (Self::F64(a), Self::F64(b)) => a $op b,

            (Self::Usize(a), Self::Usize(b)) => a $op b,

            _ => unreachable!(
                "numeric promotion produced non-numeric values of lhs: '{:#?}' and rhs: '{:#?}'",
                lhs,
                rhs
            ),
        }))
    }};
}

macro_rules! numeric_bitwise {
    ($lhs:expr, $rhs:expr, $op:tt) => {{
        let (lhs, rhs) = Self::promote_numeric_pair($lhs, $rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(a $op b)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(a $op b)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(a $op b)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(a $op b)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a $op b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a $op b)),

            (Self::Usize(a), Self::Usize(b)) => Ok(Self::Usize(a $op b)),

            _ => unreachable!(),
        }
    }};
}

macro_rules! numeric_shift {
    ($lhs:expr, $rhs:expr, $op:tt) => {{
        let amount = Self::determine_shift_amount($rhs)?;

        match $lhs {
            Self::S8(v)     => Ok(Self::S8(v $op amount)),
            Self::U8(v)     => Ok(Self::U8(v $op amount)),

            Self::S16(v)    => Ok(Self::S16(v $op amount)),
            Self::U16(v)    => Ok(Self::U16(v $op amount)),

            Self::S32(v)    => Ok(Self::S32(v $op amount)),
            Self::U32(v)    => Ok(Self::U32(v $op amount)),

            Self::Usize(v)  => Ok(Self::Usize(v $op amount)),

            _ => unreachable!()
        }
    }};
}

macro_rules! numeric_arithmetic {
    ($lhs:expr, $rhs:expr, $op:tt) => {{
        let (lhs, rhs) = Self::promote_numeric_pair($lhs, $rhs)?;

        match (lhs, rhs) {
            (Self::S8(a), Self::S8(b)) => Ok(Self::S8(a $op b)),
            (Self::U8(a), Self::U8(b)) => Ok(Self::U8(a $op b)),

            (Self::S16(a), Self::S16(b)) => Ok(Self::S16(a $op b)),
            (Self::U16(a), Self::U16(b)) => Ok(Self::U16(a $op b)),

            (Self::S32(a), Self::S32(b)) => Ok(Self::S32(a $op b)),
            (Self::U32(a), Self::U32(b)) => Ok(Self::U32(a $op b)),

            (Self::F32(a), Self::F32(b)) => Ok(Self::F32(a $op b)),
            (Self::F64(a), Self::F64(b)) => Ok(Self::F64(a $op b)),

            (Self::Usize(a), Self::Usize(b)) => Ok(Self::Usize(a $op b)),

            _ => unreachable!(),
        }
    }};
}

impl RuntimeValue {
    /* helpers */

    fn determine_shift_amount(value: RuntimeValue) -> RuntimeResult<u32> {
        match value {
            Self::S8(v) if v >= 0 => Ok(v as u32),
            Self::U8(v) => Ok(v as u32),

            Self::S16(v) if v >= 0 => Ok(v as u32),
            Self::U16(v) => Ok(v as u32),

            Self::S32(v) if v >= 0 => Ok(v as u32),
            Self::U32(v) => Ok(v),

            Self::Usize(v) => Ok(v as u32),

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
            (DataType::Usize, _) | (_, DataType::Usize) => DataType::Usize,

            (DataType::S16, DataType::S16)
            | (DataType::S16, DataType::S8)
            | (DataType::S8, DataType::S16) => DataType::S16,

            (DataType::U16, DataType::U16)
            | (DataType::U16, DataType::U8)
            | (DataType::U8, DataType::U16) => DataType::U16,

            (DataType::S8, DataType::S8) => DataType::S8,
            (DataType::U8, DataType::U8) => DataType::U8,

            (DataType::F32, DataType::F32) => DataType::F32,
            (DataType::F32, DataType::F64) => DataType::F64,
            (DataType::F64, DataType::F32) => DataType::F64,
            (DataType::F64, DataType::F64) => DataType::F64,

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

            (RuntimeValue::F32(v), DataType::F64) => Ok(Self::F64(v as f64)),

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
        numeric_arithmetic!(self, rhs, +)
    }

    pub fn subtract(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_arithmetic!(self, rhs, -)
    }

    pub fn multiply(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_arithmetic!(self, rhs, *)
    }

    pub fn divide(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_arithmetic!(self, rhs, /)
    }

    pub fn modulo(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_arithmetic!(self, rhs, %)
    }

    /* comparisons */

    pub fn compare_eq(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        let lhs_type = self.data_type()?.to_string();
        let rhs_type = rhs.data_type()?.to_string();

        let compare = |result: bool| Ok(Self::Bool(result));

        // numeric equality
        if self.data_type()?.is_numeric() && rhs.data_type()?.is_numeric() {
            return numeric_equality!(self, rhs, ==);
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
            return numeric_equality!(self, rhs, !=);
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
        numeric_compare!(self, rhs, <)
    }

    pub fn compare_lte(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_compare!(self, rhs, <=)
    }

    pub fn compare_gt(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_compare!(self, rhs, >)
    }

    pub fn compare_gte(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_compare!(self, rhs, >=)
    }

    /* bitwise */

    pub fn bitwise_and(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_bitwise!(self, rhs, &)
    }

    pub fn bitwise_or(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_bitwise!(self, rhs, |)
    }

    pub fn bitwise_xor(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_bitwise!(self, rhs, ^)
    }

    /* shifts */
    pub fn shift_left(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_shift!(self, rhs, <<)
    }

    pub fn shift_right(self, rhs: RuntimeValue) -> RuntimeResult<Self> {
        numeric_shift!(self, rhs, >>)
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

    /* unary operators */

    pub fn negate(self) -> RuntimeResult<Self> {
        match self {
            Self::S8(v) => Ok(Self::S8(-v)),
            Self::S16(v) => Ok(Self::S16(-v)),
            Self::S32(v) => Ok(Self::S32(-v)),

            Self::U8(_) | Self::U16(_) | Self::U32(_) | Self::Usize(_) => Err(
                RuntimeError::UnsupportedUnaryOperation("-", self.data_type()?.to_string()),
            ),

            _ => Err(RuntimeError::UnsupportedUnaryOperation(
                "-",
                self.data_type()?.to_string(),
            )),
        }
    }

    pub fn logical_not(self) -> RuntimeResult<Self> {
        match self {
            Self::Bool(value) => Ok(Self::Bool(!value)),

            _ => Err(RuntimeError::UnsupportedUnaryOperation(
                "!",
                self.data_type()?.to_string(),
            )),
        }
    }

    pub fn bitwise_not(self) -> RuntimeResult<Self> {
        match self {
            Self::S8(v) => Ok(Self::S8(!v)),
            Self::U8(v) => Ok(Self::U8(!v)),

            Self::S16(v) => Ok(Self::S16(!v)),
            Self::U16(v) => Ok(Self::U16(!v)),

            Self::S32(v) => Ok(Self::S32(!v)),
            Self::U32(v) => Ok(Self::U32(!v)),

            Self::Usize(v) => Ok(Self::Usize(!v)),

            _ => Err(RuntimeError::UnsupportedUnaryOperation(
                "~",
                self.data_type()?.to_string(),
            )),
        }
    }
}
