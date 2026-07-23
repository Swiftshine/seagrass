use crate::core::{lang::ast::{Assignment, AssignmentTarget, Block, ControlStatement, Expression, Program, Return, Statement}, runtime::{ControlFlow, Runtime, RuntimeResult, StatementResult}};

impl Runtime {
    pub fn execute(&mut self, program: &Program) -> RuntimeResult<()> {
        // collect sg:: functions
        self.register_native_functions();

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

        // execute normal statements
        for statement in &program.statements {
            match statement {
                Statement::FunctionDefinition(_)
                | Statement::StructDefinition(_)
                | Statement::StructImpl(_) => {}
                _ => {
                    self.execute_statement(statement)?;
                }
            }
        }

        // call main()
        if self.get_function("main").is_ok() {
            self.call_function("main", vec![])?;
        }

        Ok(())
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> StatementResult {
        match statement {
            Statement::Assignment(assignment) => self.execute_assignment(assignment),

            Statement::Expression(expression) => {
                self.evaluate_expression(expression)?;
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

            _ => unreachable!("{:?}", control_statement),
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
        let mut value = self.evaluate_expression(&assignment.expression)?;

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
        let value = self.evaluate_expression(&ret.expression)?;
        Ok(ControlFlow::Return(value))
    }

    fn execute_block(&mut self, block: &Block) -> StatementResult {
        self.push_scope();

        let result = (|| {
            for statement in &block.statements {
                let flow = self.execute_statement(statement)?;

                match flow {
                    ControlFlow::Continue => {}
                    ControlFlow::Return(_) | ControlFlow::Break => return Ok(flow),
                }
            }

            Ok(ControlFlow::Continue)
        })();

        self.pop_scope();

        result
    }
}
