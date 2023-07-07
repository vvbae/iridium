#[derive(Debug, PartialEq, Clone, Copy)]
/// An 8-bit integer (0 ~ 255)
pub enum Opcode {
    LOAD,
    ADD,
    SUB,
    MUL,
    DIV,
    HLT,
    JMP,
    JMPF,
    JMPB,
    EQ,
    NEQ,
    GTE,
    LTE,
    LT,
    GT,
    JMPE,
    NOP,
    ALOC,
    INC,
    DEC,
    DJMPE,
    IGL,
    PRTS,
    LOADF64,
    ADDF64,
    SUBF64,
    MULF64,
    DIVF64,
    EQF64,
    NEQF64,
    GTF64,
    GTEF64,
    LTF64,
    LTEF64,
    SHL,
    SHR,
    AND,
    OR,
    XOR,
    NOT,
    LUI,
    CLOOP,
    LOOP,
    LOADM,
    SETM,
    PUSH,
    POP,
    CALL,
    RET,
}

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::LOAD,
            1 => Opcode::ADD,
            2 => Opcode::SUB,
            3 => Opcode::MUL,
            4 => Opcode::DIV,
            5 => Opcode::HLT,
            6 => Opcode::JMP,
            7 => Opcode::JMPF,
            8 => Opcode::JMPB,
            9 => Opcode::EQ,
            10 => Opcode::NEQ,
            11 => Opcode::GTE,
            12 => Opcode::LTE,
            13 => Opcode::LT,
            14 => Opcode::GT,
            15 => Opcode::JMPE,
            16 => Opcode::NOP,
            17 => Opcode::ALOC,
            18 => Opcode::INC,
            19 => Opcode::DEC,
            20 => Opcode::DJMPE,
            21 => Opcode::PRTS,
            22 => Opcode::LOADF64,
            23 => Opcode::ADDF64,
            24 => Opcode::SUBF64,
            25 => Opcode::MULF64,
            26 => Opcode::DIVF64,
            27 => Opcode::EQF64,
            28 => Opcode::NEQF64,
            29 => Opcode::GTF64,
            30 => Opcode::GTEF64,
            31 => Opcode::LTF64,
            32 => Opcode::LTEF64,
            33 => Opcode::SHL,
            34 => Opcode::SHR,
            35 => Opcode::AND,
            36 => Opcode::OR,
            37 => Opcode::XOR,
            38 => Opcode::NOT,
            39 => Opcode::LUI,
            40 => Opcode::CLOOP,
            41 => Opcode::LOOP,
            42 => Opcode::LOADM,
            43 => Opcode::SETM,
            44 => Opcode::PUSH,
            45 => Opcode::POP,
            46 => Opcode::CALL,
            47 => Opcode::RET,
            _ => Opcode::IGL,
        }
    }
}

impl<'a> From<&'a str> for Opcode {
    fn from(value: &'a str) -> Self {
        match value {
            "load" => Opcode::LOAD,
            "add" => Opcode::ADD,
            "sub" => Opcode::SUB,
            "mul" => Opcode::MUL,
            "div" => Opcode::DIV,
            "hlt" => Opcode::HLT,
            "jmp" => Opcode::JMP,
            "jmpf" => Opcode::JMPF,
            "jmpb" => Opcode::JMPB,
            "eq" => Opcode::EQ,
            "neq" => Opcode::NEQ,
            "gte" => Opcode::GTE,
            "gt" => Opcode::GT,
            "lte" => Opcode::LTE,
            "lt" => Opcode::LT,
            "jmpe" => Opcode::JMPE,
            "nop" => Opcode::NOP,
            "aloc" => Opcode::ALOC,
            "inc" => Opcode::INC,
            "dec" => Opcode::DEC,
            "djmpe" => Opcode::DJMPE,
            "prts" => Opcode::PRTS,
            "loadf64" => Opcode::LOADF64,
            "addf64" => Opcode::ADDF64,
            "subf64" => Opcode::SUBF64,
            "mulf64" => Opcode::MULF64,
            "divf64" => Opcode::DIVF64,
            "eqf64" => Opcode::EQF64,
            "neqf64" => Opcode::NEQF64,
            "gtf64" => Opcode::GTF64,
            "gtef64" => Opcode::GTEF64,
            "ltf64" => Opcode::LTF64,
            "ltef64" => Opcode::LTEF64,
            "shl" => Opcode::SHL,
            "shr" => Opcode::SHR,
            "and" => Opcode::AND,
            "or" => Opcode::OR,
            "xor" => Opcode::XOR,
            "not" => Opcode::NOT,
            "lui" => Opcode::LUI,
            "cloop" => Opcode::CLOOP,
            "loop" => Opcode::LOOP,
            "loadm" => Opcode::LOADM,
            "setm" => Opcode::SETM,
            "push" => Opcode::PUSH,
            "pop" => Opcode::POP,
            "call" => Opcode::CALL,
            "ret" => Opcode::RET,
            _ => Opcode::IGL,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Instruction {
        Self { opcode }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hlt() {
        let opcode = Opcode::HLT;
        assert_eq!(opcode, Opcode::HLT);
    }

    #[test]
    fn test_create_instruction() {
        let instruction = Instruction::new(Opcode::HLT);
        assert_eq!(instruction.opcode, Opcode::HLT);
    }

    #[test]
    fn test_str_to_opcode() {
        let opcode = Opcode::from("load");
        assert_eq!(opcode, Opcode::LOAD);
        let opcode = Opcode::from("illegal");
        assert_eq!(opcode, Opcode::IGL);
    }
}
