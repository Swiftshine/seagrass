use crate::core::{
    lang::{ast::Program, build_program},
    runtime::Runtime,
};

pub mod lang;
pub mod native;
pub mod runtime;

fn execute_program(program: &Program) -> Runtime {
    let mut runtime = Runtime::new();
    let _ = runtime.execute(program);
    runtime
}

pub fn execute_source(source: &str) -> anyhow::Result<Runtime> {
    let program = build_program(source)?;
    let runtime = execute_program(&program);
    Ok(runtime)
}
