use anyhow::Result;
use seagrass::core::{execute_source, runtime::RuntimeValue};

#[test]
pub fn declarative_assignment() -> Result<()> {
    let runtime = execute_source("let my_ident = 123;", &vec![])?;
    let variable = runtime.get_variable("my_ident")?;
    assert!(matches!(*variable, RuntimeValue::S32(123)));
    Ok(())
}

#[test]
pub fn reassignment() -> Result<()> {
    let source = "
        let my_ident = 123;
        my_ident = 456;
    ";

    let runtime = execute_source(source, &vec![])?;
    let variable = runtime.get_variable("my_ident")?;
    assert!(matches!(*variable, RuntimeValue::S32(456)));
    Ok(())
}

#[test]
pub fn check_data_types() -> Result<()> {
    let source = "
        let unsigned: u32 = 1;
        let signed: s32 = 2;
        let implicitly_signed = 3;
    ";

    let runtime = execute_source(source, &vec![])?;
    let first = runtime.get_variable("unsigned")?;
    let second = runtime.get_variable("signed")?;
    let third = runtime.get_variable("implicitly_signed")?;

    assert!(matches!(*first, RuntimeValue::U32(1)));
    assert!(matches!(*second, RuntimeValue::S32(2)));
    assert!(matches!(*third, RuntimeValue::S32(3)));
    Ok(())
}
