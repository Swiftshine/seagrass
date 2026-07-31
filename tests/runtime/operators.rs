use anyhow::Result;
use seagrass::core::{execute_source, runtime::RuntimeValue};

#[test]
pub fn addition() -> Result<()> {
    let runtime = execute_source("let result = 1 + 2;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(3));

    Ok(())
}

#[test]
pub fn subtraction() -> Result<()> {
    let runtime = execute_source("let result = 5 - 3;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(2));

    Ok(())
}

#[test]
pub fn multiplication() -> Result<()> {
    let runtime = execute_source("let result = 4 * 5;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(20));

    Ok(())
}

#[test]
pub fn division() -> Result<()> {
    let runtime = execute_source("let result = 20 / 4;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(5));

    Ok(())
}

#[test]
pub fn modulo() -> Result<()> {
    let runtime = execute_source("let result = 10 % 3;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(1));

    Ok(())
}

#[test]
pub fn shift_left() -> Result<()> {
    let runtime = execute_source("let result = 1 << 4;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(16));

    Ok(())
}

#[test]
pub fn shift_right() -> Result<()> {
    let runtime = execute_source("let result = 16 >> 2;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(4));

    Ok(())
}

#[test]
pub fn less_than() -> Result<()> {
    let runtime = execute_source("let result = 1 < 2;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(true));

    Ok(())
}

#[test]
pub fn less_than_or_equal_to() -> Result<()> {
    let runtime = execute_source("let result = 2 <= 2;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(true));

    Ok(())
}

#[test]
pub fn greater_than() -> Result<()> {
    let runtime = execute_source("let result = 2 > 1;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(true));

    Ok(())
}

#[test]
pub fn greater_than_or_equal_to() -> Result<()> {
    let runtime = execute_source("let result = 2 >= 2;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(true));

    Ok(())
}

#[test]
pub fn equality() -> Result<()> {
    let runtime = execute_source("let result = 123 == 123;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(true));

    Ok(())
}

#[test]
pub fn inequality() -> Result<()> {
    let runtime = execute_source("let result = 123 != 456;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(true));

    Ok(())
}

#[test]
pub fn bitwise_and() -> Result<()> {
    let runtime = execute_source("let result = 12 & 10;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(8));

    Ok(())
}

#[test]
pub fn bitwise_or() -> Result<()> {
    let runtime = execute_source("let result = 12 | 3;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(15));

    Ok(())
}

#[test]
pub fn bitwise_xor() -> Result<()> {
    let runtime = execute_source("let result = 12 ^ 10;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(6));

    Ok(())
}

#[test]
pub fn logical_and() -> Result<()> {
    let runtime = execute_source("let result = true && false;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(false));

    Ok(())
}

#[test]
pub fn logical_or() -> Result<()> {
    let runtime = execute_source("let result = true || false;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(true));

    Ok(())
}

/* Unary operators */

#[test]
pub fn negation() -> Result<()> {
    let runtime = execute_source("let result = -123;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(-123));

    Ok(())
}

#[test]
pub fn logical_not() -> Result<()> {
    let runtime = execute_source("let result = !true;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::Bool(false));

    Ok(())
}

#[test]
pub fn bitwise_not() -> Result<()> {
    let runtime = execute_source("let result = ~0;", &vec![])?;
    let result = runtime.get_global_variable("result")?;

    assert_eq!(result.borrow().copy_value(), RuntimeValue::S32(-1));

    Ok(())
}
