// This module provides the following structs:
// Key: a key (one of the 12 semitones in Western tuning)
// Note: a note, with same values as MIDI (0 is C(-1), 60 is C4, etc.)
// NamedKey: a key that is called a certain way (e.g. D# or Eb).
// NamedNote: a note that is called a certain way (e.g. D#4 or Eb4).

// All these structs are not scale-aware. For high-level use, you probably
// want the ScaleKey and ScaleNote structs in the scale module.

use regex::Regex;
use std::fmt::{self, Debug, Display};
use std::ops::Add;
use std::str::FromStr;

// Represents any of the 12 distinct keys in Western tuning
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Key(i8);

impl Key {
    pub fn new(key: i8) -> Self {
        Self(key.rem_euclid(12))
    }

    pub fn get_named_key_starting_with(&self, base_key: &BaseKey) -> Option<NamedKey> {
        match (self.0, base_key) {
            (0, BaseKey::B) => Some(NamedKey::new(BaseKey::B, KeyModifier::Sharp)),
            (0, BaseKey::C) => Some(NamedKey::new(BaseKey::C, KeyModifier::Natural)),
            (1, BaseKey::C) => Some(NamedKey::new(BaseKey::C, KeyModifier::Sharp)),
            (1, BaseKey::D) => Some(NamedKey::new(BaseKey::D, KeyModifier::Flat)),
            (2, BaseKey::C) => Some(NamedKey::new(BaseKey::D, KeyModifier::DoubleSharp)),
            (2, BaseKey::D) => Some(NamedKey::new(BaseKey::D, KeyModifier::Natural)),
            (3, BaseKey::D) => Some(NamedKey::new(BaseKey::D, KeyModifier::Sharp)),
            (3, BaseKey::E) => Some(NamedKey::new(BaseKey::E, KeyModifier::Flat)),
            (4, BaseKey::E) => Some(NamedKey::new(BaseKey::E, KeyModifier::Natural)),
            (4, BaseKey::F) => Some(NamedKey::new(BaseKey::F, KeyModifier::Flat)),
            (5, BaseKey::E) => Some(NamedKey::new(BaseKey::E, KeyModifier::Sharp)),
            (5, BaseKey::F) => Some(NamedKey::new(BaseKey::F, KeyModifier::Natural)),
            (6, BaseKey::F) => Some(NamedKey::new(BaseKey::F, KeyModifier::Sharp)),
            (6, BaseKey::G) => Some(NamedKey::new(BaseKey::G, KeyModifier::Flat)),
            (7, BaseKey::F) => Some(NamedKey::new(BaseKey::G, KeyModifier::DoubleSharp)),
            (7, BaseKey::G) => Some(NamedKey::new(BaseKey::G, KeyModifier::Natural)),
            (8, BaseKey::G) => Some(NamedKey::new(BaseKey::G, KeyModifier::Sharp)),
            (8, BaseKey::A) => Some(NamedKey::new(BaseKey::A, KeyModifier::Flat)),
            (9, BaseKey::G) => Some(NamedKey::new(BaseKey::A, KeyModifier::DoubleSharp)),
            (9, BaseKey::A) => Some(NamedKey::new(BaseKey::A, KeyModifier::Natural)),
            (10, BaseKey::A) => Some(NamedKey::new(BaseKey::A, KeyModifier::Sharp)),
            (10, BaseKey::B) => Some(NamedKey::new(BaseKey::B, KeyModifier::Flat)),
            (11, BaseKey::B) => Some(NamedKey::new(BaseKey::B, KeyModifier::Natural)),
            (11, BaseKey::C) => Some(NamedKey::new(BaseKey::C, KeyModifier::Flat)),
            _ => None,
        }
    }

    pub fn get_default_named_key(&self) -> NamedKey {
        match self.0 {
            0 => NamedKey::new(BaseKey::C, KeyModifier::Natural),
            1 => NamedKey::new(BaseKey::C, KeyModifier::Sharp),
            2 => NamedKey::new(BaseKey::D, KeyModifier::Natural),
            3 => NamedKey::new(BaseKey::D, KeyModifier::Sharp),
            4 => NamedKey::new(BaseKey::E, KeyModifier::Natural),
            5 => NamedKey::new(BaseKey::F, KeyModifier::Natural),
            6 => NamedKey::new(BaseKey::F, KeyModifier::Sharp),
            7 => NamedKey::new(BaseKey::G, KeyModifier::Natural),
            8 => NamedKey::new(BaseKey::G, KeyModifier::Sharp),
            9 => NamedKey::new(BaseKey::A, KeyModifier::Natural),
            10 => NamedKey::new(BaseKey::A, KeyModifier::Sharp),
            11 => NamedKey::new(BaseKey::B, KeyModifier::Natural),
            _ => panic!("Normally keys should be between 0 and 11"),
        }
    }
}

