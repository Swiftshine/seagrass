use crate::core::lang::parser::Rule;
use anyhow::Result;
use pest::iterators::Pair;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/* Statements */
#[derive(Debug, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Expression(Expression),
}

/* Expressions */

#[derive(Debug, PartialEq)]
pub enum Expression {
    Value(Value),
}

#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub declarative: bool,
    pub identifier: String,
    pub expression: Expression,
}

#[derive(Debug, PartialEq)]
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

    let value = build_value(pair.into_inner().next().unwrap())?;

    Ok(Expression::Value(value))
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
