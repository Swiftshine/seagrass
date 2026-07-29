use crate::core::{
    lang::ast::{AssignmentTarget, BinaryOperator, DataType, Expression, Value},
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
            (val, ty) if val.data_type()?.can_be_coerced_into(ty) => self.coerce(val, ty),

            _ => Err(RuntimeError::AnnotationError {
                expected: expected.to_string(),
                found: value_data_type_string,
            }),
        }
    }

    // pub fn evaluate_lvalue(&mut self, target: &AssignmentTarget) -> RuntimeResult<LValue> {
    //     match target {
    //         AssignmentTarget::Identifier(identifier) => {
    //             Ok(LValue::Variable(self.get_variable(identifier)?))
    //         }

    //         AssignmentTarget::Dereference(target) => {
    //             let value = self.evaluate_lvalue(target)?.read()?;

    //             match value {
    //                 RuntimeValue::Reference(reference) => Ok(LValue::Variable(reference)),

    //                 other => Err(RuntimeError::ExpectedReference(
    //                     other.data_type()?.to_string(),
    //                 )),
    //             }
    //         }

    //         AssignmentTarget::ArrayAccess {
    //             target,
    //             index_expression,
    //         } => {
    //             let array = Box::new(self.evaluate_lvalue(target)?);

    //             let index = match self.evaluate_expression(index_expression)? {
    //                 RuntimeValue::U32(value) => value as usize,

    //                 RuntimeValue::S32(value) if value >= 0 => value as usize,

    //                 other => {
    //                     return Err(RuntimeError::InvalidArrayIndex(
    //                         other.data_type()?.to_string(),
    //                     ));
    //                 }
    //             };

    //             let value = array.read()?;

    //             match value {
    //                 RuntimeValue::Array { contents, .. } => {
    //                     if index >= contents.len() {
    //                         return Err(RuntimeError::ArrayIndexOutOfBounds {
    //                             index,
    //                             length: contents.len(),
    //                         });
    //                     }
    //                 }

    //                 other => {
    //                     return Err(RuntimeError::CannotIndexNonArrayType(
    //                         other.data_type()?.to_string(),
    //                     ));
    //                 }
    //             }

    //             Ok(LValue::ArrayElement { array, index })
    //         }

    //         AssignmentTarget::FieldAccess {
    //             target,
    //             field_identifier,
    //         } => {
    //             let object = Box::new(self.evaluate_lvalue(target)?);

    //             let value = object.read()?;

    //             match value {
    //                 RuntimeValue::Struct { definition, fields } => {
    //                     if !fields.contains_key(field_identifier) {
    //                         return Err(RuntimeError::InvalidStructFieldAccess {
    //                             field_name: field_identifier.clone(),
    //                             struct_name: definition.identifier.clone(),
    //                         });
    //                     }
    //                 }

    //                 other => {
    //                     return Err(RuntimeError::InvalidStructFieldAccessTarget {
    //                         field: field_identifier.clone(),
    //                         data_type: other.data_type()?.to_string(),
    //                     });
    //                 }
    //             }

    //             Ok(LValue::StructField {
    //                 object,
    //                 field: field_identifier.clone(),
    //             })
    //         }
    //     }
    // }

    pub fn evaluate_expression(&mut self, expression: &Expression) -> RuntimeResult<RuntimeValue> {
        match expression {
            Expression::Value(value) => self.resolve_ast_value(value),
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
                todo!("fix struct field access")

                // let value = self.evaluate_expression(expression)?.resolve();

                // match value {
                //     RuntimeValue::Struct { definition, fields } => fields
                //         .get(field)
                //         .cloned()
                //         .ok_or(RuntimeError::InvalidStructFieldAccess {
                //             field_name: field.clone(),
                //             struct_name: definition.identifier.clone(),
                //         }),

                //     _ => Err(RuntimeError::InvalidStructFieldAccessTarget {
                //         field: field.clone(),
                //         data_type: value.data_type()?.to_string(),
                //     }),
                // }
            }

            Expression::ArrayAccess {
                expression,
                index_expression,
            } => {
                todo!("fix array access")

                // let index = match self.evaluate_expression(index_expression)? {
                //     RuntimeValue::U32(i) => i as usize,
                //     RuntimeValue::S32(i) if i >= 0 => i as usize,
                //     value => {
                //         return Err(RuntimeError::InvalidArrayIndex(
                //             value.data_type()?.to_string(),
                //         ));
                //     }
                // };

                // let value = self.evaluate_expression(expression)?.resolve();

                // match value {
                //     RuntimeValue::Array { contents, .. } => {
                //         contents
                //             .get(index)
                //             .cloned()
                //             .ok_or(RuntimeError::ArrayIndexOutOfBounds {
                //                 index,
                //                 length: contents.len(),
                //             })
                //     }

                //     _ => Err(RuntimeError::CannotIndexNonArrayType(
                //         value.data_type()?.to_string(),
                //     )),
                // }
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
                let value = self.evaluate_method_receiver(expression)?;
                value.assert_reference()?;

                let data_type = if value.is_reference() {
                    value.dereference()?.data_type()?
                } else {
                    value.data_type()?
                };

                let data_type_identifier = data_type.to_string();
                match data_type {
                    DataType::Array { .. } => {
                        let args = arguments
                            .iter()
                            .flat_map(|expr| self.evaluate_expression(expr))
                            .collect();

                        self.invoke_method_for_array(
                            data_type.clone(),
                            method_identifier,
                            value,
                            args,
                        )
                    }
                    DataType::UserDefined(_) => {
                        let args = arguments
                            .iter()
                            .flat_map(|expr| self.evaluate_expression(expr))
                            .collect();

                        self.invoke_method_for_struct(
                            &data_type_identifier,
                            method_identifier,
                            value,
                            args,
                        )
                    }
                    _ => Err(RuntimeError::CannotInvokeMethodOnType(data_type_identifier)),
                }
            }

            _ => unreachable!("{:?}", expression),
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
            // arithmetic
            BinaryOperator::Add => lhs.add(rhs),
            BinaryOperator::Subtract => lhs.subtract(rhs),
            BinaryOperator::Multiply => lhs.multiply(rhs),
            BinaryOperator::Divide => lhs.divide(rhs),
            BinaryOperator::Modulo => lhs.modulo(rhs),

            // shifts
            BinaryOperator::ShiftLeft => lhs.shift_left(rhs),
            BinaryOperator::ShiftRight => lhs.shift_right(rhs),

            // comparisons
            BinaryOperator::LessThan => lhs.compare_lt(rhs),
            BinaryOperator::LessThanOrEqualTo => lhs.compare_lte(rhs),
            BinaryOperator::GreaterThan => lhs.compare_gt(rhs),
            BinaryOperator::GreaterThanOrEqualTo => lhs.compare_gte(rhs),
            BinaryOperator::EqualTo => lhs.compare_eq(rhs),
            BinaryOperator::NotEqualTo => lhs.compare_neq(rhs),

            // bitwise
            BinaryOperator::BitwiseAnd => lhs.bitwise_and(rhs),
            BinaryOperator::BitwiseOr => lhs.bitwise_or(rhs),
            BinaryOperator::BitwiseXor => lhs.bitwise_xor(rhs),

            // logical
            BinaryOperator::LogicalAnd => lhs.logical_and(rhs),
            BinaryOperator::LogicalOr => lhs.logical_or(rhs),
        }
    }

    fn resolve_ast_value(&self, value: &Value) -> RuntimeResult<RuntimeValue> {
        match value {
            Value::S32(i) => Ok(RuntimeValue::S32(*i)),

            Value::U32(i) => Ok(RuntimeValue::U32(*i)),

            Value::String(string) => Ok(RuntimeValue::String(string.clone())),

            Value::Bool(b) => Ok(RuntimeValue::Bool(*b)),

            Value::Identifier(name) => {
                let var = self.get_variable(name)?;
                Ok(var.borrow().copy_value())
            }
        }
    }
}
