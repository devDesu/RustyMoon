use std::rc::Rc;
#[derive(Debug)]
pub enum TValue {
    NIL,
    TBOOLEAN(bool),
    NUMFLT(f64),
    NUMINT(i64),
    STR(Rc<String>),

}
