use std::rc::Rc;

use crate::core::{types::{Closure, TValue, LuaThread, stack::LuaStackView, StackIndex, CallInfo}, opcodes::LuaOpcode};

pub struct LuaVm ();

enum Order {
    Less,
    LessOrEqual,
    Equal,
    Greater,
    GreaterOrEqual,
}

fn compare(left: &TValue, right: &TValue, k: bool, mode: Order) -> usize {
    let mut result = 0_usize;
    match mode {
        Order::Less => {
            if let (TValue::NUMINT(l_val), TValue::NUMINT(r_val)) = (left, right) {
                result = if (l_val < r_val) != k { 1 } else { 0 };
            } else if let (TValue::NUMFLT(l_val), TValue::NUMFLT(r_val)) = (left, right) {
                result = if (l_val < r_val) != k { 1 } else { 0 };
            } else {
                panic!("Don't know how to compare otjer types yet")
            }
        },
        Order::LessOrEqual => {
            if let (TValue::NUMINT(l_val), TValue::NUMINT(r_val)) = (left, right) {
                result = if (l_val <= r_val) != k { 1 } else { 0 };
            } else if let (TValue::NUMFLT(l_val), TValue::NUMFLT(r_val)) = (left, right) {
                result = if (l_val <= r_val) != k { 1 } else { 0 };
            } else {
                panic!("Don't know how to compare otjer types yet")
            }
        }
        Order::Equal => {
            if let (TValue::NUMINT(l_val), TValue::NUMINT(r_val)) = (left, right) {
                result = if (l_val == r_val) != k { 1 } else { 0 };
            } else if let (TValue::NUMFLT(l_val), TValue::NUMFLT(r_val)) = (left, right) {
                result = if (l_val == r_val) != k { 1 } else { 0 };
            } else {
                panic!("Don't know how to compare otjer types yet")
            }
        },
        Order::Greater => {
            if let (TValue::NUMINT(l_val), TValue::NUMINT(r_val)) = (left, right) {
                result = if (l_val > r_val) != k { 1 } else { 0 };
            } else if let (TValue::NUMFLT(l_val), TValue::NUMFLT(r_val)) = (left, right) {
                result = if (l_val > r_val) != k { 1 } else { 0 };
            } else {
                panic!("Don't know how to compare otjer types yet")
            }
        },
        Order::GreaterOrEqual => {
            if let (TValue::NUMINT(l_val), TValue::NUMINT(r_val)) = (left, right) {
                result = if (l_val >= r_val) != k { 1 } else { 0 };
            } else if let (TValue::NUMFLT(l_val), TValue::NUMFLT(r_val)) = (left, right) {
                result = if (l_val >= r_val) != k { 1 } else { 0 };
            } else {
                panic!("Don't know how to compare otjer types yet")
            }
        },
    };

    result
}

impl LuaVm {

