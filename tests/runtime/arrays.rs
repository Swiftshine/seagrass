use anyhow::Result;
use seagrass::core::{execute_source, runtime::RuntimeValue};

#[test]
pub fn define_array() -> Result<()> {
    let source = "
        let arr = { 1, 2, 3 };
    ";

    let arr = execute_source(source, &vec![])?.get_global_variable("arr")?;

    assert!(matches!(
        arr.borrow().copy_value(),
        RuntimeValue::Array { .. }
    ));
    Ok(())
}

#[test]
pub fn read_from_array() -> Result<()> {
    let source = "
        let arr = { 1, 2, 3 };
        let out = arr[0] + arr[1] + arr[2];
    ";

    let out = execute_source(source, &vec![])?.get_global_variable("out")?;

    assert_eq!(out.borrow().copy_value(), RuntimeValue::S32(1 + 2 + 3));

    Ok(())
}

#[test]
pub fn assign_to_array() -> Result<()> {
    let source = "
        let arr = { 1, 2, 3 };
        arr[1] = 4;
    ";

    let arr = execute_source(source, &vec![])?.get_global_variable("arr")?;

    if let RuntimeValue::Array { contents, .. } = arr.borrow().copy_value() {
        assert_eq!(contents[1].borrow().copy_value(), RuntimeValue::S32(4));
        Ok(())
    } else {
        panic!("this should be an array")
    }
}
