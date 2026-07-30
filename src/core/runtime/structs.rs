use std::collections::HashMap;

use crate::core::lang::ast::{ArrayInitialization, StructFieldDefinition, StructMember};

use crate::core::runtime::value::RuntimeReference;
use crate::core::{
    lang::ast::{
        Attribute, DataType, Expression, MethodDefinition, StructDefinition, StructImpl,
        StructInitialization, Value,
    },
    runtime::{ControlFlow, Runtime, RuntimeError, RuntimeResult, RuntimeValue},
};

impl StructImpl {
    pub fn get_method_definition(&self, identifier: &str) -> RuntimeResult<&MethodDefinition> {
        self.method_definitions
            .iter()
            .find(|m| m.identifier == identifier)
            .ok_or(RuntimeError::MethodNotFound {
                method_identifier: identifier.to_string(),
                struct_identifier: self.struct_identifier.clone(),
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteOrder {
    Little,
    Big,
}

pub enum StringSerialization {
    Ascii,
}

impl Attribute {
    pub fn assert_argument_count(&self, expected: usize) -> RuntimeResult<()> {
        if self.arguments.len() != expected {
            Err(RuntimeError::InvalidAttributeArgumentCount {
                attribute: self.identifier.clone(),
                expected,
                found: self.arguments.len(),
            })
        } else {
            Ok(())
        }
    }
}

pub trait Attributable {
    fn has_attribute(&self, identifier: &'static str) -> bool;

    fn get_attribute(&self, identifier: &'static str) -> RuntimeResult<&Attribute>;
}

impl Attributable for StructDefinition {
    fn has_attribute(&self, identifier: &'static str) -> bool {
        self.attributes.iter().any(|a| a.identifier == identifier)
    }

    fn get_attribute(&self, identifier: &'static str) -> RuntimeResult<&Attribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.identifier == identifier)
            .ok_or(RuntimeError::AttributeNotFound(identifier))
    }
}

impl StructDefinition {
    // "is declared" because the user could say it's pod when it's really not
    pub fn is_declared_pod(&self) -> bool {
        self.has_attribute("pod")
    }

    pub fn byte_order(&self) -> RuntimeResult<ByteOrder> {
        let Ok(attribute) = self.get_attribute("byte_order") else {
            return Ok(ByteOrder::Little);
        };

        attribute.assert_argument_count(1)?;

        match &attribute.arguments[0] {
            Expression::Value(Value::String(value)) => match value.as_str() {
                "little" => Ok(ByteOrder::Little),
                "big" => Ok(ByteOrder::Big),

                other => Err(RuntimeError::InvalidAttributeArgument {
                    attribute: "byte_order".to_string(),
                    expected: "\"little\" or \"big\"".to_string(),
                    found: other.to_string(),
                }),
            },

            other => Err(RuntimeError::InvalidAttributeArgument {
                attribute: "byte_order".to_string(),
                expected: "string literal".to_string(),
                found: format!("{:?}", other),
            }),
        }
    }

    pub fn fields(&self) -> impl Iterator<Item = &StructFieldDefinition> {
        self.members.iter().filter_map(|member| match member {
            StructMember::Field(field) => Some(field),
            _ => None,
        })
    }
}

impl Attributable for StructFieldDefinition {
    fn has_attribute(&self, identifier: &'static str) -> bool {
        self.attributes.iter().any(|a| a.identifier == identifier)
    }

    fn get_attribute(&self, identifier: &'static str) -> RuntimeResult<&Attribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.identifier == identifier)
            .ok_or(RuntimeError::AttributeNotFound(identifier))
    }
}

impl StructFieldDefinition {
    pub fn string_serialization(&self) -> RuntimeResult<Option<StringSerialization>> {
        let Ok(attribute) = self.get_attribute("serialize_as") else {
            return Ok(None);
        };

        attribute.assert_argument_count(1)?;

        match &attribute.arguments[0] {
            Expression::Value(Value::String(s)) => match s.as_str() {
                "ascii" => Ok(Some(StringSerialization::Ascii)),
                _ => Err(RuntimeError::UnexpectedAttributeArgument {
                    attribute: attribute.identifier.clone(),
                    found: s.clone(),
                }),
            },
            _ => unreachable!(),
        }
    }

    pub fn alignment(&self) -> RuntimeResult<Option<usize>> {
        let Ok(attribute) = self.get_attribute("align") else {
            return Ok(None);
        };

        attribute.assert_argument_count(1)?;

        match &attribute.arguments[0] {
            Expression::Value(Value::U32(value)) => Ok(Some(*value as usize)),

            Expression::Value(Value::S32(value)) if *value >= 0 => Ok(Some(*value as usize)),

            other => Err(RuntimeError::InvalidAttributeArgument {
                attribute: "align".to_string(),
                expected: "positive integer".to_string(),
                found: format!("{:?}", other),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ValidationException {
    SerializeAsAscii,
}

impl Runtime {
    pub fn validate_pod(&self, struct_definition: &StructDefinition) -> RuntimeResult<()> {
        for field in struct_definition.fields() {
            self.validate_pod_type(field)?;
        }

        Ok(())
    }

    fn get_validation_exceptions(
        &self,
        field_definition: &StructFieldDefinition,
    ) -> RuntimeResult<Vec<ValidationException>> {
        let mut exceptions = Vec::new();

        if matches!(field_definition.data_type, DataType::String)
            && let Ok(attribute) = field_definition.get_attribute("serialize_as")
            && let Expression::Value(Value::String(string)) = &attribute.arguments[0]
        {
            match string.as_str() {
                "ascii" => exceptions.push(ValidationException::SerializeAsAscii),
                _ => {
                    return Err(RuntimeError::UnexpectedAttributeArgument {
                        attribute: attribute.identifier.clone(),
                        found: string.clone(),
                    });
                }
            }
        }

        Ok(exceptions)
    }

    fn validate_pod_type(&self, field_definition: &StructFieldDefinition) -> RuntimeResult<()> {
        // special exceptions
        let exceptions = self.get_validation_exceptions(field_definition)?;
        self.validate_pod_for_data_type(&field_definition.data_type, exceptions)
    }

    fn validate_pod_for_data_type(
        &self,
        data_type: &DataType,
        exceptions: Vec<ValidationException>,
    ) -> RuntimeResult<()> {
        match data_type {
            DataType::U32 | DataType::S32 | DataType::Bool => Ok(()),

            DataType::String => {
                if exceptions.contains(&ValidationException::SerializeAsAscii) {
                    Ok(())
                } else {
                    Err(RuntimeError::NonPODType(data_type.to_string()))
                }
            }
            DataType::Reference(_) | DataType::Iterator(_) => {
                Err(RuntimeError::NonPODType(data_type.to_string()))
            }
            DataType::Array { data_type, .. } => {
                self.validate_pod_for_data_type(data_type, exceptions)
            }
            DataType::UserDefined(identifier) => {
                let struct_field_definition = self.get_struct_definition(identifier)?;

                if !struct_field_definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(identifier.clone()));
                }

                self.validate_pod(struct_field_definition)
            }
        }
    }

    pub fn invoke_method_for_struct(
        &mut self,
        struct_identifier: &str,
        method_identifier: &str,
        struct_reference: RuntimeValue,
        args: Vec<RuntimeValue>,
    ) -> RuntimeResult<RuntimeValue> {
        // make sure we have the struct struct_field_definition and the function the method call
        // is asking for
        self.get_struct_definition(struct_identifier)?;

        let parameters = &self
            .get_struct_impl(struct_identifier)?
            .get_method_definition(method_identifier)?
            .parameters;

        let mut args = Self::collect_runtime_function_arguments(parameters, args)?;

        let scope_resolved_name = struct_identifier.to_string() + "::" + method_identifier;

        struct_reference.assert_reference()?;
        args.insert(0, ("self".to_string(), struct_reference));

        self.push_frame(scope_resolved_name, args);

        let block = &self
            .get_struct_impl(struct_identifier)?
            .get_method_definition(method_identifier)?
            .body
            .clone();

        let result = self.execute_function_body(block);
        self.pop_frame();

        match result? {
            ControlFlow::Continue => Ok(RuntimeValue::None),
            ControlFlow::Return(value) => Ok(value),
            _ => unreachable!("expected ControlFlow::Continue or ControlFlow::Return"),
        }
    }

    pub fn serialize_into(&self, value: &RuntimeValue, output: &mut Vec<u8>) -> RuntimeResult<()> {
        match value {
            RuntimeValue::Reference(reference) => {
                let value = reference.borrow();

                self.serialize_value(&value, output, self.byte_order)
            }

            value => self.serialize_value(value, output, self.byte_order),
        }
    }

    fn serialize_field(
        &self,
        field: &StructFieldDefinition,
        value: &RuntimeValue,
        output: &mut Vec<u8>,
        byte_order: ByteOrder,
    ) -> RuntimeResult<()> {
        if let RuntimeValue::String(s) = value {
            match field.string_serialization()? {
                Some(StringSerialization::Ascii) => {
                    if !s.is_ascii() {
                        return Err(RuntimeError::NonAsciiString(s.clone()));
                    }

                    output.extend(s.bytes());
                    output.push(0);
                    return Ok(());
                }
                None => {}
            }
        }

        self.serialize_value(value, output, byte_order)
    }

    fn serialize_value(
        &self,
        value: &RuntimeValue,
        output: &mut Vec<u8>,
        byte_order: ByteOrder,
    ) -> RuntimeResult<()> {
        match value {
            RuntimeValue::U32(value) => match byte_order {
                ByteOrder::Little => output.extend(value.to_le_bytes()),

                ByteOrder::Big => output.extend(value.to_be_bytes()),
            },

            RuntimeValue::S32(value) => match byte_order {
                ByteOrder::Little => output.extend(value.to_le_bytes()),

                ByteOrder::Big => output.extend(value.to_be_bytes()),
            },

            RuntimeValue::Bool(value) => {
                output.push(if *value { 1 } else { 0 });
            }

            RuntimeValue::Array { contents, .. } => {
                for item in contents {
                    self.serialize_value(&item.borrow(), output, byte_order)?;
                }
            }

            RuntimeValue::Struct { definition, fields } => {
                if !definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(definition.identifier.clone()));
                }

                let struct_byte_order = definition.byte_order()?;

                for member in &definition.members {
                    match member {
                        StructMember::Field(field) => {
                            if let Some(alignment) = field.alignment()? {
                                let remainder = output.len() % alignment;

                                if remainder != 0 {
                                    let padding = alignment - remainder;
                                    output.resize(output.len() + padding, 0);
                                }
                            }

                            let value = fields.get(&field.identifier).unwrap();
                            self.serialize_field(
                                field,
                                &value.borrow(),
                                output,
                                struct_byte_order,
                            )?;
                        }

                        StructMember::Padding(count) => {
                            output.resize(output.len() + *count, 0);
                        }
                    }
                }
            }

            _ => {
                return Err(RuntimeError::NonPODType(value.data_type()?.to_string()));
            }
        }

        Ok(())
    }

    pub fn default_value(&self, data_type: &DataType) -> RuntimeResult<RuntimeValue> {
        match data_type {
            DataType::U32 => Ok(RuntimeValue::U32(0)),

            DataType::S32 => Ok(RuntimeValue::S32(0)),

            DataType::Bool => Ok(RuntimeValue::Bool(false)),

            DataType::String => Ok(RuntimeValue::String(String::new())),

            DataType::Reference(_) => Err(RuntimeError::CannotDefaultInitializeReference),

            DataType::Iterator(_) => Err(RuntimeError::CannotDefaultInitializeIterator),

            DataType::Array { data_type, count } => {
                let contents =
                    vec![self.default_value(data_type)?.into_runtime_reference(); *count]
                        .into_boxed_slice();

                Ok(RuntimeValue::Array {
                    inner_data_type: *data_type.clone(),
                    contents,
                })
            }

            DataType::UserDefined(identifier) => {
                let definition = self.get_struct_definition(identifier)?.clone();

                let mut fields = HashMap::new();

                for field in definition.fields() {
                    fields.insert(
                        field.identifier.clone(),
                        self.default_value(&field.data_type)?
                            .into_runtime_reference(),
                    );
                }

                Ok(RuntimeValue::Struct { definition, fields })
            }
        }
    }

    pub fn initialize_array(&mut self, init: &ArrayInitialization) -> RuntimeResult<RuntimeValue> {
        // in the case that the type is annotated:
        // if it turns out later on that there are more elements in the array than are initialized here,
        // that's okay, because the rest can be default-initialized.
        // if there are more, however, raise an error

        if init.initialized_fields.is_empty() {
            return Err(RuntimeError::CannotInferEmptyArrayType);
        }

        let contents: Box<[RuntimeReference]> = init
            .initialized_fields
            .iter()
            .flat_map(|expr| self.evaluate_expression_to_value(expr))
            .map(RuntimeValue::into_runtime_reference)
            .collect();

        assert_eq!(contents.len(), init.initialized_fields.len());

        let data_type = contents[0].borrow().data_type()?;

        for item in &contents {
            assert_eq!(item.borrow().data_type()?, data_type);
        }

        Ok(RuntimeValue::Array {
            inner_data_type: data_type,
            contents,
        })
    }

    pub fn initialize_struct(
        &mut self,
        init: &StructInitialization,
    ) -> RuntimeResult<RuntimeValue> {
        let struct_definition = self.get_struct_definition(&init.identifier)?.clone();

        let mut struct_fields = HashMap::new();

        for field_definition in struct_definition.fields() {
            if let Some(initialized_field) = init
                .initialized_fields
                .iter()
                .find(|f| f.identifier == field_definition.identifier)
            {
                let value = self.evaluate_expression_to_value(&initialized_field.expression)?;

                let value = match self.apply_type_annotation(value, &field_definition.data_type) {
                    Ok(value) => value,
                    Err(RuntimeError::AnnotationError { expected, found }) => {
                        return Err(RuntimeError::InvalidStructFieldInitialization {
                            field_name: field_definition.identifier.clone(),
                            struct_name: struct_definition.identifier.clone(),
                            expected,
                            found,
                        });
                    }

                    Err(err) => return Err(err),
                };

                struct_fields.insert(
                    field_definition.identifier.clone(),
                    value.into_runtime_reference(),
                );
            } else if init.use_defaults {
                let value = self.default_value(&field_definition.data_type)?;
                struct_fields.insert(
                    field_definition.identifier.clone(),
                    value.into_runtime_reference(),
                );
            } else {
                return Err(RuntimeError::IncompleteStructInitialization(
                    struct_definition.identifier.clone(),
                ));
            }
        }

        Ok(RuntimeValue::Struct {
            definition: struct_definition,
            fields: struct_fields,
        })
    }

    pub fn deserialize(&self, data_type: &DataType, bytes: &[u8]) -> RuntimeResult<RuntimeValue> {
        let mut offset = 0;

        self.deserialize_value(data_type, bytes, &mut offset, self.byte_order)
    }

    fn deserialize_value(
        &self,
        data_type: &DataType,
        bytes: &[u8],
        offset: &mut usize,
        byte_order: ByteOrder,
    ) -> RuntimeResult<RuntimeValue> {
        match data_type {
            DataType::U32 => {
                let slice = self.read_exact::<4>(bytes, offset)?;

                Ok(RuntimeValue::U32(match byte_order {
                    ByteOrder::Little => u32::from_le_bytes(slice),
                    ByteOrder::Big => u32::from_be_bytes(slice),
                }))
            }

            DataType::S32 => {
                let slice = self.read_exact::<4>(bytes, offset)?;

                Ok(RuntimeValue::S32(match byte_order {
                    ByteOrder::Little => i32::from_le_bytes(slice),
                    ByteOrder::Big => i32::from_be_bytes(slice),
                }))
            }

            DataType::Bool => {
                let value = self.read_byte(bytes, offset)?;

                match value {
                    0 => Ok(RuntimeValue::Bool(false)),
                    1 => Ok(RuntimeValue::Bool(true)),
                    other => Err(RuntimeError::SerializationError(format!(
                        "expected bool, found a value of 0x{:02}",
                        other
                    ))),
                }
            }

            DataType::String => {
                let mut string = Vec::new();

                loop {
                    let byte = self.read_byte(bytes, offset)?;

                    if byte == 0 {
                        break;
                    }

                    string.push(byte);
                }

                let string = String::from_utf8(string)
                    .map_err(|_| RuntimeError::SerializationError("Invalid UTF-8".to_string()))?;

                Ok(RuntimeValue::String(string))
            }

            DataType::Array { data_type, count } => {
                let count = *count;

                let contents: Box<[RuntimeReference]> = (0..count)
                    .flat_map(|_| self.deserialize_value(data_type, bytes, offset, byte_order))
                    .map(RuntimeValue::into_runtime_reference)
                    .collect();

                Ok(RuntimeValue::Array {
                    inner_data_type: *data_type.clone(),
                    contents,
                })
            }

            DataType::Reference(_) => Err(RuntimeError::CannotDeserialize("reference")),

            DataType::Iterator(_) => Err(RuntimeError::CannotDeserialize("iterator")),

            DataType::UserDefined(identifier) => {
                let definition = self.get_struct_definition(identifier)?.clone();

                if !definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(identifier.clone()));
                }

                let byte_order = definition.byte_order()?;

                let mut fields = HashMap::new();

                for member in &definition.members {
                    match member {
                        StructMember::Padding(count) => {
                            *offset += *count;
                        }

                        StructMember::Field(field) => {
                            if let Some(alignment) = field.alignment()? {
                                let remainder = *offset % alignment;

                                if remainder != 0 {
                                    *offset += alignment - remainder;
                                }
                            }

                            let value = if matches!(field.data_type, DataType::String) {
                                match field.string_serialization()? {
                                    Some(StringSerialization::Ascii) => self.deserialize_value(
                                        &DataType::String,
                                        bytes,
                                        offset,
                                        byte_order,
                                    )?,

                                    None => self.deserialize_value(
                                        &field.data_type,
                                        bytes,
                                        offset,
                                        byte_order,
                                    )?,
                                }
                            } else {
                                self.deserialize_value(&field.data_type, bytes, offset, byte_order)?
                            };

                            fields.insert(field.identifier.clone(), value.into_runtime_reference());
                        }
                    }
                }

                Ok(RuntimeValue::Struct { definition, fields })
            }
        }
    }

    fn read_byte(&self, bytes: &[u8], offset: &mut usize) -> RuntimeResult<u8> {
        if *offset >= bytes.len() {
            return Err(RuntimeError::UnexpectedEOF);
        }

        let value = bytes[*offset];
        *offset += 1;

        Ok(value)
    }

    fn read_exact<const N: usize>(
        &self,
        bytes: &[u8],
        offset: &mut usize,
    ) -> RuntimeResult<[u8; N]> {
        if *offset + N > bytes.len() {
            return Err(RuntimeError::UnexpectedEOF);
        }

        let mut result = [0u8; N];
        result.copy_from_slice(&bytes[*offset..*offset + N]);

        *offset += N;

        Ok(result)
    }
}
