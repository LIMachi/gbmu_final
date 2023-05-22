use env_logger::{Builder, Env, Target};
pub use log::*;

pub fn init() {
    let env = Env::default().default_filter_or("app=debug,error");
    let mut builder = Builder::from_env(env);
    builder.target(Target::Stdout).init();
}
