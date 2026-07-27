use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::{
    lang::ast::{DataType, StructDefinition},
    runtime::{Runtime, RuntimeError, RuntimeResult},
};

pub type RuntimeReference = Rc<RefCell<RuntimeVariable>>;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeVariable {
    pub value: RuntimeValue,
}

impl RuntimeVariable {
    pub fn from_value(value: RuntimeValue) -> Self {
        Self { value }
    }

    pub fn value(&self) -> RuntimeValue {
        self.value.clone()
    }

    pub fn set_value(&mut self, value: RuntimeValue) {
        self.value = value;
    }
}

impl DataType {
    pub fn can_be_coerced_into(&self, into: &Self) -> bool {
        match (self, into) {
            (Self::S32, Self::U32) => true,
            (Self::U32, Self::S32) => true,
            (
                Self::Array {
                    data_type: self_type,
                    count: self_count,
                },
                Self::Array {
                    data_type: into_type,
                    count: into_count,
                },
            ) if self_type.can_be_coerced_into(into_type) => into_count >= self_count,
            _ => *self == *into,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    None,
    U32(u32),
    S32(i32),
    String(String),
    Bool(bool),
    Struct {
        definition: Rc<StructDefinition>,
        fields: HashMap<String, RuntimeValue>,
    },
    Reference(RuntimeReference),
    Array {
        inner_data_type: DataType,
        contents: Box<[RuntimeValue]>,
    },
}

impl RuntimeValue {
    pub fn data_type(&self) -> RuntimeResult<DataType> {
        match self {
            Self::None => Err(RuntimeError::NoDataTypeAttached),
            Self::U32(_) => Ok(DataType::U32),
            Self::S32(_) => Ok(DataType::S32),
            Self::String(_) => Ok(DataType::String),
            Self::Bool(_) => Ok(DataType::Bool),
            Self::Struct { definition, .. } => {
                Ok(DataType::UserDefined(definition.identifier.clone()))
            }
            Self::Array {
                inner_data_type: data_type,
                contents,
            } => Ok(DataType::Array {
                data_type: Box::new(data_type.clone()),
                count: contents.len(),
            }),

            Self::Reference(variable) => Ok(DataType::Reference(Box::new(
                variable.borrow().value().data_type()?,
            ))),
        }
    }

    pub fn struct_access(&self, identifier: &str) -> RuntimeResult<RuntimeValue> {
        match self {
            Self::Struct { definition, fields } => {
                fields
                    .get(identifier)
                    .cloned()
                    .ok_or(RuntimeError::InvalidStructFieldAccess {
                        field_name: identifier.to_string(),
                        struct_name: definition.identifier.clone(),
                    })
            }

            Self::Reference(_) => self.dereference(),

            _ => Err(RuntimeError::InvalidStructFieldAccessTarget {
                field: identifier.to_string(),
                data_type: self.data_type()?.to_string(),
            }),
        }
    }

    pub fn resolve(&self) -> RuntimeValue {
        // this function should only be called for struct
        match self {
            Self::Reference(reference) => reference.borrow().value().resolve(),
            _ => self.clone(),
        }
    }

    // pub fn reference(self) -> RuntimeValue {
    //     RuntimeValue::Reference(
    //         Rc::new(RefCell::new(self))
    //     )
    // }

    pub fn dereference(&self) -> RuntimeResult<RuntimeValue> {
        match self {
            RuntimeValue::Reference(variable) => Ok(variable.borrow().value().clone()),
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
}

impl Runtime {
    pub fn coerce(&self, value: RuntimeValue, into: &DataType) -> RuntimeResult<RuntimeValue> {
        let data_type = value.data_type()?;

        if data_type == *into {
            Ok(value)
        } else if data_type.can_be_coerced_into(into) {
            match (value, into) {
                (RuntimeValue::U32(val), DataType::S32) => Ok(RuntimeValue::S32(val as i32)),
                (RuntimeValue::S32(val), DataType::U32) => Ok(RuntimeValue::U32(val as u32)),
                (RuntimeValue::Array { contents, .. }, DataType::Array { data_type, count }) => {
                    let mut vec = contents.to_vec();

                    // make sure the existing type gets coerced into the new one
                    for i in 0..vec.len() {
                        vec[i] = self.coerce(vec[i].clone(), data_type)?
                    }

                    vec.resize(*count, self.default_value(data_type)?);
                    let contents = vec.into_boxed_slice();

                    Ok(RuntimeValue::Array {
                        inner_data_type: *data_type.clone(),
                        contents,
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
