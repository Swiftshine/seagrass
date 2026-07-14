use anyhow::Result;
use seagrass::core::lang;
use seagrass::core::runtime::{Runtime, RuntimeConfigOption};
use std::fs;

fn main() -> Result<()> {
    let contents = fs::read_to_string("scratch/1.sgs")?;
    let program = lang::build_program(&contents)?;
    let mut runtime = Runtime::new().with_config(RuntimeConfigOption::PreserveExpiredFrames(true));
    runtime.execute(&program)?;
    Ok(())
}
