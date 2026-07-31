use crate::core::lang::{
    ast::{DataType, Value},
    parser::Rule,
};
use anyhow::Result;
use pest::iterators::Pair;

fn parse_i32_literal(s: &str) -> Result<i32> {
    if let Some(bits) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        Ok(i32::from_str_radix(bits, 2)?)
    } else if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Ok(i32::from_str_radix(hex, 16)?)
    } else {
        Ok(s.parse()?)
    }
}

pub fn parse_usize_literal(s: &str) -> Result<usize> {
    if let Some(bits) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        Ok(usize::from_str_radix(bits, 2)?)
    } else if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Ok(usize::from_str_radix(hex, 16)?)
    } else {
        Ok(s.parse()?)
    }
}

fn parse_f32_literal(s: &str) -> Result<f32> {
    let s = s.strip_suffix('f').expect("f32 literal must end in 'f'");

    Ok(s.parse()?)
}

fn parse_f64_literal(s: &str) -> Result<f64> {
    let s = s.strip_suffix('d').expect("f64 literal must end in 'd'");

    Ok(s.parse()?)
}

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

            let value = parse_i32_literal(s)?;

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

        Rule::Float => {
            let s = inner.as_str();

            if s.ends_with('f') {
                Ok(Value::F32(parse_f32_literal(s)?))
            } else if s.ends_with('d') {
                Ok(Value::F64(parse_f64_literal(s)?))
            } else {
                unreachable!("float literal missing suffix: {}", s);
            }
        }

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

pub fn build_data_type(pair: Pair<Rule>) -> Result<DataType> {
    assert_eq!(pair.as_rule(), Rule::DataType);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::ArrayType => build_array_type(inner),
        Rule::SingleDataType => build_single_type(inner),

        _ => unreachable!("{:?}", inner.as_rule()),
    }
}

fn build_array_type(pair: Pair<Rule>) -> Result<DataType> {
    assert_eq!(pair.as_rule(), Rule::ArrayType);

    let mut inner = pair.into_inner();

    // DataType
    let data_type = build_data_type(inner.next().unwrap())?;

    // Integer

    let count = if let Some(count) = inner.next() {
        let s = count.as_str();
        let count = parse_usize_literal(s)?;

        Some(count)
    } else {
        None
    };

    Ok(DataType::Array {
        inner_data_type: Box::new(data_type),
        count,
    })
}

fn build_single_type(pair: Pair<Rule>) -> Result<DataType> {
    assert_eq!(pair.as_rule(), Rule::SingleDataType);

    match pair.as_str() {
        "s8" => Ok(DataType::S8),
        "u8" => Ok(DataType::U8),
        "s16" => Ok(DataType::S16),
        "u16" => Ok(DataType::U16),
        "s32" => Ok(DataType::S32),
        "u32" => Ok(DataType::U32),
        "f32" => Ok(DataType::F32),
        "f64" => Ok(DataType::F64),
        "string" => Ok(DataType::String),
        "bool" => Ok(DataType::Bool),
        other => Ok(DataType::UserDefined(other.to_string())),
    }
}
