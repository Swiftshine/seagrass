use anyhow::Result;
use seagrass::core::{execute_source, runtime::RuntimeValue};

#[test]
pub fn compare_values() -> Result<()> {
    let source = "
        let one = 1 == 1;
        let two = 1 != 1;
    ";

    let runtime = execute_source(source, &vec![])?;
    let one = runtime.get_global_variable("one")?;
    let two = runtime.get_global_variable("two")?;

    assert_eq!(one.borrow().value(), RuntimeValue::Bool(true));
    assert_eq!(two.borrow().value(), RuntimeValue::Bool(false));
    Ok(())
}

#[test]
pub fn compare_structs() -> Result<()> {
    let struct_definitions = "
        struct StructOne {
            field_1: u32,
            field_2: u32
        }

        struct StructTwo {
            field_a: u32,
            field_b: u32
        }
    ";

    let source = struct_definitions.to_string()
        + "
        let one = StructOne { field_1: 1, field_2: 2 };
        let two = StructOne { field_1: 1, field_2: 3 };
        let three = StructOne { field_1: 1, field_2: 2 };

        let result_1 = one == two;
        let result_2 = one == three;
    ";

    // compare against structs of identical types
    let runtime = execute_source(&source, &vec![])?;
    let one = runtime.get_global_variable("result_1")?;
    let two = runtime.get_global_variable("result_2")?;

    assert_eq!(one.borrow().value(), RuntimeValue::Bool(false));
    assert_eq!(two.borrow().value(), RuntimeValue::Bool(true));

    // fail against structs of mismatching types
    let source = struct_definitions.to_string()
        + "
        let one = StructOne { field_1: 1, field_2: 2 };
        let two = StructTwo { field_a: 1, field_b: 2 };

        let result = one == two;
    ";

    let runtime = execute_source(&source, &vec![])?;
    let result = runtime.get_variable("result");
    assert!(result.is_err()); // the variable "result" will not exisst due to the runtime failing

    Ok(())
}
