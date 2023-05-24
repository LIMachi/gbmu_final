use std::cmp::Ordering;
use std::fs::File;
use std::io::{ErrorKind, Read, Result};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use header::{Capabilities, Header, Mbc};

mod header;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rom {
    pub filename: String,
    pub location: PathBuf,
    pub header: header::Header,
    pub cover: Option<String>,
    content: Vec<u8>,
}

impl PartialOrd for Rom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.filename.partial_cmp(&other.filename)
    }
}

impl Ord for Rom {
    fn cmp(&self, other: &Self) -> Ordering {
        self.filename.cmp(&other.filename)
    }
}

impl Rom {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let f = path.file_stem().unwrap().to_str().expect("invalid rom name").to_string();
        let location = path.parent().unwrap().to_path_buf();
        let mut v = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut v)?;
        if v.len() < 0x150 {
            return Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid rom"));
        }
        Ok(Self {
            filename: f,
            location,
            header: Header::new(&v[0..=0x14F]),
            content: v,
            cover: None,
        })
    }

    pub fn raw(&self) -> Vec<u8> {
        self.content.clone()
    }

    pub fn find_roms<P: AsRef<std::path::Path>>(_path: P) -> Vec<Self> {
        vec![]
    }
}
