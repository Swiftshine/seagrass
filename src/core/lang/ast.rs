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
    Import(String),
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
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    StructInitialization(StructInitialization),
    ArrayInitialization(ArrayInitialization),
    StructFieldAccess {
        expression: Box<Expression>,
        field_identifier: String,
    },
    ArrayAccess {
        expression: Box<Expression>,
        index_expression: Box<Expression>,
    },
    MethodCall {
        expression: Box<Expression>,
        method_identifier: String,
        arguments: Vec<Expression>,
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },
    TypeCast {
        expression: Box<Expression>,
        target_type: DataType,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOperator {
    // arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // shifts
    ShiftLeft,
    ShiftRight,

    // comparison
    LessThan,
    LessThanOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
    EqualTo,
    NotEqualTo,

    // bitwise
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,

    // logical
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOperator {
    Reference,   // &expr
    Dereference, // *expr
    Negate,      // -expr
    LogicalNot,  // !expr
    BitwiseNot,  // ~expr
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
    Dereference(Box<AssignmentTarget>),
    FieldAccess {
        target: Box<AssignmentTarget>,
        field_identifier: String,
    },
    ArrayAccess {
        target: Box<AssignmentTarget>,
        index_expression: Expression,
    },
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
    S8(i8),
    U8(u8),
    S16(i16),
    U16(u16),
    S32(i32),
    U32(u32),
    Usize(usize),
    F32(f32),
    F64(f64),
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
    S8,
    U8,
    S16,
    U16,
    S32,
    U32,
    Usize,
    F32,
    F64,
    String,
    Bool,
    Reference(Box<DataType>),
    Iterator(Box<DataType>),
    UserDefined(String),
    Array {
        inner_data_type: Box<DataType>,
        count: Option<usize>,
    },
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S8 => write!(f, "s8"),
            Self::U8 => write!(f, "u8"),
            Self::S16 => write!(f, "s16"),
            Self::U16 => write!(f, "u16"),
            Self::S32 => write!(f, "s32"),
            Self::U32 => write!(f, "u32"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::String => write!(f, "string"),
            Self::Bool => write!(f, "bool"),
            Self::Usize => write!(f, "usize"),
            Self::Reference(data_type) => write!(f, "&{}", data_type),
            Self::UserDefined(user_defined_type) => write!(f, "{user_defined_type}"),
            Self::Array {
                inner_data_type: data_type,
                count,
            } => {
                if let Some(count) = count {
                    write!(f, "[{};{}]", data_type, count)
                } else {
                    unreachable!("[{}] should only exist when serializing", data_type)
                }
            }
            Self::Iterator(data_type) => write!(f, "iterator<{}>", data_type),
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
    For {
        identifier: String,
        iterable: Expression,
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

    fn build_for(identifier: String, iterable: Expression, block: Block) -> Self {
        Self::For {
            identifier,
            iterable,
            block,
        }
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
        Rule::ForStatement => Ok(Statement::ControlStatement(build_for_statement(inner)?)),
        Rule::BreakStatement => Ok(Statement::Break),
        Rule::StructImplBlock => Ok(Statement::StructImpl(build_struct_impl(inner)?)),
        Rule::ImportStatement => Ok(Statement::Import(build_import(inner)?)),
        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_import(pair: Pair<Rule>) -> Result<String> {
    assert_eq!(pair.as_rule(), Rule::ImportStatement);

    let mut inner = pair.into_inner();

    // KeywordImport
    inner.next();

    // String
    Ok(inner
        .next()
        .unwrap()
        .as_str()
        .strip_prefix("\"")
        .unwrap()
        .strip_suffix("\"")
        .unwrap()
        .to_string())
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

fn build_for_statement(pair: Pair<Rule>) -> Result<ControlStatement> {
    assert_eq!(pair.as_rule(), Rule::ForStatement);

    let mut inner = pair.into_inner();

    // KeywordFor
    inner.next();

    // Identifier
    let identifier = inner.next().unwrap().as_str().to_string();

    // KeywordIn
    inner.next();

    // Expression
    let iterable = build_expression(inner.next().unwrap())?;

    // Block
    let block = build_block(inner.next().unwrap())?;

    Ok(ControlStatement::build_for(identifier, iterable, block))
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

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::AssignmentDereference => {
            let target = inner.into_inner().next().unwrap();

            Ok(AssignmentTarget::Dereference(Box::new(
                build_assignment_target(target)?,
            )))
        }

        Rule::AssignmentPostfix => build_assignment_postfix(inner),

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_assignment_postfix(pair: Pair<Rule>) -> Result<AssignmentTarget> {
    assert_eq!(pair.as_rule(), Rule::AssignmentPostfix);

    let mut inner = pair.into_inner();

    let identifier = inner.next().unwrap();

    let mut target = AssignmentTarget::Identifier(identifier.as_str().to_string());

    for pair in inner {
        match pair.as_rule() {
            Rule::FieldAccess => {
                let field = pair.into_inner().next().unwrap();

                target = AssignmentTarget::FieldAccess {
                    target: Box::new(target),
                    field_identifier: field.as_str().to_string(),
                };
            }

            Rule::ArrayAccess => {
                let index_expression = build_expression(pair.into_inner().next().unwrap())?;

                target = AssignmentTarget::ArrayAccess {
                    target: Box::new(target),
                    index_expression,
                };
            }

            _ => unreachable!("{:?}", pair.as_rule()),
        }
    }

    Ok(target)
}

// fn build_dereference_target(pair: Pair<Rule>) -> Result<Expression> {
//     assert_eq!(pair.as_rule(), Rule::Dereference);

//     let unary = pair.into_inner().next().unwrap();
//     build_unary(unary)
// }

// fn build_dereference(pair: Pair<Rule>) -> Result<Expression> {
//     assert_eq!(pair.as_rule(), Rule::Dereference);

//     let mut inner = pair.into_inner();

//     let expression = build_unary(inner.next().unwrap())?;

//     Ok(Expression::Dereference(Box::new(expression)))
// }

fn build_binary_expression<BuildOperand, BuildOperator>(
    pair: Pair<Rule>,
    build_operand: BuildOperand,
    build_operator: BuildOperator,
) -> Result<Expression>
where
    BuildOperand: Fn(Pair<Rule>) -> Result<Expression>,
    BuildOperator: Fn(Rule) -> BinaryOperator,
{
    let mut inner = pair.into_inner();
    let mut expr = build_operand(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let rhs = build_operand(
            inner
                .next()
                .expect("binary operator must have a right-hand operand"),
        )?;

        expr = Expression::Binary {
            lhs: Box::new(expr),
            operator: build_operator(op.as_rule()),
            rhs: Box::new(rhs),
        };
    }

    Ok(expr)
}

fn build_expression(pair: Pair<Rule>) -> Result<Expression> {
    build_range(pair.into_inner().next().unwrap())
}

fn build_range(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Range);

    let mut inner = pair.into_inner();

    let start = build_logical_or(inner.next().unwrap())?;

    if inner.next().is_some() {
        let end = build_logical_or(inner.next().unwrap())?;

        Ok(Expression::Range {
            start: Box::new(start),
            end: Box::new(end),
        })
    } else {
        Ok(start)
    }
}

fn build_logical_or(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::LogicalOr);

    build_binary_expression(pair, build_logical_and, |rule| match rule {
        Rule::OrOr => BinaryOperator::LogicalOr,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_logical_and(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::LogicalAnd);

    build_binary_expression(pair, build_bitwise_or, |rule| match rule {
        Rule::AndAnd => BinaryOperator::LogicalAnd,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_bitwise_or(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::BitwiseOr);

    build_binary_expression(pair, build_bitwise_xor, |rule| match rule {
        Rule::Pipe => BinaryOperator::BitwiseOr,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_bitwise_xor(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::BitwiseXor);

    build_binary_expression(pair, build_bitwise_and, |rule| match rule {
        Rule::Caret => BinaryOperator::BitwiseXor,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_bitwise_and(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::BitwiseAnd);

    build_binary_expression(pair, build_comparison, |rule| match rule {
        Rule::Amp => BinaryOperator::BitwiseAnd,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_comparison(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Comparison);

    build_binary_expression(pair, build_shift, |rule| match rule {
        Rule::EqualTo => BinaryOperator::EqualTo,
        Rule::NotEqualTo => BinaryOperator::NotEqualTo,
        Rule::LessThan => BinaryOperator::LessThan,
        Rule::LessThanOrEqualTo => BinaryOperator::LessThanOrEqualTo,
        Rule::GreaterThan => BinaryOperator::GreaterThan,
        Rule::GreaterThanOrEqualTo => BinaryOperator::GreaterThanOrEqualTo,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_shift(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Shift);

    build_binary_expression(pair, build_addition, |rule| match rule {
        Rule::ShiftLeft => BinaryOperator::ShiftLeft,
        Rule::ShiftRight => BinaryOperator::ShiftRight,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_addition(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Addition);

    build_binary_expression(pair, build_multiplication, |rule| match rule {
        Rule::Plus => BinaryOperator::Add,
        Rule::Subtract => BinaryOperator::Subtract,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_multiplication(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Multiplication);

    build_binary_expression(pair, build_type_cast, |rule| match rule {
        Rule::Multiply => BinaryOperator::Multiply,
        Rule::Slash => BinaryOperator::Divide,
        Rule::Percent => BinaryOperator::Modulo,
        _ => unreachable!("{:?}", rule),
    })
}

fn build_type_cast(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::TypeCast);

    let mut inner = pair.into_inner();

    let mut expression = build_unary(inner.next().unwrap())?;

    // KeywordAs
    inner.next();

    for pair in inner {
        let data_type = build_data_type(pair)?;

        expression = Expression::TypeCast {
            expression: Box::new(expression),
            target_type: data_type,
        };
    }

    Ok(expression)
}

fn build_unary(pair: Pair<Rule>) -> Result<Expression> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Reference => build_unary_op(inner, UnaryOperator::Reference),
        Rule::Dereference => build_unary_op(inner, UnaryOperator::Dereference),
        Rule::Negation => build_unary_op(inner, UnaryOperator::Negate),
        Rule::LogicalNot => build_unary_op(inner, UnaryOperator::LogicalNot),
        Rule::BitwiseNot => build_unary_op(inner, UnaryOperator::BitwiseNot),
        Rule::Postfix => build_postfix(inner),
        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_unary_op(pair: Pair<Rule>, operator: UnaryOperator) -> Result<Expression> {
    let mut inner = pair.into_inner();

    let operand = build_unary(inner.next().unwrap())?;

    Ok(Expression::Unary {
        operator,
        expression: Box::new(operand),
    })
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

            Rule::ArrayAccess => {
                // Expression
                let index_expression = Box::new(build_expression(inner.next().unwrap())?);

                expr = Expression::ArrayAccess {
                    expression: Box::new(expr),
                    index_expression,
                };
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

    let mut initialized_fields = Vec::new();
    let mut use_defaults = false;

    for pair in pair.into_inner() {
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

    for pair in inner {
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
    let amount = parse_usize_literal(inner.next().unwrap().as_str())?;

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

    if let Some(pair) = inner.peek()
        && pair.as_rule() == Rule::GenericArguments
    {
        let pair = inner.next().unwrap(); // consume it

        for pair in pair.into_inner() {
            generics.push(build_data_type(pair)?);
        }
    }

    Ok(generics)
}

fn build_argument_list(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    assert_eq!(pair.as_rule(), Rule::ArgumentList);

    pair.into_inner().map(build_expression).collect()
}
