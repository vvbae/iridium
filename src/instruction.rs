#[derive(Debug, PartialEq, Clone, Copy)]
/// An 8-bit integer (0 ~ 255)
pub enum Opcode {
    LOAD,
    ADD,
    SUB,
    MUL,
    DIV,
    HLT, // halt
    JMP,
    JMPF,
    JMPB,
    EQ,
    NEQ,
    GT,
    LT,
    GTQ,
    LTQ,
    JEQ, // jump if equal
    IGL, // illegal
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
            // "gte" => Opcode::GTE,
            "gt" => Opcode::GT,
            // "lte" => Opcode::LTE,
            "lt" => Opcode::LT,
            // "jmpe" => Opcode::JMPE,
            // "nop" => Opcode::NOP,
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
