use midly::{Format, Header, MetaMessage, MidiMessage, Timing, TrackEvent, TrackEventKind};

use super::{Scale, ScaleNote};

pub const TICKS_PER_BEAT: u8 = 24;

// A ScaleNote or silence, with associated duration.
pub type TimedNote = (Option<ScaleNote>, u8);

pub struct ScaleNoteTrack {
    pub id: String,
    pub scale: Scale,
    pub octave: i8,
    pub bpm: u8,
    pub start: u32,
    pub notes: Vec<TimedNote>,
}

impl ScaleNoteTrack {
    /// Create a track of MIDI events, from an iterable of (Note, u8)
    /// representing notes and their durations.
    pub fn to_midi_harpsichord(&self) -> Vec<TrackEvent> {
        let mut track_events = Vec::<TrackEvent>::new();

        // MIDI sets tempo in microseconds per beat, e.g. 120bpm is 500000 microseconds/beat.
        // Note that the number of MIDI ticks per beat is set with the TICKS_PER_BEAT constant.
        let microseconds_per_beat = 500000 * 120 / u32::from(self.bpm);
        track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(microseconds_per_beat.into())),
        });

        // Set harpsichord as instrument
        track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::ProgramChange { program: 6.into() },
            },
        });

        let mut next_note_delta = self.start;

        for (note, duration) in self.notes.iter() {
            let duration = u32::from(duration.clone());

            if let Some(note) = note {
                track_events.push(TrackEvent {
                    delta: (next_note_delta).into(),
                    kind: TrackEventKind::Midi {
                        channel: 0.into(),
                        message: MidiMessage::NoteOn {
                            key: note.to_note(&self.scale).0.into(),
                            vel: 127.into(),
                        },
                    },
                });

                track_events.push(TrackEvent {
                    delta: duration.into(),
                    kind: TrackEventKind::Midi {
                        channel: 0.into(),
                        message: MidiMessage::NoteOff {
                            key: note.to_note(&self.scale).0.into(),
                            vel: 127.into(),
                        },
                    },
                });

                next_note_delta = 0;
            } else {
                next_note_delta += duration;
            }
        }

        // Track end
        track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        track_events
    }
}

pub struct ScaleNotePiece(pub Vec<ScaleNoteTrack>);

impl ScaleNotePiece {
    pub fn write_midi_harpsichord<W>(&self, w: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let header = Header::new(
            Format::Parallel,
            Timing::Metrical(u16::from(TICKS_PER_BEAT).into()),
        );
        let tracks: Vec<Vec<TrackEvent>> = self
            .0
            .iter()
            .map(|track| track.to_midi_harpsichord())
            .collect();
        midly::write_std(&header, tracks.iter(), w)
    }
}

#[cfg(test)]
mod tests {
    use super::super::NamedKey;
    use super::*;
    use std::io::Cursor;

    #[test]
    fn can_generate_midi_harpsichord() {
        let c = str::parse::<NamedKey>("C").unwrap();
        let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap();
        let octave = 4;

        let wtc_1_1_prelude = ScaleNotePiece(vec![ScaleNoteTrack {
            id: "voice_1".to_string(),
            start: 0,
            scale: c_major_scale,
            octave,
            bpm: 120,
            notes: [0, 2, 4, 7, 9, 4, 7, 9]
                .into_iter()
                .map(|position| (Some(ScaleNote { position, octave }), TICKS_PER_BEAT / 2))
                .collect(),
        }]);

        let mut buffer = Cursor::new(vec![0; 100]);
        wtc_1_1_prelude.write_midi_harpsichord(&mut buffer).unwrap();
    }
}
