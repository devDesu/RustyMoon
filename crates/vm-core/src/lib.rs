pub struct ExecuteError {}

pub trait Context {
    fn display_opcode<O: Opcode>(&self, opcode: &O) -> String;
}

pub trait VM {
    fn step<O: Opcode>(&mut self, opcode: &O) -> Result<(), ExecuteError>;
}

pub trait Opcode: std::fmt::Display {}