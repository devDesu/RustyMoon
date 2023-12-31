## Intro
Lua VM is a register-based virtual machine with garbage collector.
Everything in Lua is TValue object.
Every "function" in lua is a so called "closure"- closure contains pointer to function prototype and static variables


## Entitites
### TValue
Basic type in Lua - stored in stack, could either be immediate "stack" value or heap ptr (both with metadata)

```c
// C

/* Common type for all collectable objects */
typedef struct GCObject {
  struct GCObject *next; 
  lu_byte tt; 
  lu_byte marked;
} GCObject;

typedef struct TValue {
  union {
    struct GCObject *gc;    /* collectable objects */
    void *p;         /* light userdata */
    lua_CFunction f; /* light C functions */
    lua_Integer i;   /* integer numbers */
    lua_Number n;    /* float numbers */
    /* not used, but may avoid warnings for uninitialized value */
    lu_byte ub;
  } value_; 
  lu_byte tt_;
} TValue;
```

Collectable object in C is any heap-based data: strings, tables, closure*, ...
NB!: to be C-compatible TValue size should be 9 bytes. 
NB2!: not really, we can construct C-like structures before C-call, but for the rest cases just use normal rust structures

```rust
// Rust

pub enum TValue {
    NIL,
    TBOOLEAN(bool),
    NUMFLT(f64),
    NUMINT(i64),
    STR(Rc<String>),
    CLOSURE(Rc<Closure>),
    TABLE(Rc<HashTable>)
}
```

TODO!: What's the better way to repr "collectable" objects in rust? Box? Rc?

### Function Prototype
Function prototype contains opcodes, constants, function description and debug info, initialized during input file parsing
```c
// C

typedef struct Proto {
  // collectable header
  struct GCObject *next; 
  lu_byte tt; 
  lu_byte marked;
  
  lu_byte numparams;  /* number of fixed (named) parameters */
  lu_byte is_vararg;
  lu_byte maxstacksize;  /* number of registers needed by this function */
  int sizeupvalues;  /* size of 'upvalues' */
  int sizek;  /* size of 'k' */
  int sizecode;
  int sizelineinfo;
  int sizep;  /* size of 'p' */
  int sizelocvars;
  int sizeabslineinfo;  /* size of 'abslineinfo' */
  int linedefined;  /* debug information  */
  int lastlinedefined;  /* debug information  */
  TValue *k;  /* constants used by the function */
  Instruction *code;  /* opcodes */
  struct Proto **p;  /* functions defined inside the function */
  Upvaldesc *upvalues;  /* upvalue information */
  ls_byte *lineinfo;  /* information about source lines (debug information) */
  AbsLineInfo *abslineinfo;  /* idem */
  LocVar *locvars;  /* information about local variables (debug information) */
  TString  *source;  /* used for debug information */
  GCObject *gclist;
} Proto;
```

Which can be described in rust (omitting some gc-related params and debug info) as
```rust
// Rust

pub struct UpvalueDescription {
    pub instack: bool,
    pub idx: u8,
    pub kind: u8,
    pub name: Option<Box<String>>
}

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
```

Since proto couldn't be constructed in runtime (in normal conditions) we don't really need to implement it as "colectable" object
It could just be `&'static` - init/alloc all protos during file parsing, drop on program exit

### Closure
Proto and Closure are like Class and instance of the Class in OOP. While Proto contains some shared metadata/constants, instance of the Proto (Closure) contains instance-related data: UpValues
UpValues are persistent variables preserved during function call, like `static` but not `global` 

```c
// C

typedef struct CClosure {
  // collectable header
  struct GCObject *next; 
  lu_byte tt; 
  lu_byte marked; 
  
  lu_byte nupvalues; 
  GCObject *gclist;
  lua_CFunction f;
  TValue upvalue[1];  /* list of upvalues */
} CClosure;


typedef struct LClosure {
  // collectable header
  struct GCObject *next; 
  lu_byte tt; 
  lu_byte marked; 
  
  lu_byte nupvalues; 
  GCObject *gclist;
  struct Proto *p;
  UpVal *upvals[1];  /* list of upvalues */
} LClosure;


typedef union Closure {
  CClosure c;
  LClosure l;
} Closure;
```

Since we want to implement C compatibility (maybe C-gate with artificial LuaStack construction approach could help) we want both variants to be handled

```rust
// Rust

pub struct LuaClosure {
    pub proto: &'static Proto,
    pub upvalues: Vec<UpVal>,
}

pub struct CClosure {
    pub fn_ptr: fn() -> (), // todo!: better proto
    pub upvalues: &'static mut TValue,
}

pub enum Closure {
    Lua(LuaClosure),
    C(CClosure),
}
```

