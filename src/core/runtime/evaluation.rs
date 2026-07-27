use crate::core::{
    lang::ast::{BinaryOperator, DataType, Expression, Value},
    runtime::{Runtime, RuntimeError, RuntimeResult, RuntimeValue},
};

impl Runtime {
    pub fn evaluate_boolean_expression(&mut self, expression: &Expression) -> RuntimeResult<bool> {
        let value = self.evaluate_expression(expression)?;

        if let RuntimeValue::Bool(boolean) = value {
            Ok(boolean)
        } else {
            Err(RuntimeError::ExpectedBoolean)
        }
    }

    pub fn apply_type_annotation(
        &self,
        value: RuntimeValue,
        expected: &DataType,
    ) -> RuntimeResult<RuntimeValue> {
        let value_data_type_string = value.data_type()?.to_string();

        match (value, expected) {
            (RuntimeValue::S32(i), DataType::U32) if i >= 0 => Ok(RuntimeValue::U32(i as u32)),

            (RuntimeValue::Bool(b), DataType::Bool) => Ok(RuntimeValue::Bool(b)),

            (RuntimeValue::String(s), DataType::String) => Ok(RuntimeValue::String(s)),

            (value, DataType::UserDefined(expected))
                if value.data_type()? == DataType::UserDefined(expected.clone()) =>
            {
                Ok(value)
            }

            // implicit type coercion
            (val, ty) if val.data_type()?.can_be_coereced_into(ty) => self.coerce(val, ty),

            // todo: handle type annotations of struct initialization
            _ => Err(RuntimeError::AnnotationError {
                expected: expected.to_string(),
                found: value_data_type_string,
            }),
        }
    }

    pub fn evaluate_expression(&mut self, expression: &Expression) -> RuntimeResult<RuntimeValue> {
        match expression {
            Expression::Value(value) => self.resolve_value(value),
            Expression::FunctionCall(call) => {
                let args = call
                    .arguments
                    .iter()
                    .map(|expr| self.evaluate_expression(expr))
                    .collect::<RuntimeResult<Vec<_>>>()?;

                let generics = &call.generics;

                self.call_function(&call.identifier, args, generics)
            }

            Expression::Binary { lhs, rhs, operator } => {
                let lhs = self.evaluate_expression(lhs)?;
                let rhs = self.evaluate_expression(rhs)?;

                self.evaluate_binary(*operator, lhs, rhs)
            }

            Expression::StructInitialization(init) => self.initialize_struct(init),

            Expression::ArrayInitialization(init) => self.initialize_array(init),

            Expression::StructFieldAccess {
                expression,
                field_identifier: field,
            } => {
                let value = self.evaluate_expression(expression)?.resolve();

                match value {
                    RuntimeValue::Struct { definition, fields } => fields
                        .get(field)
                        .cloned()
                        .ok_or(RuntimeError::InvalidStructFieldAccess {
                            field_name: field.clone(),
                            struct_name: definition.identifier.clone(),
                        }),

                    _ => Err(RuntimeError::InvalidStructFieldAccessTarget {
                        field: field.clone(),
                        data_type: value.data_type()?.to_string(),
                    }),
                }
            }

            Expression::Reference(expression) => match expression.as_ref() {
                Expression::Value(Value::Identifier(identifier)) => {
                    let variable = self.get_variable(identifier)?;

                    Ok(RuntimeValue::Reference(variable))
                }

                _ => Err(RuntimeError::InvalidReferenceTarget),
            },

            Expression::Dereference(expression) => {
                let value = self.evaluate_expression(expression)?;
                value.dereference()
            }

            Expression::MethodCall {
                expression,
                method_identifier,
                arguments,
            } => {
                // later on i plan on implementing custom functions for native types
                // but for now, structs only

                let value = self.evaluate_method_receiver(expression)?;
                value.assert_reference()?;

                // find name of the struct definition
                let struct_identifier = if value.is_reference() {
                    value.dereference()?.data_type()?
                } else {
                    value.data_type()?
                }
                .to_string();

                let args = arguments
                    .iter()
                    .flat_map(|expr| self.evaluate_expression(expr))
                    .collect();

                self.invoke_method(&struct_identifier, method_identifier, value, args)
            }
        }
    }

    fn evaluate_method_receiver(&mut self, expression: &Expression) -> RuntimeResult<RuntimeValue> {
        match expression {
            Expression::Reference(_) => self.evaluate_expression(expression),

            Expression::Value(Value::Identifier(name)) => {
                Ok(RuntimeValue::Reference(self.get_variable(name)?))
            }

            _ => Err(RuntimeError::InvalidReferenceTarget),
        }
    }

    fn evaluate_binary(
        &self,
        operator: BinaryOperator,
        lhs: RuntimeValue,
        rhs: RuntimeValue,
    ) -> RuntimeResult<RuntimeValue> {
        match operator {
            BinaryOperator::Add => lhs.add(rhs),
            BinaryOperator::Subtract => lhs.subtract(rhs),
            BinaryOperator::Multiply => lhs.multiply(rhs),
            BinaryOperator::Divide => lhs.divide(rhs),
            BinaryOperator::EqualTo => lhs.compare_eq(rhs),
            BinaryOperator::NotEqualTo => lhs.compare_neq(rhs),
        }
    }

    fn resolve_value(&self, value: &Value) -> RuntimeResult<RuntimeValue> {
        match value {
            Value::S32(i) => Ok(RuntimeValue::S32(*i)),

            Value::U32(i) => Ok(RuntimeValue::U32(*i)),

            Value::String(string) => Ok(RuntimeValue::String(string.clone())),

            Value::Bool(b) => Ok(RuntimeValue::Bool(*b)),

            Value::Identifier(name) => {
                let var = self.get_variable(name)?;
                Ok(var.borrow().value())
            }
        }
    }
}
