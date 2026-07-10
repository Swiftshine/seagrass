use anyhow::Result;
use seagrass::core::lang;
use seagrass::core::runtime::Runtime;
use std::fs;

fn main() -> Result<()> {
    let contents = fs::read_to_string("scratch/1.sgs")?;
    let program = lang::build_program(&contents)?;
    // let _ = dbg!(&program);
    let mut runtime = Runtime::new();
    dbg!(&runtime);
    let _ = dbg!(runtime.execute(&program));
    dbg!(&runtime);

    Ok(())
}
