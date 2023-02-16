pub use log::*;

use std::fs::File;
use std::io::BufWriter;
use env_logger::{Builder, Env, Target};

pub fn file_writer() -> std::io::Result<BufWriter<File>> {
    const FILE: &str = "gbmu.log";
    let file = File::create(FILE)?;
    Ok(BufWriter::with_capacity(65_536, file))
}

pub fn init() {
    let env = Env::default().default_filter_or("wgpu_core=warn,wgpu_hal=warn,naga=warn,debug");
    let mut builder = Builder::from_env(env);
    builder.target(match file_writer() {
        Ok(writer) => Target::Pipe(Box::new(writer)),
        Err(e) => Target::Stdout
    }).init();
}