### UpValue
AKA closure variables, list of TValues for C closure and UpValue object for Lua closure. 
UpValue could be either `Open` or `Closed`

If open it references stack TValue (stack offset), on upvalue_close call, value from stack offset is copied into UpValue value field ( `UpValue::Closed(stack.get_mut_at_offset(open_upvalue.0))` ), UpVal becomes `closed` and it references TValue in direct way (*TValue)

All upvalues are added to open_upvalues field of thread_state

```c
// C

typedef struct UpVal {
  // collectable header
  struct GCObject *next; 
  lu_byte tt; 
  lu_byte marked;

  union {
    TValue *p;  /* points to stack or to its own value */
    ptrdiff_t offset;  /* used while the stack is being reallocated */
  } v;
  union {
    struct {  /* (when open) */
      struct UpVal *next;  /* linked list */
      struct UpVal **previous;
    } open;
    TValue value;  /* the value (when closed) */
  } u;
} UpVal;
```


```rust
// Rust
type StackOffset = u16;

pub struct UpVal {
    Closed(&'static mut TValue),
    Open(StackOffset)
}
```

### Thread state
In Lua thread state contains A LOT of information: mentioned upvalues, debug-related info and call infos

```c
// C

/*
** 'per thread' state
*/
struct lua_State {
  CommonHeader; // collectable object header
  lu_byte status;
  lu_byte allowhook;
  unsigned short nci;  /* number of items in call info list */
  StkIdRel top;  /* first free slot in the stack */
  global_State *l_G;
  CallInfo *ci;  /* call info for current function */
  StkIdRel stack_last;  /* end of stack (last element + 1) */
  StkIdRel stack;  /* stack base */
  UpVal *openupval;  /* list of open upvalues in this stack */
  StkIdRel tbclist;  /* list of to-be-closed variables */
  GCObject *gclist;
  struct lua_State *twups;  /* list of threads with open upvalues */
  struct lua_longjmp *errorJmp;  /* current error recover point */
  CallInfo base_ci;  /* CallInfo for first level (C calling Lua) */
  volatile lua_Hook hook;
  ptrdiff_t errfunc;  /* current error handling function (stack index) */
  l_uint32 nCcalls;  /* number of nested (non-yieldable | C)  calls */
  int oldpc;  /* last pc traced */
  int basehookcount;
  int hookcount;
  volatile l_signalT hookmask;
};
```

Most of the information are used for debug purposes, in rust we could omit most of the fields and create nice structure

```rust
// Rust

pub enum ThreadMode {
    Running,
    Stopped,
}

pub struct LuaThread {
    pub stack: stack::LuaStack,
    pub current_call: LinkedList<CallInfo>, // linked list of CallInfo frames, current call should be on top
    pub mode: ThreadMode,
    pub open_upvalues: LinkedList<UpVal>,
    pub top: usize, // stack current top ptr
    // todo!: read about C calls, yields and error recovery
}
```

### Lua stack
Stack was mentioned earlier multiple times because it's the main part of the Lua VM.
Despite that Lua is register-based VM everything in Lua is done using stack.
Stack is an array of `TValue`s, it's initialized per-thread

| Item                                                	| Index                                                  	| Referenced by       	|
|-----------------------------------------------------	|--------------------------------------------------------	|---------------------	|
| ...                                                 	|                                                        	|                     	|
| top of the function                                 	| 2 + var arg count + fixed args count + registers count 	| `call_info.top`     	|
| function registers                                  	| 2+varArg count + fixed args count                      	|                     	|
| `TValue` - first fixed fn arg                       	| 2+varArg count                                         	| `call_info.base`    	|
| `TValue` - variable fn args                         	| ...                                                    	|                     	|
| `TValue::Closure(Closure::LuaClosure(lua_closure))` 	| 2                                                      	| `call_info.closure` 	|
| ...                                                 	| 1                                                      	|                     	|
| stack base                                          	| 0                                                      	|                     	|

## Execution flow
1. Parse input file, init function protos
2. Prepare thread instance - alloc stack, init root closure, push closure to the stack
3. start execution
 - 3.1 on `call` opcode init new closure, push it to the stack, initialize new call_info, prepare call frame (varargs, etc), add call_info to thread call_info list, finish step
 - 3.2 on `return` opcode clear call_frame, place return values (if any) on the stack starting from closure ptr 
 - 3.3 `vararg` stuff modifies stack! Variable arguments are placed after closure ptr, fixed args are placed after, local variables/fn registers are placed after fixed args and stack top is being updated

 ## Entities relationsships
[![Test](./relationships.drawio.svg)](https://app.diagrams.net/?mode=github#HdevDesu%2FRustyMoon%2Fmaster%2Frelationships.drawio.svg)