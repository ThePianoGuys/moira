use midly::{MidiMessage, TrackEvent, TrackEventKind};

use super::scale::Scale;
use super::track::{Track, Piece};

// struct JazzPiece {
//     length: u8,
//     left_hand: Vec<Chord>,
//     // right_hand: Vec<RightHand>,
// }

/// What the left hand is playing during a bar
pub struct Chord {
    pub scale: Scale,
    pub chord: Vec<i8>,  // the positions of the scale played
    pub octave: i8,
    pub notes: Vec<(bool, u8)>  // True means a note is played, False means a silence.
}

impl Track for Chord {
    fn to_midi(&self, channel: u8) -> Vec<TrackEvent> {
        let mut track_events = Vec::<TrackEvent>::new();

        // Set piano as instrument
        track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::ProgramChange { program: 1.into() },
            },
        });

        let mut next_note_delta = 0;

        for (is_played, duration) in self.notes.iter() {
            let duration = u32::from(duration.clone());

            if *is_played {
                for position in self.chord.iter() {
                    track_events.push(TrackEvent {
                        delta: (next_note_delta).into(),
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOn {
                                key: self.scale.get_note(*position, self.octave).0.into(),
                                vel: 127.into(),
                            },
                        },
                    });
                    next_note_delta = 0;
                }

                for position in self.chord.iter() {
                    track_events.push(TrackEvent {
                        delta: duration.into(),
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOff {
                                key: self.scale.get_note(*position, self.octave).0.into(),
                                vel: 127.into(),
                            },
                        },
                    });
                }
            } else {
                next_note_delta += duration;
            }
        };
        track_events
    }
}

#[cfg(test)]
mod tests {
    use super::super::NamedKey;
    use super::*;
    use std::io::Cursor;

    #[test]
    fn can_generate_left_hand_midi() {
        let c = str::parse::<NamedKey>("C").unwrap();
        let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap();

        let left_hand = Piece {
            bpm: 120, 
            tracks: vec![Chord{
                scale: c_major_scale,
                chord: vec![0, 2, 6],
                octave: 3,
                notes: vec![(true, 12), (true, 24), (true, 24), (false, 24), (true, 12)],
            }]
        };

        let mut buffer = Cursor::new(vec![0; 100]);
        left_hand.write_midi(&mut buffer).unwrap();
    }
}