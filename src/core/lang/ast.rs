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
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinition {
    pub identifier: String,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Value(Value),
    FunctionCall(FunctionCall),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub declarative: bool,
    pub identifier: String,
    pub expression: Expression,
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
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Integer(i64),
    Identifier(String),
}

// pub fn dump(pair: Pair<Rule>, indent: usize) {
//     println!(
//         "{}{:?}: {:?}",
//         "  ".repeat(indent),
//         pair.as_rule(),
//         pair.as_str()
//     );

//     for child in pair.into_inner() {
//         dump(child, indent + 1);
//     }
// }

pub fn build_program(pair: Pair<Rule>) -> Result<Program> {
    assert_eq!(pair.as_rule(), Rule::Program);

    let statements = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::Statement)
        .map(build_statement)
        .flatten()
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
        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_assignment(pair: Pair<Rule>) -> Result<Assignment> {
    let mut inner = pair.into_inner().peekable();

    // KeywordLet?
    let declarative = matches!(inner.peek().map(|p| p.as_rule()), Some(Rule::KeywordLet));

    if declarative {
        inner.next();
    }

    // Identifier
    let identifier = inner.next().unwrap().as_str().to_string();

    // Equals
    inner.next();

    // Expression
    let expression = build_expression(inner.next().unwrap())?;

    Ok(Assignment {
        declarative,
        identifier,
        expression,
    })
}

fn build_expression(pair: Pair<Rule>) -> Result<Expression> {
    assert_eq!(pair.as_rule(), Rule::Expression);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Value => Ok(Expression::Value(build_value(inner)?)),
        Rule::FunctionCall => Ok(Expression::FunctionCall(build_function_call(inner)?)),
        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_function_definition(pair: Pair<Rule>) -> Result<FunctionDefinition> {
    assert_eq!(pair.as_rule(), Rule::FunctionDefinition);

    let mut inner = pair.into_inner();

    // KeywordFn
    inner.next();

    // Identifier
    let identifier = inner.next().unwrap().as_str().to_string();

    // L and R parens
    inner.next();
    inner.next();

    let body = build_block(inner.next().unwrap())?;

    Ok(FunctionDefinition { identifier, body })
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

fn build_value(pair: Pair<Rule>) -> Result<Value> {
    assert_eq!(pair.as_rule(), Rule::Value);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Identifier => Ok(Value::Identifier(inner.as_str().to_string())),

        Rule::Integer => Ok(Value::Integer(inner.as_str().parse()?)),

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_function_call(pair: Pair<Rule>) -> Result<FunctionCall> {
    assert_eq!(pair.as_rule(), Rule::FunctionCall);

    let mut inner = pair.into_inner();

    // Identifier
    let identifier = inner.next().unwrap().as_str().to_string();

    // L and R parens
    inner.next();
    inner.next();

    Ok(FunctionCall { identifier })
}
