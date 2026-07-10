use anyhow::Result;
use seagrass::core::{execute_source, runtime::RuntimeValue};

#[test]
fn declarative_assignment() -> Result<()> {
    let runtime = execute_source("let my_ident = 123;")?;
    let variable = runtime.get_variable("my_ident")?;
    assert!(matches!(*variable, RuntimeValue::Integer(123)));
    Ok(())
}

#[test]
fn reassignment() -> Result<()> {
    let source = "
        let my_ident = 123;
        my_ident = 456;
    ";

    let runtime = execute_source(source)?;
    let variable = runtime.get_variable("my_ident")?;
    assert!(matches!(*variable, RuntimeValue::Integer(456)));
    Ok(())
}
