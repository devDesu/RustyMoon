use std::{io::{Read, BufReader}, rc::Rc};
use std::fmt::Write;

use crate::core::types::TValue;

use super::{opcodes::{LuaInstruction, LuaOpcode, self}, types::{UpvalueDescription, Proto}};

struct LuaReader<R: Read> {
    reader: R,
    int_size: u8,
    number_size: u8,
}

impl<R: Read> LuaReader<R> {
    fn new(reader: R, int_size: u8, number_size: u8 ) -> Self {
        LuaReader { reader, int_size, number_size }
    }

    pub fn load_byte(&mut self) -> u8 {
        let mut result = [0u8];
        self.reader.read(&mut result).expect("Failed to read");
        result[0]
    }

    pub fn read_unsigned(&mut self, limit: u64) -> u64 {
        let mut x = 0_u64;
        let mut b = 0_u8;
        loop {
            b = self.load_byte();
            if x >= limit { panic!("integer overflow") };
            x = (x << 7) | (b & 0x7f) as u64;
            if (b & 0x80) != 0 { break; }
        }
        x
    }

    pub fn read_size(&mut self) -> u64 {
        self.read_unsigned(u64::MAX - 1)
    }

    pub fn read_instruction(&mut self) -> u32 {
        let mut bytes: [u8; 4] = [0, 0, 0, 0];
        self.reader.read(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    pub fn read_number(&mut self) -> f64 {
        match self.number_size {
            4 => {
                let mut bytes: [u8; 4] = [0, 0, 0, 0];
                self.reader.read(&mut bytes);
                f32::from_le_bytes(bytes) as f64
            }
            8 => {
                let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                self.reader.read(&mut bytes);
                f64::from_le_bytes(bytes)
            }
            _ => { panic!("Bad num size") },
        }
    }

    pub fn read_integer(&mut self) -> i64 {
        match self.int_size {
            4 => {
                let mut bytes: [u8; 4] = [0, 0, 0, 0];
                self.reader.read(&mut bytes);
                i32::from_le_bytes(bytes) as i64
            }
            8 => {
                let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                self.reader.read(&mut bytes);
                i64::from_le_bytes(bytes)
            }
            _ => { panic!("Bad int size") },
        }
    }

    pub fn read_int(&mut self) -> i64 {
        self.read_unsigned(u64::MAX) as i64
    }

    pub fn read_string(&mut self) -> Option<Rc<String>> {
        const MAXSHORTLEN: usize = 40;
        let str_size = self.read_size() as usize;

        if str_size == 0 {
            None
        } else { 
            //let mut s = String::with_capacity(str_size - 1);
            //let raw_s_ptr = unsafe { s.as_bytes_mut() };
            let mut empty_buf = vec![0u8; str_size - 1];
            let got_bytes = self.reader.read(&mut empty_buf).expect("Failed to read exact");
            assert_eq!(got_bytes, { str_size-1 });
            Some(Rc::new(String::from_utf8(empty_buf).expect("Failed to convert bytes to str")))
        }
    }

    pub fn read_code(&mut self) -> Vec<LuaInstruction> {
        let opcodes_count = self.read_int();
        
        let mut result: Vec<LuaInstruction> = Vec::new();
        for _ in 0..opcodes_count {
            let opcode = opcodes::decode(self.read_instruction());
            result.push(opcode);
        }

        result
    }

    pub fn read_protos(&mut self) -> Vec<Proto> {
        let proto_count = self.read_int();
        let mut result: Vec<Proto> = Vec::new();
        for _ in 0..proto_count {
            result.push(self.read_function());
        }

        result
    }

    pub fn read_upvalues(&mut self) -> Vec<UpvalueDescription> {
        let upval_count = self.read_int();
        let mut upvals: Vec<UpvalueDescription> = Vec::new();
        for _ in 0..upval_count {
            let instack = self.load_byte() != 0;
            let idx = self.load_byte();
            let kind = self.load_byte();
            upvals.push( UpvalueDescription {
                name: None,
                instack,
                idx,
                kind
            } )
        }

        upvals
    }

    pub fn read_constants(&mut self) -> Vec<TValue> {
        let const_count = self.read_int();

        /*
        * #define LUA_TNIL                0
            #define LUA_TBOOLEAN            1
            #define LUA_TLIGHTUSERDATA      2
            #define LUA_TNUMBER             3
            #define LUA_TSTRING             4
            #define LUA_TTABLE              5
            #define LUA_TFUNCTION           6
            #define LUA_TUSERDATA           7
            #define LUA_TTHREAD             8

            #define LUA_NUMTYPES            9

        */
        let mut result: Vec<TValue> = Vec::new();
        for _ in 0..const_count {
            let const_type = self.load_byte();
            let const_val = match const_type {
                0 => { TValue::NIL },
                1 => TValue::TBOOLEAN(true),
                17 => TValue::TBOOLEAN(false),
                19 => TValue::NUMFLT(self.read_number()),
                3 => TValue::NUMINT(self.read_integer()),
                4 | 20 => TValue::STR(self.read_string().unwrap()),
                _ => panic!("Unknown type"),
            };
            result.push(const_val);
        }

        result
    }

    fn read_function(&mut self) -> Proto {
        let fn_name = self.read_string();
        let line_defined = self.read_int() as usize;
        let last_line_defined = self.read_int() as usize;
        let num_params = self.load_byte();
        let is_vararg = self.load_byte() == 1;
        let max_stack_size = self.load_byte();
    
        let opcodes = self.read_code();
        let constants = self.read_constants();
        let upvals = self.read_upvalues();
        let protos = self.read_protos();
    
    
        
    
        Proto {
            fn_name,
            is_vararg,
            max_stack_size,
            num_params,
            upvalues: upvals,
            code: opcodes,
            constants,
            fns: protos,
        }
    }
    
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Header {
    signature: [u8; 4],
    version: u8,
    format: u8,
    data_chunk: [u8; 6],
    instruction_size: u8,
    int_size: u8,
    number_size: u8,
}

pub fn parse_header(data: &[u8]) -> Result<(usize, Header), std::io::Error> {    
    let mut offset = 0;
    const HEADER_SIZE: usize = std::mem::size_of::<Header>();
    let p: *const [u8; HEADER_SIZE] = data.as_ptr() as *const [u8; HEADER_SIZE];                                                                          
    let header: Header = unsafe { std::mem::transmute(*p)};
    
    //reader.consume(HEADER_SIZE);
    //reader.fill_buf().unwrap();

    offset += HEADER_SIZE;
    let mut l_reader = LuaReader {
        int_size: header.int_size,
        number_size: header.number_size,
        reader: BufReader::new(&data[offset..])
    };

    assert_eq!(header.signature, *b"\x1bLua");
    assert_eq!(header.data_chunk, *b"\x19\x93\r\n\x1a\n");
    
    let test_int_value = l_reader.read_integer();
    assert_eq!(test_int_value, 0x5678);

    let test_f_value = l_reader.read_number();
    assert_eq!(test_f_value, 370.5f64);

    offset += header.number_size as usize + header.int_size as usize;

    Result::Ok((offset, header))

    }

pub fn parse_all(data: &[u8]) -> Proto {
    let (offset, header) = parse_header(data).unwrap();
    let mut lua_reader = LuaReader::new(BufReader::new(&data[offset..]), header.int_size, header.number_size);
    let _upvalues = lua_reader.load_byte();
    let func = lua_reader.read_function();
    println!("{:?}", func);
    func
}


impl Proto {
    pub fn display_opcode(&self, opcode: &LuaInstruction) -> String {
        let mut result = String::new();
        write!(&mut result, "{:?}; ", opcode.opcode).expect("Failed to write");
        let err = match opcode.opcode {
            LuaOpcode::LOADI_AsBx => write!(&mut result, "R[{}] = {}", opcode.args.get_A(), opcode.args.get_sBx()),
            LuaOpcode::LOADK_ABx => write!(&mut result, "R[{}] = Const[{}] ({})", opcode.args.get_A(), opcode.args.get_Bx(), self.constants.get(opcode.args.get_sBx() as usize).unwrap()),
        //Opcode::OP_ADD(abc) => write!(&mut result, "R[{}] = R[{}] + R[{}]", abc.0, abc.2, abc.3),
        //Opcode::OP_LOADK(abx) => write!(&mut result, "R[{}] = Const[{}] ({})", abx.0, abx.1, constants.get(abx.1 as usize).unwrap()),
        //Opcode::OP_SETTABUP(abc) => write!(&mut result, "UpValue({})[{}] = {}", upvalues.get(abc.0 as usize).unwrap(), constants.get(abc.2 as usize).unwrap(), constants.get(abc.3 as usize).unwrap()),
           _ => write!(&mut result, "TODO"),
        };
        err.expect("Failed to write");
        result
    }
}

impl std::fmt::Display for Proto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "Fn: {:#?}, {} params, vararg: {}, max stack: {}\nUpvalues: {}\nConstants: {}\nPrototypes: {}\nCode: {}", 
            self.fn_name,
            self.num_params, 
            self.is_vararg, 
            self.max_stack_size,
            self.upvalues.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join("\n"),
            self.constants.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join("\n"),
            self.fns.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join("\n"),
            self.code.iter().map( |x| { self.display_opcode(x) } ).collect::<Vec<String>>().join("\n")
        )
    }
}