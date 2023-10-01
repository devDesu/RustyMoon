use core::fmt;



/*===========================================================================
  We assume that instructions are unsigned 32-bit integers.
  All instructions have an opcode in the first 7 bits.
  Instructions can have the following formats:

        3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1 0 0 0 0 0 0 0 0 0 0
        1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
iABC          C(8)     |      B(8)     |k|     A(8)      |   Op(7)     |
iABx                Bx(17)               |     A(8)      |   Op(7)     |
iAsBx              sBx (signed)(17)      |     A(8)      |   Op(7)     |
iAx                           Ax(25)                     |   Op(7)     |
isJ                           sJ (signed)(25)            |   Op(7)     |

  A signed argument is represented in excess K: the represented value is
  the written unsigned value minus K, where K is half the maximum for the
  corresponding unsigned argument.
===========================================================================*/
const IX_BITSIZE: usize = 32;
const OPCODE_BITSIZE: usize = 7;
const A_ARG_POS: usize = OPCODE_BITSIZE;
const A_ARG_SIZE: usize = 8;
const B_ARG_POS: usize = OPCODE_BITSIZE + A_ARG_SIZE + 1;
const B_ARG_SIZE: usize = A_ARG_SIZE;
const C_ARG_POS: usize = B_ARG_POS + B_ARG_SIZE;
const C_ARG_SIZE: usize = A_ARG_SIZE; 
const BX_ARG_POS: usize = OPCODE_BITSIZE + A_ARG_SIZE;
const BX_ARG_SIZE: usize = IX_BITSIZE - OPCODE_BITSIZE - A_ARG_SIZE;
const AX_ARG_POS: usize = A_ARG_POS;
const AX_ARG_SIZE: usize = IX_BITSIZE - OPCODE_BITSIZE;
const SJ_ARG_POS: usize = A_ARG_POS;
const SJ_ARG_SIZE: usize = AX_ARG_SIZE;

macro_rules! bitmask {
    ($x:expr) => {
        (1 << $x) - 1
    };
}

macro_rules! get_operand {
    ($data:expr, $shift:expr, $size:expr) => {
        (($data) >> $shift) & bitmask!($size)
    };
}

// #[rustc_layout_scalar_valid_range_end(0b11111111111111111)]
type u17 = u32;
type i17 = i32;
type u25 = u32;
type i25 = i32;

#[derive(Debug)]
pub struct LuaArgs {
    raw: u32
}

impl LuaArgs {
    pub fn get_A(&self) -> u8 {
        get_operand!(self.raw, A_ARG_POS, A_ARG_SIZE) as u8
    }
    
    pub fn get_Ax(&self) -> u25 {
        get_operand!(self.raw, AX_ARG_POS, AX_ARG_SIZE) as u25
    }

    pub fn get_B(&self) -> u8 {
        get_operand!(self.raw, B_ARG_POS, B_ARG_SIZE) as u8
    }

    pub fn get_sB(&self) -> i8 {
        let b = get_operand!(self.raw, B_ARG_POS, B_ARG_SIZE) as u8;
        const K: i32 = 0xff >> 1;
        (b as i32 - K) as i8
    }

    pub fn get_Bx(&self) -> u17 {
        get_operand!(self.raw, BX_ARG_POS, BX_ARG_SIZE) as u17
    }

    pub fn get_sBx(&self) -> i17 {
        let bx = self.get_Bx();
        
        const K: i32 = 0x1ffff >> 1; // half max for u17
        (bx as i32 - K) as i17
    }

    pub fn get_C(&self) -> u8 {
        get_operand!(self.raw, C_ARG_POS, C_ARG_SIZE) as u8
    }

    pub fn get_sC(&self) -> i8 {
        let c = self.get_C();
        const K: i32 = 0xff >> 1;
        (c as i32 - K) as i8
    }

    pub fn get_k(&self) -> bool {
        (self.raw >> (B_ARG_POS - 1) & 1) != 0
    }

    pub fn get_sJ(&self) -> i25 {
        let uj = get_operand!(self.raw, SJ_ARG_POS, SJ_ARG_SIZE) as u25;

        const K: i32 = 0x1ffffff >> 1_i32;
        (uj as i32 - K) as i25 
    }

}

/*
** Grep "ORDER OP" if you change these enums. Opcodes marked with a (*)
** has extra descriptions in the notes after the enumeration.
*/

