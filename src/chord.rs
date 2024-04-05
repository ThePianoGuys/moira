use super::scale::Scale;
use std::rc::Rc;

pub struct Chord {
    name: String,
    scale: Rc<Scale>,
    notes: Vec<u8>,
}
