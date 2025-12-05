use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use rodio::{OutputStream, Sink, Source};
use minimp3::{Decoder as Mp3Decoder, Frame};
use std::{
    fs::File,
    sync::{Arc},
    thread,
    time::{Duration, Instant},
};

struct PlaybackState {
    playing: bool,
    paused: bool,
    stopped: bool,
    start_time: Option<Instant>,
    paused_offset: f32, // seconds
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::env::args()
        .nth(1)
        .unwrap_or("Your-Song.mp3".to_string());

    println!("loading file: {}", file_path);

    let (sample_rate, channels, samples) = decode_mp3(&file_path).expect("failed to decode");

    let total_samples = samples.len() as f32;
    let total_duration =
        total_samples / (sample_rate as f32) / (channels as f32);

    println!(
        "mp3 loaded {} Hz, {} channels, {:.2} seconds",
        sample_rate, channels, total_duration
    );
    enable_raw_mode()?;

    // Versuch: Default Output-Device nutzen
    let (_stream, handle) = match OutputStream::try_default() {
        Ok((s, h)) => (s, h),
        Err(e) => {
            eprintln!("Did not find default audio output device: {e}");
            eprintln!("Please make sure that PipeWire/ALSA is running and a device is connected to 3.5mm output, bluetooth, HDMI, GPIO or whatever your audio device connects to.");
            return Ok(());
        }
    };

    let sink = Sink::try_new(&handle).expect("Failed to create Sink");

    let source = rodio::buffer::SamplesBuffer::new(
        channels as u16,
        sample_rate as u32,
        samples,
    ).convert_samples::<f32>(); // <-- Typ explizit auf f32 setzen

    sink.append(source);
    //sink.pause();

    let sink = std::sync::Arc::new(std::sync::Mutex::new(sink));

    let playback = std::sync::Arc::new(std::sync::Mutex::new(PlaybackState {
        playing: true,
        paused: false,
        stopped: false,
        start_time: Some(Instant::now()),
        paused_offset: 0.0,
    }));

    // progress thread
    {
        let _sink = Arc::clone(&sink);
        let playback = Arc::clone(&playback);

        thread::spawn(move || loop {
            {
                let p = playback.lock().unwrap();
        
                let elapsed = if p.stopped {
                    0.0
                } else if p.paused {
                    p.paused_offset
                } else if p.playing {
                    p.paused_offset + p.start_time.unwrap().elapsed().as_secs_f32()
                } else {
                    0.0
                };
        
                let status = if p.stopped {
                    "⏹ stop"
                } else if p.paused {
                    "⏸ pause"
                } else if p.playing {
                    "▶ play"
                } else {
                    ""
                };
        
                print_progress(elapsed, total_duration, status);
            }
        
            thread::sleep(Duration::from_millis(200));
        });
        
        
        
    }

    // user controls
    println!("controls: SPACE = toggle play/pause | s = stop | q = quit");

loop {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    println!("x quitting...");
                    break;
                }
                KeyCode::Char('s') => {
                    let mut p = playback.lock().unwrap();
                    sink.lock().unwrap().stop();
                    p.playing = false;
                    p.paused = false;
                    p.stopped = true;
                    p.paused_offset = 0.0;
                    println!("⏹ stop");
                }
                KeyCode::Char('p') => {
                    // optional: Play-Taste behalten
                }
                KeyCode::Char('a') => {
                    // optional: Pause-Taste behalten
                }

                // 🔥 HIER ist die gewünschte Leertaste ohne Enter
                KeyCode::Char(' ') => {
                    let mut p = playback.lock().unwrap();
                    let s = sink.lock().unwrap();
                
                    if !p.playing || p.stopped {
                        s.play();
                        p.start_time = Some(Instant::now());
                        p.paused_offset = 0.0;
                        p.playing = true;
                        p.paused = false;
                        p.stopped = false;
                    } else if p.paused {
                        s.play();
                        p.start_time = Some(Instant::now());
                        p.paused = false;
                    } else {
                        s.pause();
                        p.paused_offset += p.start_time.unwrap().elapsed().as_secs_f32();
                        p.paused = true;
                    }
                }
                

                _ => {}
            }
        }
    }
}
disable_raw_mode()?;
Ok(())
}

fn format_time(sec: f32) -> String {
    let total = sec.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// progressbar
fn print_progress(elapsed: f32, total: f32, status: &str) {
    let width = 30;
    let ratio = (elapsed / total).clamp(0.0, 1.0);
    let filled = (ratio * width as f32) as usize;
    let empty = width - filled;

    let bar = format!(
        "[{}{}]",
        "█".repeat(filled),
        "-".repeat(empty)
    );

    let t_elapsed = format_time(elapsed);
    let t_total   = format_time(total);

    print!("\r{} {} / {} {}", bar, t_elapsed, t_total, status);
    print!("{}", " ".repeat(10)); // alte Zeichen überschreiben
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}



/// MP3 → PCM
fn decode_mp3(path: &str) -> Result<(usize, usize, Vec<i16>), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut decoder = Mp3Decoder::new(file);

    let mut sample_rate = 44100;
    let mut channels = 2;
    let mut pcm: Vec<i16> = Vec::new();

    loop {
        match decoder.next_frame() {
            Ok(Frame {
                data,
                sample_rate: sr,
                channels: ch,
                ..
            }) => {
                sample_rate = sr;
                channels = ch;
                pcm.extend_from_slice(&data);
            }
            Err(_) => break,
        }
    }

    Ok((sample_rate.try_into().unwrap(), channels, pcm))
}
