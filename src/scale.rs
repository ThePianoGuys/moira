use std::str::FromStr;

use log::warn;
use regex::Regex;

use super::key::{BaseKey, Key, NamedKey, NamedNote, Note};


pub struct Scale {
    start: NamedKey,         // starting note of the scale: 0 is C, 11 is B
    offsets: Vec<i8>,        // the offsets of the scale
    elements: Vec<NamedKey>, // Will be filled in at struct initialization.
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
        let elements = Self::generate_elements(&start, &offsets);

        Ok(Self {
            start,
            offsets,
            elements,
        })
    }
    fn generate_elements(start: &NamedKey, offsets: &Vec<i8>) -> Vec<NamedKey> {
        // This bit of logic tries to assign NamedKeys to the offsets, such that,
        // as far as possible, the NamedKeys start with different BaseKeys.
        // If this is not possible, we default to the key's default NamedKey.

        let (base_key, _) = start.get_components();
        // Get all base keys in reverse order (so we can use this as a stack)
        let keys_in_order: Vec<BaseKey> = base_key.get_keys_in_order().collect();
        let mut keys_stack: Vec<BaseKey> = keys_in_order.into_iter().rev().collect();

        let mut elements = Vec::<NamedKey>::new();
        for offset in offsets.iter() {
            let key = start.to_key() + offset;

            let get_default_key = |key: Key| -> NamedKey {
                let default_key = key.get_default_named_key();
                warn!("Could not generate consecutive NamedKey, for {} {:?} offset {}, reverting to default {}", start, offsets, offset, default_key);
                default_key
            };

            // First, try the last element of keys_in_order.
            let named_key: NamedKey = match keys_stack.last() {
                Some(last_key) => match key.get_named_key_starting_with(last_key) {
                    Some(named_key) => {
                        keys_stack.pop();
                        named_key
                    }
                    None => get_default_key(key),
                },
                None => get_default_key(key),
            };

            elements.push(named_key)
        }
        elements
    }
    fn get_index_and_additional_octaves(&self, position: i8) -> (usize, i8) {
        let len = i8::try_from(self.offsets.len()).unwrap();
        let (index, additional_octaves) = (position.rem_euclid(len), position.div_euclid(len));
        let index_usize = usize::try_from(index).unwrap();
        (index_usize, additional_octaves)
    }
    pub fn get_note(&self, position: i8, octave: i8) -> Note {
        let (index_usize, additional_octaves) = self.get_index_and_additional_octaves(position);
        Note::compose(self.start.to_key(), octave + additional_octaves) + &self.offsets[index_usize]
    }
    pub fn get_named_note(&self, position: i8, octave: i8) -> NamedNote {
        let (index_usize, _) = self.get_index_and_additional_octaves(position);
        let note = self.get_note(position, octave);
        note.get_named_note_starting_with(&self.elements[index_usize].base_key)
            .unwrap()
    }
}

impl FromStr for Scale {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new("^([A-G][b♭#♯x𝄪]?)(M|maj|m|min)?$").unwrap();
        let captures = re
            .captures(s)
            .ok_or_else(|| format!("Invalid scale:{}", s))?;

        let start = NamedKey::from_str(&captures[1])?;

        let offsets = match captures.get(2) {
            None => Ok(vec![0, 2, 4, 5, 7, 9, 11]), // major
            Some(scale_mode) => match scale_mode.as_str() {
                "M" | "maj" => Ok(vec![0, 2, 4, 5, 7, 9, 11]),
                "m" | "min" => Ok(vec![0, 2, 3, 5, 7, 8, 11]),
                mode => Err(format!("Invalid scale mode: {}", mode)),
            },
        }?;

        Self::new(start, offsets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    #[test]
    fn can_init_scales() {
        env_logger::init();

        let major_scales = [
            "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
        ];
        for key_name in major_scales {
            let key = str::parse::<NamedKey>(key_name).unwrap();
            let _scale = Scale::new(key, vec![0, 2, 4, 5, 7, 9, 11]).unwrap();
        }

        let minor_scales = [
            "C", "C#", "D", "Eb", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        for key_name in minor_scales {
            let key = str::parse::<NamedKey>(key_name).unwrap();
            let _scale = Scale::new(key, vec![0, 2, 3, 5, 7, 8, 11]).unwrap();
        }
    }

    #[test]
    fn can_get_notes() {
        let c = str::parse::<NamedKey>("C").unwrap();
        let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap(); // C-major

        let note_positions = [-2, -1, 0, 2, 4, 7, 9];
        let notes = note_positions.into_iter().map(|position| {
            c_major_scale.get_named_note(position, 4)
        });

        let expected_notes =
            ["A3", "B3", "C4", "E4", "G4", "C5", "E5"].map(|s| str::parse::<NamedNote>(s).unwrap());
        for (note, expected_note) in iter::zip(notes, expected_notes) {
            assert_eq!(note, expected_note);
        }

        let eb = str::parse::<NamedKey>("Eb").unwrap();
        let eb_minor_scale = Scale::new(eb, vec![0, 2, 3, 5, 7, 8, 11]).unwrap(); // E-flat minor harmonic

        let note_positions = [0, 1, 2, 3, 4, 5, 6, 7];
        let notes = note_positions.into_iter().map(|position| {
            eb_minor_scale.get_named_note(position, 4)
        });

        let expected_notes = ["Eb4", "F4", "Gb4", "Ab4", "Bb4", "Cb5", "D5", "Eb5"]
            .map(|s| str::parse::<NamedNote>(s).unwrap());
        for (note, expected_note) in iter::zip(notes, expected_notes) {
            assert_eq!(note, expected_note);
        }
    }
}
