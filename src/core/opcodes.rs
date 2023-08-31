use core::fmt;
use std::error::Error;

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
const SBX_ARG_POS: usize = BX_ARG_POS;
const SBX_ARG_SIZE: usize = IX_BITSIZE - OPCODE_BITSIZE - A_ARG_SIZE;
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

type u17 = u32;
type i17 = i32;
type u25 = u32;
type i25 = i32;

pub trait Args {
    fn decode(raw: u32) -> Self;
}

#[derive(PartialEq, Debug)]
struct ABC (u8, bool, u8, u8);
impl Args for ABC {
    fn decode(raw: u32) -> Self {
        let a = get_operand!(raw, A_ARG_POS, A_ARG_SIZE) as u8;
        let k = (raw >> (B_ARG_POS - 1) & 1) != 0;
        let b = get_operand!(raw, B_ARG_POS, B_ARG_SIZE) as u8;
        let c = get_operand!(raw, C_ARG_POS, C_ARG_SIZE) as u8;
        ABC(a, k, b, c)
    }
}

#[derive(PartialEq, Debug)]
struct ABx (u8, u17);
impl Args for ABx {
    fn decode(raw: u32) -> Self {
        let a = get_operand!(raw, A_ARG_POS, A_ARG_SIZE) as u8;
        let bx = get_operand!(raw, BX_ARG_POS, BX_ARG_SIZE) as u17;
        ABx(a, bx)
    }
}

#[derive(PartialEq, Debug)]
struct AsBx (u8, i17);
impl Args for AsBx {
    fn decode(raw: u32) -> Self {
        let a = get_operand!(raw, A_ARG_POS, A_ARG_SIZE) as u8;
        let bx = get_operand!(raw, SBX_ARG_POS, SBX_ARG_SIZE) as u17;
        fn to_signed(unsigned: u17) -> i17 {
            const K: i32 = 0x1ffff >> 1 as i32; // half max for u17

            unsigned as i32 - K
        }

        AsBx(a, to_signed(bx))
    }
}

#[derive(PartialEq, Debug)]
struct Ax (u25);

impl Args for Ax {
    fn decode(raw: u32) -> Self {
        Ax(get_operand!(raw, AX_ARG_POS, AX_ARG_SIZE) as u25)
    }
}

#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types)]
struct sJ (i25);

impl Args for sJ {
    fn decode(raw: u32) -> Self {
        sJ(get_operand!(raw, SJ_ARG_POS, SJ_ARG_SIZE) as i25)
    }
}

#[allow(non_camel_case_types)]
pub enum OpcodeMode {
    ABC(ABC), // { A: u8, B: u9, C: u9 },
    ABx(ABx),
    AsBx(AsBx),
    Ax(Ax),
    sJ(sJ),
}


/*
** Grep "ORDER OP" if you change these enums. Opcodes marked with a (*)
** has extra descriptions in the notes after the enumeration.
*/

