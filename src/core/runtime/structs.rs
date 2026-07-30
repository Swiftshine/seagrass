use std::collections::HashMap;

use crate::core::lang::ast::ArrayInitialization;

use crate::core::runtime::value::RuntimeReference;
use crate::core::{
    lang::ast::{DataType, MethodDefinition, StructImpl, StructInitialization},
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

impl Runtime {
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

    pub fn default_value(&self, data_type: &DataType) -> RuntimeResult<RuntimeValue> {
        match data_type {
            DataType::S8 => Ok(RuntimeValue::S8(0)),

            DataType::U8 => Ok(RuntimeValue::U8(0)),

            DataType::S16 => Ok(RuntimeValue::S16(0)),

            DataType::U16 => Ok(RuntimeValue::U16(0)),

            DataType::S32 => Ok(RuntimeValue::S32(0)),

            DataType::U32 => Ok(RuntimeValue::U32(0)),

            DataType::Usize => Ok(RuntimeValue::Usize(0)),

            DataType::Bool => Ok(RuntimeValue::Bool(false)),

            DataType::String => Ok(RuntimeValue::String(String::new())),

            DataType::Reference(_) => Err(RuntimeError::CannotDefaultInitializeReference),

            DataType::Iterator(_) => Err(RuntimeError::CannotDefaultInitializeIterator),

            DataType::Array { data_type, count } => {
                let contents =
                    vec![self.default_value(data_type)?.into_runtime_reference(); count.unwrap()]
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
}
