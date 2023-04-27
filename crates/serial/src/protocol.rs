pub enum Command {
    SendMaster,
    SendSlave,
    Sync,
}

impl TryFrom<u8> for Command {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            104 => Ok(Command::SendMaster),
            105 => Ok(Command::SendSlave),
            106 => Ok(Command::Sync),
            n => Err(format!("Unknown command {n}"))
        }
    }
}


