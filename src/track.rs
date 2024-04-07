use midly::{MetaMessage, MidiMessage, TrackEvent, TrackEventKind};

use super::key::Note;

/// Create a track of MIDI events, from an iterable of (Note, u8)
/// representing notes and their durations.

pub fn create_track_harpsichord<'a, N>(notes: N) -> Vec<TrackEvent<'a>>
where
    N: IntoIterator<Item = (Note, u32)>,
    N::IntoIter: ExactSizeIterator + Clone + Send,
{
    let mut track_events = Vec::<TrackEvent>::new();

    // Set tempo to 120 bpm
    track_events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(500000.into())),
    });

    // Set harpsichord as instrument
    track_events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: 0.into(),
            message: MidiMessage::ProgramChange { program: 6.into() },
        },
    });

    for (note, duration) in notes {
        track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::NoteOn {
                    key: note.0.into(),
                    vel: 127.into(),
                },
            },
        });

        track_events.push(TrackEvent {
            delta: duration.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::NoteOff {
                    key: note.0.into(),
                    vel: 127.into(),
                },
            },
        });
    }

    // Track end
    track_events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    track_events
}
