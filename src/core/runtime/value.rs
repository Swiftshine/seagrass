use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::{
    lang::ast::{DataType, StructDefinition},
    runtime::{Runtime, RuntimeError, RuntimeResult},
};

pub type RuntimeReference = Rc<RefCell<RuntimeValue>>;

#[derive(Debug, Clone, PartialEq)]
pub enum LValue {
    Reference(RuntimeReference),
    ArrayElement { array: Box<LValue>, index: usize },
    StructField { object: Box<LValue>, field: String },
}

impl LValue {
    pub fn get_reference(&self) -> RuntimeReference {
        match self {
            Self::Reference(reference) => Rc::clone(reference),
            _ => unreachable!(),
        }
    }

    pub fn read_value(&self) -> RuntimeResult<RuntimeValue> {
        match self {
            Self::Reference(reference) => Ok(reference.borrow().copy_value()),

            Self::ArrayElement { array, index } => {
                let array_runtime_value = array.read_value()?;

                if let RuntimeValue::Array { contents, .. } = array_runtime_value {
                    let reference = contents.get(*index).cloned().ok_or(
                        RuntimeError::ArrayIndexOutOfBounds {
                            index: *index,
                            length: contents.len(),
                        },
                    )?;

                    Ok(reference.borrow().copy_value())
                } else {
                    Err(RuntimeError::CannotIndexNonArrayType(
                        array_runtime_value.data_type()?.to_string(),
                    ))
                }
            }

            Self::StructField { object, field } => {
                let struct_runtime_value = object.read_value()?;

                if let RuntimeValue::Struct { definition, fields } = struct_runtime_value {
                    let value =
                        fields
                            .get(field)
                            .ok_or(RuntimeError::InvalidStructFieldAccess {
                                field_name: field.clone(),
                                struct_name: definition.identifier.clone(),
                            })?;

                    Ok(value.borrow().copy_value())
                } else {
                    Err(RuntimeError::InvalidStructFieldAccessTarget {
                        field: field.clone(),
                        data_type: struct_runtime_value.data_type()?.to_string(),
                    })
                }
            }
        }
    }

    pub fn write_value(&self, value: RuntimeValue) -> RuntimeResult<()> {
        match self {
            Self::Reference(reference) => {
                *reference.borrow_mut() = value;
                Ok(())
            }

            Self::ArrayElement { array, index } => {
                let array_reference = array.get_reference();

                if let RuntimeValue::Array { contents, .. } = &mut *array_reference.borrow_mut() {
                    *contents[*index].borrow_mut() = value;
                    Ok(())
                } else {
                    Err(RuntimeError::CannotIndexNonArrayType(
                        array_reference.borrow().data_type()?.to_string(),
                    ))
                }
            }

            Self::StructField { object, field } => {
                let struct_reference = object.get_reference();

                if let RuntimeValue::Struct { fields, .. } = &mut *struct_reference.borrow_mut() {
                    let field = fields.get(field).unwrap();
                    *field.borrow_mut() = value;
                    Ok(())
                } else {
                    Err(RuntimeError::InvalidStructFieldAccessTarget {
                        field: field.clone(),
                        data_type: struct_reference.borrow().data_type()?.to_string(),
                    })
                }
            }
        }
    }
}

impl DataType {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::S8 | Self::U8 | Self::S16 | Self::U16 | Self::S32 | Self::U32 | Self::Usize
        )
    }

    pub fn can_be_coerced_into(&self, into: &Self) -> bool {
        match (self, into) {
            // identical types
            _ if self == into => true,

            // integer widening
            (Self::S8, Self::S16)
            | (Self::S8, Self::S32)
            | (Self::U8, Self::U16)
            | (Self::U8, Self::S16)
            | (Self::U8, Self::U32)
            | (Self::S16, Self::S32)
            | (Self::U16, Self::U32)
            | (Self::U16, Self::S32)
            | (Self::S8, Self::Usize)
            | (Self::U8, Self::Usize)
            | (Self::S16, Self::Usize)
            | (Self::U16, Self::Usize)
            | (Self::S32, Self::Usize)
            | (Self::U32, Self::Usize) => true,

            (
                Self::Array {
                    inner_data_type: self_type,
                    count: self_count,
                },
                Self::Array {
                    inner_data_type: into_type,
                    count: into_count,
                },
            ) if self_type.can_be_coerced_into(into_type) => into_count >= self_count,

            _ => false,
        }
    }
}

