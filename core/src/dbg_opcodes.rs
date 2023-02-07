use std::fmt::{Display, Formatter, Write};
use crate::CBOpcode;
use super::Opcode;

pub struct DebugInfo {
    sz: usize,
    op: &'static str
}

impl From<(usize, &'static str)> for DebugInfo {
    fn from((sz, op): (usize, &'static str)) -> Self {
        Self { sz, op }
    }
}

impl Display for DebugInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.op)
    }
}

pub fn dbg_opcodes(opcode: Opcode) -> (usize, &'static str) {
    match opcode {
        Opcode::Nop          => (1, "NOP"),
        Opcode::LdBCd16      => (3, "LD BC,u16"),
        Opcode::LdIndBCA     => (1, "LD (BC),A"),
        Opcode::IncBC        => (1, "INC BC"),
        Opcode::IncB         => (1, "INC B"),
        Opcode::DecB         => (1, "DEC B"),
        Opcode::LdBd8        => (2, "LD B,u8"),
        Opcode::Rlca         => (1, "RLCA"),
        Opcode::LdInda16SP   => (3, "LD (a16),SP"),
        Opcode::AddHLBC      => (1, "ADD HL,BC"),
        Opcode::LdAIndBC     => (1, "LD A,(BC)"),
        Opcode::DecBC        => (1, "DEC BC"),
        Opcode::IncC         => (1, "INC C"),
        Opcode::DecC         => (1, "DEC C"),
        Opcode::LdCd8        => (2, "LD C,u8"),
        Opcode::Rrca         => (1, "RRCA"),
        Opcode::Stop0        => (2, "STOP 0"),
        Opcode::LdDEd16      => (3, "LD DE,u16"),
        Opcode::LdIndDEA     => (1, "LD (DE),A"),
        Opcode::IncDE        => (1, "INC DE"),
        Opcode::IncD         => (1, "INC D"),
        Opcode::DecD         => (1, "DEC D"),
        Opcode::LdDd8        => (2, "LD D,u8"),
        Opcode::Rla          => (1, "RLA"),
        Opcode::Jrr8         => (2, "JR i8"),
        Opcode::AddHLDE      => (1, "ADD HL,DE"),
        Opcode::LdAIndDE     => (1, "LD A,(DE)"),
        Opcode::DecDE        => (1, "DEC DE"),
        Opcode::IncE         => (1, "INC E"),
        Opcode::DecE         => (1, "DEC E"),
        Opcode::LdEd8        => (2, "LD E,u8"),
        Opcode::Rra          => (1, "RRA"),
        Opcode::JrNZr8       => (2, "JR NZ,i8"),
        Opcode::LdHLd16      => (3, "LD HL,u16"),
        Opcode::LdIndHLIncA  => (1, "LD (HL+),A"),
        Opcode::IncHL        => (1, "INC HL"),
        Opcode::IncH         => (1, "INC H"),
        Opcode::DecH         => (1, "DEC H"),
        Opcode::LdHd8        => (2, "LD H,u8"),
        Opcode::Daa          => (1, "DAA"),
        Opcode::JrZr8        => (2, "JR Z,i8"),
        Opcode::AddHLHL      => (1, "ADD HL,HL"),
        Opcode::LdAIndHLInc  => (1, "LD A,(HL+)"),
        Opcode::DecHL        => (1, "DEC HL"),
        Opcode::IncL         => (1, "INC L"),
        Opcode::DecL         => (1, "DEC L"),
        Opcode::LdLd8        => (2, "LD L,u8"),
        Opcode::Cpl          => (1, "CPL"),
        Opcode::JrNCr8       => (2, "JR NC,i8"),
        Opcode::LdSPd16      => (3, "LD SP,u16"),
        Opcode::LdIndHLDecA  => (1, "LD (HL-),A"),
        Opcode::IncSP        => (1, "INC SP"),
        Opcode::IncIndHL     => (1, "INC (HL)"),
        Opcode::DecIndHL     => (1, "DEC (HL)"),
        Opcode::LdIndHLd8    => (2, "LD (HL),u8"),
        Opcode::Scf          => (1, "SCF"),
        Opcode::JrCr8        => (2, "JR C,i8"),
        Opcode::AddHLSP      => (1, "ADD HL,SP"),
        Opcode::LdAIndHLDec  => (1, "LD A,(HL-)"),
        Opcode::DecSP        => (1, "DEC SP"),
        Opcode::IncA         => (1, "INC A"),
        Opcode::DecA         => (1, "DEC A"),
        Opcode::LdAd8        => (2, "LD A,u8"),
        Opcode::Ccf          => (1, "CCF"),
        Opcode::LdBB         => (1, "LD B,B"),
        Opcode::LdBC         => (1, "LD B,C"),
        Opcode::LdBD         => (1, "LD B,D"),
        Opcode::LdBE         => (1, "LD B,E"),
        Opcode::LdBH         => (1, "LD B,H"),
        Opcode::LdBL         => (1, "LD B,L"),
        Opcode::LdBIndHL     => (1, "LD B,(HL)"),
        Opcode::LdBA         => (1, "LD B,A"),
        Opcode::LdCB         => (1, "LD C,B"),
        Opcode::LdCC         => (1, "LD C,C"),
        Opcode::LdCD         => (1, "LD C,D"),
        Opcode::LdCE         => (1, "LD C,E"),
        Opcode::LdCH         => (1, "LD C,H"),
        Opcode::LdCL         => (1, "LD C,L"),
        Opcode::LdCIndHL     => (1, "LD C,(HL)"),
        Opcode::LdCA         => (1, "LD C,A"),
        Opcode::LdDB         => (1, "LD D,B"),
        Opcode::LdDC         => (1, "LD D,C"),
        Opcode::LdDD         => (1, "LD D,D"),
        Opcode::LdDE         => (1, "LD D,E"),
        Opcode::LdDH         => (1, "LD D,H"),
        Opcode::LdDL         => (1, "LD D,L"),
        Opcode::LdDIndHL     => (1, "LD D,(HL)"),
        Opcode::LdDA         => (1, "LD D,A"),
        Opcode::LdEB         => (1, "LD E,B"),
        Opcode::LdEC         => (1, "LD E,C"),
        Opcode::LdED         => (1, "LD E,D"),
        Opcode::LdEE         => (1, "LD E,E"),
        Opcode::LdEH         => (1, "LD E,H"),
        Opcode::LdEL         => (1, "LD E,L"),
        Opcode::LdEIndHL     => (1, "LD E,(HL)"),
        Opcode::LdEA         => (1, "LD E,A"),
        Opcode::LdHB         => (1, "LD H,B"),
        Opcode::LdHC         => (1, "LD H,C"),
        Opcode::LdHD         => (1, "LD H,D"),
        Opcode::LdHE         => (1, "LD H,E"),
        Opcode::LdHH         => (1, "LD H,H"),
        Opcode::LdHL         => (1, "LD H,L"),
        Opcode::LdHIndHL     => (1, "LD H,(HL)"),
        Opcode::LdHA         => (1, "LD H,A"),
        Opcode::LdLB         => (1, "LD L,B"),
        Opcode::LdLC         => (1, "LD L,C"),
        Opcode::LdLD         => (1, "LD L,D"),
        Opcode::LdLE         => (1, "LD L,E"),
        Opcode::LdLH         => (1, "LD L,H"),
        Opcode::LdLL         => (1, "LD L,L"),
        Opcode::LdLIndHL     => (1, "LD L,(HL)"),
        Opcode::LdLA         => (1, "LD L,A"),
        Opcode::LdIndHLB     => (1, "LD (HL),B"),
        Opcode::LdIndHLC     => (1, "LD (HL),C"),
        Opcode::LdIndHLD     => (1, "LD (HL),D"),
        Opcode::LdIndHLE     => (1, "LD (HL),E"),
        Opcode::LdIndHLH     => (1, "LD (HL),H"),
        Opcode::LdIndHLL     => (1, "LD (HL),L"),
        Opcode::Halt         => (1, "HALT"),
        Opcode::LdIndHLA     => (1, "LD (HL),A"),
        Opcode::LdAB         => (1, "LD A,B"),
        Opcode::LdAC         => (1, "LD A,C"),
        Opcode::LdAD         => (1, "LD A,D"),
        Opcode::LdAE         => (1, "LD A,E"),
        Opcode::LdAH         => (1, "LD A,H"),
        Opcode::LdAL         => (1, "LD A,L"),
        Opcode::LdAIndHL     => (1, "LD A,(HL)"),
        Opcode::LdAA         => (1, "LD A,A"),
        Opcode::AddAB        => (1, "ADD A,B"),
        Opcode::AddAC        => (1, "ADD A,C"),
        Opcode::AddAD        => (1, "ADD A,D"),
        Opcode::AddAE        => (1, "ADD A,E"),
        Opcode::AddAH        => (1, "ADD A,H"),
        Opcode::AddAL        => (1, "ADD A,L"),
        Opcode::AddAIndHL    => (1, "ADD A,(HL)"),
        Opcode::AddAA        => (1, "ADD A,A"),
        Opcode::AdcAB        => (1, "ADC A,B"),
        Opcode::AdcAC        => (1, "ADC A,C"),
        Opcode::AdcAD        => (1, "ADC A,D"),
        Opcode::AdcAE        => (1, "ADC A,E"),
        Opcode::AdcAH        => (1, "ADC A,H"),
        Opcode::AdcAL        => (1, "ADC A,L"),
        Opcode::AdcAIndHL    => (1, "ADC A,(HL)"),
        Opcode::AdcAA        => (1, "ADC A,A"),
        Opcode::SubB         => (1, "SUB B"),
        Opcode::SubC         => (1, "SUB C"),
        Opcode::SubD         => (1, "SUB D"),
        Opcode::SubE         => (1, "SUB E"),
        Opcode::SubH         => (1, "SUB H"),
        Opcode::SubL         => (1, "SUB L"),
        Opcode::SubIndHL     => (1, "SUB (HL)"),
        Opcode::SubA         => (1, "SUB A"),
        Opcode::SbcAB        => (1, "SBC A,B"),
        Opcode::SbcAC        => (1, "SBC A,C"),
        Opcode::SbcAD        => (1, "SBC A,D"),
        Opcode::SbcAE        => (1, "SBC A,E"),
        Opcode::SbcAH        => (1, "SBC A,H"),
        Opcode::SbcAL        => (1, "SBC A,L"),
        Opcode::SbcAIndHL    => (1, "SBC A,(HL)"),
        Opcode::SbcAA        => (1, "SBC A,A"),
        Opcode::AndB         => (1, "AND B"),
        Opcode::AndC         => (1, "AND C"),
        Opcode::AndD         => (1, "AND D"),
        Opcode::AndE         => (1, "AND E"),
        Opcode::AndH         => (1, "AND H"),
        Opcode::AndL         => (1, "AND L"),
        Opcode::AndIndHL     => (1, "AND (HL)"),
        Opcode::AndA         => (1, "AND A"),
        Opcode::XorB         => (1, "XOR B"),
        Opcode::XorC         => (1, "XOR C"),
        Opcode::XorD         => (1, "XOR D"),
        Opcode::XorE         => (1, "XOR E"),
        Opcode::XorH         => (1, "XOR H"),
        Opcode::XorL         => (1, "XOR L"),
        Opcode::XorIndHL     => (1, "XOR (HL)"),
        Opcode::XorA         => (1, "XOR A"),
        Opcode::OrB          => (1, "OR B"),
        Opcode::OrC          => (1, "OR C"),
        Opcode::OrD          => (1, "OR D"),
        Opcode::OrE          => (1, "OR E"),
        Opcode::OrH          => (1, "OR H"),
        Opcode::OrL          => (1, "OR L"),
        Opcode::OrIndHL      => (1, "OR (HL)"),
        Opcode::OrA          => (1, "OR A"),
        Opcode::CpB          => (1, "CP B"),
        Opcode::CpC          => (1, "CP C"),
        Opcode::CpD          => (1, "CP D"),
        Opcode::CpE          => (1, "CP E"),
        Opcode::CpH          => (1, "CP H"),
        Opcode::CpL          => (1, "CP L"),
        Opcode::CpIndHL      => (1, "CP (HL)"),
        Opcode::CpA          => (1, "CP A"),
        Opcode::RetNZ        => (1, "RET NZ"),
        Opcode::PopBC        => (1, "POP BC"),
        Opcode::JpNZa16      => (3, "JP NZ,a16"),
        Opcode::Jpa16        => (3, "JP a16"),
        Opcode::CallNZa16    => (3, "CALL NZ,a16"),
        Opcode::PushBC       => (1, "PUSH BC"),
        Opcode::AddAd8       => (2, "ADD A,u8"),
        Opcode::Rst00H       => (1, "RST 00H"),
        Opcode::RetZ         => (1, "RET Z"),
        Opcode::Ret          => (1, "RET"),
        Opcode::JpZa16       => (3, "JP Z,a16"),
        Opcode::PrefixCB     => (1, "PREFIX CB"),
        Opcode::CallZa16     => (3, "CALL Z,a16"),
        Opcode::Calla16      => (3, "CALL a16"),
        Opcode::AdcAd8       => (2, "ADC A,u8"),
        Opcode::Rst08H       => (1, "RST 08H"),
        Opcode::RetNC        => (1, "RET NC"),
        Opcode::PopDE        => (1, "POP DE"),
        Opcode::JpNCa16      => (3, "JP NC,u16"),
        Opcode::CallNCa16    => (3, "CALL NC,u16"),
        Opcode::PushDE       => (1, "PUSH DE"),
        Opcode::Subd8        => (2, "SUB u8"),
        Opcode::Rst10H       => (1, "RST 10H"),
        Opcode::RetC         => (1, "RET C"),
        Opcode::Reti         => (1, "RETI"),
        Opcode::JpCa16       => (3, "JP C,a16"),
        Opcode::CallCa16     => (3, "CALL C,a16"),
        Opcode::SbcAd8       => (2, "SBC A,u8"),
        Opcode::Rst18H       => (1, "RST 18H"),
        Opcode::LdhInda8A    => (2, "LDH (a8),A"),
        Opcode::PopHL        => (1, "POP HL"),
        Opcode::LdIndCA      => (2, "LD (C),A"),
        Opcode::PushHL       => (1, "PUSH HL"),
        Opcode::Andd8        => (2, "AND u8"),
        Opcode::Rst20H       => (1, "RST 20H"),
        Opcode::AddSPr8      => (2, "ADD SP,i8"),
        Opcode::JpIndHL      => (1, "JP (HL)"),
        Opcode::LdInda16A    => (3, "LD (a16),A"),
        Opcode::Xord8        => (2, "XOR u8"),
        Opcode::Rst28H       => (1, "RST 28H"),
        Opcode::LdhAInda8    => (2, "LDH A,(a8)"),
        Opcode::PopAF        => (1, "POP AF"),
        Opcode::LdAIndC      => (2, "LD A,(C)"),
        Opcode::Di           => (1, "DI"),
        Opcode::PushAF       => (1, "PUSH AF"),
        Opcode::Ord8         => (2, "OR u8"),
        Opcode::Rst30H       => (1, "RST 30H"),
        Opcode::LdHLSPaddr8  => (2, "LD HL,SP+i8"),
        Opcode::LdSPHL       => (1, "LD SP,HL"),
        Opcode::LdAInda16    => (3, "LD A,(a16)"),
        Opcode::Ei           => (1, "EI"),
        Opcode::Cpd8         => (2, "CP u8"),
        Opcode::Rst38H       => (1, "RST 38H")
    }
}

