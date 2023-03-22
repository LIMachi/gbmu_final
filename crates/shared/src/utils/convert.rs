use std::str::FromStr;

pub trait Converter {
    fn convert(str: &str) -> Self;
}

impl Converter for u8 {
    fn convert(raw: &str) -> Self {
        let _a = 2;

        let str = raw.trim_start_matches("0x").trim_start_matches("0X");
        if str != raw {
            u8::from_str_radix(str, 16).unwrap_or(0)
        } else {
            u8::from_str(str).or_else(|_| u8::from_str_radix(str, 16)).unwrap_or(0)
        }
    }
}

impl Converter for u16 {
    fn convert(raw: &str) -> Self {
        let str = raw.trim_start_matches("0x").trim_start_matches("0X");
        if str != raw {
            u16::from_str_radix(str, 16).unwrap_or(0)
        } else {
            u16::from_str(str).or_else(|_| u16::from_str_radix(str, 16)).unwrap_or(0)
        }
    }
}
