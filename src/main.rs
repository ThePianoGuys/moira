mod chord;
mod key;
mod scale;

use key::NamedKey;
use scale::{Scale, ScaleNote};

fn main() {
    env_logger::init();

    let c = str::parse::<NamedKey>("C").unwrap();
    let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap(); // C-major

    let wtc_1_1_prelude = [0, 2, 4, 7, 9, 4, 7, 9]
        .into_iter()
        .map(|position| ScaleNote::new(&c_major_scale, position, 4).to_named_note());

    for note in wtc_1_1_prelude {
        println!("{}", note);
    }
}
