use std::rc::Rc;
use log::warn;

mod chord;
mod key;
mod scale;

use key::{NamedNote, NamedKey};
use scale::{Scale, ScaleNote};

fn main() {
    env_logger::init();
    warn!("This is a warning!");

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

    let c = str::parse::<NamedKey>("C").unwrap();
    let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap(); // C-major
    let c_major_scale = Rc::new(c_major_scale);

    let eb = str::parse::<NamedKey>("Eb").unwrap();
    let eb_minor_scale = Scale::new(eb, vec![0, 2, 3, 5, 7, 8, 11]).unwrap(); // E-flat minor harmonic
    let eb_minor_scale = Rc::new(eb_minor_scale);

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
