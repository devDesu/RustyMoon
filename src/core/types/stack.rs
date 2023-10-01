use super::{TValue, CallInfo, StackIndex};

/**
 * Stack is an array of TValues
 */
pub struct LuaStack {
    raw: Vec<TValue>,
    top: usize,
}

impl LuaStack {
    pub fn new(&self, capacity: usize) -> Self {
        Self {
            top: 0,
            raw: Vec::<TValue>::with_capacity(capacity),
        }
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn pop(&mut self) {
        self.top -= 1;
    }

    pub fn push(&mut self, value: TValue) {
        self.raw[self.top] = value;
        self.top += 1;
    }

    pub fn set_at_offset(&mut self, value: TValue, offset: StackIndex) {
        self.raw[offset] = value;
    }

    pub fn get_at_offset(&self, offset: StackIndex) -> &TValue {
        self.raw.get(offset).unwrap()
    }

    pub fn get_at_offset_mut(&mut self, offset: StackIndex) -> &mut TValue {
        self.raw.get_mut(offset).unwrap()
    }
}

type MutStackRef<'s> = &'s mut LuaStack;
pub struct LuaStackView<'stack> {
    stack: &'stack mut LuaStack,
    base: usize,
    top: usize,
    num_args: u8,
    num_varargs: usize,
}

impl<'stack> LuaStackView<'stack> {

    pub fn new(stack: &'stack mut LuaStack, call_info: &CallInfo) -> Self {
        let closure = call_info.get_lua_closure(stack);
        Self { stack, base: call_info.base, top: call_info.top, num_varargs: call_info.nextraargs, num_args: closure.proto.num_params }
    }

    pub fn set_register(self, register: StackIndex, value: TValue) {
        self.stack.set_at_offset(value, self.base + register as usize);
    }

    pub fn get_register(&self, register: StackIndex) -> &TValue {
        self.stack.get_at_offset(self.base + register as usize)
    }

    pub fn get_register_mut(&mut self, register: StackIndex) -> &mut TValue {
        self.stack.get_at_offset_mut(register)
    }

    pub fn get_arg(&self, arg: usize) -> &TValue {  panic!("")  }

    pub fn set_return_values(&mut self, values: &[TValue]) {

    }
}