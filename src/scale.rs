use std::rc::Rc;

use log::warn;

use super::key::{BaseKey, Key, NamedKey, NamedNote, Note};

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

fn get_named_keys_for_scale(start: &NamedKey, offsets: &Vec<i8>) -> Vec<NamedKey> {
    // This bit of logic tries to assign NamedKeys to the offsets, such that,
    // as far as possible, the NamedKeys start with different BaseKeys.
    // If this is not possible, we default to the key's default NamedKey.

    let (base_key, _) = start.get_components();
    // Get all base keys in reverse order (so we can use this as a stack)
    let mut keys_in_order: Vec<BaseKey> = base_key.get_keys_in_order().into_iter().rev().collect();

    let mut elements = Vec::<NamedKey>::new();
    for offset in offsets.iter() {
        let key = start.to_key() + offset;

        let get_default_key = |key: Key| -> NamedKey {
            let default_key = key.get_default_named_key();
            warn!("Could not generate consecutive NamedKey, for {} {:?} offset {}, reverting to default {}", start, offsets, offset, default_key);
            default_key
        };

        // First, try the last element of keys_in_order.
        let named_key: NamedKey = match keys_in_order.last() {
            Some(last_key) => match key.get_named_key_starting_with(last_key) {
                Some(named_key) => {
                    keys_in_order.pop();
                    named_key
                },
                None => get_default_key(key)
            }
            None => get_default_key(key)
        };

        elements.push(named_key)
    }
    elements
}

impl Scale {
    /// Create a new scale, starting from the given key and with the specified offsets.
    /// 
    /// # Errors
    ///     - if the offsets are not strictly increasing;
    ///     - if any offset is not comprised between 0 and 11.
    pub fn new(start: NamedKey, offsets: Vec<i8>) -> Result<Self, String> {
        // Validate offsets.
        let mut previous_offset: Option<i8> = None;
        for offset in offsets.iter() {
            if *offset < 0 || *offset > 11 {
                return Err("All offsets must be between 0 and 11!".to_string());
            }
            if let Some(previous_offset) = previous_offset {
                if previous_offset >= *offset {
                    return Err("Offsets must be in strictly increasing order!".to_string());
                }
            }
            previous_offset = Some(offset.clone());
        }

        // Get the named keys of the scale.
        let elements = get_named_keys_for_scale(&start, &offsets);

        Ok(Self {
            start,
            offsets,
            elements,
        })
    }
    fn get_named_key(&self, position: i8) -> NamedKey {
        let index = position.rem_euclid(i8::try_from(self.offsets.len()).unwrap());
        self.elements[usize::try_from(index).unwrap()]
    }
    fn get_named_note(&self, position: i8, octave: i8) -> NamedNote {
        let len = i8::try_from(self.offsets.len()).unwrap();
        let (index, additional_octaves) = (position.rem_euclid(len), position.div_euclid(len));
        println!("{}", index);
        let index_usize = usize::try_from(index).unwrap();
        let note = Note::compose(self.start.to_key(), octave + additional_octaves)
            + &self.offsets[index_usize];
        note.get_named_note_starting_with(&self.elements[index_usize].base_key).unwrap()
    }
    pub fn get_scale(&self) -> &Vec<i8> {
        &self.offsets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    #[test]
    fn can_init_scales() {
        env_logger::init();

        let major_scales = ["C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];
        for key_name in major_scales {
            let key = str::parse::<NamedKey>(key_name).unwrap();
            let _scale = Scale::new(key, vec![0, 2, 4, 5, 7, 9, 11]).unwrap();
        }

        let minor_scales = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        for key_name in minor_scales {
            let key = str::parse::<NamedKey>(key_name).unwrap();
            let _scale = Scale::new(key, vec![0, 2, 3, 5, 7, 8, 11]).unwrap();
        }
    }

    #[test]
    fn can_get_notes() {
        let c = str::parse::<NamedKey>("C").unwrap();
        let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap(); // C-major
        let c_major_scale = Rc::new(c_major_scale);

        let note_positions = [-2, -1, 0, 2, 4, 7, 9];
        let notes: Vec<NamedNote> = note_positions
            .into_iter()
            .map(|position| ScaleNote::new(Rc::clone(&c_major_scale), position, 4).to_named_note())
            .collect();

        let expected_notes = ["A3", "B3", "C4", "E4", "G4", "C5", "E5"].map(|s| str::parse::<NamedNote>(s).unwrap());
        for (note, expected_note) in iter::zip(notes, expected_notes) {
            assert_eq!(note, expected_note);
        }

        let eb = str::parse::<NamedKey>("Eb").unwrap();
        let eb_minor_scale = Scale::new(eb, vec![0, 2, 3, 5, 7, 8, 11]).unwrap(); // E-flat minor harmonic
        let eb_minor_scale = Rc::new(eb_minor_scale);

        let note_positions = [0, 1, 2, 3, 4, 5, 6, 7];
        let notes: Vec<NamedNote> = note_positions
            .into_iter()
            .map(|position| ScaleNote::new(Rc::clone(&eb_minor_scale), position, 4).to_named_note())
            .collect();

        let expected_notes = ["Eb4", "F4", "Gb4", "Ab4", "Bb4", "Cb5", "D5", "Eb5"].map(|s| str::parse::<NamedNote>(s).unwrap());
        for (note, expected_note) in iter::zip(notes, expected_notes) {
            assert_eq!(note, expected_note);
        }
    }
}