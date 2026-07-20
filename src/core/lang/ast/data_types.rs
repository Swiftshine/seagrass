use crate::core::lang::{
    ast::{DataType, Value},
    parser::Rule,
};
use anyhow::Result;
use pest::iterators::Pair;

pub fn build_value(pair: Pair<Rule>) -> Result<Value> {
    assert_eq!(pair.as_rule(), Rule::Value);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Identifier => Ok(Value::Identifier(inner.as_str().to_string())),

        Rule::String => {
            let string = inner
                .as_str()
                .strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap();
            Ok(Value::String(string.to_string()))
        }

        Rule::Integer => {
            let s = inner.as_str();

            let value = if let Some(hex) = s.strip_prefix("0x") {
                i32::from_str_radix(hex, 16)?
            } else if let Some(hex) = s.strip_prefix("0X") {
                i32::from_str_radix(hex, 16)?
            } else {
                s.parse()?
            };

            Ok(Value::S32(value))
        }

        Rule::Bool => {
            let rule = inner.into_inner().next().unwrap().as_rule();

            match rule {
                Rule::KeywordTrue => Ok(Value::Bool(true)),
                Rule::KeywordFalse => Ok(Value::Bool(false)),
                _ => unreachable!("{:?}", rule),
            }
        }

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

pub fn build_data_type(pair: Pair<Rule>) -> Result<DataType> {
    match pair.as_str() {
        "u32" => Ok(DataType::U32),
        "s32" => Ok(DataType::S32),
        "string" => Ok(DataType::String),
        "bool" => Ok(DataType::Bool),
        other => Ok(DataType::UserDefined(other.to_string())),
    }
}
