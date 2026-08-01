use std::{collections::HashMap, io::Cursor};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

use crate::core::{
    lang::ast::{
        Attribute, DataType, Expression, StructDefinition, StructFieldDefinition, StructMember,
        Value,
    },
    runtime::{Runtime, RuntimeError, RuntimeResult, RuntimeValue, value::RuntimeReference},
};

impl DataType {
    pub fn static_size(&self, runtime: &Runtime) -> RuntimeResult<usize> {
        match self {
            Self::S8 | Self::U8 | Self::Bool => Ok(1),
            Self::S16 | Self::U16 => Ok(2),
            Self::S32 | Self::U32 | Self::F32 => Ok(4),
            Self::F64 => Ok(8),

            Self::Array {
                inner_data_type,
                count,
            } => {
                let count =
                    count.ok_or(RuntimeError::CannotDetermineDataTypeSize(self.to_string()))?;

                Ok(inner_data_type.static_size(runtime)? * count)
            }

            Self::Iterator(_)
            | Self::String
            | Self::Usize
            | Self::Reference(_)
            | Self::NativeObject(_) => {
                Err(RuntimeError::CannotDetermineDataTypeSize(self.to_string()))
            }

            Self::UserDefined(struct_name) => {
                let struct_definition = runtime.get_struct_definition(struct_name)?;

                if !struct_definition.is_declared_pod() {
                    return Err(RuntimeError::CannotDetermineDataTypeSize(self.to_string()));
                }

                let mut size = 0;

                for member in &struct_definition.members {
                    match member {
                        StructMember::Padding(pad) => size += *pad,

                        StructMember::Field(field) => {
                            // strings are only fixed-size if explicitly ascii serialized,
                            // but even, then we don't know the *actual* length,
                            // so we say that they don't have a statically known size

                            if let Some(alignment) = field.alignment()? {
                                let remainder = size % alignment;

                                if remainder != 0 {
                                    size += alignment - remainder;
                                }
                            }

                            size += field.data_type.static_size(runtime)?;
                        }
                    }
                }

                Ok(size)
            }
        }
    }
}

impl RuntimeValue {
    pub fn serialized_size(&self, runtime: &Runtime) -> RuntimeResult<usize> {
        match self {
            RuntimeValue::S8(_) | RuntimeValue::U8(_) | RuntimeValue::Bool(_) => Ok(1),

            RuntimeValue::S16(_) | RuntimeValue::U16(_) => Ok(2),

            RuntimeValue::S32(_) | RuntimeValue::U32(_) | RuntimeValue::F32(_) => Ok(4),

            RuntimeValue::F64(_) => Ok(8),

            RuntimeValue::String(value) => {
                // strings serialize as null-terminated UTF-8
                Ok(value.len() + 1)
            }

            RuntimeValue::Array { contents, .. } => {
                let mut size = 0;

                for value in contents {
                    size += value.borrow().serialized_size(runtime)?;
                }

                Ok(size)
            }

            RuntimeValue::Struct { definition, fields } => {
                if !definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(definition.identifier.clone()));
                }

                let mut size = 0;

                for member in &definition.members {
                    match member {
                        StructMember::Padding(count) => {
                            size += *count;
                        }

                        StructMember::Field(field) => {
                            if let Some(alignment) = field.alignment()? {
                                let remainder = size % alignment;

                                if remainder != 0 {
                                    size += alignment - remainder;
                                }
                            }

                            let value = fields.get(&field.identifier).ok_or_else(|| {
                                RuntimeError::InvalidStructFieldAccess {
                                    field_name: field.identifier.clone(),
                                    struct_name: definition.identifier.clone(),
                                }
                            })?;

                            let value = value.borrow();

                            if matches!(*value, RuntimeValue::String(_)) {
                                match field.string_serialization()? {
                                    Some(StringSerialization::Ascii) => {
                                        size += match &*value {
                                            RuntimeValue::String(string) => string.len() + 1,
                                            _ => unreachable!(),
                                        };

                                        continue;
                                    }

                                    None => {}
                                }
                            }

                            size += value.serialized_size(runtime)?;
                        }
                    }
                }

                Ok(size)
            }

            RuntimeValue::None
            | RuntimeValue::Usize(_)
            | RuntimeValue::Iterator { .. }
            | RuntimeValue::Reference(_)
            | RuntimeValue::NativeObject(_) => Err(RuntimeError::CannotDetermineDataTypeSize(
                self.data_type()?.to_string(),
            )),
        }
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

