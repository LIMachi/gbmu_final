use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Read, Result};

mod header;

pub struct Rom {
    pub filename: String,
    pub location: PathBuf,
    pub header: header::Header,
    content: Vec<u8>,
}

impl Rom {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let f = path.file_stem().unwrap().to_str().expect("invalid rom name").to_string();
        let location = path.parent().unwrap().to_path_buf();
        let mut v = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut v)?;
        let h = header::Header::new(&v[0..=0x14F]);
        println!("{:?}", h);
        Ok(Self {
            filename: f,
            location,
            header: h,
            content: v
        })
    }

    pub fn raw(&self) -> Vec<u8> {
        self.content.clone()
    }
}
