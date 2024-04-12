use std::fs::File;

mod chord;
mod json_input;
mod key;
mod scale;
mod track;

use key::NamedKey;
use scale::Scale;
use track::{Piece, Voice, TICKS_PER_BEAT};

fn main() {
    env_logger::init();

    let c = str::parse::<NamedKey>("C").unwrap();
    let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap();

    let wtc_1_1_prelude_voice = Box::new(Voice {
        id: "voice_1".to_string(),
        start: 0,
        scale: c_major_scale.clone(),
        octave: 4,
        notes: [0, 2, 4, 7, 9, 4, 7, 9]
            .into_iter()
            .map(|position| (Some(position), TICKS_PER_BEAT / 2))
            .collect(),
    });

    let wtc_1_1_prelude = Piece {
        bpm: 120,
        tracks: vec![wtc_1_1_prelude_voice.clone()],
    };

    println!("{}", wtc_1_1_prelude_voice);

    let mut buffer = File::create("results/wtc_1_1_prelude.mid").unwrap();

    wtc_1_1_prelude.write_midi(&mut buffer).unwrap();

    let wtc_1_1_fugue =
        json_input::parse_piece(include_str!("../examples/wtc_1_1_fugue.json")).unwrap();
    let mut buffer = File::create("results/wtc_1_1_fugue.mid").unwrap();
    wtc_1_1_fugue.write_midi(&mut buffer).unwrap();

    let ballad = 
        json_input::parse_piece(include_str!("../examples/ballad.json")).unwrap();
    let mut buffer = File::create("results/ballad.mid").unwrap();
    ballad.write_midi(&mut buffer).unwrap();
}
