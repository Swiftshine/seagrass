use anyhow::Result;
use seagrass::core::{
    execute_source,
    runtime::{RuntimeConfigOption, RuntimeValue},
};

#[test]
pub fn check_scopes() -> Result<()> {
    let source = "
        fn function_one() {
            let one = 1;
            return one;
        }

        fn function_two() {
            let two = 2;
            return two;
        }

        fn main() {
            let one = function_one();
            let two = function_two();
        }

        let global_var = 123;
    ";

    let runtime = execute_source(source, &[RuntimeConfigOption::PreserveExpiredFrames(true)])?;

    let one = runtime
        .get_dead_frame("function_one")?
        .current_scope()
        .get_variable("one")?;

    let two = runtime
        .get_dead_frame("function_two")?
        .current_scope()
        .get_variable("two")?;

    assert!(matches!(one.value, RuntimeValue::S32(1)));
    assert!(matches!(two.value, RuntimeValue::S32(2)));

    let one = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("one")?;
    let two = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("two")?;

    assert!(matches!(one.value, RuntimeValue::S32(1)));
    assert!(matches!(two.value, RuntimeValue::S32(2)));

    let global_var = runtime.get_global_variable("global_var")?;

    assert!(matches!(global_var.value, RuntimeValue::S32(123)));
    Ok(())
}
