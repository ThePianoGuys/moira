use indexmap::IndexMap;

use serde_json::Value;

use super::track::{TimedNote, TICKS_PER_BEAT};
use super::{Scale, ScaleNote, ScaleNotePiece, ScaleNoteTrack};

// This is the definition of the JSON data format we are using.
//
// Piece  = [ Track* ]
// Track  = { "id": String, "scale": string, "bpm": int, "start": Start, "notes": Notes }
// Start  = int | { String: offset<int> }
// Notes  = [ Note | { Note: duration<int>} | Notes ]
// Note   = null | int

pub fn parse_input(input: &str) -> Result<ScaleNotePiece, String> {
    let json: Value =
        serde_json::from_str(input).or_else(|err| Err("Could not parse JSON!".to_string()))?;

    let tracks_json = json
        .as_array()
        .ok_or_else(|| "JSON should be an array!".to_string())?;
    let mut tracks_by_id: IndexMap<String, ScaleNoteTrack> = IndexMap::new();

    for track_json in tracks_json.iter() {
        let track = parse_track(track_json, &tracks_by_id)?;
        tracks_by_id.insert(track.id.clone(), track);
    }
    let tracks: Vec<ScaleNoteTrack> = tracks_by_id.into_values().collect();

    Ok(ScaleNotePiece(tracks))
}

fn parse_track(
    track_json: &Value,
    tracks_by_id: &IndexMap<String, ScaleNoteTrack>,
) -> Result<ScaleNoteTrack, String> {
    let track_json = track_json
        .as_object()
        .ok_or_else(|| "Each track should be a JSON object!".to_string())?;

    let mut id: Option<String> = None;
    let mut scale: Option<Scale> = None;
    let mut octave: Option<i8> = None;
    let mut bpm: Option<u8> = None;
    let mut start: Option<u32> = None;
    let mut notes: Option<Vec<TimedNote>> = None;

    for (key, value) in track_json {
        match key.as_str() {
            "id" => {
                id = Some(
                    value
                        .as_str()
                        .ok_or_else(|| "ID should be a string!")?
                        .to_string(),
                );
            }
            "scale" => {
                scale = Some(str::parse::<Scale>(
                    value.as_str().ok_or_else(|| "Scale should be a string!")?,
                )?);
            }
            "octave" => {
                let octave_i64 = value.as_i64().ok_or_else(|| "Octave should be an int!")?;
                octave =
                    Some(i8::try_from(octave_i64).map_err(|_| "Could not convert octave to i8!")?);
            }
            "bpm" => {
                let bpm_u64 = value.as_u64().ok_or_else(|| "bpm should be an uint!")?;
                bpm = Some(u8::try_from(bpm_u64).map_err(|_| "Could not convert bpm to u8!")?);
            }
            "start" => {
                start = Some(parse_track_start(value, tracks_by_id)?);
            }
            "notes" => {
                let scale = scale
                    .as_ref()
                    .ok_or_else(|| "Scale should be defined before notes!")?;
                let octave = octave.ok_or_else(|| "Octave should be defined before notes!")?;
                notes = Some(parse_track_notes(value, scale, octave, 0)?);
            }
            other => {
                return Err(format!("Incorrect key in JSON: {}", other));
            }
        };
    }

    if let (Some(id), Some(scale), Some(octave), Some(bpm), Some(start), Some(notes)) =
        (id, scale, octave, bpm, start, notes)
    {
        Ok(ScaleNoteTrack {
            id,
            scale,
            octave,
            bpm,
            start,
            notes,
        })
    } else {
        Err("Some parameters were missing!".to_string())
    }
}

fn parse_track_start(
    track_start_json: &Value,
    tracks_by_id: &IndexMap<String, ScaleNoteTrack>,
) -> Result<u32, String> {
    match track_start_json {
        Value::Number(start) => {
            let start = start
                .as_u64()
                .ok_or_else(|| "Track start should be a uint!")?;
            let start = u32::try_from(start).map_err(|_| "Could not cast track start to u8!")?;
            Ok(start)
        }
        Value::Object(map_track_start) => {
            let mut track_start: Option<u32> = None;
            for (key, value) in map_track_start {
                let reference_track = tracks_by_id
                    .get(key)
                    .ok_or_else(|| "Invalid reference track!")?;
                let offset = value
                    .as_i64()
                    .ok_or_else(|| "Offset to reference track must be int!")?;
                let offset = i64::from(reference_track.start) + offset;
                let offset = u32::try_from(offset).map_err(|_| "Could not cast start to u32!")?;
                track_start = Some(offset);
            }
            if let Some(track_start) = track_start {
                Ok(track_start)
            } else {
                Err("Empty object!".to_string())
            }
        }
        _ => Err("start should be int or Json object!".to_string()),
    }
}

fn parse_track_notes(
    track_notes_json: &Value,
    scale: &Scale,
    octave: i8,
    depth: u8,
) -> Result<Vec<TimedNote>, String> {
    let mut notes: Vec<TimedNote> = Vec::new();
    let mut push_note = |position: Option<i8>, duration: u8| {
        let duration = duration * TICKS_PER_BEAT / depth;
        notes.push(match position {
            Some(position) => {
                let note = ScaleNote { position, octave };
                (Some(note), duration)
            }
            None => (None, duration),
        });
    };
    match track_notes_json {
        Value::Number(num) => {
            let position = num.as_i64().ok_or_else(|| "Note value must be int!")?;
            let position =
                i8::try_from(position).map_err(|_| "Could not cast note value to i8!")?;
            push_note(Some(position), 1);
        }
        Value::String(string) => {
            if string.as_str() != "" {
                return Err("Only an empty string can be used to signify a silence!".to_string());
            }
            push_note(None, 1);
        }
        Value::Null => {
            push_note(None, 1);
        }
        Value::Array(track_notes_json) => {
            for track_notes_json_deeper in track_notes_json {
                let notes_deeper =
                    parse_track_notes(track_notes_json_deeper, scale, octave, depth + 1)?;
                notes.extend(notes_deeper.into_iter());
            }
        }
        Value::Object(map_note_value) => {
            for (key, value) in map_note_value {
                let duration = value.as_u64().ok_or_else(|| "Note duration must be int!")?;
                let duration =
                    u8::try_from(duration).map_err(|_| "Could not cast duration to u8!")?;

                if key.as_str() == "" {
                    push_note(None, duration);
                } else {
                    let position =
                        str::parse::<i8>(key).map_err(|_| "Could not cast note value to i8!")?;
                    push_note(Some(position), duration);
                }
            }
        }
        _ => {
            return Err("Notes must be a number, string, null, Array or Object!".to_string());
        }
    };
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_data() {
        let data = r#"
        [
            {
                "id": "voice_1", "scale": "Cmaj", "octave": 4, "bpm": 120, "start": 0,
                "notes": [
                    "", 1, 2, 3, {"4": 2}, [5, 4], 3,
                    6, 2, [{"5": 3}, 6, 5, 4],
                    [3, 4, 3, 2, 1, 2, 1, -1, -2]
                ]
            }
        ]"#;

        let piece = parse_input(data).unwrap();
    }
}
