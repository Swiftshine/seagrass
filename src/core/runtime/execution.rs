use std::fs;

use crate::core::{
    lang::{
        self,
        ast::{
            Assignment, AssignmentTarget, Block, ControlStatement, Expression, Program, Return,
            Statement,
        },
    },
    runtime::{
        ControlFlow, Runtime, RuntimeError, RuntimeResult, RuntimeValue, StatementResult,
        value::RuntimeReference,
    },
};

enum IterationMode {
    Value,
    Reference,
}

impl Runtime {
    fn define_from_program(&mut self, program: &Program) -> RuntimeResult<()> {
        // collect struct and function definitions
        for statement in &program.statements {
            match statement {
                Statement::FunctionDefinition(func) => {
                    self.define_function(func)?;
                }
                Statement::StructDefinition(struct_definition) => {
                    self.define_struct(struct_definition)?;
                }
                Statement::StructImpl(struct_impl) => {
                    self.impl_struct(struct_impl)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn define_from_file(&mut self, filename: &str) -> anyhow::Result<()> {
        let path = self.base_dir.join(filename).canonicalize()?;

        if !self.loaded_files.insert(path.clone()) {
            return Ok(());
        }

        // save previous directory
        let previous_base = self.base_dir.clone();

        // switch to imported file's directory
        self.base_dir = path.parent().unwrap().to_owned();

        let file_contents = fs::read_to_string(&path)?;
        let program = lang::build_program(&file_contents)?;
        self.define_from_program(&program)?;

        // restore
        self.base_dir = previous_base;

        Ok(())
    }

    pub fn execute(&mut self, program: &Program) -> RuntimeResult<()> {
        // collect functions for built-in data types
        self.register_builtin_methods();

        // collect sg:: functions
        self.register_native_functions();

        // define things from this program
        self.define_from_program(program)?;

        for import in program
            .statements
            .iter()
            .filter(|s| matches!(s, &Statement::Import(_)))
        {
            if let Statement::Import(filename) = import {
                if let Err(e) = self.define_from_file(filename) {
                    eprintln!("Error importing file: {e}");
                }
            }
        }

        // execute normal statements
        for statement in &program.statements {
            match statement {
                Statement::FunctionDefinition(_)
                | Statement::StructDefinition(_)
                | Statement::StructImpl(_)
                | Statement::Import(_) => {}
                _ => {
                    self.execute_statement(statement)?;
                }
            }
        }

        // call main()
        // todo! handle the case where main() is absent in the executed script but present
        // in imports
        if self.get_function("main").is_ok() {
            self.call_function("main", vec![], &vec![])?;
        }

        Ok(())
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> StatementResult {
        match statement {
            Statement::Assignment(assignment) => self.execute_assignment(assignment),

            Statement::Expression(expression) => {
                self.evaluate_expression_to_value(expression)?;
                Ok(ControlFlow::Continue)
            }

            Statement::Return(ret) => self.execute_return(ret),

            Statement::ControlStatement(control_statement) => {
                self.execute_control_statement(control_statement)
            }

            Statement::Break => Ok(ControlFlow::Break),

            _ => unreachable!("{:?}", statement),
        }
    }

    fn execute_control_statement(
        &mut self,
        control_statement: &ControlStatement,
    ) -> StatementResult {
        match control_statement {
            ControlStatement::If {
                expression,
                block,
                children,
            } => self.execute_if(expression, block, children),

            ControlStatement::While { expression, block } => self.execute_while(expression, block),

            ControlStatement::Loop { block } => self.execute_loop(block),

            ControlStatement::For {
                identifier,
                iterable,
                block,
            } => self.execute_for(identifier, iterable, block),

            _ => unreachable!("{:?}", control_statement),
        }
    }

    fn execute_iteration(
        &mut self,
        iterator_identifier: &String,
        contents: &[RuntimeReference],
        mode: IterationMode,
        block: &Block,
    ) -> StatementResult {
        for item in contents {
            let value = match mode {
                IterationMode::Value => item.borrow().copy_value(),
                IterationMode::Reference => RuntimeValue::Reference(item.clone()),
            };

            self.push_scope();
            self.assign_variable(iterator_identifier.clone(), value);

            let result = self.execute_block_contents(block);

            self.pop_scope();

            match result? {
                ControlFlow::Continue => {}
                ControlFlow::Break => break,
                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
            }
        }

        Ok(ControlFlow::Continue)
    }

    fn execute_for(
        &mut self,
        iterator_identifier: &String,
        iterable: &Expression,
        block: &Block,
    ) -> StatementResult {
        match iterable {
            Expression::Range { start, end } => {
                let start = self.evaluate_expression_to_value(start)?;
                let end = self.evaluate_expression_to_value(end)?;

                let start = match start {
                    RuntimeValue::U32(v) => v as usize,
                    RuntimeValue::S32(v) if v >= 0 => v as usize,

                    other => {
                        return Err(RuntimeError::ExpectedIntegerForRange(
                            other.data_type()?.to_string(),
                        ));
                    }
                };

                let end = match end {
                    RuntimeValue::U32(v) => v as usize,
                    RuntimeValue::S32(v) if v >= 0 => v as usize,

                    other => {
                        return Err(RuntimeError::ExpectedIntegerForRange(
                            other.data_type()?.to_string(),
                        ));
                    }
                };

                for i in start..end {
                    self.push_scope();

                    self.assign_variable(iterator_identifier.clone(), RuntimeValue::Usize(i));

                    let result = self.execute_block_contents(block);

                    self.pop_scope();

                    match result? {
                        ControlFlow::Continue => {}
                        ControlFlow::Break => break,
                        ControlFlow::Return(value) => {
                            return Ok(ControlFlow::Return(value));
                        }
                    }
                }

                Ok(ControlFlow::Continue)
            }

            Expression::ArrayInitialization(init) => {
                for expression in &init.initialized_fields {
                    let value = self.evaluate_expression_to_value(expression)?;

                    self.push_scope();

                    self.assign_variable(iterator_identifier.clone(), value);

                    let result = self.execute_block_contents(block);

                    self.pop_scope();

                    match result? {
                        ControlFlow::Continue => {}
                        ControlFlow::Break => break,
                        ControlFlow::Return(value) => {
                            return Ok(ControlFlow::Return(value));
                        }
                    }
                }

                Ok(ControlFlow::Continue)
            }

            _ => {
                let value = self.evaluate_expression_to_value(iterable)?;

                match value {
                    RuntimeValue::Array { contents, .. } => self.execute_iteration(
                        iterator_identifier,
                        &contents,
                        IterationMode::Value,
                        block,
                    ),

                    RuntimeValue::Iterator { contents, .. } => self.execute_iteration(
                        iterator_identifier,
                        &contents,
                        IterationMode::Reference,
                        block,
                    ),

                    other => Err(RuntimeError::CannotIterateOnType(
                        other.data_type()?.to_string(),
                    )),
                }
            }
        }
    }

    fn execute_loop(&mut self, block: &Block) -> StatementResult {
        loop {
            let flow = self.execute_block(block)?;

            if matches!(flow, ControlFlow::Break) {
                break;
            }
        }

        Ok(ControlFlow::Continue)
    }

    fn execute_while(&mut self, expression: &Expression, block: &Block) -> StatementResult {
        while self.evaluate_boolean_expression(expression)? {
            let flow = self.execute_block(block)?;

            if matches!(flow, ControlFlow::Break) {
                break;
            }
        }

        Ok(ControlFlow::Continue)
    }

    fn execute_if(
        &mut self,
        expression: &Expression,
        block: &Block,
        children: &Vec<ControlStatement>,
    ) -> StatementResult {
        // execute our statement first

        if self.evaluate_boolean_expression(expression)? {
            return self.execute_block(block);
        }

        // check children

        for child in children {
            match child {
                ControlStatement::ElseIf { expression, block } => {
                    if self.evaluate_boolean_expression(expression)? {
                        return self.execute_block(block);
                    }
                }

                ControlStatement::Else { block } => {
                    return self.execute_block(block);
                }

                _ => {
                    unreachable!("{:?}", child);
                }
            }
        }

        // continue anyway
        Ok(ControlFlow::Continue)
    }

    fn execute_assignment(&mut self, assignment: &Assignment) -> StatementResult {
        let mut value = self.evaluate_expression_to_value(&assignment.expression)?;

        if let Some(expected_type) = &assignment.data_type {
            value = self.apply_type_annotation(value, expected_type)?;
        }

        if assignment.declarative {
            match &assignment.target {
                AssignmentTarget::Identifier(ident) => {
                    self.assign_variable(ident.clone(), value);
                }

                _ => unreachable!("invalid declarative assignment target"),
            }
        } else {
            self.assign_to_target(&assignment.target, value)?;
        }

        Ok(ControlFlow::Continue)
    }

    fn execute_return(&mut self, ret: &Return) -> StatementResult {
        let value = self.evaluate_expression_to_value(&ret.expression)?;
        Ok(ControlFlow::Return(value))
    }

    fn execute_block(&mut self, block: &Block) -> StatementResult {
        self.push_scope();
        let result = self.execute_block_contents(block);
        self.pop_scope();
        result
    }

    fn execute_block_contents(&mut self, block: &Block) -> StatementResult {
        for statement in &block.statements {
            let flow = self.execute_statement(statement)?;

            match flow {
                ControlFlow::Continue => {}
                ControlFlow::Return(_) | ControlFlow::Break => return Ok(flow),
            }
        }

        Ok(ControlFlow::Continue)
    }
}
