use indexmap::IndexMap;

use regex::Regex;
use serde_json::Value;

use super::track::{TimedNote, TICKS_PER_BEAT};
use super::{Scale, Piece, VoiceTrack};

// This is the definition of the JSON data format we are using.
//
// Piece  = [ Track* ]
// Track  = { "id": String, "scale": string, "bpm": int, "start": Start, "notes": Notes }
// Start  = int | { String: offset<int> }
// Notes  = [ Note | { duration<int>: Notes } | Notes ]
// Note   = null | int

pub fn parse_piece(json_str: &str) -> Result<Piece<VoiceTrack>, String> {
    let json: Value =
        serde_json::from_str(json_str).or_else(|_| Err("Could not parse JSON!".to_string()))?;

    let piece_json = json
        .as_object()
        .ok_or_else(|| "JSON should be an object!")?;

    let bpm = piece_json.get("bpm").ok_or_else(|| "bpm missing!")?;
    let bpm = bpm.as_u64().ok_or_else(|| "bpm must be uint!")?;
    let bpm = u8::try_from(bpm).map_err(|_| "Could not cast bpm to u8!")?;

    let tracks_json = piece_json
        .get("tracks")
        .ok_or_else(|| "tracks missing!")?
        .as_array()
        .ok_or_else(|| "tracks should be an array!")?;
    let mut tracks_by_id: IndexMap<String, VoiceTrack> = IndexMap::new();

    for track_json in tracks_json.iter() {
        let track = parse_track(track_json, &tracks_by_id)?;
        tracks_by_id.insert(track.id.clone(), track);
    }
    let tracks: Vec<VoiceTrack> = tracks_by_id.into_values().collect();

    Ok(Piece { bpm, tracks })
}

fn parse_track(
    track_json: &Value,
    tracks_by_id: &IndexMap<String, VoiceTrack>,
) -> Result<VoiceTrack, String> {
    let track_json = track_json
        .as_object()
        .ok_or_else(|| "Each track should be a JSON object!")?;

    let id = track_json
        .get("id")
        .ok_or_else(|| "id missing!")?
        .as_str()
        .ok_or_else(|| "id should be string!")?
        .to_string();

    let scale = track_json
        .get("scale")
        .ok_or_else(|| "scale missing!")?
        .as_str()
        .ok_or_else(|| "scale should be string!")?;
    let scale = str::parse::<Scale>(scale)?;

    let octave = track_json
        .get("octave")
        .ok_or_else(|| "octave missing!")?
        .as_i64()
        .ok_or_else(|| "octave should be int!")?;
    let octave = i8::try_from(octave).map_err(|_| "Could not convert octave to i8!")?;

    let start = track_json.get("start").ok_or_else(|| "start missing!")?;
    let start = parse_track_start(start, tracks_by_id)?;

    let notes = track_json.get("notes").ok_or_else(|| "notes missing!")?;
    let notes = parse_track_notes(notes, &scale, octave)?;

    Ok(VoiceTrack {
        id,
        scale,
        octave,
        start,
        notes,
    })
}

fn parse_track_start(
    track_start_json: &Value,
    tracks_by_id: &IndexMap<String, VoiceTrack>,
) -> Result<u32, String> {
    match track_start_json {
        Value::Number(start) => {
            let start = start
                .as_u64()
                .ok_or_else(|| "VoiceTrack start should be a uint!")?;
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
) -> Result<Vec<TimedNote>, String> {
    // matches e.g. 3, 1/3, /3.
    let duration_regex = Regex::new("^(\\d+)?(?:\\/(\\d+))?$").unwrap();
    parse_track_notes_recursive(track_notes_json, scale, octave, TICKS_PER_BEAT, &duration_regex, false)
}

fn parse_track_notes_recursive(
    track_notes_json: &Value,
    scale: &Scale,
    octave: i8,
    duration: u8,
    duration_regex: &Regex,
    halve_array: bool,
) -> Result<Vec<TimedNote>, String> {
    let mut notes: Vec<TimedNote> = Vec::new();
    let mut push_note = |position: Option<i8>, duration: u8| {
        notes.push(match position {
            Some(position) => {
                (Some(position), duration)
            }
            None => (None, duration),
        });
    };
    match track_notes_json {
        Value::Number(num) => {
            let position = num.as_i64().ok_or_else(|| "Note value must be int!")?;
            let position =
                i8::try_from(position).map_err(|_| "Could not cast note value to i8!")?;
            push_note(Some(position), duration);
        }
        Value::String(string) => {
            if string.as_str() != "" {
                return Err("Only an empty string can be used to signify a silence!".to_string());
            }
            push_note(None, duration);
        }
        Value::Null => {
            push_note(None, duration);
        }
        Value::Array(track_notes_json) => {
            for value in track_notes_json {
                let duration = if halve_array { duration / 2 } else { duration };
                let notes_deeper =
                    parse_track_notes_recursive(value, scale, octave, duration, &duration_regex, true)?;
                notes.extend(notes_deeper.into_iter());
            }
        }
        Value::Object(map_note_value) => {
            for (key, value) in map_note_value {
                let captures = duration_regex
                    .captures(key)
                    .ok_or_else(|| format!("Invalid duration specifier: {}", key))?;

                let numerator = match captures.get(1) {
                    None => 1,
                    Some(numerator) => str::parse::<u8>(numerator.as_str()).unwrap()
                };
                let denominator = match captures.get(2) {
                    None => 1,
                    Some(denominator) => str::parse::<u8>(denominator.as_str()).unwrap()
                };

                let duration = duration * numerator / denominator;
                let notes_deeper = parse_track_notes_recursive(value, scale, octave, duration, &duration_regex, false)?;
                notes.extend(notes_deeper.into_iter());
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
        {
            "bpm": 120,
            "tracks": [
                {
                    "id": "voice_1", "scale": "Cmaj", "octave": 4, "start": 0,
                    "notes": [
                        "", 0, 1, 2,
                        [{"3": 3}, [4, 3]], 2, 5,
                        1, [{"4": 3}, 5, 4, 3],
                        [2, 3, 2, 1, 0, 1, 0, -1]
                    ]
                },
                {
                    "id": "voice_2", "scale": "Gmaj", "octave": 4, "start": 12,
                    "notes": [
                        "", 0, 1, 2
                    ]
                }
            ]
        }"#;

        let _piece = parse_piece(data).unwrap();
    }
}
