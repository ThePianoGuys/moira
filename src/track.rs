use std::fmt::{self, Display};

use midly::{Format, Header, MetaMessage, MidiMessage, Timing, TrackEvent, TrackEventKind};

use super::Scale;

pub const TICKS_PER_BEAT: u8 = 24;

/// A note or silence, with associated duration.
pub type TimedNote = (Option<i8>, u8);

pub trait Track {
    fn to_midi(&self, channel: u8) -> Vec<TrackEvent>;
}

pub struct VoiceTrack {
    pub id: String,
    pub scale: Scale,
    pub octave: i8,
    pub start: u32,
    pub notes: Vec<TimedNote>,
}

impl Track for VoiceTrack {
    /// Create a track of MIDI events, writing notes to the given MIDI channel.
    fn to_midi(&self, channel: u8) -> Vec<TrackEvent> {
        let mut track_events = Vec::<TrackEvent>::new();

        // Set harpsichord as instrument
        track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::ProgramChange { program: 6.into() },
            },
        });

        let mut next_note_delta = self.start * u32::from(TICKS_PER_BEAT);

        for (note, duration) in self.notes.iter() {
            let duration = u32::from(duration.clone());

            if let Some(note) = note {
                track_events.push(TrackEvent {
                    delta: (next_note_delta).into(),
                    kind: TrackEventKind::Midi {
                        channel: channel.into(),
                        message: MidiMessage::NoteOn {
                            key: self.scale.get_note(*note, self.octave).0.into(),
                            vel: 127.into(),
                        },
                    },
                });

                track_events.push(TrackEvent {
                    delta: duration.into(),
                    kind: TrackEventKind::Midi {
                        channel: channel.into(),
                        message: MidiMessage::NoteOff {
                            key: self.scale.get_note(*note, self.octave).0.into(),
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

impl Display for VoiceTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut note_names = String::new();
        let mut note_symbols = String::new();
        for (position, duration) in self.notes.iter() {
            let note_name = match position {
                Some(position) => {
                    format!("{:4}", self.scale.get_named_note(*position, self.octave).to_string())
                }
                None => "    ".to_string(),
            };
            let note_symbol = match duration * 16 / TICKS_PER_BEAT {
                64 => "𝅝   ",
                48 => "𝅗𝅥𝅭   ",
                32 => "𝅗𝅥   ",
                24 => "𝅘𝅥𝅭   ",
                16 => "𝅘𝅥   ",
                12 => "𝅘𝅥𝅮𝅭   ",
                8 => "𝅘𝅥𝅮   ",
                4 => "𝅘𝅥𝅯   ",
                2 => "𝅘𝅥𝅰   ",
                _ => "?   ",
            };
            note_names.extend(note_name.chars());
            note_symbols.extend(note_symbol.chars());
        }
        write!(f, "{}\n{}", note_names, note_symbols)
    }
}

pub struct Piece<T: Track> {
    pub bpm: u8,
    pub tracks: Vec<T>,
}

impl<T: Track> Piece<T> {
    pub fn write_midi<W>(&self, w: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let header = Header::new(
            Format::Parallel,
            Timing::Metrical(u16::from(TICKS_PER_BEAT).into()),
        );

        let microseconds_per_beat = 500000 * 120 / u32::from(self.bpm);

        // The first track must contain tempo and time signature information.
        let mut tracks: Vec<Vec<TrackEvent>> = vec![vec![
            // MIDI sets tempo in microseconds per beat, e.g. 120bpm is 500000 microseconds/beat.
            // Note that the number of MIDI ticks per beat is set with the TICKS_PER_BEAT constant.
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(microseconds_per_beat.into())),
            },
            // Set the time signature
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
            },
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            },
        ]];

        for (i, track) in self.tracks.iter().enumerate() {
            let track_to_midi = track.to_midi(u8::try_from(i).unwrap() % 16);
            tracks.push(track_to_midi);
        }
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

        let wtc_1_1_prelude = Piece {
            bpm: 120,
            tracks: vec![VoiceTrack {
                id: "voice_1".to_string(),
                start: 0,
                scale: c_major_scale,
                octave,
                notes: [0, 2, 4, 7, 9, 4, 7, 9]
                    .into_iter()
                    .map(|position| (Some(position), TICKS_PER_BEAT / 2))
                    .collect(),
            }],
        };

        let mut buffer = Cursor::new(vec![0; 100]);
        wtc_1_1_prelude.write_midi(&mut buffer).unwrap();
    }

    #[test]
    fn can_format_track() {
        let c = str::parse::<NamedKey>("C").unwrap();
        let c_major_scale = Scale::new(c, vec![0, 2, 4, 5, 7, 9, 11]).unwrap();
        let octave = 4;

        let wtc_1_1_prelude_track = VoiceTrack {
            id: "voice_1".to_string(),
            start: 0,
            scale: c_major_scale,
            octave,
            notes: [0, 2, 4, 7, 9, 4, 7, 9]
                .into_iter()
                .map(|position| (Some(position), TICKS_PER_BEAT / 2))
                .collect(),
        };

        wtc_1_1_prelude_track.to_string();
    }
}
