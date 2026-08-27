mod core;
use anyhow::Result;
use crate::core::lang;
use crate::core::runtime::{Runtime, RuntimeConfigOption};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let path = Path::new("scratch/1.sg");
    let contents = fs::read_to_string(path)?;

    let program = lang::build_program(&contents)?;
    let mut runtime = Runtime::new(path.parent().unwrap().to_path_buf())
        .with_config(RuntimeConfigOption::PreserveExpiredFrames(true));
    runtime.execute(&program)?;
    // dbg!(&runtime);
    Ok(())
}
