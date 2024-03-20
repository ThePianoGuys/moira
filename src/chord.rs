enum ChordStyle {
    Major,
    Minor,
}

enum ChordScale {
    Major,
    MinorHarmonic,
    MinorMelodic,
}

pub struct Chord {
    name: String,
    style: ChordStyle,
    tone: String,
    scale: ChordScale,
    notes: Vec<u8>,
}