// Allow adding an offset to a key. This wraps around.
impl Add<&i8> for Key {
    type Output = Key;
    fn add(self, rhs: &i8) -> Self::Output {
        Key::new(self.0 + *rhs)
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_default_named_key())
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// A wrapper around a note, with the height being the same as in MIDI
// (0 is C-1, 60 is C4 etc.)
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Note(pub u8);

impl Note {
    // Decompose a Note into its Key and octave
    pub fn decompose(&self) -> (Key, i8) {
        let key = self.0 % 12;
        let octave = (self.0 - key) / 12 - 1;
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

    pub fn get_named_note_starting_with(&self, base_key: &BaseKey) -> Option<NamedNote> {
        let (key, octave) = self.decompose();
        let named_key = key.get_named_key_starting_with(base_key)?;

        // handle the B#, Cb case correctly
        Some(match (named_key.base_key, named_key.key_modifier) {
            (BaseKey::B, KeyModifier::Sharp) => NamedNote::new(named_key, octave - 1),
            (BaseKey::C, KeyModifier::Flat) => NamedNote::new(named_key, octave + 1),
            _ => NamedNote::new(named_key, octave),
        })
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

impl Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum KeyModifier {
    Natural,
    Flat,
    Sharp,
    DoubleSharp,
}

impl KeyModifier {
    pub fn get_value(&self) -> i8 {
        match self {
            KeyModifier::Flat => -1,
            KeyModifier::Natural => 0,
            KeyModifier::Sharp => 1,
            KeyModifier::DoubleSharp => 2,
        }
    }
}

impl Display for KeyModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key_modifier_str = match self {
            KeyModifier::Natural => "",
            KeyModifier::Flat => "♭",
            KeyModifier::Sharp => "♯",
            KeyModifier::DoubleSharp => "𝄪",
        };
        write!(f, "{}", key_modifier_str)
    }
}

impl Debug for KeyModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
    pub fn get_keys_in_order(&self) -> impl Iterator<Item = BaseKey> {
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
        keys_in_order.into_iter().cycle().skip(start_idx).take(7)
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

impl Debug for BaseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub fn to_key(&self) -> Key {
        self.base_key.to_key() + &self.key_modifier.get_value()
    }
}

impl FromStr for NamedKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new("^([A-G])([b♭#♯x𝄪])?$").unwrap();
        let captures = re
            .captures(s)
            .ok_or_else(|| format!("Invalid key: {}", s))?;

        let base_key = match &captures[1] {
            "C" => Ok(BaseKey::C),
            "D" => Ok(BaseKey::D),
            "E" => Ok(BaseKey::E),
            "F" => Ok(BaseKey::F),
            "G" => Ok(BaseKey::G),
            "A" => Ok(BaseKey::A),
            "B" => Ok(BaseKey::B),
            _ => Err(format!("Invalid key: {} ", s)),
        }?;

        let key_modifier = match captures.get(2) {
            None => Ok(KeyModifier::Natural),
            Some(modifier_match) => match modifier_match.as_str() {
                "b" | "♭" => Ok(KeyModifier::Flat),
                "#" | "♯" => Ok(KeyModifier::Sharp),
                "x" | "𝄪" => Ok(KeyModifier::DoubleSharp),
                _ => Err(format!("Invalid key: {}", s)),
            },
        }?;

        Ok(Self::new(base_key, key_modifier))
    }
}

impl Display for NamedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.base_key, self.key_modifier)
    }
}

impl Debug for NamedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NamedNote {
    key: NamedKey,
    octave: i8,
}

impl NamedNote {
    pub fn new(key: NamedKey, octave: i8) -> Self {
        NamedNote { key, octave }
    }
    pub fn to_note(&self) -> Note {
        // Do it this way to handle Cb5 is B4, B#4 is C5
        Note::compose(self.key.base_key.to_key(), self.octave) + &self.key.key_modifier.get_value()
    }
}

impl FromStr for NamedNote {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new("^([A-G][b♭#♯x𝄪]?)(-1|[0-9])$").unwrap();
        let captures = re
            .captures(s)
            .ok_or_else(|| format!("Invalid note:{}", s))?;

        let key = NamedKey::from_str(&captures[1])?;
        let octave: i8 = str::parse(&captures[2]).map_err(|_| format!("Invalid note: {}", s))?;

        Ok(Self::new(key, octave))
    }
}

impl Display for NamedNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.key, self.octave)
    }
}

impl Debug for NamedNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
