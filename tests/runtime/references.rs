use anyhow::Result;
use seagrass::core::{
    execute_source,
    runtime::{RuntimeConfigOption, RuntimeValue},
};

#[test]
pub fn read_from_reference() -> Result<()> {
    let source = "
        let my_var = 1;
        let my_ref = &my_var;

        let value_1 = *my_ref;

        my_var = 2;

        let value_2 = *my_ref;
    ";

    let runtime = execute_source(source, &[RuntimeConfigOption::PreserveExpiredFrames(true)])?;

    let one = runtime.get_global_variable("value_1")?;

    let two = runtime.get_global_variable("value_2")?;

    assert_eq!(one.borrow().value(), RuntimeValue::S32(1));
    assert_eq!(two.borrow().value(), RuntimeValue::S32(2));

    Ok(())
}

#[test]
pub fn write_to_reference() -> Result<()> {
    let source = "
        let my_var = 1;
        let my_ref = &my_var;

        *my_ref = 2;
    ";

    let runtime = execute_source(source, &[RuntimeConfigOption::PreserveExpiredFrames(true)])?;

    let var = runtime.get_global_variable("my_var")?;

    assert_eq!(var.borrow().value(), RuntimeValue::S32(2));

    Ok(())
}
