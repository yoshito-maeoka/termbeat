// src/sequencer/pattern.rs
#[derive(Clone, Debug)]
pub struct Step {
    pub active: bool,
    pub velocity: u8,  // 0-127
    pub note: u8,      // MIDI note number
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub steps: Vec<Vec<Step>>,  // [track][step]
    pub length: usize,           // 通常16ステップ
}

#[derive(Clone, Debug)]
pub enum InstrumentType {
    Kick,
    Snare,
    HiHat,
    Bass,
    Pad,
    Lead,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub instrument: InstrumentType,
    pub volume: f32,
    pub pan: f32,        // -1.0 (L) to 1.0 (R)
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
}