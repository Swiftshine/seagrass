use anyhow::Result;
use seagrass::core::{execute_source, runtime::RuntimeValue};

#[test]
pub fn if_statements() -> Result<()> {
    let base = "
        let out = \"\";

        if my_val == 1 {
            out = \"ONE\";
        } else if my_val == 2 {
            out = \"TWO\";
        } else {
            out = \"DEFAULT\";
        }
    ";

    let source = "let my_val = 1;".to_string() + base;
    let out = execute_source(&source, &vec![])?.get_global_variable("out")?;
    assert_eq!(
        out.borrow().value(),
        RuntimeValue::String("ONE".to_string())
    );

    let source = "let my_val = 2;".to_string() + base;
    let out = execute_source(&source, &vec![])?.get_global_variable("out")?;
    assert_eq!(
        out.borrow().value(),
        RuntimeValue::String("TWO".to_string())
    );

    let source = "let my_val = 3;".to_string() + base;
    let out = execute_source(&source, &vec![])?.get_global_variable("out")?;
    assert_eq!(
        out.borrow().value(),
        RuntimeValue::String("DEFAULT".to_string())
    );
    Ok(())
}

#[test]
pub fn while_statements() -> Result<()> {
    let source = "
        let val = 5;

        while val != 0 {
            val = val - 1;
        }
    ";

    let val = execute_source(source, &vec![])?.get_global_variable("val")?;

    assert_eq!(val.borrow().value(), RuntimeValue::S32(0));

    Ok(())
}
