use crate::core::{
    lang::{ast::Program, build_program},
    runtime::{Runtime, RuntimeConfigOption},
};

pub mod lang;
pub mod native;
pub mod runtime;

fn execute_program(program: &Program, configs: &[RuntimeConfigOption]) -> Runtime {
    let mut runtime = Runtime::new().with_configs(configs);
    let _ = runtime.execute(program);
    runtime
}

pub fn execute_source(source: &str, configs: &[RuntimeConfigOption]) -> anyhow::Result<Runtime> {
    let program = build_program(source)?;
    let runtime = execute_program(&program, configs);
    Ok(runtime)
}