pub fn dbg_cb_opcodes(opcode: CBOpcode) -> &'static str {
    match opcode {
        CBOpcode::RlcB       => "RLC B",
        CBOpcode::RlcC       => "RLC C",
        CBOpcode::RlcD       => "RLC D",
        CBOpcode::RlcE       => "RLC E",
        CBOpcode::RlcH       => "RLC H",
        CBOpcode::RlcL       => "RLC L",
        CBOpcode::RlcIndHL   => "RLC (HL)",
        CBOpcode::RlcA       => "RLC A",
        CBOpcode::RrcB       => "RRC B",
        CBOpcode::RrcC       => "RRC C",
        CBOpcode::RrcD       => "RRC D",
        CBOpcode::RrcE       => "RRC E",
        CBOpcode::RrcH       => "RRC H",
        CBOpcode::RrcL       => "RRC L",
        CBOpcode::RrcIndHL   => "RRC (HL)",
        CBOpcode::RrcA       => "RRC A",
        CBOpcode::RlB        => "RL B",
        CBOpcode::RlC        => "RL C",
        CBOpcode::RlD        => "RL D",
        CBOpcode::RlE        => "RL E",
        CBOpcode::RlH        => "RL H",
        CBOpcode::RlL        => "RL L",
        CBOpcode::RlIndHL    => "RL (HL)",
        CBOpcode::RlA        => "RL A",
        CBOpcode::RrB        => "RR B",
        CBOpcode::RrC        => "RR C",
        CBOpcode::RrD        => "RR D",
        CBOpcode::RrE        => "RR E",
        CBOpcode::RrH        => "RR H",
        CBOpcode::RrL        => "RR L",
        CBOpcode::RrIndHL    => "RR (HL)",
        CBOpcode::RrA        => "RR A",
        CBOpcode::SlaB       => "SLA B",
        CBOpcode::SlaC       => "SLA C",
        CBOpcode::SlaD       => "SLA D",
        CBOpcode::SlaE       => "SLA E",
        CBOpcode::SlaH       => "SLA H",
        CBOpcode::SlaL       => "SLA L",
        CBOpcode::SlaIndHL   => "SLA (HL)",
        CBOpcode::SlaA       => "SLA A",
        CBOpcode::SraB       => "SRA B",
        CBOpcode::SraC       => "SRA C",
        CBOpcode::SraD       => "SRA D",
        CBOpcode::SraE       => "SRA E",
        CBOpcode::SraH       => "SRA H",
        CBOpcode::SraL       => "SRA L",
        CBOpcode::SraIndHL   => "SRA (HL)",
        CBOpcode::SraA       => "SRA A",
        CBOpcode::SwapB      => "SWAP B",
        CBOpcode::SwapC      => "SWAP C",
        CBOpcode::SwapD      => "SWAP D",
        CBOpcode::SwapE      => "SWAP E",
        CBOpcode::SwapH      => "SWAP H",
        CBOpcode::SwapL      => "SWAP L",
        CBOpcode::SwapIndHL  => "SWAP (HL)",
        CBOpcode::SwapA      => "SWAP A",
        CBOpcode::SrlB       => "SRL B",
        CBOpcode::SrlC       => "SRL C",
        CBOpcode::SrlD       => "SRL D",
        CBOpcode::SrlE       => "SRL E",
        CBOpcode::SrlH       => "SRL H",
        CBOpcode::SrlL       => "SRL L",
        CBOpcode::SrlIndHL   => "SRL (HL)",
        CBOpcode::SrlA       => "SRL A",
        CBOpcode::Bit0B      => "BIT 0,B",
        CBOpcode::Bit0C      => "BIT 0,C",
        CBOpcode::Bit0D      => "BIT 0,D",
        CBOpcode::Bit0E      => "BIT 0,E",
        CBOpcode::Bit0H      => "BIT 0,H",
        CBOpcode::Bit0L      => "BIT 0,L",
        CBOpcode::Bit0IndHL  => "BIT 0,(HL)",
        CBOpcode::Bit0A      => "BIT 0,A",
        CBOpcode::Bit1B      => "BIT 1,B",
        CBOpcode::Bit1C      => "BIT 1,C",
        CBOpcode::Bit1D      => "BIT 1,D",
        CBOpcode::Bit1E      => "BIT 1,E",
        CBOpcode::Bit1H      => "BIT 1,H",
        CBOpcode::Bit1L      => "BIT 1,L",
        CBOpcode::Bit1IndHL  => "BIT 1,(HL)",
        CBOpcode::Bit1A      => "BIT 1,A",
        CBOpcode::Bit2B      => "BIT 2,B",
        CBOpcode::Bit2C      => "BIT 2,C",
        CBOpcode::Bit2D      => "BIT 2,D",
        CBOpcode::Bit2E      => "BIT 2,E",
        CBOpcode::Bit2H      => "BIT 2,H",
        CBOpcode::Bit2L      => "BIT 2,L",
        CBOpcode::Bit2IndHL  => "BIT 2,(HL)",
        CBOpcode::Bit2A      => "BIT 2,A",
        CBOpcode::Bit3B      => "BIT 3,B",
        CBOpcode::Bit3C      => "BIT 3,C",
        CBOpcode::Bit3D      => "BIT 3,D",
        CBOpcode::Bit3E      => "BIT 3,E",
        CBOpcode::Bit3H      => "BIT 3,H",
        CBOpcode::Bit3L      => "BIT 3,L",
        CBOpcode::Bit3IndHL  => "BIT 3,(HL)",
        CBOpcode::Bit3A      => "BIT 3,A",
        CBOpcode::Bit4B      => "BIT 4,B",
        CBOpcode::Bit4C      => "BIT 4,C",
        CBOpcode::Bit4D      => "BIT 4,D",
        CBOpcode::Bit4E      => "BIT 4,E",
        CBOpcode::Bit4H      => "BIT 4,H",
        CBOpcode::Bit4L      => "BIT 4,L",
        CBOpcode::Bit4IndHL  => "BIT 4,(HL)",
        CBOpcode::Bit4A      => "BIT 4,A",
        CBOpcode::Bit5B      => "BIT 5,B",
        CBOpcode::Bit5C      => "BIT 5,C",
        CBOpcode::Bit5D      => "BIT 5,D",
        CBOpcode::Bit5E      => "BIT 5,E",
        CBOpcode::Bit5H      => "BIT 5,H",
        CBOpcode::Bit5L      => "BIT 5,L",
        CBOpcode::Bit5IndHL  => "BIT 5,(HL)",
        CBOpcode::Bit5A      => "BIT 5,A",
        CBOpcode::Bit6B      => "BIT 6,B",
        CBOpcode::Bit6C      => "BIT 6,C",
        CBOpcode::Bit6D      => "BIT 6,D",
        CBOpcode::Bit6E      => "BIT 6,E",
        CBOpcode::Bit6H      => "BIT 6,H",
        CBOpcode::Bit6L      => "BIT 6,L",
        CBOpcode::Bit6IndHL  => "BIT 6,(HL)",
        CBOpcode::Bit6A      => "BIT 6,A",
        CBOpcode::Bit7B      => "BIT 7,B",
        CBOpcode::Bit7C      => "BIT 7,C",
        CBOpcode::Bit7D      => "BIT 7,D",
        CBOpcode::Bit7E      => "BIT 7,E",
        CBOpcode::Bit7H      => "BIT 7,H",
        CBOpcode::Bit7L      => "BIT 7,L",
        CBOpcode::Bit7IndHL  => "BIT 7,(HL)",
        CBOpcode::Bit7A      => "BIT 7,A",
        CBOpcode::Res0B      => "RES 0,B",
        CBOpcode::Res0C      => "RES 0,C",
        CBOpcode::Res0D      => "RES 0,D",
        CBOpcode::Res0E      => "RES 0,E",
        CBOpcode::Res0H      => "RES 0,H",
        CBOpcode::Res0L      => "RES 0,L",
        CBOpcode::Res0IndHL  => "RES 0,(HL)",
        CBOpcode::Res0A      => "RES 0,A",
        CBOpcode::Res1B      => "RES 1,B",
        CBOpcode::Res1C      => "RES 1,C",
        CBOpcode::Res1D      => "RES 1,D",
        CBOpcode::Res1E      => "RES 1,E",
        CBOpcode::Res1H      => "RES 1,H",
        CBOpcode::Res1L      => "RES 1,L",
        CBOpcode::Res1IndHL  => "RES 1,(HL)",
        CBOpcode::Res1A      => "RES 1,A",
        CBOpcode::Res2B      => "RES 2,B",
        CBOpcode::Res2C      => "RES 2,C",
        CBOpcode::Res2D      => "RES 2,D",
        CBOpcode::Res2E      => "RES 2,E",
        CBOpcode::Res2H      => "RES 2,H",
        CBOpcode::Res2L      => "RES 2,L",
        CBOpcode::Res2IndHL  => "RES 2,(HL)",
        CBOpcode::Res2A      => "RES 2,A",
        CBOpcode::Res3B      => "RES 3,B",
        CBOpcode::Res3C      => "RES 3,C",
        CBOpcode::Res3D      => "RES 3,D",
        CBOpcode::Res3E      => "RES 3,E",
        CBOpcode::Res3H      => "RES 3,H",
        CBOpcode::Res3L      => "RES 3,L",
        CBOpcode::Res3IndHL  => "RES 3,(HL)",
        CBOpcode::Res3A      => "RES 3,A",
        CBOpcode::Res4B      => "RES 4,B",
        CBOpcode::Res4C      => "RES 4,C",
        CBOpcode::Res4D      => "RES 4,D",
        CBOpcode::Res4E      => "RES 4,E",
        CBOpcode::Res4H      => "RES 4,H",
        CBOpcode::Res4L      => "RES 4,L",
        CBOpcode::Res4IndHL  => "RES 4,(HL)",
        CBOpcode::Res4A      => "RES 4,A",
        CBOpcode::Res5B      => "RES 5,B",
        CBOpcode::Res5C      => "RES 5,C",
        CBOpcode::Res5D      => "RES 5,D",
        CBOpcode::Res5E      => "RES 5,E",
        CBOpcode::Res5H      => "RES 5,H",
        CBOpcode::Res5L      => "RES 5,L",
        CBOpcode::Res5IndHL  => "RES 5,(HL)",
        CBOpcode::Res5A      => "RES 5,A",
        CBOpcode::Res6B      => "RES 6,B",
        CBOpcode::Res6C      => "RES 6,C",
        CBOpcode::Res6D      => "RES 6,D",
        CBOpcode::Res6E      => "RES 6,E",
        CBOpcode::Res6H      => "RES 6,H",
        CBOpcode::Res6L      => "RES 6,L",
        CBOpcode::Res6IndHL  => "RES 6,(HL)",
        CBOpcode::Res6A      => "RES 6,A",
        CBOpcode::Res7B      => "RES 7,B",
        CBOpcode::Res7C      => "RES 7,C",
        CBOpcode::Res7D      => "RES 7,D",
        CBOpcode::Res7E      => "RES 7,E",
        CBOpcode::Res7H      => "RES 7,H",
        CBOpcode::Res7L      => "RES 7,L",
        CBOpcode::Res7IndHL  => "RES 7,(HL)",
        CBOpcode::Res7A      => "RES 7,A",
        CBOpcode::Set0B      => "SET 0,B",
        CBOpcode::Set0C      => "SET 0,C",
        CBOpcode::Set0D      => "SET 0,D",
        CBOpcode::Set0E      => "SET 0,E",
        CBOpcode::Set0H      => "SET 0,H",
        CBOpcode::Set0L      => "SET 0,L",
        CBOpcode::Set0IndHL  => "SET 0,(HL)",
        CBOpcode::Set0A      => "SET 0,A",
        CBOpcode::Set1B      => "SET 1,B",
        CBOpcode::Set1C      => "SET 1,C",
        CBOpcode::Set1D      => "SET 1,D",
        CBOpcode::Set1E      => "SET 1,E",
        CBOpcode::Set1H      => "SET 1,H",
        CBOpcode::Set1L      => "SET 1,L",
        CBOpcode::Set1IndHL  => "SET 1,(HL)",
        CBOpcode::Set1A      => "SET 1,A",
        CBOpcode::Set2B      => "SET 2,B",
        CBOpcode::Set2C      => "SET 2,C",
        CBOpcode::Set2D      => "SET 2,D",
        CBOpcode::Set2E      => "SET 2,E",
        CBOpcode::Set2H      => "SET 2,H",
        CBOpcode::Set2L      => "SET 2,L",
        CBOpcode::Set2IndHL  => "SET 2,(HL)",
        CBOpcode::Set2A      => "SET 2,A",
        CBOpcode::Set3B      => "SET 3,B",
        CBOpcode::Set3C      => "SET 3,C",
        CBOpcode::Set3D      => "SET 3,D",
        CBOpcode::Set3E      => "SET 3,E",
        CBOpcode::Set3H      => "SET 3,H",
        CBOpcode::Set3L      => "SET 3,L",
        CBOpcode::Set3IndHL  => "SET 3,(HL)",
        CBOpcode::Set3A      => "SET 3,A",
        CBOpcode::Set4B      => "SET 4,B",
        CBOpcode::Set4C      => "SET 4,C",
        CBOpcode::Set4D      => "SET 4,D",
        CBOpcode::Set4E      => "SET 4,E",
        CBOpcode::Set4H      => "SET 4,H",
        CBOpcode::Set4L      => "SET 4,L",
        CBOpcode::Set4IndHL  => "SET 4,(HL)",
        CBOpcode::Set4A      => "SET 4,A",
        CBOpcode::Set5B      => "SET 5,B",
        CBOpcode::Set5C      => "SET 5,C",
        CBOpcode::Set5D      => "SET 5,D",
        CBOpcode::Set5E      => "SET 5,E",
        CBOpcode::Set5H      => "SET 5,H",
        CBOpcode::Set5L      => "SET 5,L",
        CBOpcode::Set5IndHL  => "SET 5,(HL)",
        CBOpcode::Set5A      => "SET 5,A",
        CBOpcode::Set6B      => "SET 6,B",
        CBOpcode::Set6C      => "SET 6,C",
        CBOpcode::Set6D      => "SET 6,D",
        CBOpcode::Set6E      => "SET 6,E",
        CBOpcode::Set6H      => "SET 6,H",
        CBOpcode::Set6L      => "SET 6,L",
        CBOpcode::Set6IndHL  => "SET 6,(HL)",
        CBOpcode::Set6A      => "SET 6,A",
        CBOpcode::Set7B      => "SET 7,B",
        CBOpcode::Set7C      => "SET 7,C",
        CBOpcode::Set7D      => "SET 7,D",
        CBOpcode::Set7E      => "SET 7,E",
        CBOpcode::Set7H      => "SET 7,H",
        CBOpcode::Set7L      => "SET 7,L",
        CBOpcode::Set7IndHL  => "SET 7,(HL)",
        CBOpcode::Set7A      => "SET 7,A"
    }
}