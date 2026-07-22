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

    let runtime = execute_source(source, &[RuntimeConfigOption::PreserveExpiredFrames(true)])?;
    let variable = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("my_ident")?;

    assert!(matches!(variable.borrow().value(), RuntimeValue::S32(123)));
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

    let runtime = execute_source(
        source,
        &vec![RuntimeConfigOption::PreserveExpiredFrames(true)],
    )?;

    let variable = runtime
        .get_dead_frame("main")?
        .current_scope()
        .get_variable("my_ident")?;

    assert!(matches!(variable.borrow().value(), RuntimeValue::S32(123)));
    Ok(())
}

#[test]
pub fn struct_method() -> Result<()> {
    let source = "
        struct MyStruct {
            value: s32
        }

        impl MyStruct {
            fn my_func(&self) {
                return self.value * 2;
            }
        }

        let my_struct = MyStruct { value: 2 };
        let out = (&my_struct).my_func();
    ";

    let out = execute_source(source, &vec![])?.get_global_variable("out")?;

    assert_eq!(out.borrow().value(), RuntimeValue::S32(4));
    
    Ok(())
}
