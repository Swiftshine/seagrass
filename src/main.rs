mod core;

use anyhow::Result;
use core::lang;
use core::runtime::Runtime;
use std::fs;

fn main() -> Result<()> {
    let contents = fs::read_to_string("scratch/1.sgs")?;
    let program = lang::build_program(&contents)?;

    let mut runtime = Runtime::new();
    dbg!(&runtime);
    runtime.execute(&program);
    dbg!(&runtime);
    // let _ = dbg!(program);

    Ok(())
}