/// NEVER CALL `clone()` MANUALLY! ALWAYS use `copy_value()` INSTEAD!
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    None,
    S8(i8),
    U8(u8),
    S16(i16),
    U16(u16),
    S32(i32),
    U32(u32),
    Usize(usize),
    String(String),
    Bool(bool),
    Struct {
        definition: Rc<StructDefinition>,
        fields: HashMap<String, RuntimeReference>,
    },
    Reference(RuntimeReference),
    // ik this looks identical to Array but they are different for a reason
    Iterator {
        inner_data_type: DataType,
        contents: Box<[RuntimeReference]>,
    },
    Array {
        inner_data_type: DataType,
        contents: Box<[RuntimeReference]>,
    },
}

impl RuntimeValue {
    pub fn into_runtime_reference(self) -> RuntimeReference {
        Rc::new(RefCell::new(self))
    }

    pub fn cast_to(self, target: &DataType) -> RuntimeResult<Self> {
        match target {
            DataType::S8
            | DataType::U8
            | DataType::S16
            | DataType::U16
            | DataType::S32
            | DataType::U32
            | DataType::Usize => self.cast_numeric(target),

            _ => Err(RuntimeError::InvalidCast(
                self.data_type()?.to_string(),
                target.to_string(),
            )),
        }
    }

    fn cast_numeric(self, target: &DataType) -> RuntimeResult<Self> {
        let data_type = self.data_type()?;

        let value = match self {
            Self::S8(v) => v as i128,
            Self::U8(v) => v as i128,
            Self::S16(v) => v as i128,
            Self::U16(v) => v as i128,
            Self::S32(v) => v as i128,
            Self::U32(v) => v as i128,
            Self::Usize(v) => v as i128,

            value => {
                return Err(RuntimeError::InvalidCast(
                    value.data_type()?.to_string(),
                    target.to_string(),
                ));
            }
        };

        match target {
            DataType::S8 => Ok(Self::S8(value as i8)),
            DataType::U8 => Ok(Self::U8(value as u8)),

            DataType::S16 => Ok(Self::S16(value as i16)),
            DataType::U16 => Ok(Self::U16(value as u16)),

            DataType::S32 => Ok(Self::S32(value as i32)),
            DataType::U32 => Ok(Self::U32(value as u32)),

            DataType::Usize => Ok(Self::Usize(value as usize)),

            _ => Err(RuntimeError::InvalidCast(
                data_type.to_string(),
                target.to_string(),
            )),
        }
    }

