// src/audio/engine.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    pub current_step: Arc<Mutex<usize>>,
    pub bpm: f32,
    sample_rate: f32,
}

impl AudioEngine {
    pub fn new(bpm: f32) -> Self {
        Self {
            current_step: Arc::new(Mutex::new(0)),
            bpm,
            sample_rate: 44100.0,
        }
    }

    pub fn start(&self, pattern: Arc<Mutex<Pattern>>) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("No output device available")?;
        
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;
        
        let samples_per_step = (60.0 / self.bpm * sample_rate / 4.0) as usize;
        let current_step = self.current_step.clone();
        
        let mut sample_counter = 0;
        
        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    // ステップ進行
                    if sample_counter >= samples_per_step {
                        sample_counter = 0;
                        let mut step = current_step.lock().unwrap();
                        *step = (*step + 1) % 16;
                    }
                    
                    // ここでパターンに基づいてオーディオ生成
                    *sample = 0.0;  // TODO: 実際の音声生成
                    sample_counter += 1;
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None
        )?;
        
        stream.play()?;
        std::mem::forget(stream);  // ストリームを保持
        
        Ok(())
    }
}