    pub fn step(&mut self, thread: &mut LuaThread) {
        let call_info = thread.current_call.front_mut().unwrap();
        let lua_closure = match call_info.get_closure(&thread.stack) {
            Closure::Lua(lua_closure) => { lua_closure },
            Closure::C(_) => { panic!("C closure shouldn't occur here") }
        };

        let instruction = lua_closure.proto.code.get(call_info.pc).expect("No more opcodes");
        let mut frame = LuaStackView::new(&mut thread.stack, &mut call_info);
        call_info.pc += 1;
        match instruction.opcode {
            LuaOpcode::MOVE_AB => {
                frame.set_register(instruction.args.get_A().into(), frame.get_register(instruction.args.get_B().into()).clone());
            },
            LuaOpcode::LOADI_AsBx => {
                frame.set_register(instruction.args.get_A().into(), TValue::NUMINT(instruction.args.get_sBx().into()));
            },
            LuaOpcode::LOADF_AsBx => {
                frame.set_register(instruction.args.get_A().into(), TValue::NUMFLT(instruction.args.get_sBx().into()));
            },
            LuaOpcode::LOADK_ABx => {
                frame.set_register(instruction.args.get_A().into(), lua_closure.proto.constants.get(instruction.args.get_Bx() as usize).unwrap().clone());
            },
            LuaOpcode::LOADKX_A => {
                let extra_arg = lua_closure.proto.code.get(call_info.pc).unwrap();
                let val = lua_closure.proto.constants.get(extra_arg.args.get_Ax() as usize).unwrap();
                frame.set_register(instruction.args.get_A().into(), val.clone());
                call_info.pc +=1;
            },
            LuaOpcode::LOADFALSE_A => {
                frame.set_register(instruction.args.get_A().into(), TValue::TBOOLEAN(false));
            },
            LuaOpcode::LFALSESKIP_A => 
            {
                frame.set_register(instruction.args.get_A().into(), TValue::TBOOLEAN(false));
                call_info.pc += 1;
            },
            LuaOpcode::LOADTRUE_A => {
                frame.set_register(instruction.args.get_A().into(), TValue::TBOOLEAN(true));
            },
            LuaOpcode::LOADNIL_ABC => {
                let count = instruction.args.get_B();
                let base = instruction.args.get_A() as StackIndex;
                for i in 0..count {
                    frame.set_register(base + (i as StackIndex), TValue::NIL);
                }
            },
            LuaOpcode::GETUPVAL_AB => {
                let upval = lua_closure.upvalues.get(instruction.args.get_B() as usize).unwrap();
                frame.set_register(instruction.args.get_A().into(), *upval.get_value(&thread.stack));
            },
            LuaOpcode::SETUPVAL_AB => {
                let val = frame.get_register_mut(instruction.args.get_A().into());
                lua_closure.upvalues[instruction.args.get_B() as usize].set_value(&mut thread.stack, val);
            },
            LuaOpcode::ADDI_ABsC => {
                let immediate = instruction.args.get_sC();
                let dst = instruction.args.get_A();
                let tval = frame.get_register(instruction.args.get_B().into());
                let result = match tval {
                    TValue::NUMFLT(f) => TValue::NUMFLT(f + immediate as f64),
                    TValue::NUMINT(i) => TValue::NUMINT(i + immediate as i64),
                    _ => {
                        todo!("Add meta methods for arith support")
                    }
                };
                frame.set_register(dst.into(), result);
            },
            LuaOpcode::JMP_sJ => {
                call_info.pc = (call_info.pc as i64 + instruction.args.get_sJ() as i64) as usize;
            },
            LuaOpcode::EQ_ABk => {
                // if ((R[A] == R[B]) ~= k) then pc++
                
                call_info.pc += compare(
                    frame.get_register(instruction.args.get_A().into()),
                    frame.get_register(instruction.args.get_B().into()),
                    instruction.args.get_k(),
                    Order::Equal
                );
            },
            LuaOpcode::LT_ABk => {
                call_info.pc += compare(
                    frame.get_register(instruction.args.get_A().into()),
                    frame.get_register(instruction.args.get_B().into()),
                    instruction.args.get_k(),
                    Order::Less
                );
            },
            LuaOpcode::LE_ABk => {
                call_info.pc += compare(
                    frame.get_register(instruction.args.get_A().into()),
                    frame.get_register(instruction.args.get_B().into()),
                    instruction.args.get_k(),
                    Order::LessOrEqual
                );
            },
            LuaOpcode::EQK_ABk => {
                call_info.pc += compare(
                    frame.get_register(instruction.args.get_A().into()),
                    lua_closure.proto.constants.get(instruction.args.get_B() as usize).unwrap(),
                    instruction.args.get_k(),
                    Order::Equal
                );
            },
            LuaOpcode::EQI_AsBk => {
                let left = frame.get_register(instruction.args.get_A().into());
                let right = instruction.args.get_sB();
                let k = instruction.args.get_k();

                if let TValue::NUMINT(val_int) = left {
                    call_info.pc += if (*val_int == (right as i64)) != k { 1 } else { 0 }    
                } else if let TValue::NUMFLT(val_flt) = left {
                    call_info.pc += if (*val_flt == (right as f64)) != k { 1 } else { 0 }
                } else {
                    todo!();
                }
            },
            LuaOpcode::LTI_AsBk => {
                let left = frame.get_register(instruction.args.get_A().into());
                let right = instruction.args.get_sB();
                let k = instruction.args.get_k();

                if let TValue::NUMINT(val_int) = left {
                    call_info.pc += if (*val_int < (right as i64)) != k { 1 } else { 0 }    
                } else if let TValue::NUMFLT(val_flt) = left {
                    call_info.pc += if (*val_flt < (right as f64)) != k { 1 } else { 0 }
                } else {
                    todo!();
                }
            },
            LuaOpcode::LEI_AsBk => {
                let left = frame.get_register(instruction.args.get_A().into());
                let right = instruction.args.get_sB();
                let k = instruction.args.get_k();

                if let TValue::NUMINT(val_int) = left {
                    call_info.pc += if (*val_int <= (right as i64)) != k { 1 } else { 0 }    
                } else if let TValue::NUMFLT(val_flt) = left {
                    call_info.pc += if (*val_flt <= (right as f64)) != k { 1 } else { 0 }
                } else {
                    todo!();
                }
            },
            LuaOpcode::GTI_AsBk => {
                let left = frame.get_register(instruction.args.get_A().into());
                let right = instruction.args.get_sB();
                let k = instruction.args.get_k();

                if let TValue::NUMINT(val_int) = left {
                    call_info.pc += if (*val_int > (right as i64)) != k { 1 } else { 0 }    
                } else if let TValue::NUMFLT(val_flt) = left {
                    call_info.pc += if (*val_flt > (right as f64)) != k { 1 } else { 0 }
                } else {
                    todo!();
                }
            },
            LuaOpcode::GEI_AsBk => {
                let left = frame.get_register(instruction.args.get_A().into());
                let right = instruction.args.get_sB();
                let k = instruction.args.get_k();

                if let TValue::NUMINT(val_int) = left {
                    call_info.pc += if (*val_int >= (right as i64)) != k { 1 } else { 0 }    
                } else if let TValue::NUMFLT(val_flt) = left {
                    call_info.pc += if (*val_flt >= (right as f64)) != k { 1 } else { 0 }
                } else {
                    todo!();
                }
            },
            LuaOpcode::TEST_Ak => {
                // if (not R[A] == k) then pc++
                let left = frame.get_register(instruction.args.get_A().into());
                let k = instruction.args.get_k();
                let result = match left {
                    TValue::NIL => true,
                    TValue::TBOOLEAN(b) => !b,
                    _ => true, 
                };
                if result != k {
                    call_info.pc += 1;
                }
            },
            LuaOpcode::CALL_ABC => {
                let fn_idx = instruction.args.get_A() as StackIndex;
                let number_of_args = instruction.args.get_B() as usize;
                let nresults = instruction.args.get_C() - 1;
                if number_of_args != 0 {
                    thread.top = fn_idx + number_of_args;
                }
                let closure = match frame.get_register(fn_idx) {
                    TValue::CLOSURE(closure) => {
                        closure.as_ref()
                    },
                    _ => panic!("Expected TValue at offset to be closure"),
                };
                match closure {
                    Closure::C(c_closure) => {
                        // Implement C-gate here
                        (c_closure.fn_ptr)();
                    },
                    Closure::Lua(lua_closure) => {
                        let mut new_call_info = CallInfo::new_lua(lua_closure.proto, fn_idx);
                        new_call_info.nresults = nresults.into();
                        thread.current_call.push_front(new_call_info);
                    }
                }

                /*
                Proto *p = clLvalue(s2v(func))->p;
                int narg = cast_int(L->top.p - func) - 1;  /* number of real arguments */
                int nfixparams = p->numparams;
                int fsize = p->maxstacksize;  /* frame size */
                checkstackGCp(L, fsize, func);
                CallInfo *ci = L->ci = next_ci(L);  /* new frame */
                ci->func.p = func;
                ci->nresults = nresults;
                ci->callstatus = 0;
                ci->top.p = func + 1 + fsize;
                ci->u.l.savedpc = p->code;  /* starting point */
                for (; narg < nfixparams; narg++)
                    setnilvalue(s2v(L->top.p++));  /* complete missing arguments */
                lua_assert(ci->top.p <= L->stack_last.p);
                return ci;
                 */
            },
            LuaOpcode::TAILCALL_ABCk => {

            },
            LuaOpcode::RETURN0 => {
                // pop call info
                thread.current_call.pop_front();
                // empty stack frame
                todo!();
            },
            LuaOpcode::RETURN1_A => {
                // pop call info
                thread.current_call.pop_front();
                // empty stack frame
                todo!();
                // set return values
            },
            LuaOpcode::CLOSURE_ABx => {
                let proto = lua_closure.proto.fns.get(instruction.args.get_Bx() as usize).unwrap();
                let mut new_closure = Closure::new_lua(proto);
                thread.stack.set_at_offset(TValue::CLOSURE(Rc::new(new_closure)), instruction.args.get_A().into());
                new_closure.init_upvalues(proto.upvalues, lua_closure.upvalues);
            }
            LuaOpcode::VARARG_AC => {
                let n = instruction.args.get_C() as i64 - 1;
                let wanted = call_info.nextraargs;
                let count = if n < 0 { wanted } else { n as usize };
                let wh = instruction.args.get_A() as usize;
                for i in 0..wanted {
                    match i < count {
                        true =>  todo!("vararg"), //self.stack.set_at_offset(value, wh + i),
                        false => frame.set_register(wh + i, TValue::NIL),
                    };
                };
            },
            LuaOpcode::VARARGPREP_A => {
                //ProtectNT(luaT_adjustvarargs(L, GETARG_A(i), ci, cl->p));
                //updatebase(ci);  /* function has new base after adjustment */ 
                
                let nfixparams = instruction.args.get_A();
                let total_args_count = thread.top - call_info.fn_idx;
                let vararg_count = total_args_count - 1 - nfixparams as usize;
                call_info.nextraargs = vararg_count;
                
                thread.stack.set_at_offset(thread.stack.get_at_offset(call_info.fn_idx).clone(), thread.top);
                thread.top += 1;

                for i in 1..=nfixparams as usize {
                    thread.stack.set_at_offset(thread.stack.get_at_offset(call_info.fn_idx + i).clone(), thread.top);
                    thread.stack.set_at_offset(TValue::NIL, call_info.fn_idx + i);
                    thread.top += 1;
                }

                call_info.fn_idx = total_args_count;
                call_info.top = total_args_count;

                
                
            },
            LuaOpcode::EXTRAARG_Ax => {
                panic!("That shouldn't be executed");
            }
            _ => todo!("Not implemented")
        }
    }

    pub fn init(&mut self) {

    }
}