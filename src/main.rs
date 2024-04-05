use std::rc::Rc;

mod chord;
mod key;
mod scale;

use key::NamedNote;
use scale::{Scale, ScaleNote};

fn main() {
    let c = key::NamedKey::from_str("C");
    let c_major_scale = Rc::new(Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11])); // C-major

    let eb = key::NamedKey::from_str("Eb");
    let eb_minor_scale = Rc::new(Scale::new(eb, vec![0, 2, 3, 5, 7, 8, 11])); // E-flat minor harmonic

    let notes = [0, 2, 4, 7, 9, 4, 7, 9];
    let wtc_1_1_prelude: Vec<ScaleNote> = notes
        .into_iter()
        .map(|position| ScaleNote::new(Rc::clone(&c_major_scale), position, 4))
        .collect();

    let actual_notes: Vec<NamedNote> = wtc_1_1_prelude
        .into_iter()
        .map(|note| note.to_named_note())
        .collect();

    for note in actual_notes {
        println!("{}", note);
    }
}
