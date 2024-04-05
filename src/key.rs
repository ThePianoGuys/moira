// This module provides the following structs:
// Key: a key (one of the 12 semitones in Western tuning)
// Note: a note, with same values as MIDI (0 is C(-1), 60 is C4, etc.)
// NamedKey: a key that is called a certain way (e.g. D# or Eb).
// NamedNote: a note that is called a certain way (e.g. D#4 or Eb4).

// All these structs are not scale-aware. For high-level use, you probably
// want the ScaleKey and ScaleNote structs in the scale module.

use regex::Regex;
use std::fmt::{self, Display};
use std::ops::Add;

// Represents any of the 12 distinct keys in Western tuning
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Key(i8);

impl Key {
    pub fn new(key: i8) -> Self {
        Self(key % 12)
    }

    pub fn get_named_keys(&self) -> [Option<NamedKey>; 2] {
        match self.0 {
            0 => [
                Some(NamedKey::new(BaseKey::C, KeyModifier::Natural)),
                Some(NamedKey::new(BaseKey::B, KeyModifier::Sharp)),
            ],
            1 => [
                Some(NamedKey::new(BaseKey::C, KeyModifier::Sharp)),
                Some(NamedKey::new(BaseKey::D, KeyModifier::Flat)),
            ],
            2 => [Some(NamedKey::new(BaseKey::D, KeyModifier::Natural)), None],
            3 => [
                Some(NamedKey::new(BaseKey::D, KeyModifier::Sharp)),
                Some(NamedKey::new(BaseKey::E, KeyModifier::Flat)),
            ],
            4 => [
                Some(NamedKey::new(BaseKey::E, KeyModifier::Natural)),
                Some(NamedKey::new(BaseKey::F, KeyModifier::Flat)),
            ],
            5 => [
                Some(NamedKey::new(BaseKey::F, KeyModifier::Natural)),
                Some(NamedKey::new(BaseKey::E, KeyModifier::Sharp)),
            ],
            6 => [
                Some(NamedKey::new(BaseKey::F, KeyModifier::Sharp)),
                Some(NamedKey::new(BaseKey::G, KeyModifier::Flat)),
            ],
            7 => [Some(NamedKey::new(BaseKey::G, KeyModifier::Natural)), None],
            8 => [
                Some(NamedKey::new(BaseKey::G, KeyModifier::Sharp)),
                Some(NamedKey::new(BaseKey::A, KeyModifier::Flat)),
            ],
            9 => [Some(NamedKey::new(BaseKey::A, KeyModifier::Natural)), None],
            10 => [
                Some(NamedKey::new(BaseKey::A, KeyModifier::Sharp)),
                Some(NamedKey::new(BaseKey::B, KeyModifier::Flat)),
            ],
            11 => [
                Some(NamedKey::new(BaseKey::B, KeyModifier::Natural)),
                Some(NamedKey::new(BaseKey::C, KeyModifier::Flat)),
            ],
            _ => panic!("Normally keys should be between 0 and 11"),
        }
    }
}

// Allow adding an offset to a key. This wraps around.
impl Add<&i8> for Key {
    type Output = Key;
    fn add(self, rhs: &i8) -> Self::Output {
        Key::new((self.0 + *rhs) % 12)
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.0 {
            0 => "C",
            1 => "Cs",
            2 => "D",
            3 => "Ds",
            4 => "E",
            5 => "F",
            6 => "Fs",
            7 => "G",
            8 => "Gs",
            9 => "A",
            10 => "As",
            11 => "B",
            _ => panic!("Normally keys should be between 0 and 11"),
        };
        write!(f, "{}", name)
    }
}

// A wrapper around a note, with the height being the same as in MIDI
// (0 is C-1, 60 is C4 etc.)
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Note(u8);

impl Note {
    // Decompose a Note into its Key and octave
    pub fn decompose(&self) -> (Key, i8) {
        let key = self.0 % 12;
        let octave = (self.0 - key) / 12 + 1;
        (
            Key::new(key.try_into().unwrap()),
            octave.try_into().unwrap(),
        )
    }

    // Create a Note from a Key and octave
    pub fn compose(key: Key, octave: i8) -> Self {
        // Note: C-1 is 0, C0 is 12.
        Self(key.0.try_into().unwrap()) + &((octave + 1) * 12)
    }
}