#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
pub enum LuaOpcode {
    /*----------------------------------------------------------------------
      name          args    description
    ------------------------------------------------------------------------*/
    MOVE_AB=0,/*      A B     R[A] := R[B]                                    */
    LOADI_AsBx,/*     A sBx   R[A] := sBx                                     */
    LOADF_AsBx,/*     A sBx   R[A] := (lua_Number)sBx                         */
    LOADK_ABx,/*     A Bx    R[A] := K[Bx]                                   */
    LOADKX_A,/*    A       R[A] := K[extra arg]                            */
    LOADFALSE_A,/* A       R[A] := false                                   */
    LFALSESKIP_A,/*A       R[A] := false; pc++     (*)                     */
    LOADTRUE_A,/*  A       R[A] := true                                    */
    LOADNIL_ABC,/*   A B     R[A], R[A+1], ..., R[A+B] := nil                */
    GETUPVAL_AB,/*  A B     R[A] := UpValue[B]                              */
    SETUPVAL_AB,/*  A B     UpValue[B] := R[A]                              */
    GETTABUP_AB,/*  A B C   R[A] := UpValue[B][K[C]:string]                 */
    GETTABLE_ABC,/*  A B C   R[A] := R[B][R[C]]                              */
    GETI_ABC,/*      A B C   R[A] := R[B][C]                                 */
    GETFIELD_ABC,/*  A B C   R[A] := R[B][K[C]:string]                       */
    SETTABUP_ABC,/*  A B C   UpValue[A][K[B]:string] := RK(C)                */
    SETTABLE_ABC,/*  A B C   R[A][R[B]] := RK(C)                             */
    SETI_ABC,/*      A B C   R[A][B] := RK(C)                                */
    SETFIELD_ABC,/*  A B C   R[A][K[B]:string] := RK(C)                      */
    NEWTABLE_ABCk,/*  A B C k R[A] := {}                                      */
    SELF_ABC,/*      A B C   R[A+1] := R[B]; R[A] := R[B][RK(C):string]      */
    ADDI_ABsC,/*      A B sC  R[A] := R[B] + sC                               */
    ADDK_ABC,/*      A B C   R[A] := R[B] + K[C]:number                      */
    SUBK_ABC,/*      A B C   R[A] := R[B] - K[C]:number                      */
    MULK_ABC,/*      A B C   R[A] := R[B] * K[C]:number                      */
    MODK_ABC,/*      A B C   R[A] := R[B] % K[C]:number                      */
    POWK_ABC,/*      A B C   R[A] := R[B] ^ K[C]:number                      */
    DIVK_ABC,/*      A B C   R[A] := R[B] / K[C]:number                      */
    IDIVK_ABC,/*     A B C   R[A] := R[B] // K[C]:number                     */
    BANDK_ABC,/*     A B C   R[A] := R[B] & K[C]:integer                     */
    BORK_ABC,/*      A B C   R[A] := R[B] | K[C]:integer                     */
    BXORK_ABC,/*     A B C   R[A] := R[B] ~ K[C]:integer                     */
    SHRI_ABsC,/*      A B sC  R[A] := R[B] >> sC                              */
    SHLI_ABsC,/*      A B sC  R[A] := sC << R[B]                              */
    ADD_ABC,/*       A B C   R[A] := R[B] + R[C]                             */
    SUB_ABC,/*       A B C   R[A] := R[B] - R[C]                             */
    MUL_ABC,/*       A B C   R[A] := R[B] * R[C]                             */
    MOD_ABC,/*       A B C   R[A] := R[B] % R[C]                             */
    POW_ABC,/*       A B C   R[A] := R[B] ^ R[C]                             */
    DIV_ABC,/*       A B C   R[A] := R[B] / R[C]                             */
    IDIV_ABC,/*      A B C   R[A] := R[B] // R[C]                            */
    BAND_ABC,/*      A B C   R[A] := R[B] & R[C]                             */
    BOR_ABC,/*       A B C   R[A] := R[B] | R[C]                             */
    BXOR_ABC,/*      A B C   R[A] := R[B] ~ R[C]                             */
    SHL_ABC,/*       A B C   R[A] := R[B] << R[C]                            */
    SHR_ABC,/*       A B C   R[A] := R[B] >> R[C]                            */
    MMBIN_ABC,/*     A B C   call C metamethod over R[A] and R[B]    (*)     */
    MMBINI_AsBCk,/*    A sB C k        call C metamethod over R[A] and sB      */
    MMBINK_ABCk,/*    A B C k         call C metamethod over R[A] and K[B]    */
    UNM_AB,/*       A B     R[A] := -R[B]                                   */
    BNOT_AB,/*      A B     R[A] := ~R[B]                                   */
    NOT_AB,/*       A B     R[A] := not R[B]                                */
    LEN_AB,/*       A B     R[A] := #R[B] (length operator)                 */
    CONCAT_AB,/*    A B     R[A] := R[A].. ... ..R[A + B - 1]               */
    CLOSE_A,/*     A       close all upvalues >= R[A]                      */
    TBC_A,/*       A       mark variable A "to be closed"                  */
    JMP_sJ,/*       sJ      pc += sJ                                        */
    EQ_ABk,/*        A B k   if ((R[A] == R[B]) ~= k) then pc++              */
    LT_ABk,/*        A B k   if ((R[A] <  R[B]) ~= k) then pc++              */
    LE_ABk,/*        A B k   if ((R[A] <= R[B]) ~= k) then pc++              */
    EQK_ABk,/*       A B k   if ((R[A] == K[B]) ~= k) then pc++              */
    EQI_AsBk,/*       A sB k  if ((R[A] == sB) ~= k) then pc++                */
    LTI_AsBk,/*       A sB k  if ((R[A] < sB) ~= k) then pc++                 */
    LEI_AsBk,/*       A sB k  if ((R[A] <= sB) ~= k) then pc++                */
    GTI_AsBk,/*       A sB k  if ((R[A] > sB) ~= k) then pc++                 */
    GEI_AsBk,/*       A sB k  if ((R[A] >= sB) ~= k) then pc++                */
    TEST_Ak,/*      A k     if (not R[A] == k) then pc++                    */
    TESTSET_ABk,/*   A B k   if (not R[B] == k) then pc++ else R[A] := R[B] (*) */
    CALL_ABC,/*      A B C   R[A], ... ,R[A+C-2] := R[A](R[A+1], ... ,R[A+B-1]) */
    TAILCALL_ABCk,/*  A B C k return R[A](R[A+1], ... ,R[A+B-1])              */
    RETURN_ABCk,/*    A B C k return R[A], ... ,R[A+B-2]      (see note)      */
    RETURN0,/*           return                                          */
    RETURN1_A,/*   A       return R[A]                                     */
    FORLOOP_ABx,/*   A Bx    update counters; if loop continues then pc-=Bx; */
    FORPREP_ABx,/*   A Bx    <check values and prepare counters>;
                            if not to run then pc+=Bx+1;                    */
    TFORPREP_ABx,/*  A Bx    create upvalue for R[A + 3]; pc+=Bx             */
    TFORCALL_AC,/*  A C     R[A+4], ... ,R[A+3+C] := R[A](R[A+1], R[A+2]);  */
    TFORLOOP_ABx,/*  A Bx    if R[A+2] ~= nil then { R[A]=R[A+2]; pc -= Bx } */
    SETLIST_ABCk,/*   A B C k R[A][C+i] := R[A+i], 1 <= i <= B                */
    CLOSURE_ABx,/*   A Bx    R[A] := closure(KPROTO[Bx])                     */
    VARARG_AC,/*    A C     R[A], R[A+1], ..., R[A+C-2] = vararg            */
    VARARGPREP_A,/*A       (adjust vararg parameters)                      */
    EXTRAARG_Ax/*   Ax      extra (larger) argument for previous opcode     */
}

impl From<u8> for LuaOpcode {
    fn from(value: u8) -> Self {
        if value > 82 { panic!("Invalid opcode ix") };
        unsafe { core::mem::transmute::<u8, LuaOpcode>(value) }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    opcode: u32,
    ix: u8,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse opcode: {:04X}, unknown ix: {}", self.opcode, self.ix)
    }
}

#[derive(Debug)]
pub struct LuaInstruction {
    pub opcode: LuaOpcode,
    pub args: LuaArgs,
}

impl std::fmt::Display for LuaInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.opcode)
    }
}

pub fn decode(raw: u32) -> LuaInstruction {
        let ix = (raw & bitmask!(OPCODE_BITSIZE)) as u8;
        LuaInstruction {
            opcode: ix.try_into().expect("Failed to convert u8 to opcode"),
            args: LuaArgs { raw },
        }
    }

#[cfg(test)]
mod test {
    use crate::core::opcodes::{LuaOpcode, decode};

    #[test]
    fn decode_move() {
        let op = decode(0x42030000_u32);
        assert_eq!(op.opcode, LuaOpcode::MOVE_AB);
        assert_eq!(op.args.get_A(), 0);
        assert_eq!(op.args.get_B(), 3);
    }
}