pub use log::*;

use std::fs::File;
use std::io::{BufWriter, ErrorKind};
use env_logger::{Builder, Env, Target};

pub fn file_writer() -> std::io::Result<BufWriter<File>> {

    #[cfg(feature = "file_log")]
    {
        const FILE: &str = "gbmu.log";
        let file = File::create(FILE)?;
        return Ok(BufWriter::with_capacity(65_536, file))
    }
    Err(std::io::Error::new(ErrorKind::Other, "not allowed"))
}

pub fn init() {
    let env = Env::default().default_filter_or("wgpu_core=warn,wgpu_hal=warn,naga=warn,debug");
    let mut builder = Builder::from_env(env);
    builder.target(match file_writer() {
        Ok(writer) => Target::Pipe(Box::new(writer)),
        Err(_e) => Target::Stdout
    }).init();
}
