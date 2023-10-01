pub mod stack;
use std::{rc::Rc, collections::LinkedList};

use self::stack::LuaStack;

use super::opcodes::LuaInstruction;

pub type StackIndex = usize;

#[derive(Debug, Clone)]
pub enum TValue {
    NIL,
    TBOOLEAN(bool),
    NUMFLT(f64),
    NUMINT(i64),
    STR(Rc<String>),
    CLOSURE(Rc<Closure>),
    EMPTY,
}

impl std::fmt::Display for TValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TValue::NIL => write!(f, "Nil"),
            TValue::TBOOLEAN(val) => write!(f, "TBoolean({})", val),
            TValue::NUMFLT(flt) => write!(f, "Float({})", flt),
            TValue::NUMINT(i) => write!(f, "Int({})", i),
            TValue::STR(s) => write!(f, "Str({})", s),
            TValue::CLOSURE(c) => write!(f, "Closure:\n{:?}", c),
            TValue::EMPTY => write!(f, "Empty _system_ value"),
        }
    }
}

#[derive(Debug)]
pub struct UpvalueDescription {
    pub instack: bool,
    pub idx: u8,
    pub kind: u8,
    pub name: Option<Box<String>>
}

impl std::fmt::Display for UpvalueDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "In stack: {}, at index {}, kind: {}, name: {:#?}", self.instack, self.idx, self.kind, self.name)
    }
}

#[derive(Debug)]
pub struct Proto {
    pub fn_name: Option<Rc<String>>,
    pub num_params: u8,  /* number of fixed (named) parameters */
    pub is_vararg: bool,
    pub max_stack_size: u8, /* number of registers needed by this function */
    pub constants: Vec<TValue>,  /* constants used by the function */
    pub code: Vec<LuaInstruction>,  /* opcodes */
    pub fns: Vec<Proto>,  /* functions defined inside the function */
    pub upvalues: Vec<UpvalueDescription>,  /* upvalue information */
}

#[derive(Debug)]
pub enum  UpVal {
    Open(StackIndex),
    Closed(&'static mut TValue),
}

impl UpVal {
    pub fn new(stack_index: StackIndex) -> Self {
        Self::Open(stack_index)
    }

    pub fn get_value(&self, stack: &stack::LuaStack) -> &TValue {
        match self {
            Self::Open(offset) => stack.get_at_offset(*offset),
            Self::Closed(val) => val,
        }
    } 

    pub fn set_value(&mut self, stack: &mut stack::LuaStack, value: &mut TValue) -> () {
        match self {
            Self::Open(offset) => stack.set_at_offset(value.clone(), *offset),
            Self::Closed(old) => {
                *old = value;
            }
        }
    }
}

#[derive(Debug)]
pub struct LuaClosure {
    pub proto: &'static Proto,
    pub upvalues: Vec<UpVal>,
}

#[derive(Debug)]
pub struct CClosure {
    pub fn_ptr: fn() -> (),
    pub upvalues: Vec<TValue>,
}

#[derive(Debug)]
pub enum Closure {
    Lua(LuaClosure),
    C(CClosure),
}


impl Closure {
    pub fn new_lua(proto: &'static Proto) -> Self {
        Closure::Lua(LuaClosure { proto, upvalues: Vec::with_capacity(proto.upvalues.len()), })
    }

    pub fn init_upvalues(&mut self, upval_descrs: Vec<UpvalueDescription>, parent_upvalues: Vec<UpVal>) {
        for (i, descr) in upval_descrs.iter().enumerate() {
            if descr.instack {
                todo!("Local upvalues are not supported yet");
            } else {
                self.set_upvalue(i, *parent_upvalues.get(descr.idx as usize).unwrap());
            }
        }
    }

    pub fn set_upvalue(&mut self, index: usize, value: UpVal) {
        match self {
            Closure::C(c_closure) => todo!(),
            Closure::Lua(lua_closure) => { lua_closure.upvalues[index] = value; }
        }
    } 
}

pub enum ThreadMode {
    Running,
    Stopped,
}

pub struct LuaThread {
    pub stack: stack::LuaStack,
    pub current_call: LinkedList<CallInfo>,
    pub mode: ThreadMode,
    pub open_upvalues: LinkedList<UpVal>,
    pub top: StackIndex, // stack current top ptr
}

pub struct CallInfo {
    pub nresults: i16,
    pub nextraargs: usize,
    pub base: StackIndex,
    pub top: StackIndex,
    pub fn_idx: StackIndex,
    pub pc: usize,
}

impl CallInfo {
    pub fn new_lua(proto: &Proto, fn_idx: StackIndex) -> Self {
        let lua_closure = Closure::new_lua(proto);
        Self {
            nextraargs: 0,
            fn_idx,
            pc: 0,
            base: 0,
            nresults: 0,
            top: fn_idx + 1 + proto.max_stack_size as usize,
        }
    }

    pub fn get_closure<'a, 'stack>(&'a self, stack: &'stack LuaStack) -> &'stack Closure {
        let value_at_index = stack.get_at_offset(self.fn_idx);
        match value_at_index {
            TValue::CLOSURE(closure) => {
                closure.as_ref()
            },
            _ => { panic!("CallInfo points to {:?} instead of closure", value_at_index) }
        }
    }

    pub fn get_lua_closure<'a, 'stack>(&self, stack: &'stack LuaStack) -> &'stack LuaClosure {
        match self.get_closure(stack) {
            Closure::Lua(closure) => {
                closure
            },
            _ => {
                panic!("Trying to get non-lua closure");
            }
        }
    }
}