struct SerializationContext<'a> {
    runtime: &'a Runtime,
    input: Option<Cursor<&'a [u8]>>,
    output: Vec<u8>,
    already_read_fields: HashMap<String, RuntimeValue>,
}

impl<'a> SerializationContext<'a> {
    fn for_writing(runtime: &'a Runtime) -> Self {
        Self {
            runtime,
            input: None,
            output: Vec::new(),
            already_read_fields: HashMap::new(),
        }
    }

    fn for_reading(runtime: &'a Runtime, input: &'a [u8]) -> Self {
        Self {
            runtime,
            input: Some(Cursor::new(input)),
            output: Vec::new(),
            already_read_fields: HashMap::new(),
        }
    }
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

    fn find_element_count_for_counted_array(
        &self,
        ctx: &SerializationContext,
    ) -> RuntimeResult<usize> {
        let attribute = self.get_attribute("counted_by")?;

        attribute.assert_argument_count(1)?;

        match &attribute.arguments[0] {
            Expression::Value(value) => {
                match value {
                    Value::Identifier(ident) => {
                        // something within this struct
                        if let Some(value) = ctx.already_read_fields.get(ident)
                            && value.data_type()?.is_numeric()
                        {
                            let value = value.copy_value().cast_to(&DataType::Usize)?;

                            if let RuntimeValue::Usize(size) = value {
                                Ok(size)
                            } else {
                                unreachable!("expected it to resolve to a usize")
                            }
                        } else {
                            Err(RuntimeError::CountedByFail(ident.clone()))
                        }
                    }

                    _ => {
                        // for constants
                        let value = ctx.runtime.resolve_ast_value(value)?;

                        let data_type = value.data_type()?;

                        if data_type.is_numeric()
                            && let RuntimeValue::Usize(size) = value.cast_to(&DataType::Usize)?
                        {
                            Ok(size)
                        } else {
                            Err(RuntimeError::InvalidAttributeArgument {
                                attribute: "counted_by".to_string(),
                                expected: "positive integer".to_string(),
                                found: format!("{:?}", data_type),
                            })
                        }
                    }
                }
            }
            other => Err(RuntimeError::InvalidAttributeArgument {
                attribute: "counted_by".to_string(),
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
            DataType::S8
            | DataType::U8
            | DataType::S16
            | DataType::U16
            | DataType::S32
            | DataType::U32
            | DataType::F32
            | DataType::F64
            | DataType::Bool => Ok(()),

            DataType::String => {
                if exceptions.contains(&ValidationException::SerializeAsAscii) {
                    Ok(())
                } else {
                    Err(RuntimeError::NonPODType(data_type.to_string()))
                }
            }

            DataType::Reference(_)
            | DataType::Iterator(_)
            | DataType::Usize
            | DataType::NativeObject(_) => Err(RuntimeError::NonPODType(data_type.to_string())),

            DataType::Array {
                inner_data_type: data_type,
                ..
            } => self.validate_pod_for_data_type(data_type, exceptions),

            DataType::UserDefined(identifier) => {
                let struct_field_definition = self.get_struct_definition(identifier)?;

                if !struct_field_definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(identifier.clone()));
                }

                self.validate_pod(struct_field_definition)
            }
        }
    }

    pub fn serialize_into(&self, value: &RuntimeValue) -> RuntimeResult<Vec<u8>> {
        let mut ctx = SerializationContext::for_writing(self);

        match value {
            RuntimeValue::Reference(reference) => {
                let value = reference.borrow();

                let _ = self.serialize_value(&value, self.byte_order, &mut ctx);

                Ok(ctx.output)
            }

            value => {
                let _ = self.serialize_value(value, self.byte_order, &mut ctx);

                Ok(ctx.output)
            }
        }
    }

    fn serialize_field(
        &self,
        field: &StructFieldDefinition,
        value: &RuntimeValue,
        byte_order: ByteOrder,
        ctx: &mut SerializationContext,
    ) -> RuntimeResult<()> {
        if let RuntimeValue::String(s) = value {
            match field.string_serialization()? {
                Some(StringSerialization::Ascii) => {
                    if !s.is_ascii() {
                        return Err(RuntimeError::NonAsciiString(s.clone()));
                    }

                    ctx.output.extend(s.bytes());
                    ctx.output.push(0);
                    return Ok(());
                }
                None => {}
            }
        }

        self.serialize_value(value, byte_order, ctx)
    }

