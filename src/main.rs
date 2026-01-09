use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};
use std::sync::{Arc, Mutex};
use std::thread;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};

// アプリケーションの状態
struct App {
    pattern: Pattern,
    current_step: usize,
    selected_track: usize,
    selected_step: usize,
    playing: bool,
    bpm: u32,
    audio_engine: Arc<Mutex<AudioEngine>>,
    export_message: Option<String>,
}

// パターンデータ
struct Pattern {
    tracks: Vec<Track>,
    length: usize,
}

struct Track {
    name: String,
    steps: Vec<bool>,
}

// オーディオエンジン
struct AudioEngine {
    sample_rate: f32,
    time: f32,
}

impl AudioEngine {
    fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            time: 0.0,
        }
    }

    // キックドラム生成（低周波数のサイン波 + ピッチダウン）
    fn generate_kick(&mut self) -> f32 {
        let duration = 0.3;
        let freq_start = 150.0;
        let freq_end = 40.0;
        
        if self.time < duration {
            let t = self.time / duration;
            let freq = freq_start + (freq_end - freq_start) * t;
            let phase = 2.0 * std::f32::consts::PI * freq * self.time;
            let envelope = (1.0 - t).powf(2.0);
            
            self.time += 1.0 / self.sample_rate;
            (phase.sin() * envelope * 0.5).clamp(-1.0, 1.0)
        } else {
            self.time = 0.0;
            0.0
        }
    }

    // スネア（ノイズ + サイン波）
    fn generate_snare(&mut self) -> f32 {
        let duration = 0.15;
        
        if self.time < duration {
            let t = self.time / duration;
            let envelope = (1.0 - t).powf(1.5);
            
            // ノイズ成分
            let noise = (rand_xorshift() % 1000) as f32 / 1000.0 - 0.5;
            // トーン成分
            let tone = (2.0 * std::f32::consts::PI * 180.0 * self.time).sin();
            
            self.time += 1.0 / self.sample_rate;
            ((noise * 0.7 + tone * 0.3) * envelope * 0.4).clamp(-1.0, 1.0)
        } else {
            self.time = 0.0;
            0.0
        }
    }

    // ハイハット（高周波ノイズ）
    fn generate_hihat(&mut self) -> f32 {
        let duration = 0.05;
        
        if self.time < duration {
            let t = self.time / duration;
            let envelope = (1.0 - t).powf(3.0);
            let noise = (rand_xorshift() % 1000) as f32 / 1000.0 - 0.5;
            
            self.time += 1.0 / self.sample_rate;
            (noise * envelope * 0.2).clamp(-1.0, 1.0)
        } else {
            self.time = 0.0;
            0.0
        }
    }

    // ベース（低音サイン波）
    fn generate_bass(&mut self, note: f32) -> f32 {
        let duration = 0.2;
        
        if self.time < duration {
            let t = self.time / duration;
            let freq = 440.0 * 2.0_f32.powf((note - 69.0) / 12.0);
            let phase = 2.0 * std::f32::consts::PI * freq * self.time;
            let envelope = (1.0 - t).powf(0.5);
            
            self.time += 1.0 / self.sample_rate;
            (phase.sin() * envelope * 0.3).clamp(-1.0, 1.0)
        } else {
            self.time = 0.0;
            0.0
        }
    }

    fn reset(&mut self) {
        self.time = 0.0;
    }
}

// シンプルな乱数生成器（std::randomを避けるため）
static mut XORSHIFT_STATE: u32 = 123456789;
fn rand_xorshift() -> u32 {
    unsafe {
        XORSHIFT_STATE ^= XORSHIFT_STATE << 13;
        XORSHIFT_STATE ^= XORSHIFT_STATE >> 17;
        XORSHIFT_STATE ^= XORSHIFT_STATE << 5;
        XORSHIFT_STATE
    }
}

// トリガーされた音声
#[derive(Clone, Copy)]
enum SoundTrigger {
    Kick,
    Snare,
    HiHat,
    Bass(f32), // MIDI note
}

impl App {
    fn new() -> App {
        let tracks = vec![
            Track {
                name: "Kick".to_string(),
                steps: vec![false; 16],
            },
            Track {
                name: "Snare".to_string(),
                steps: vec![false; 16],
            },
            Track {
                name: "Hi-Hat".to_string(),
                steps: vec![false; 16],
            },
            Track {
                name: "Bass".to_string(),
                steps: vec![false; 16],
            },
        ];

        App {
            pattern: Pattern { tracks, length: 16 },
            current_step: 0,
            selected_track: 0,
            selected_step: 0,
            playing: false,
            bpm: 120,
            audio_engine: Arc::new(Mutex::new(AudioEngine::new(44100.0))),
            export_message: None,
        }
    }

