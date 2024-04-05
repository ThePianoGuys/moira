use std::rc::Rc;

use super::key::{NamedKey, NamedNote, Note};

// A key in a scale
#[derive(Clone)]
pub struct ScaleKey {
    scale: Rc<Scale>,
    position: i8,
}

impl ScaleKey {
    pub fn new(scale: Rc<Scale>, position: i8) -> Self {
        Self { scale, position }
    }
    pub fn to_named_key(&self) -> NamedKey {
        self.scale.get_named_key(self.position)
    }
}

// A note in a scale
#[derive(Clone)]
pub struct ScaleNote {
    scale: Rc<Scale>,
    position: i8,
    octave: i8,
}
impl ScaleNote {
    pub fn new(scale: Rc<Scale>, position: i8, octave: i8) -> Self {
        Self {
            scale,
            position,
            octave,
        }
    }
    pub fn to_named_note(&self) -> NamedNote {
        self.scale.get_named_note(self.position, self.octave)
    }
}

pub struct Scale {
    start: NamedKey,         // starting note of the scale: 0 is C, 11 is B
    offsets: Vec<i8>,        // the offsets of the scale
    elements: Vec<NamedKey>, // Will be filled in at struct initialization.
}

impl Scale {
    pub fn new(start: NamedKey, offsets: Vec<i8>) -> Self {
        let (base_key, _) = start.get_components();
        let keys_in_order = base_key.get_keys_in_order();
        let mut elements = Vec::<NamedKey>::new();

        for (i, offset) in offsets.iter().enumerate() {
            let key = start.to_key() + offset;
            let possible_named_keys = key.get_named_keys();
            let mut possible_named_keys_iter = possible_named_keys.iter();

            // Ensure keys are always with non-duplicate names in the scale (e.g. A, B, C#, D, etc.)
            let named_key = loop {
                let named_key_option = *possible_named_keys_iter.next().unwrap();
                let named_key = named_key_option.unwrap();
                if named_key.base_key == keys_in_order[i] {
                    break named_key;
                }
            };

            elements.push(named_key)
        }

        Self {
            start,
            offsets,
            elements,
        }
    }
    fn get_named_key(&self, position: i8) -> NamedKey {
        let len: i8 = self.offsets.len().try_into().unwrap();
        let index_usize: usize = (position % len).try_into().unwrap();
        self.elements[index_usize]
    }
    fn get_named_note(&self, position: i8, octave: i8) -> NamedNote {
        let len: i8 = self.offsets.len().try_into().unwrap();
        let index = position % len;
        let additional_octaves = (position - index) / len;
        let index_usize: usize = index.try_into().unwrap();
        let note = Note::compose(self.start.to_key(), octave + additional_octaves)
            + &self.offsets[index_usize];
        let (_, octave) = note.decompose();
        NamedNote::new(self.elements[index_usize], octave)
    }
    pub fn get_scale(&self) -> &Vec<i8> {
        &self.offsets
    }
}
