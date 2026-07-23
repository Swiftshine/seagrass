use std::collections::HashMap;

use crate::core::{lang::ast::{Attribute, DataType, Expression, MethodDefinition, StructDefinition, StructImpl, StructInitialization, Value}, runtime::{ControlFlow, Runtime, RuntimeError, RuntimeResult, RuntimeValue}};


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


impl StructDefinition {
    pub fn has_attribute(&self, identifier: &str) -> bool {
        self.attributes
            .iter()
            .any(|a| a.identifier == identifier)
    }

    pub fn get_attribute(&self, identifier: &str) -> Option<&Attribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.identifier == identifier)
    }

    // "is declared" because the user could say it's pod when it's really not
    pub fn is_declared_pod(&self) -> bool {
        self.has_attribute("pod")
    }

    pub fn byte_order(&self) -> RuntimeResult<ByteOrder> {
        let Some(attribute) = self.get_attribute("byte_order") else {
            return Ok(ByteOrder::Little);
        };
    
        if attribute.arguments.len() != 1 {
            return Err(RuntimeError::InvalidAttributeArgumentCount {
                attribute: "byte_order".to_string(),
                expected: 1,
                found: attribute.arguments.len(),
            });
        }
    
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
}

impl Runtime {
    pub fn validate_pod(&self, struct_definition: &StructDefinition) -> RuntimeResult<()> {
        for field in &struct_definition.fields {
            self.validate_pod_type(&field.data_type)?;
        }

        Ok(())
    }

    fn validate_pod_type(&self, data_type: &DataType) -> RuntimeResult<()> {
        match data_type {
            DataType::U32
            | DataType::S32
            | DataType::Bool => Ok(()),
    
            DataType::String | DataType::Reference(_) => {
                Err(RuntimeError::NonPODType(data_type.to_string()))
            }
    
            DataType::UserDefined(name) => {
                let definition = self.get_struct_definition(name)?;
    
                if !definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(name.clone()));
                }
    
                self.validate_pod(definition)
            }
        }
    }

    pub fn invoke_method(
        &mut self,
        struct_identifier: &str,
        method_identifier: &str,
        struct_reference: RuntimeValue,
        args: Vec<RuntimeValue>,
    ) -> RuntimeResult<RuntimeValue> {
        // make sure we have the struct definition and the function the method call
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

    pub fn serialize_into(
        &self,
        value: &RuntimeValue,
        output: &mut Vec<u8>,
    ) -> RuntimeResult<()> {
        let byte_order = ByteOrder::Little; // todo! do self.byte_order instead and make this a runtime configuration

        match value {
            RuntimeValue::Reference(reference) => {
                let value = reference.borrow();
    
                self.serialize_value(
                    &value.value(),
                    output,
                    byte_order
                )
            }
    
            value => {
                self.serialize_value(
                    value,
                    output,
                    byte_order
                )
            }
        }
    }

    fn serialize_value(
        &self,
        value: &RuntimeValue,
        output: &mut Vec<u8>,
        byte_order: ByteOrder,
    ) -> RuntimeResult<()> {
        match value {
            RuntimeValue::U32(value) => {
                match byte_order {
                    ByteOrder::Little => {
                        output.extend(value.to_le_bytes())
                    }

                    ByteOrder::Big => {
                        output.extend(value.to_be_bytes())
                    }
                }
            }

            RuntimeValue::S32(value) => {
                match byte_order {
                    ByteOrder::Little => {
                        output.extend(value.to_le_bytes())
                    }

                    ByteOrder::Big => {
                        output.extend(value.to_be_bytes())
                    }
                }
            }

            RuntimeValue::Bool(value) => {
                output.push(if *value { 1 } else { 0 });
            }

            RuntimeValue::Struct {
                definition,
                fields,
            } => {
                if !definition.is_declared_pod() {
                    return Err(RuntimeError::NonPODType(
                        definition.identifier.clone()
                    ));
                }

                let struct_byte_order = definition.byte_order()?;

                for field in &definition.fields {
                    let value = fields
                        .get(&field.identifier)
                        .unwrap();

                    self.serialize_value(
                        value,
                        output,
                        struct_byte_order,
                    )?;
                }
            }

            _ => {
                return Err(RuntimeError::NonPODType(
                    value.data_type()?.to_string()
                ));
            }
        }

        Ok(())
    }

    pub fn initialize_struct(&mut self, init: &StructInitialization) -> RuntimeResult<RuntimeValue> {
        let definition = self.get_struct_definition(&init.identifier)?.clone();

        let mut runtime_struct = RuntimeValue::Struct {
            definition: definition.clone(),
            fields: HashMap::new(),
        };

        let mut struct_fields = HashMap::new();

        for field_definition in &definition.fields {
            if !init
                .initialized_fields
                .iter().any(|f| f.identifier == field_definition.identifier)
            {
                if self.config.error_on_incomplete_struct_initialization {
                    return Err(RuntimeError::IncompleteStructInitialization(
                        definition.identifier.clone(),
                    ));
                } else {
                    todo!("implement default values");
                }
            } else {
                let initialized_field = init
                    .initialized_fields
                    .iter()
                    .find(|f| f.identifier == field_definition.identifier)
                    .unwrap();

                let value = self.evaluate_expression(&initialized_field.expression)?;

                let value = match self.apply_type_annotation(value, &field_definition.data_type) {
                    Ok(value) => value,
                    Err(RuntimeError::AnnotationError { expected, found }) => {
                        return Err(RuntimeError::InvalidStructFieldInitialization {
                            field_name: field_definition.identifier.clone(),
                            struct_name: definition.identifier.clone(),
                            expected,
                            found,
                        });
                    }

                    Err(err) => return Err(err),
                };
                let value = self.apply_type_annotation(value, &field_definition.data_type)?;

                struct_fields.insert(initialized_field.identifier.clone(), value);
            }
        }

        if let RuntimeValue::Struct { fields, .. } = &mut runtime_struct {
            *fields = struct_fields;
        }

        Ok(runtime_struct)
    }
}
