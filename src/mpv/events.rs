use serde::{Serialize, Deserialize};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "kebab-case")]
pub enum MpvEvent {
    StartFile{ playlist_entry_id: u32 },
    EndFile { reason: EndFileReason, },
    Pause,
    Unpause,
    PlaybackRestart,
    FileLoaded,
    TracksChanged,
    ChapterChange,
    PropertyChange { name: String, data: f64 },
    UnknownEvent(String),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum EndFileReason {
    EOF,
    Stop,
    Quit,
    Error,
    Redirect,
    Unknown,
}
