use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::{
    lang::ast::{DataType, StructDefinition},
    runtime::{Runtime, RuntimeError, RuntimeResult},
};

pub type RuntimeReference = Rc<RefCell<RuntimeValue>>;
// pub type RuntimeIterator = Rc<RefCell<RuntimeValue>>;

// #[derive(Debug, Clone, PartialEq)]
// pub enum LValue {
//     Variable(RuntimeReference),
//     ArrayElement { array: Box<LValue>, index: usize },
//     StructField { object: Box<LValue>, field: String },
// }

// impl LValue {
//     pub fn read(&self) -> RuntimeResult<RuntimeValue> {
//         match self {
//             Self::Variable(reference) => Ok(reference.borrow().value()),

//             Self::ArrayElement { array, index } => {
//                 let value = array.read()?;

//                 match value {
//                     RuntimeValue::Array { contents, .. } => {
//                         contents
//                             .get(*index)
//                             .cloned()
//                             .ok_or(RuntimeError::ArrayIndexOutOfBounds {
//                                 index: *index,
//                                 length: contents.len(),
//                             })
//                     }

//                     other => Err(RuntimeError::CannotIndexNonArrayType(
//                         other.data_type()?.to_string(),
//                     )),
//                 }
//             }

//             Self::StructField { object, field } => {
//                 let value = object.read()?;

//                 match value {
//                     RuntimeValue::Struct { definition, fields } => fields
//                         .get(field)
//                         .cloned()
//                         .ok_or(RuntimeError::InvalidStructFieldAccess {
//                             field_name: field.clone(),
//                             struct_name: definition.identifier.clone(),
//                         }),

//                     other => Err(RuntimeError::InvalidStructFieldAccessTarget {
//                         field: field.clone(),
//                         data_type: other.data_type()?.to_string(),
//                     }),
//                 }
//             }
//         }
//     }

//     pub fn write(&self, value: RuntimeValue) -> RuntimeResult<()> {
//         match self {
//             Self::Variable(variable) => {
//                 variable.borrow_mut().set_value(value)?;
//                 Ok(())
//             }

//             Self::ArrayElement { array, index } => {
//                 let mut array_value = array.read()?;

//                 match &mut array_value {
//                     RuntimeValue::Array { contents, .. } => {
//                         contents[*index] = value;
//                         array.write(array_value)
//                     }

//                     other => Err(RuntimeError::CannotIndexNonArrayType(
//                         other.data_type()?.to_string(),
//                     )),
//                 }
//             }

//             Self::StructField { object, field } => {
//                 let mut object_value = object.read()?;

//                 match &mut object_value {
//                     RuntimeValue::Struct { fields, .. } => {
//                         fields.insert(field.clone(), value);
//                         object.write(object_value)
//                     }

//                     other => Err(RuntimeError::InvalidStructFieldAccessTarget {
//                         field: field.clone(),
//                         data_type: other.data_type()?.to_string(),
//                     }),
//                 }
//             }
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct RuntimeVariable {
//     pub value: RuntimeValue,
//     pub data_type: DataType,
// }

// impl RuntimeVariable {
//     pub fn from_value(value: RuntimeValue) -> Self {
//         let data_type = value.data_type().unwrap();

//         Self { value, data_type }
//     }

//     pub fn value(&self) -> RuntimeValue {
//         self.value.clone()
//     }

//     pub fn set_value(&mut self, value: RuntimeValue) -> RuntimeResult<()> {
//         let value = match (&self.data_type, value) {
//             (DataType::U32, RuntimeValue::S32(i)) if i >= 0 => RuntimeValue::U32(i as u32),

//             (expected, value) if value.data_type()? == *expected => value,

//             (expected, value) => {
//                 return Err(RuntimeError::AnnotationError {
//                     expected: expected.to_string(),
//                     found: value.data_type()?.to_string(),
//                 });
//             }
//         };

//         self.value = value;

//         Ok(())
//     }
// }

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

/// NEVER CALL `clone()` MANUALLY! ALWAYS use `copy_value()` INSTEAD!
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
    pub fn into_runtime_reference(self) -> RuntimeReference {
        Rc::new(RefCell::new(self))
    }

    pub fn copy_value(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::U32(v) => Self::U32(*v),
            Self::S32(v) => Self::S32(*v),
            Self::String(s) => Self::String(s.clone()),
            Self::Bool(v) => Self::Bool(*v),
            Self::Struct { definition, fields } => Self::Struct {
                definition: Rc::clone(definition),
                fields: fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.copy_value()))
                    .collect(),
            },
            Self::Reference(r) => Self::Reference(r.clone()),
            Self::Array {
                inner_data_type,
                contents,
            } => Self::Array {
                inner_data_type: inner_data_type.clone(),
                contents: contents
                    .iter()
                    .map(Self::copy_value)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
        }
    }

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
                variable.borrow().data_type()?,
            ))),
        }
    }

    pub fn struct_access(&self, identifier: &str) -> RuntimeResult<RuntimeValue> {
        match self {
            Self::Struct { definition, fields } => {
                let value = fields
                    .get(identifier)
                    .map(|value| value.copy_value())
                    .ok_or(RuntimeError::InvalidStructFieldAccess {
                        field_name: identifier.to_string(),
                        struct_name: definition.identifier.clone(),
                    });

                value
            }

            Self::Reference(_) => self.dereference(),

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
                (RuntimeValue::U32(val), DataType::S32) => Ok(RuntimeValue::S32(val as i32)),
                (RuntimeValue::S32(val), DataType::U32) => Ok(RuntimeValue::U32(val as u32)),
                (RuntimeValue::Array { contents, .. }, DataType::Array { data_type, count }) => {
                    let mut vec = contents
                        .into_iter()
                        .flat_map(|value| self.coerce(value, data_type))
                        .collect::<Vec<RuntimeValue>>();

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
