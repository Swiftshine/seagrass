use anyhow::Result;
use seagrass::core::{
    execute_source,
    runtime::{RuntimeConfigOption, RuntimeValue},
};

#[test]
pub fn check_main() -> Result<()> {
    let source = "
        fn main() {
            let my_ident = 123;
        }
    ";

    let runtime = execute_source(source, &[RuntimeConfigOption::PreserveScope(true)])?;
    let variable = runtime.get_dead_variable("my_ident")?;
    assert!(matches!(*variable, RuntimeValue::S32(123)));
    Ok(())
}

#[test]
pub fn return_value() -> Result<()> {
    let source = "
        fn do_something() {
            return 123;
        }

        fn main() {
            let my_ident = do_something();
        }
    ";

    let runtime = execute_source(source, &vec![RuntimeConfigOption::PreserveScope(true)])?;
    let variable = runtime.get_dead_variable("my_ident")?;
    assert!(matches!(*variable, RuntimeValue::S32(123)));
    Ok(())
}
