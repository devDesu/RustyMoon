use std::{io::{Read, BufReader}, rc::Rc};

use crate::core::{opcodes, types::TValue};

use super::opcodes::Opcode;

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
        u32::from_le_bytes(bytes) as u32
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
            assert_eq!(got_bytes, (str_size-1) as usize);
            Some(Rc::new(String::from_utf8(empty_buf).expect("Failed to convert bytes to str")))
        }
    }

    pub fn read_code(&mut self) -> Vec<Opcode> {
        println!("Reading code");
        let opcodes_count = self.read_int();
        println!("Opcodes: {opcodes_count}");

        let mut result: Vec<Opcode> = Vec::new();
        for _ in 0..opcodes_count {
            let opcode = opcodes::decode(self.read_instruction()).expect("Failed to parse opcode");
            println!("{opcode:?}");
            result.push(opcode);
        }

        result
    }

    pub fn read_protos(&mut self) {
        println!("Reading protos");
        let proto_count = self.read_int();
        println!("Proto count {proto_count}");
    }

    pub fn read_upvalues(&mut self) -> Vec<Upvalue> {
        let upval_count = self.read_int();
        println!("Upvals {upval_count}");

        let mut upvals: Vec<Upvalue> = Vec::new();
        for _ in 0..upval_count {
            let instack = if self.load_byte() == 0 { false } else { true };
            let idx = self.load_byte();
            let kind = self.load_byte();
            upvals.push( Upvalue {
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
        println!("Constants {const_count}");

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
    

    println!("{:?}", header);
    
    let test_int_value = l_reader.read_integer();
    assert_eq!(test_int_value, 0x5678);

    let test_f_value = l_reader.read_number();
    assert_eq!(test_f_value, 370.5f64);

    offset += header.number_size as usize + header.int_size as usize;

    Result::Ok((offset, header))

    }

pub fn parse_all(data: &[u8]) {
    let (offset, header) = parse_header(data).unwrap();
    let mut lua_reader = LuaReader::new(BufReader::new(&data[offset..]), header.int_size, header.number_size);
    let upvalues = lua_reader.load_byte();
    let func = read_function(&mut lua_reader);
    println!("{:?}", func);
}

#[derive(Debug)]
struct FnInfo {
    fn_name: Option<Rc<String>>,
    line_defined: i64,
    last_line_defined: i64,
    num_params: u8,
    is_vararg: bool,
    max_stack_size: u8,
    upvalues: Vec<Upvalue>,
    opcodes: Vec<Opcode>,
    constants: Vec<TValue>,
}

fn read_function<R: Read>(reader: &mut LuaReader<R>) -> FnInfo {
    let fn_name = reader.read_string();
    let line_defined = reader.read_int();
    let last_line_defined = reader.read_int();
    let num_params = reader.load_byte();
    let is_vararg = reader.load_byte() == 1;
    let max_stack_size = reader.load_byte();

    println!("Fn name: {fn_name:?}");

    let opcodes = reader.read_code();
    let constants = reader.read_constants();
    let upvals = reader.read_upvalues();
    reader.read_protos();


    let fn_info = FnInfo {
        fn_name,
        line_defined,
        last_line_defined,
        is_vararg,
        max_stack_size,
        num_params,
        upvalues: upvals,
        opcodes,
        constants,
    };

    fn_info
}

#[derive(Debug)]
struct Upvalue {
    instack: bool,
    idx: u8,
    kind: u8,
    name: Option<Rc<String>>
}