    fn toggle_step(&mut self) {
        let step = &mut self.pattern.tracks[self.selected_track].steps[self.selected_step];
        *step = !*step;
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        self.selected_step = ((self.selected_step as i32 + dx)
            .rem_euclid(self.pattern.length as i32)) as usize;
        self.selected_track = ((self.selected_track as i32 + dy)
            .rem_euclid(self.pattern.tracks.len() as i32)) as usize;
    }

    fn toggle_play(&mut self) {
        self.playing = !self.playing;
    }

    fn export_to_wav(&self, filename: &str, loops: usize) -> Result<(), Box<dyn Error>> {
        let sample_rate = 44100;
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(filename, spec)?;
        
        let samples_per_step = (60.0 / self.bpm as f32 * sample_rate as f32 / 4.0) as usize;
        let total_steps = self.pattern.length * loops;

        let mut active_sounds: Vec<(SoundTrigger, AudioEngine)> = Vec::new();

        for step in 0..total_steps {
            let current_step = step % self.pattern.length;
            
            // このステップでトリガーされる音を追加
            for (track_idx, track) in self.pattern.tracks.iter().enumerate() {
                if track.steps[current_step] {
                    let trigger = match track_idx {
                        0 => SoundTrigger::Kick,
                        1 => SoundTrigger::Snare,
                        2 => SoundTrigger::HiHat,
                        3 => SoundTrigger::Bass(36.0),
                        _ => continue,
                    };
                    let engine = AudioEngine::new(sample_rate as f32);
                    active_sounds.push((trigger, engine));
                }
            }

            // このステップのサンプルを生成
            for _ in 0..samples_per_step {
                let mut mix = 0.0f32;

                active_sounds.retain_mut(|(trigger, engine)| {
                    let s = match trigger {
                        SoundTrigger::Kick => engine.generate_kick(),
                        SoundTrigger::Snare => engine.generate_snare(),
                        SoundTrigger::HiHat => engine.generate_hihat(),
                        SoundTrigger::Bass(note) => engine.generate_bass(*note),
                    };
                    
                    mix += s;
                    engine.time > 0.0 || s.abs() > 0.001
                });

                let sample = (mix.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                writer.write_sample(sample)?;
            }
        }

        writer.finalize()?;
        Ok(())
    }

    fn get_active_sounds(&self) -> Vec<SoundTrigger> {
        let mut triggers = Vec::new();
        
        for (track_idx, track) in self.pattern.tracks.iter().enumerate() {
            if track.steps[self.current_step] {
                let trigger = match track_idx {
                    0 => SoundTrigger::Kick,
                    1 => SoundTrigger::Snare,
                    2 => SoundTrigger::HiHat,
                    3 => SoundTrigger::Bass(36.0), // C1
                    _ => continue,
                };
                triggers.push(trigger);
            }
        }
        
        triggers
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // セットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    
    // オーディオストリーム起動
    let audio_engine = app.audio_engine.clone();
    let triggered_sounds = Arc::new(Mutex::new(Vec::<SoundTrigger>::new()));
    let triggered_sounds_clone = triggered_sounds.clone();
    
    thread::spawn(move || {
        start_audio_stream(audio_engine, triggered_sounds_clone).unwrap();
    });
    
    let res = run_app(&mut terminal, &mut app, triggered_sounds);

    // クリーンアップ
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn start_audio_stream(
    audio_engine: Arc<Mutex<AudioEngine>>,
    triggered_sounds: Arc<Mutex<Vec<SoundTrigger>>>,
) -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("No output device")?;
    let config = device.default_output_config()?;

    let mut active_sounds: Vec<(SoundTrigger, AudioEngine)> = Vec::new();

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // 新しいトリガーをチェック
            if let Ok(mut triggers) = triggered_sounds.try_lock() {
                for trigger in triggers.drain(..) {
                    let engine = AudioEngine::new(44100.0);
                    active_sounds.push((trigger, engine));
                }
            }

            for sample in data.iter_mut() {
                let mut mix = 0.0f32;

                // すべてのアクティブな音を生成してミックス
                active_sounds.retain_mut(|(trigger, engine)| {
                    let s = match trigger {
                        SoundTrigger::Kick => engine.generate_kick(),
                        SoundTrigger::Snare => engine.generate_snare(),
                        SoundTrigger::HiHat => engine.generate_hihat(),
                        SoundTrigger::Bass(note) => engine.generate_bass(*note),
                    };
                    
                    mix += s;
                    
                    // 音が終わったら削除
                    engine.time > 0.0 || s.abs() > 0.001
                });

                *sample = mix.clamp(-1.0, 1.0);
            }
        },
        |err| eprintln!("Audio error: {}", err),
        None,
    )?;