    fn serialize_value(
        &self,
        value: &RuntimeValue,
        byte_order: ByteOrder,
        ctx: &mut SerializationContext,
    ) -> RuntimeResult<()> {
        match value {
            RuntimeValue::S8(value) => ctx.output.push(*value as u8),

            RuntimeValue::U8(value) => ctx.output.push(*value),

            RuntimeValue::S16(value) => match byte_order {
                ByteOrder::Little => ctx.output.extend(value.to_le_bytes()),

                ByteOrder::Big => ctx.output.extend(value.to_be_bytes()),
            },

            RuntimeValue::U16(value) => match byte_order {
                ByteOrder::Little => ctx.output.extend(value.to_le_bytes()),

                ByteOrder::Big => ctx.output.extend(value.to_be_bytes()),
            },

            RuntimeValue::S32(value) => match byte_order {
                ByteOrder::Little => ctx.output.extend(value.to_le_bytes()),

                ByteOrder::Big => ctx.output.extend(value.to_be_bytes()),
            },

            RuntimeValue::U32(value) => match byte_order {
                ByteOrder::Little => ctx.output.extend(value.to_le_bytes()),

                ByteOrder::Big => ctx.output.extend(value.to_be_bytes()),
            },

            RuntimeValue::F32(value) => match byte_order {
                ByteOrder::Little => ctx.output.extend(value.to_le_bytes()),
                ByteOrder::Big => ctx.output.extend(value.to_be_bytes()),
            },

            RuntimeValue::F64(value) => match byte_order {
                ByteOrder::Little => ctx.output.extend(value.to_le_bytes()),
                ByteOrder::Big => ctx.output.extend(value.to_be_bytes()),
            },

            RuntimeValue::Bool(value) => {
                ctx.output.push(if *value { 1 } else { 0 });
            }

            RuntimeValue::Array { contents, .. } => {
                for item in contents {
                    self.serialize_value(&item.borrow(), byte_order, ctx)?;
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
                                let remainder = ctx.output.len() % alignment;

                                if remainder != 0 {
                                    let padding = alignment - remainder;
                                    ctx.output.resize(ctx.output.len() + padding, 0);
                                }
                            }

                            let value = fields.get(&field.identifier).unwrap();
                            self.serialize_field(field, &value.borrow(), struct_byte_order, ctx)?;
                        }

                        StructMember::Padding(count) => {
                            ctx.output.resize(ctx.output.len() + *count, 0);
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

    pub fn deserialize(&self, data_type: &DataType, bytes: &[u8]) -> RuntimeResult<RuntimeValue> {
        let mut ctx = SerializationContext::for_reading(self, bytes);

        self.deserialize_value(data_type, self.byte_order, &mut ctx)
    }

    fn deserialize_value(
        &self,
        data_type: &DataType,
        byte_order: ByteOrder,
        ctx: &mut SerializationContext,
    ) -> RuntimeResult<RuntimeValue> {
        match data_type {
            DataType::S8 => {
                let byte = ctx.input.as_mut().unwrap().read_i8()?;
                Ok(RuntimeValue::S8(byte))
            }

            DataType::U8 => {
                let byte = ctx.input.as_mut().unwrap().read_u8()?;
                Ok(RuntimeValue::U8(byte))
            }

            DataType::S16 => Ok(RuntimeValue::S16(match byte_order {
                ByteOrder::Little => ctx.input.as_mut().unwrap().read_i16::<LittleEndian>()?,
                ByteOrder::Big => ctx.input.as_mut().unwrap().read_i16::<BigEndian>()?,
            })),

            DataType::U16 => Ok(RuntimeValue::U16(match byte_order {
                ByteOrder::Little => ctx.input.as_mut().unwrap().read_u16::<LittleEndian>()?,
                ByteOrder::Big => ctx.input.as_mut().unwrap().read_u16::<BigEndian>()?,
            })),

            DataType::S32 => Ok(RuntimeValue::S32(match byte_order {
                ByteOrder::Little => ctx.input.as_mut().unwrap().read_i32::<LittleEndian>()?,
                ByteOrder::Big => ctx.input.as_mut().unwrap().read_i32::<BigEndian>()?,
            })),

            DataType::U32 => Ok(RuntimeValue::U32(match byte_order {
                ByteOrder::Little => ctx.input.as_mut().unwrap().read_u32::<LittleEndian>()?,
                ByteOrder::Big => ctx.input.as_mut().unwrap().read_u32::<BigEndian>()?,
            })),

            DataType::F32 => Ok(RuntimeValue::F32(match byte_order {
                ByteOrder::Little => ctx.input.as_mut().unwrap().read_f32::<LittleEndian>()?,
                ByteOrder::Big => ctx.input.as_mut().unwrap().read_f32::<BigEndian>()?,
            })),

            DataType::F64 => Ok(RuntimeValue::F64(match byte_order {
                ByteOrder::Little => ctx.input.as_mut().unwrap().read_f64::<LittleEndian>()?,
                ByteOrder::Big => ctx.input.as_mut().unwrap().read_f64::<BigEndian>()?,
            })),

            DataType::Bool => {
                let value = ctx.input.as_mut().unwrap().read_u8()?;

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
                    let byte = ctx.input.as_mut().unwrap().read_u8()?;

                    if byte == 0 {
                        break;
                    }

                    string.push(byte);
                }

                let string = String::from_utf8(string)
                    .map_err(|_| RuntimeError::SerializationError("Invalid UTF-8".to_string()))?;

                Ok(RuntimeValue::String(string))
            }

            DataType::Array {
                inner_data_type: data_type,
                count,
            } => {
                if let Some(count) = count {
                    // regular array
                    let count = *count;

                    let contents: Box<[RuntimeReference]> = (0..count)
                        .flat_map(|_| self.deserialize_value(data_type, byte_order, ctx))
                        .map(RuntimeValue::into_runtime_reference)
                        .collect();

                    Ok(RuntimeValue::Array {
                        inner_data_type: *data_type.clone(),
                        contents,
                    })
                } else {
                    Err(RuntimeError::UncountedArray)
                }
            }

            DataType::Reference(_) => Err(RuntimeError::CannotDeserialize("reference".to_string())),

            DataType::Iterator(_) => Err(RuntimeError::CannotDeserialize("iterator".to_string())),

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
                            let pos = ctx.input.as_ref().unwrap().position();
                            ctx.input
                                .as_mut()
                                .unwrap()
                                .set_position(pos + *count as u64);
                        }

                        StructMember::Field(field) => {
                            if let Some(alignment) = field.alignment()? {
                                let remainder =
                                    ctx.input.as_ref().unwrap().position() % alignment as u64;

                                if remainder != 0 {
                                    let pos = ctx.input.as_ref().unwrap().position();

                                    ctx.input
                                        .as_mut()
                                        .unwrap()
                                        .set_position(pos + alignment as u64 - remainder);
                                }
                            }

                            // some values should or shouldn't be added based on need
                            // we do need numeric values. we DON'T need strings or arrays
                            let mut should_add_to_ctx = false;

                            let value = if matches!(field.data_type, DataType::String) {
                                match field.string_serialization()? {
                                    Some(StringSerialization::Ascii) => {
                                        self.deserialize_value(&DataType::String, byte_order, ctx)?
                                    }

                                    None => {
                                        self.deserialize_value(&field.data_type, byte_order, ctx)?
                                    }
                                }
                            } else if let DataType::Array {
                                inner_data_type, ..
                            } = &field.data_type
                                && let Ok(element_count) =
                                    field.find_element_count_for_counted_array(ctx)
                            {
                                let new_array_type = DataType::Array {
                                    inner_data_type: inner_data_type.clone(),
                                    count: Some(element_count),
                                };

                                self.deserialize_value(&new_array_type, byte_order, ctx)?
                            } else {
                                should_add_to_ctx = true;
                                self.deserialize_value(&field.data_type, byte_order, ctx)?
                            };

                            if should_add_to_ctx {
                                ctx.already_read_fields
                                    .insert(field.identifier.clone(), value.copy_value());
                            }

                            fields.insert(field.identifier.clone(), value.into_runtime_reference());
                        }
                    }
                }

                Ok(RuntimeValue::Struct { definition, fields })
            }

            _ => Err(RuntimeError::CannotDeserialize(data_type.to_string())),
        }
    }
}
