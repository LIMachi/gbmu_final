
fn main() {
    let v = std::env::var("ahaha").unwrap_or("12".to_string()).parse::<u8>().unwrap_or(12);
    let op = Opcode::try_from(v);
    println!("{op:?}")
}
