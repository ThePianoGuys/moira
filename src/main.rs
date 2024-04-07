use std::fs::File;

use midly::{self, Format, Header, Timing};

mod chord;
mod key;
mod scale;
mod track;

use key::NamedKey;
use scale::{Scale, ScaleNote};

fn main() {
    env_logger::init();

    let header = Header::new(Format::Parallel, Timing::Metrical(100.into()));

    let c = str::parse::<NamedKey>("C").unwrap();
    let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap();

    let wtc_1_1_prelude = [0, 2, 4, 7, 9, 4, 7, 9].into_iter().map(|position| {
        (
            ScaleNote::new(&c_major_scale, position, 4)
                .to_named_note()
                .to_note(),
            50,
        )
    });

    let track = track::create_track_harpsichord(wtc_1_1_prelude);

    let mut buffer = File::create("wtc_1_1_prelude.mid").unwrap();

    midly::write_std(&header, vec![track.iter()], buffer).unwrap();
}
