use anyhow::Result;
use seagrass::core::{
    execute_source,
    runtime::{RuntimeConfigOption, RuntimeValue},
};

#[test]
pub fn addition() -> Result<()> {
    let source = "
        fn func_one() {
            return 1;
        }

        fn func_two() {
            return 2;
        }

        fn main() {
            let result_plus = func_one() + func_two();
            let result_minus = func_two() - func_one();
        }
    ";

    let runtime = execute_source(source, &[RuntimeConfigOption::PreserveExpiredFrames(true)])?;
    let result_plus = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("result_plus")?;
    let result_minus = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("result_minus")?;

    assert!(matches!(result_plus.borrow().value(), RuntimeValue::S32(3)));
    assert!(matches!(result_minus.borrow().value(), RuntimeValue::S32(1)));
    Ok(())
}

#[test]
pub fn multiplication() -> Result<()> {
    let source = "
        fn main() {
            let result_mul = 3 * 2;
            let result_div = 6 / 3;
        }
    ";

    let runtime = execute_source(source, &[RuntimeConfigOption::PreserveExpiredFrames(true)])?;
    let result_mul = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("result_mul")?;
    let result_div = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("result_div")?;

    assert!(matches!(result_mul.borrow().value(), RuntimeValue::S32(6)));
    assert!(matches!(result_div.borrow().value(), RuntimeValue::S32(2)));
    Ok(())
}
