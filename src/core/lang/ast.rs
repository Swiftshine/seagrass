mod data_types;

use data_types::*;

use crate::core::lang::parser::Rule;
use anyhow::Result;
use pest::iterators::Pair;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/* Statements */

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assignment(Assignment),
    Expression(Expression),
    FunctionDefinition(FunctionDefinition),
    Return(Return),
    Break,
    StructDefinition(StructDefinition),
    ControlStatement(ControlStatement),
    StructImpl(StructImpl),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructImpl {
    pub struct_identifier: String,
    pub function_definitions: Vec<FunctionDefinition>,
    pub method_definitions: Vec<MethodDefinition>,
}

impl StructImpl {
    pub fn has_method(&self, identifier: &str) -> bool {
        self.method_definitions
            .iter()
            .any(|m| m.identifier == identifier)
    }
}

// these two structs are identical but they're different types
// for the sake of clarity

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDefinition {
    pub attributes: Vec<Attribute>,
    pub identifier: String,
    pub body: Block,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinition {
    pub attributes: Vec<Attribute>,
    pub identifier: String,
    pub body: Block,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Value(Value),
    FunctionCall(FunctionCall),
    Binary {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        operator: BinaryOperator,
    },
    StructInitialization(StructInitialization),
    ArrayInitialization(ArrayInitialization),
    StructFieldAccess {
        expression: Box<Expression>,
        field_identifier: String,
    },
    Reference(Box<Expression>),
    Dereference(Box<Expression>),
    MethodCall {
        expression: Box<Expression>,
        method_identifier: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOperator {
    // arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,

    // comparison
    EqualTo,
    NotEqualTo,
    // todo: bitwise
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructInitialization {
    pub identifier: String,
    pub initialized_fields: Vec<StructFieldInitialization>,
    pub use_defaults: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructFieldInitialization {
    pub identifier: String,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayInitialization {
    pub initialized_fields: Vec<Expression>,
    pub use_defaults: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub declarative: bool,
    pub target: AssignmentTarget,
    pub data_type: Option<DataType>,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentTarget {
    Identifier(String),
    Dereference(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub identifier: String,
    pub arguments: Vec<Expression>,
    pub generics: Vec<DataType>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    S32(i32),
    U32(u32),
    String(String),
    Bool(bool),
    Identifier(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub identifier: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    U32,
    S32,
    String,
    Bool,
    Reference(Box<DataType>),
    UserDefined(String),
    Array {
        data_type: Box<DataType>,
        count: usize,
    },
}

impl DataType {
    pub fn to_string(&self) -> String {
        match self {
            Self::U32 => "u32".to_string(),
            Self::S32 => "s32".to_string(),
            Self::String => "string".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Reference(data_type) => format!("&{}", data_type.to_string()),
            Self::UserDefined(user_defined_type) => user_defined_type.clone(),
            Self::Array { data_type, count } => format!("[{}; {}]", data_type.to_string(), count),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDefinition {
    pub attributes: Vec<Attribute>,
    pub identifier: String,
    // using vec here because the field order matters
    pub members: Vec<StructMember>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructMember {
    Field(StructFieldDefinition),
    Padding(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructFieldDefinition {
    pub attributes: Vec<Attribute>,
    pub identifier: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlStatement {
    If {
        expression: Expression,
        block: Block,
        children: Vec<ControlStatement>,
    },
    ElseIf {
        expression: Expression,
        block: Block,
    },
    Else {
        block: Block,
    },
    While {
        expression: Expression,
        block: Block,
    },
    Loop {
        block: Block,
    },
}

impl ControlStatement {
    fn build_if(expression: Expression, block: Block, children: Vec<ControlStatement>) -> Self {
        Self::If {
            expression,
            block,
            children,
        }
    }

    fn build_else_if(expression: Expression, block: Block) -> Self {
        Self::ElseIf { expression, block }
    }

    fn build_else(block: Block) -> Self {
        Self::Else { block }
    }

    fn build_while(expression: Expression, block: Block) -> Self {
        Self::While { expression, block }
    }

    fn build_loop(block: Block) -> Self {
        Self::Loop { block }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub identifier: String,
    pub arguments: Vec<Expression>,
}

pub fn dump(pair: Pair<Rule>, indent: usize) {
    println!(
        "{}{:?}: {:?}",
        "  ".repeat(indent),
        pair.as_rule(),
        pair.as_str()
    );

    for child in pair.into_inner() {
        dump(child, indent + 1);
    }
}

pub fn build_program(pair: Pair<Rule>) -> Result<Program> {
    assert_eq!(pair.as_rule(), Rule::Program);

    let statements = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::Statement)
        .flat_map(build_statement)
        .collect();

    Ok(Program { statements })
}

fn build_statement(pair: Pair<Rule>) -> Result<Statement> {
    assert_eq!(pair.as_rule(), Rule::Statement);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Assignment => Ok(Statement::Assignment(build_assignment(inner)?)),
        Rule::Expression => Ok(Statement::Expression(build_expression(inner)?)),
        Rule::FunctionDefinition => Ok(Statement::FunctionDefinition(build_function_definition(
            inner,
        )?)),
        Rule::Return => Ok(Statement::Return(build_return(inner)?)),
        Rule::StructDefinition => Ok(Statement::StructDefinition(build_struct_definition(inner)?)),
        Rule::IfStatement => Ok(Statement::ControlStatement(build_if_statement(inner)?)),
        Rule::WhileStatement => Ok(Statement::ControlStatement(build_while_statement(inner)?)),
        Rule::LoopStatement => Ok(Statement::ControlStatement(build_loop_statement(inner)?)),
        Rule::BreakStatement => Ok(Statement::Break),
        Rule::StructImplBlock => Ok(Statement::StructImpl(build_struct_impl(inner)?)),
        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_struct_impl(pair: Pair<Rule>) -> Result<StructImpl> {
    assert_eq!(pair.as_rule(), Rule::StructImplBlock);

    let mut inner = pair.into_inner();

    // KeywordImpl
    inner.next();

    // Identifier
    let struct_identifier = inner.next().unwrap().to_string();

    let mut function_definitions = Vec::new();
    let mut method_definitions = Vec::new();

    // StructImplDefinitions?
    if let Some(pair) = inner.next() {
        assert_eq!(pair.as_rule(), Rule::StructImplDefinitions);

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::FunctionDefinition => {
                    function_definitions.push(build_function_definition(pair)?);
                }
                Rule::MethodDefinition => {
                    method_definitions.push(build_method_definition(pair)?);
                }
                _ => unreachable!("{:?}", pair.as_rule()),
            }
        }
    }

    Ok(StructImpl {
        struct_identifier,
        function_definitions,
        method_definitions,
    })
}

fn build_loop_statement(pair: Pair<Rule>) -> Result<ControlStatement> {
    assert_eq!(pair.as_rule(), Rule::LoopStatement);

    let mut inner = pair.into_inner();

    // KeywordLoop
    inner.next();

    // Block
    let block = build_block(inner.next().unwrap())?;

    Ok(ControlStatement::build_loop(block))
}

fn build_while_statement(pair: Pair<Rule>) -> Result<ControlStatement> {
    assert_eq!(pair.as_rule(), Rule::WhileStatement);

    let mut inner = pair.into_inner();

    // KeywordWhile
    inner.next();

    // Comparison
    let expression = build_comparison(inner.next().unwrap())?;

    // Block
    let block = build_block(inner.next().unwrap())?;

    Ok(ControlStatement::build_while(expression, block))
}

fn build_if_statement(pair: Pair<Rule>) -> Result<ControlStatement> {
    assert_eq!(pair.as_rule(), Rule::IfStatement);

    let mut inner = pair.into_inner();

    // KeywordIf
    inner.next();

    // Comparison
    let expression = build_comparison(inner.next().unwrap())?;

    // Block
    let block = build_block(inner.next().unwrap())?;

    let mut children = Vec::new();

    for statement in inner {
        match statement.as_rule() {
            Rule::ElseIfStatement => {
                children.push(build_else_if_statement(statement)?);
            }

            Rule::ElseStatement => {
                // this should be the end; break
                children.push(build_else_statement(statement)?);
                break;
            }

            _ => unreachable!("{:?}", statement.as_rule()),
        }
    }

    Ok(ControlStatement::build_if(expression, block, children))
}

fn build_else_if_statement(pair: Pair<Rule>) -> Result<ControlStatement> {
    assert_eq!(pair.as_rule(), Rule::ElseIfStatement);

    let mut inner = pair.into_inner();

    // KeywordElse
    inner.next();

    // KeywordIf
    inner.next();

    // Comparison
    let expression = build_comparison(inner.next().unwrap())?;

    // Block
    let block = build_block(inner.next().unwrap())?;

    Ok(ControlStatement::build_else_if(expression, block))
}

fn build_else_statement(pair: Pair<Rule>) -> Result<ControlStatement> {
    assert_eq!(pair.as_rule(), Rule::ElseStatement);

    let mut inner = pair.into_inner();

    // KeywordElse
    inner.next();

    // Block
    let block = build_block(inner.next().unwrap())?;

    Ok(ControlStatement::build_else(block))
}

fn build_assignment(pair: Pair<Rule>) -> Result<Assignment> {
    assert_eq!(pair.as_rule(), Rule::Assignment);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::DeclarativeAssignment => build_declarative_assignment(inner),
        Rule::Reassignment => build_reassignment(inner),
        _ => unreachable!("{:?}", inner),
    }
}

fn build_type_annotation(pair: Pair<Rule>) -> Result<DataType> {
    assert_eq!(pair.as_rule(), Rule::TypeAnnotation);
    build_data_type(pair.into_inner().next().unwrap())
}

fn build_declarative_assignment(pair: Pair<Rule>) -> Result<Assignment> {
    assert_eq!(pair.as_rule(), Rule::DeclarativeAssignment);

    let mut inner = pair.into_inner();

    // KeywordLet
    inner.next();

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    // TypeAnnotation?
    let data_type = match inner.peek().map(|p| p.as_rule()) {
        Some(Rule::TypeAnnotation) => Some(build_type_annotation(inner.next().unwrap())?),
        _ => None,
    };

    // Expression
    let expression = build_expression(inner.next().unwrap())?;

    Ok(Assignment {
        declarative: true,
        target: AssignmentTarget::Identifier(identifier),
        data_type,
        expression,
    })
}

fn build_reassignment(pair: Pair<Rule>) -> Result<Assignment> {
    assert_eq!(pair.as_rule(), Rule::Reassignment);

    let mut inner = pair.into_inner();

    let target = build_assignment_target(inner.next().unwrap())?;

    // Expression
    let expression = build_expression(inner.next().unwrap())?;

    Ok(Assignment {
        declarative: false,
        target,
        data_type: None,
        expression,
    })
}

fn build_assignment_target(pair: Pair<Rule>) -> Result<AssignmentTarget> {
    assert_eq!(pair.as_rule(), Rule::AssignmentTarget);

    let first = pair.into_inner().next().unwrap();

    match first.as_rule() {
        Rule::Identifier => Ok(AssignmentTarget::Identifier(first.as_str().to_string())),

        Rule::Dereference => Ok(AssignmentTarget::Dereference(build_dereference_target(
            first,
        )?)),

        _ => unreachable!("{:?}", first.as_rule()),
    }
}

fn build_dereference_target(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Dereference);

    let unary = pair.into_inner().next().unwrap();
    build_unary(unary)
}

fn build_dereference(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Dereference);

    let mut inner = pair.into_inner();

    let expression = build_unary(inner.next().unwrap())?;

    Ok(Expression::Dereference(Box::new(expression)))
}

fn build_expression(pair: Pair<Rule>) -> Result<Expression> {
    build_comparison(pair.into_inner().next().unwrap())
}

fn build_comparison(pair: Pair<Rule>) -> Result<Expression> {
    let mut inner = pair.into_inner();

    let mut expr = build_addition(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let rhs = build_addition(
            inner
                .next()
                .expect("operator must have a right-hand operand"),
        )?;

        let operator = match op.as_rule() {
            Rule::EqualTo => BinaryOperator::EqualTo,
            Rule::NotEqualTo => BinaryOperator::NotEqualTo,

            _ => unreachable!("{:?}", op.as_rule()),
        };

        expr = Expression::Binary {
            lhs: Box::new(expr),
            operator,
            rhs: Box::new(rhs),
        };
    }

    Ok(expr)
}

fn build_addition(pair: Pair<Rule>) -> Result<Expression> {
    let mut inner = pair.into_inner();

    let mut expr = build_multiplication(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let rhs = build_multiplication(
            inner
                .next()
                .expect("operator must have a right-hand operand"),
        )?;

        let operator = match op.as_rule() {
            Rule::Plus => BinaryOperator::Add,
            Rule::Minus => BinaryOperator::Subtract,
            _ => unreachable!("{:?}", op.as_rule()),
        };

        expr = Expression::Binary {
            lhs: Box::new(expr),
            operator,
            rhs: Box::new(rhs),
        };
    }

    Ok(expr)
}

fn build_multiplication(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Multiplication);

    let mut inner = pair.into_inner();

    let mut expr = build_unary(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let rhs = inner
            .next()
            .expect("multiplication operator must have a right-hand operand");

        let operator = match op.as_rule() {
            Rule::Multiply => BinaryOperator::Multiply,
            Rule::Slash => BinaryOperator::Divide,
            _ => unreachable!("{:?}", op.as_rule()),
        };

        expr = Expression::Binary {
            lhs: Box::new(expr),
            operator,
            rhs: Box::new(build_unary(rhs)?),
        };
    }

    Ok(expr)
}

fn build_unary(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Unary);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Reference => {
            let expression = build_unary(inner.into_inner().next().unwrap())?;
            Ok(Expression::Reference(Box::new(expression)))
        }

        Rule::Dereference => build_dereference(inner),

        Rule::Postfix => build_postfix(inner),

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_postfix(pair: Pair<Rule>) -> Result<Expression> {
    let mut inner = pair.into_inner();

    let mut expr = build_atom(inner.next().unwrap())?;

    for pair in inner {
        let rule = pair.as_rule();
        let mut inner = pair.into_inner();

        match rule {
            Rule::FieldAccess => {
                let field = inner.next().unwrap().to_string();

                expr = Expression::StructFieldAccess {
                    expression: Box::new(expr),
                    field_identifier: field,
                };
            }

            Rule::MethodCall => {
                let method_identifier = inner.next().unwrap().to_string();

                // ArgumentList?
                let arguments = match inner.peek().map(|p| p.as_rule()) {
                    Some(Rule::ArgumentList) => build_argument_list(inner.next().unwrap())?,
                    _ => Vec::new(),
                };

                expr = Expression::MethodCall {
                    expression: Box::new(expr),
                    method_identifier,
                    arguments,
                }
            }
            _ => unreachable!("{:?}", rule),
        }
    }

    Ok(expr)
}

fn build_atom(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Atom);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Value => Ok(Expression::Value(build_value(inner)?)),
        Rule::FunctionCall => Ok(Expression::FunctionCall(build_function_call(inner)?)),
        Rule::Expression => build_expression(inner), // parenthesised
        Rule::StructInitialization => Ok(Expression::StructInitialization(
            build_struct_initialization(inner)?,
        )),
        Rule::ArrayInitialization => Ok(Expression::ArrayInitialization(
            build_array_initialization(inner)?,
        )),

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_array_initialization(pair: Pair<Rule>) -> Result<ArrayInitialization> {
    assert_eq!(pair.as_rule(), Rule::ArrayInitialization);

    let mut inner = pair.into_inner();
    let mut initialized_fields = Vec::new();
    let mut use_defaults = false;

    while let Some(pair) = inner.next() {
        match pair.as_rule() {
            Rule::ArrayInitializers => {
                initialized_fields = build_array_initializers(pair)?;
            }

            Rule::DotDot => {
                use_defaults = true;
            }

            _ => unreachable!("{:?}", pair.as_rule()),
        }
    }

    Ok(ArrayInitialization {
        initialized_fields,
        use_defaults,
    })
}

fn build_array_initializers(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    assert_eq!(pair.as_rule(), Rule::ArrayInitializers);

    let expressions = pair
        .into_inner()
        .flat_map(|pair| build_expression(pair))
        .collect();

    Ok(expressions)
}

fn build_struct_initialization(pair: Pair<Rule>) -> Result<StructInitialization> {
    assert_eq!(pair.as_rule(), Rule::StructInitialization);

    let mut inner = pair.into_inner();

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    let mut initialized_fields = Vec::new();
    let mut use_defaults = false;

    while let Some(pair) = inner.next() {
        match pair.as_rule() {
            // StructFieldInitializers?
            Rule::StructFieldInitializers => {
                initialized_fields = build_struct_field_initializers(pair)?;
            }

            Rule::DotDot => {
                use_defaults = true;
                break;
            }

            _ => unreachable!(),
        }
    }

    Ok(StructInitialization {
        identifier,
        initialized_fields,
        use_defaults,
    })
}

fn build_struct_field_initializers(pair: Pair<Rule>) -> Result<Vec<StructFieldInitialization>> {
    assert_eq!(pair.as_rule(), Rule::StructFieldInitializers);

    pair.into_inner()
        .map(build_struct_field_initializer)
        .collect()
}

fn build_struct_field_initializer(pair: Pair<Rule>) -> Result<StructFieldInitialization> {
    assert_eq!(pair.as_rule(), Rule::StructFieldInitializer);

    let mut inner = pair.into_inner();

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    // // DataType
    // let data_type = build_data_type(inner.next().unwrap())?;

    // Expression
    let expression = build_expression(inner.next().unwrap())?;

    Ok(StructFieldInitialization {
        identifier,
        expression,
    })
}

fn build_struct_definition(pair: Pair<Rule>) -> Result<StructDefinition> {
    assert_eq!(pair.as_rule(), Rule::StructDefinition);

    let mut inner = pair.into_inner();

    // Attributes
    let attributes = collect_attributes(&mut inner)?;

    // KeywordStruct
    inner.next();

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    // StructMembers?
    let struct_members = match inner.next() {
        Some(pair) => build_struct_members(pair)?,
        None => Vec::new(),
    };

    Ok(StructDefinition {
        attributes,
        identifier,
        members: struct_members,
    })
}

fn build_struct_member(pair: Pair<Rule>) -> Result<StructMember> {
    assert_eq!(pair.as_rule(), Rule::StructMember);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::StructFieldDefinition => {
            Ok(StructMember::Field(build_struct_field_definition(inner)?))
        }

        Rule::PadDirective => Ok(StructMember::Padding(build_padding(inner)?)),

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_padding(pair: Pair<Rule>) -> Result<usize> {
    assert_eq!(pair.as_rule(), Rule::PadDirective);

    let mut inner = pair.into_inner();

    // KeywordPad
    inner.next();

    // Integer
    let amount = inner.next().unwrap().as_str().parse::<usize>()?;

    Ok(amount)
}

fn build_struct_members(pair: Pair<Rule>) -> Result<Vec<StructMember>> {
    assert_eq!(pair.as_rule(), Rule::StructMembers);

    pair.into_inner().map(build_struct_member).collect()
}

fn build_struct_field_definition(pair: Pair<Rule>) -> Result<StructFieldDefinition> {
    assert_eq!(pair.as_rule(), Rule::StructFieldDefinition);

    let mut inner = pair.into_inner();

    // Attributes
    let attributes = collect_attributes(&mut inner)?;

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    // DataType
    let data_type = build_data_type(inner.next().unwrap())?;

    Ok(StructFieldDefinition {
        attributes,
        identifier,
        data_type,
    })
}

fn build_attribute(pair: Pair<Rule>) -> Result<Attribute> {
    assert_eq!(pair.as_rule(), Rule::Attribute);

    let mut inner = pair.into_inner();

    // Identifier
    let identifier = inner.next().unwrap().as_str().to_string();

    let arguments = match inner.next() {
        Some(pair) if pair.as_rule() == Rule::AttributeArguments => {
            build_attribute_arguments(pair)?
        }
        None => Vec::new(),
        _ => unreachable!(),
    };

    Ok(Attribute {
        identifier,
        arguments,
    })
}

fn build_attribute_arguments(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    assert_eq!(pair.as_rule(), Rule::AttributeArguments);

    let mut inner = pair.into_inner();

    match inner.next() {
        Some(pair) => build_argument_list(pair),
        None => Ok(Vec::new()),
    }
}

fn collect_attributes(inner: &mut pest::iterators::Pairs<Rule>) -> Result<Vec<Attribute>> {
    let mut attributes = Vec::new();

    while let Some(pair) = inner.peek() {
        if pair.as_rule() == Rule::Attribute {
            attributes.push(build_attribute(inner.next().unwrap())?);
        } else {
            break;
        }
    }

    Ok(attributes)
}

fn build_function_definition(pair: Pair<Rule>) -> Result<FunctionDefinition> {
    assert_eq!(pair.as_rule(), Rule::FunctionDefinition);

    let mut inner = pair.into_inner();

    // Attributes
    let attributes = collect_attributes(&mut inner)?;

    // KeywordFn
    inner.next();

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    // ParameterList?
    let parameters = match inner.peek().map(|p| p.as_rule()) {
        Some(Rule::ParameterList) => build_parameter_list(inner.next().unwrap())?,
        _ => Vec::new(),
    };

    // Block
    let body = build_block(inner.next().unwrap())?;

    Ok(FunctionDefinition {
        attributes,
        identifier,
        body,
        parameters,
    })
}

fn build_method_definition(pair: Pair<Rule>) -> Result<MethodDefinition> {
    assert_eq!(pair.as_rule(), Rule::MethodDefinition);

    let mut inner = pair.into_inner();

    // Attributes
    let attributes = collect_attributes(&mut inner)?;

    // KeywordFn
    inner.next();

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    // KeywordSelf
    inner.next();

    // ParameterList?
    let parameters = match inner.peek().map(|p| p.as_rule()) {
        Some(Rule::ParameterList) => build_parameter_list(inner.next().unwrap())?,
        _ => Vec::new(),
    };

    // Block
    let body = build_block(inner.next().unwrap())?;

    Ok(MethodDefinition {
        attributes,
        identifier,
        body,
        parameters,
    })
}

fn build_parameter_list(pair: Pair<Rule>) -> Result<Vec<Parameter>> {
    assert_eq!(pair.as_rule(), Rule::ParameterList);

    let parameters: Vec<Parameter> = pair.into_inner().flat_map(build_parameter).collect();

    Ok(parameters)
}

fn build_parameter(pair: Pair<Rule>) -> Result<Parameter> {
    assert_eq!(pair.as_rule(), Rule::Parameter);

    let mut inner = pair.into_inner();

    // Identifier
    let identifier = inner.next().unwrap().to_string();

    // DataType
    let data_type = build_data_type(inner.next().unwrap())?;

    Ok(Parameter {
        identifier,
        data_type,
    })
}

fn build_block(pair: Pair<Rule>) -> Result<Block> {
    let statements = pair
        .into_inner()
        .filter_map(|p| match p.as_rule() {
            Rule::Statement => Some(build_statement(p)),
            _ => None,
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Block { statements })
}

fn build_return(pair: Pair<Rule>) -> Result<Return> {
    assert_eq!(pair.as_rule(), Rule::Return);

    let mut inner = pair.into_inner();

    // KeywordReturn
    inner.next();

    // Expression
    let expression = build_expression(inner.next().unwrap())?;

    Ok(Return { expression })
}

fn build_function_call(pair: Pair<Rule>) -> Result<FunctionCall> {
    assert_eq!(pair.as_rule(), Rule::FunctionCall);

    let mut inner = pair.into_inner();

    // QualifiedIdentifier
    let identifier = inner.next().unwrap().to_string();

    // GenericArguments?
    let generics = collect_generics(&mut inner)?;

    // ArgumentList?
    let arguments = match inner.next() {
        Some(pair) if pair.as_rule() == Rule::ArgumentList => build_argument_list(pair)?,

        // empty args
        _ => Vec::new(),
    };

    Ok(FunctionCall {
        identifier,
        arguments,
        generics,
    })
}

fn collect_generics(inner: &mut pest::iterators::Pairs<Rule>) -> Result<Vec<DataType>> {
    let mut generics = Vec::new();

    if let Some(pair) = inner.peek() {
        if pair.as_rule() == Rule::GenericArguments {
            let pair = inner.next().unwrap(); // consume it

            for pair in pair.into_inner() {
                generics.push(build_data_type(pair)?);
            }
        }
    }

    Ok(generics)
}

fn build_argument_list(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    assert_eq!(pair.as_rule(), Rule::ArgumentList);

    pair.into_inner().map(build_expression).collect()
}