#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    /*----------------------------------------------------------------------
      name          args    description
    ------------------------------------------------------------------------*/
    OP_MOVE(ABC),/*      A B     R[A] := R[B]                                    */
    OP_LOADI(AsBx),/*     A sBx   R[A] := sBx                                     */
    OP_LOADF(AsBx),/*     A sBx   R[A] := (lua_Number)sBx                         */
    OP_LOADK(ABx),/*     A Bx    R[A] := K[Bx]                                   */
    OP_LOADKX(ABC),/*    A       R[A] := K[extra arg]                            */
    OP_LOADFALSE(ABC),/* A       R[A] := false                                   */
    OP_LFALSESKIP(ABC),/*A       R[A] := false; pc++     (*)                     */
    OP_LOADTRUE(ABC),/*  A       R[A] := true                                    */
    OP_LOADNIL,/*   A B     R[A], R[A+1], ..., R[A+B] := nil                */
    OP_GETUPVAL,/*  A B     R[A] := UpValue[B]                              */
    OP_SETUPVAL(ABC),/*  A B     UpValue[B] := R[A]                              */
    OP_GETTABUP(ABC),/*  A B C   R[A] := UpValue[B][K[C]:string]                 */
    OP_GETTABLE(ABC),/*  A B C   R[A] := R[B][R[C]]                              */
    OP_GETI(ABC),/*      A B C   R[A] := R[B][C]                                 */
    OP_GETFIELD(ABC),/*  A B C   R[A] := R[B][K[C]:string]                       */
    OP_SETTABUP(ABC),/*  A B C   UpValue[A][K[B]:string] := RK(C)                */
    OP_SETTABLE(ABC),/*  A B C   R[A][R[B]] := RK(C)                             */
    OP_SETI(ABC),/*      A B C   R[A][B] := RK(C)                                */
    OP_SETFIELD(ABC),/*  A B C   R[A][K[B]:string] := RK(C)                      */
    OP_NEWTABLE(ABC),/*  A B C k R[A] := {}                                      */
    OP_SELF(ABC),/*      A B C   R[A+1] := R[B]; R[A] := R[B][RK(C):string]      */
    OP_ADDI,/*      A B sC  R[A] := R[B] + sC                               */
    OP_ADDK(ABC),/*      A B C   R[A] := R[B] + K[C]:number                      */
    OP_SUBK(ABC),/*      A B C   R[A] := R[B] - K[C]:number                      */
    OP_MULK(ABC),/*      A B C   R[A] := R[B] * K[C]:number                      */
    OP_MODK(ABC),/*      A B C   R[A] := R[B] % K[C]:number                      */
    OP_POWK(ABC),/*      A B C   R[A] := R[B] ^ K[C]:number                      */
    OP_DIVK(ABC),/*      A B C   R[A] := R[B] / K[C]:number                      */
    OP_IDIVK(ABC),/*     A B C   R[A] := R[B] // K[C]:number                     */
    OP_BANDK(ABC),/*     A B C   R[A] := R[B] & K[C]:integer                     */
    OP_BORK(ABC),/*      A B C   R[A] := R[B] | K[C]:integer                     */
    OP_BXORK(ABC),/*     A B C   R[A] := R[B] ~ K[C]:integer                     */
    OP_SHRI,/*      A B sC  R[A] := R[B] >> sC                              */
    OP_SHLI,/*      A B sC  R[A] := sC << R[B]                              */
    OP_ADD(ABC),/*       A B C   R[A] := R[B] + R[C]                             */
    OP_SUB(ABC),/*       A B C   R[A] := R[B] - R[C]                             */
    OP_MUL(ABC),/*       A B C   R[A] := R[B] * R[C]                             */
    OP_MOD(ABC),/*       A B C   R[A] := R[B] % R[C]                             */
    OP_POW(ABC),/*       A B C   R[A] := R[B] ^ R[C]                             */
    OP_DIV(ABC),/*       A B C   R[A] := R[B] / R[C]                             */
    OP_IDIV(ABC),/*      A B C   R[A] := R[B] // R[C]                            */
    OP_BAND(ABC),/*      A B C   R[A] := R[B] & R[C]                             */
    OP_BOR(ABC),/*       A B C   R[A] := R[B] | R[C]                             */
    OP_BXOR(ABC),/*      A B C   R[A] := R[B] ~ R[C]                             */
    OP_SHL(ABC),/*       A B C   R[A] := R[B] << R[C]                            */
    OP_SHR(ABC),/*       A B C   R[A] := R[B] >> R[C]                            */
    OP_MMBIN(ABC),/*     A B C   call C metamethod over R[A] and R[B]    (*)     */
    OP_MMBINI,/*    A sB C k        call C metamethod over R[A] and sB      */
    OP_MMBINK(ABC),/*    A B C k         call C metamethod over R[A] and K[B]    */
    OP_UNM(ABC),/*       A B     R[A] := -R[B]                                   */
    OP_BNOT(ABC),/*      A B     R[A] := ~R[B]                                   */
    OP_NOT(ABC),/*       A B     R[A] := not R[B]                                */
    OP_LEN(ABC),/*       A B     R[A] := #R[B] (length operator)                 */
    OP_CONCAT(ABC),/*    A B     R[A] := R[A].. ... ..R[A + B - 1]               */
    OP_CLOSE(ABC),/*     A       close all upvalues >= R[A]                      */
    OP_TBC(ABC),/*       A       mark variable A "to be closed"                  */
    OP_JMP(sJ),/*       sJ      pc += sJ                                        */
    OP_EQ(ABC),/*        A B k   if ((R[A] == R[B]) ~= k) then pc++              */
    OP_LT(ABC),/*        A B k   if ((R[A] <  R[B]) ~= k) then pc++              */
    OP_LE(ABC),/*        A B k   if ((R[A] <= R[B]) ~= k) then pc++              */
    OP_EQK(ABC),/*       A B k   if ((R[A] == K[B]) ~= k) then pc++              */
    OP_EQI,/*       A sB k  if ((R[A] == sB) ~= k) then pc++                */
    OP_LTI,/*       A sB k  if ((R[A] < sB) ~= k) then pc++                 */
    OP_LEI,/*       A sB k  if ((R[A] <= sB) ~= k) then pc++                */
    OP_GTI,/*       A sB k  if ((R[A] > sB) ~= k) then pc++                 */
    OP_GEI,/*       A sB k  if ((R[A] >= sB) ~= k) then pc++                */
    OP_TEST(ABC),/*      A k     if (not R[A] == k) then pc++                    */
    OP_TESTSET(ABC),/*   A B k   if (not R[B] == k) then pc++ else R[A] := R[B] (*) */
    OP_CALL(ABC),/*      A B C   R[A], ... ,R[A+C-2] := R[A](R[A+1], ... ,R[A+B-1]) */
    OP_TAILCALL(ABC),/*  A B C k return R[A](R[A+1], ... ,R[A+B-1])              */
    OP_RETURN(ABC),/*    A B C k return R[A], ... ,R[A+B-2]      (see note)      */
    OP_RETURN0(ABC),/*           return                                          */
    OP_RETURN1(ABC),/*   A       return R[A]                                     */
    OP_FORLOOP(ABx),/*   A Bx    update counters; if loop continues then pc-=Bx; */
    OP_FORPREP(ABx),/*   A Bx    <check values and prepare counters>;
                            if not to run then pc+=Bx+1;                    */
    OP_TFORPREP(ABx),/*  A Bx    create upvalue for R[A + 3]; pc+=Bx             */
    OP_TFORCALL(ABC),/*  A C     R[A+4], ... ,R[A+3+C] := R[A](R[A+1], R[A+2]);  */
    OP_TFORLOOP(ABx),/*  A Bx    if R[A+2] ~= nil then { R[A]=R[A+2]; pc -= Bx } */
    OP_SETLIST(ABC),/*   A B C k R[A][C+i] := R[A+i], 1 <= i <= B                */
    OP_CLOSURE(ABx),/*   A Bx    R[A] := closure(KPROTO[Bx])                     */
    OP_VARARG(ABC),/*    A C     R[A], R[A+1], ..., R[A+C-2] = vararg            */
    OP_VARARGPREP(ABC),/*A       (adjust vararg parameters)                      */
    OP_EXTRAARG(Ax)/*   Ax      extra (larger) argument for previous opcode     */
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