// Allow adding an offset to a note.
impl Add<&i8> for Note {
    type Output = Note;
    fn add(self, rhs: &i8) -> Self::Output {
        let key: i8 = self.0.try_into().unwrap();
        Self((key + *rhs).try_into().unwrap())
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (key, octave) = self.decompose();
        write!(f, "{}{}", key, octave)
    }
}

// Key Modifiers, for named keys/notes.
// I think we don't need double-flats, or double-sharps, since this code will be used
// for the harmony, so we don't need to express accidentations in the melody.
// And harmonic changes can be expressed by a change of scale.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum KeyModifier {
    Natural,
    Flat,
    Sharp,
}

impl Display for KeyModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key_modifier_str = match self {
            KeyModifier::Natural => "",
            KeyModifier::Flat => "b",
            KeyModifier::Sharp => "#",
        };
        write!(f, "{}", key_modifier_str)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum BaseKey {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl BaseKey {
    pub fn to_key(&self) -> Key {
        let key = match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        };
        Key::new(key)
    }
    pub fn get_keys_in_order(&self) -> Vec<BaseKey> {
        let keys_in_order = [
            Self::C,
            Self::D,
            Self::E,
            Self::F,
            Self::G,
            Self::A,
            Self::B,
        ];
        let start_idx = keys_in_order.iter().position(|x| *x == *self).unwrap();
        keys_in_order
            .into_iter()
            .cycle()
            .skip(start_idx)
            .take(7)
            .collect()
    }
}

impl Add<&KeyModifier> for BaseKey {
    type Output = Key;
    fn add(self, rhs: &KeyModifier) -> Self::Output {
        self.to_key()
            + match rhs {
                KeyModifier::Natural => &0,
                KeyModifier::Flat => &-1,
                KeyModifier::Sharp => &1,
            }
    }
}

impl Display for BaseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base_key_str = match self {
            BaseKey::C => "C",
            BaseKey::D => "D",
            BaseKey::E => "E",
            BaseKey::F => "F",
            BaseKey::G => "G",
            BaseKey::A => "A",
            BaseKey::B => "B",
        };
        write!(f, "{}", base_key_str)
    }
}

#[derive(Clone, Copy)]
pub struct NamedKey {
    pub base_key: BaseKey,
    pub key_modifier: KeyModifier,
}

impl NamedKey {
    pub fn new(base_key: BaseKey, key_modifier: KeyModifier) -> Self {
        NamedKey {
            base_key,
            key_modifier,
        }
    }
    pub fn get_components(&self) -> (BaseKey, KeyModifier) {
        (self.base_key, self.key_modifier)
    }
    pub fn from_str(name: &str) -> Self {
        let re = Regex::new("^([A-G])([b#])?$").unwrap();
        let captures = re.captures(name).expect("Invalid key name!");

        let base_key = match &captures[1] {
            "C" => BaseKey::C,
            "D" => BaseKey::D,
            "E" => BaseKey::E,
            "F" => BaseKey::F,
            "G" => BaseKey::G,
            "A" => BaseKey::A,
            "B" => BaseKey::B,
            _ => panic!("Normally the regex should only allow letters between A and G"),
        };

        let key_modifier = match captures.get(2) {
            None => KeyModifier::Natural,
            Some(modifier_match) => match modifier_match.as_str() {
                "b" => KeyModifier::Flat,
                "#" => KeyModifier::Sharp,
                _ => panic!("Normally the regex should only allow # and b"),
            },
        };

        Self {
            base_key,
            key_modifier,
        }
    }
    pub fn to_key(&self) -> Key {
        self.base_key + &self.key_modifier
    }
}

impl Display for NamedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.base_key, self.key_modifier)
    }
}

#[derive(Clone, Copy)]
pub struct NamedNote {
    key: NamedKey,
    octave: i8,
}

impl NamedNote {
    pub fn new(key: NamedKey, octave: i8) -> Self {
        NamedNote { key, octave }
    }
    pub fn to_note(&self) -> Note {
        Note::compose(self.key.to_key(), self.octave)
    }
}

impl Display for NamedNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.key, self.octave)
    }
}
