// src/audio/synth.rs
use fundsp::prelude::*;

pub fn kick_synth() -> An<impl AudioUnit> {
    // シンプルなキック: 低周波サイン波 + エンベロープ
    let freq = 60.0;
    let decay = 0.3;
    
    (sine_hz(freq) * envelope(|t| exp(-t / decay)))
        >> lowpass_hz(200.0, 1.0)
}

pub fn bass_synth(note: f32) -> An<impl AudioUnit> {
    // ベース: サイン波 + サブオクターブ
    let freq = midi_to_hz(note);
    
    (sine_hz(freq) * 0.7 + sine_hz(freq / 2.0) * 0.3)
        >> lowpass_hz(800.0, 2.0)
}

pub fn pad_synth(note: f32) -> An<impl AudioUnit> {
    // パッド: 複数のソウ波 + リバーブ風
    let freq = midi_to_hz(note);
    
    (saw_hz(freq) + saw_hz(freq * 1.01) + saw_hz(freq * 0.99))
        * 0.3
        >> lowpass_hz(2000.0, 0.5)
}

fn midi_to_hz(note: f32) -> f32 {
    440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
}