    pub fn copy_value(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::S8(v) => Self::S8(*v),
            Self::U8(v) => Self::U8(*v),
            Self::S16(v) => Self::S16(*v),
            Self::U16(v) => Self::U16(*v),
            Self::S32(v) => Self::S32(*v),
            Self::U32(v) => Self::U32(*v),
            Self::Usize(v) => Self::Usize(*v),
            Self::String(s) => Self::String(s.clone()),
            Self::Bool(v) => Self::Bool(*v),
            Self::Struct { definition, fields } => Self::Struct {
                definition: Rc::clone(definition),
                fields: fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.borrow().copy_value().into_runtime_reference()))
                    .collect(),
            },
            Self::Reference(r) => Self::Reference(r.clone()),
            Self::Array {
                inner_data_type,
                contents,
            } => Self::Array {
                inner_data_type: inner_data_type.clone(),
                contents: contents.to_vec().into_boxed_slice(),
            },
            Self::Iterator {
                inner_data_type,
                contents,
            } => Self::Iterator {
                inner_data_type: inner_data_type.clone(),
                contents: contents.clone(),
            },
        }
    }

    pub fn data_type(&self) -> RuntimeResult<DataType> {
        match self {
            Self::None => Err(RuntimeError::NoDataTypeAttached),
            Self::S8(_) => Ok(DataType::S8),
            Self::U8(_) => Ok(DataType::U8),
            Self::S16(_) => Ok(DataType::S16),
            Self::U16(_) => Ok(DataType::U16),
            Self::U32(_) => Ok(DataType::U32),
            Self::S32(_) => Ok(DataType::S32),
            Self::Usize(_) => Ok(DataType::Usize),
            Self::String(_) => Ok(DataType::String),
            Self::Bool(_) => Ok(DataType::Bool),
            Self::Struct { definition, .. } => {
                Ok(DataType::UserDefined(definition.identifier.clone()))
            }
            Self::Array {
                inner_data_type: data_type,
                contents,
            } => Ok(DataType::Array {
                inner_data_type: Box::new(data_type.clone()),
                count: Some(contents.len()),
            }),

            Self::Reference(variable) => Ok(DataType::Reference(Box::new(
                variable.borrow().data_type()?,
            ))),

            Self::Iterator {
                inner_data_type, ..
            } => Ok(DataType::Iterator(Box::new(inner_data_type.clone()))),
        }
    }

    pub fn struct_access(&self, identifier: &str) -> RuntimeResult<RuntimeValue> {
        match self {
            Self::Struct { definition, fields } => {
                let value =
                    fields
                        .get(identifier)
                        .ok_or(RuntimeError::InvalidStructFieldAccess {
                            field_name: identifier.to_string(),
                            struct_name: definition.identifier.clone(),
                        })?;

                Ok(value.borrow_mut().copy_value())
            }

            Self::Reference(reference) => reference.borrow().struct_access(identifier),

            _ => Err(RuntimeError::InvalidStructFieldAccessTarget {
                field: identifier.to_string(),
                data_type: self.data_type()?.to_string(),
            }),
        }
    }

    // pub fn resolve(&self) -> RuntimeValue {
    //     // this function should only be called for struct
    //     match self {
    //         Self::Reference(reference) => reference.borrow().value().resolve(),
    //         _ => self.clone(),
    //     }
    // }

    // pub fn reference(self) -> RuntimeValue {
    //     RuntimeValue::Reference(
    //         Rc::new(RefCell::new(self))
    //     )
    // }

    pub fn dereference(&self) -> RuntimeResult<RuntimeValue> {
        match self {
            RuntimeValue::Reference(variable) => Ok(variable.borrow().copy_value()),
            _ => Err(RuntimeError::CannotDereferenceNonReference),
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference(_))
    }

    pub fn assert_reference(&self) -> RuntimeResult<()> {
        if !self.is_reference() {
            Err(RuntimeError::ExpectedReference(
                self.data_type()?.to_string(),
            ))
        } else {
            Ok(())
        }
    }

    // pub fn assert_struct(&self) -> RuntimeResult<()> {
    //     match self {
    //         RuntimeValue::Struct { .. } => Ok(()),
    //         RuntimeValue::Reference(reference) => {
    //             // the first dereference and the first dereference only must resolve to a struct
    //             match reference.borrow().value() {
    //                 RuntimeValue::Struct { .. } => Ok(()),
    //                 _ => Err(RuntimeError::ExpectedStruct),
    //             }
    //         }
    //         _ => Err(RuntimeError::ExpectedStruct),
    //     }
    // }
    pub fn expect_reference(&self) -> RuntimeResult<RuntimeReference> {
        match self {
            RuntimeValue::Reference(reference) => Ok(reference.clone()),

            other => Err(RuntimeError::ExpectedReference(
                other.data_type()?.to_string(),
            )),
        }
    }
}

impl Runtime {
    pub fn coerce(&self, value: RuntimeValue, into: &DataType) -> RuntimeResult<RuntimeValue> {
        let data_type = value.data_type()?;

        if data_type == *into {
            Ok(value)
        } else if data_type.can_be_coerced_into(into) {
            match (value, into) {
                (RuntimeValue::S8(v), DataType::S16) => Ok(RuntimeValue::S16(v as i16)),
                (RuntimeValue::S8(v), DataType::S32) => Ok(RuntimeValue::S32(v as i32)),

                (RuntimeValue::U8(v), DataType::U16) => Ok(RuntimeValue::U16(v as u16)),
                (RuntimeValue::U8(v), DataType::S16) => Ok(RuntimeValue::S16(v as i16)),
                (RuntimeValue::U8(v), DataType::U32) => Ok(RuntimeValue::U32(v as u32)),

                (RuntimeValue::S16(v), DataType::S32) => Ok(RuntimeValue::S32(v as i32)),

                (RuntimeValue::U16(v), DataType::U32) => Ok(RuntimeValue::U32(v as u32)),
                (RuntimeValue::U16(v), DataType::S32) => Ok(RuntimeValue::S32(v as i32)),

                (
                    RuntimeValue::Array { contents, .. },
                    DataType::Array {
                        inner_data_type: data_type,
                        count,
                    },
                ) => {
                    let mut vec = contents
                        .into_iter()
                        .flat_map(|reference| {
                            self.coerce(reference.borrow().copy_value(), data_type)
                        })
                        .map(RuntimeValue::into_runtime_reference)
                        .collect::<Vec<RuntimeReference>>();

                    vec.resize(
                        count.unwrap(),
                        self.default_value(data_type)?.into_runtime_reference(),
                    );

                    Ok(RuntimeValue::Array {
                        inner_data_type: *data_type.clone(),
                        contents: vec.into_boxed_slice(),
                    })
                }

                _ => unreachable!("type must be coerceable but it isn't for some reason"),
            }
        } else {
            Err(RuntimeError::TypeCoercionFail {
                from: value.data_type()?.to_string(),
                to: into.to_string(),
            })
        }
    }
}