pub fn decode(opcode: u32) -> Result<Opcode, ParseError> {
        // get first 6 bits
        let ix = (opcode & bitmask!(OPCODE_BITSIZE)) as u8;
        match ix {
            0 => Ok(Opcode::OP_MOVE(ABC::decode(opcode))),
            1 => Ok(Opcode::OP_LOADI(AsBx::decode(opcode))),
            3 => Ok(Opcode::OP_LOADK(ABx::decode(opcode))),
            5 => Ok(Opcode::OP_LOADFALSE(ABC::decode(opcode))),
            6 => Ok(Opcode::OP_LFALSESKIP(ABC::decode(opcode))),
            7 => Ok(Opcode::OP_LOADTRUE(ABC::decode(opcode))),
            15 => Ok(Opcode::OP_SETTABUP(ABC::decode(opcode))),
            34 => Ok(Opcode::OP_ADD(ABC::decode(opcode))),
            46 => Ok(Opcode::OP_MMBIN(ABC::decode(opcode))),
            70 => Ok(Opcode::OP_RETURN(ABC::decode(opcode))),
            81 => Ok(Opcode::OP_VARARGPREP(ABC::decode(opcode))),
            _ => Err(ParseError { opcode, ix }),
        }
    }

#[cfg(test)]
mod test {
    use crate::core::opcodes::{Opcode, ABx, decode};

    #[test]
    fn decode_loadfalse() {
        assert_eq!(decode(u32::from_be_bytes([3, 0x42, 0, 0])).unwrap(), Opcode::OP_LOADK(ABx(0, 0x42)));
    }
}