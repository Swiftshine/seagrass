use anyhow::Result;
use seagrass::core::{execute_source, runtime::RuntimeValue};

#[test]
pub fn define_struct() -> Result<()> {
    let source = "
        struct MyStruct {
            field_1: u32
        }
    ";

    execute_source(source, &vec![])?;

    Ok(())
}

#[test]
pub fn initialize_struct() -> Result<()> {
    let source = "
        struct MyStruct {
            field_1: u32
        }

        let initialization = MyStruct {
            field_1: 1
        };

        let annotated_initialization: MyStruct = MyStruct {
            field_1: 2
        };
    ";

    let runtime = execute_source(source, &vec![])?;

    let one = runtime.get_global_variable("initialization")?;

    let two = runtime.get_global_variable("annotated_initialization")?;

    assert_eq!(
        one.borrow().value().struct_access("field_1")?,
        RuntimeValue::U32(1)
    );
    assert_eq!(
        two.borrow().value().struct_access("field_1")?,
        RuntimeValue::U32(2)
    );

    Ok(())
}

#[test]
pub fn access_struct() -> Result<()> {
    let source = "
        struct MyStruct {
            field_1: u32
        }

        let my_inst = MyStruct {
            field_1: 2
        };

        let my_value = my_inst.field_1;
    ";

    let runtime = execute_source(source, &vec![])?;

    let value = runtime.get_global_variable("my_value")?;

    assert_eq!(value.borrow().value(), RuntimeValue::U32(2));

    Ok(())
}

#[test]
pub fn default_initialization() -> Result<()> {
    let source = "
        struct MyStruct {
            value: u32
        }

        let s = MyStruct { .. };
        let out = s.value;
    ";

    let out = execute_source(source, &vec![])?.get_global_variable("out")?;

    assert_eq!(out.borrow().value(), RuntimeValue::U32(0));

    Ok(())
}