    stream.play()?;
    std::mem::forget(stream);

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    triggered_sounds: Arc<Mutex<Vec<SoundTrigger>>>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => app.toggle_step(),
                    KeyCode::Enter => app.toggle_play(),
                    KeyCode::Left => app.move_cursor(-1, 0),
                    KeyCode::Right => app.move_cursor(1, 0),
                    KeyCode::Up => app.move_cursor(0, -1),
                    KeyCode::Down => app.move_cursor(0, 1),
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        // WAVエクスポート
                        app.export_message = Some("Exporting...".to_string());
                        let filename = format!("rhythm-box-{}.wav", 
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                        match app.export_to_wav(&filename, 4) {
                            Ok(_) => {
                                app.export_message = Some(format!("✓ Exported to {}", filename));
                            }
                            Err(e) => {
                                app.export_message = Some(format!("✗ Export failed: {}", e));
                            }
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        // メッセージクリア
                        app.export_message = None;
                    }
                    _ => {}
                }
            }
        }

        // 再生中はステップを進める
        if app.playing {
            std::thread::sleep(std::time::Duration::from_millis(
                (60000 / app.bpm / 4) as u64,
            ));
            app.current_step = (app.current_step + 1) % app.pattern.length;
            
            // アクティブな音をトリガー
            let sounds = app.get_active_sounds();
            if let Ok(mut triggers) = triggered_sounds.try_lock() {
                triggers.extend(sounds);
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Sequencer
            Constraint::Length(5),  // Controls
            Constraint::Length(2),  // Export message
        ])
        .split(f.size());

    // ヘッダー
    let title = vec![
        Line::from(vec![
            Span::styled("🎵 ", Style::default().fg(Color::Cyan)),
            Span::styled("Rust Rhythm Box", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" | BPM: {} | ", app.bpm), Style::default().fg(Color::Yellow)),
            Span::styled(
                if app.playing { "▶ PLAYING" } else { "⏸ STOPPED" },
                Style::default().fg(if app.playing { Color::Green } else { Color::Red })
            ),
        ]),
    ];
    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // シーケンサーグリッド
    draw_sequencer(f, chunks[1], app);

    // コントロール説明
    let controls = vec![
        Line::from("Controls:"),
        Line::from("  ← → ↑ ↓  : Move cursor"),
        Line::from("  Space     : Toggle step  |  E : Export WAV (4 loops)"),
        Line::from("  Enter     : Play/Stop    |  Q : Quit"),
    ];
    let controls_widget = Paragraph::new(controls)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(controls_widget, chunks[2]);

    // エクスポートメッセージ
    if let Some(msg) = &app.export_message {
        let color = if msg.starts_with('✓') {
            Color::Green
        } else if msg.starts_with('✗') {
            Color::Red
        } else {
            Color::Yellow
        };
        
        let export_msg = Paragraph::new(msg.as_str())
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(export_msg, chunks[3]);
    }
}

fn draw_sequencer(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Step Sequencer (16 steps)");
    let inner = block.inner(area);
    f.render_widget(block, area);

    // 各トラックを描画
    let track_height = inner.height / app.pattern.tracks.len() as u16;
    
    for (track_idx, track) in app.pattern.tracks.iter().enumerate() {
        let track_area = Rect {
            x: inner.x,
            y: inner.y + (track_idx as u16 * track_height),
            width: inner.width,
            height: track_height,
        };

        let mut line_content = vec![
            Span::styled(
                format!("{:8} ", track.name),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            )
        ];

        // 16ステップを描画
        for (step_idx, &active) in track.steps.iter().enumerate() {
            let is_current = step_idx == app.current_step && app.playing;
            let is_selected = step_idx == app.selected_step && track_idx == app.selected_track;

            let symbol = if active { "●" } else { "○" };
            
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if active {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            line_content.push(Span::styled(format!("{} ", symbol), style));
        }

        let paragraph = Paragraph::new(Line::from(line_content));
        f.render_widget(paragraph, track_area);
    }
}