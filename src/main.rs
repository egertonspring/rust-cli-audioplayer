use rodio::{OutputStream, Sink, Source};
use minimp3::{Decoder as Mp3Decoder, Frame};
use std::{
    fs::File,
    io::{stdin},
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

fn main() {
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

    // Versuch: Default Output-Device nutzen
    let (_stream, handle) = match OutputStream::try_default() {
        Ok((s, h)) => (s, h),
        Err(e) => {
            eprintln!("Did not find default audio output device: {e}");
            eprintln!("Please make sure that PipeWire/ALSA is running and a device is connected to 3.5mm output, bluetooth, HDMI, GPIO or whatever your audio device connects to.");
            return;
        }
    };

    let sink = Sink::try_new(&handle).expect("Failed to create Sink");

    let source = rodio::buffer::SamplesBuffer::new(
        channels as u16,
        sample_rate as u32,
        samples,
    ).convert_samples::<f32>(); // <-- Typ explizit auf f32 setzen

    sink.append(source);
    sink.pause();

    let sink = std::sync::Arc::new(std::sync::Mutex::new(sink));

    let playback = std::sync::Arc::new(std::sync::Mutex::new(PlaybackState {
        playing: false,
        paused: false,
        stopped: false,
        start_time: None,
        paused_offset: 0.0,
    }));

    // progress thread
    {
        let sink = Arc::clone(&sink);
        let playback = Arc::clone(&playback);

        thread::spawn(move || loop {
            {
                let _s = sink.lock().unwrap();
                let p = playback.lock().unwrap();

                if p.stopped {
                    print!("\r[------------------------------] 0.0s / {:.1}s", total_duration);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }

                if p.playing && !p.paused {
                    let elapsed = p.start_time.unwrap().elapsed().as_secs_f32()
                        + p.paused_offset;

                    print_progress(elapsed, total_duration);
                }
            }

            thread::sleep(Duration::from_millis(200));
        });
    }

    // user controls
    println!("controls: p = play | a = pause | s = stop | q = quit");
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "p" => {
                let mut p = playback.lock().unwrap();
                sink.lock().unwrap().play();

                if !p.playing || p.stopped {
                    // complete restart
                    p.start_time = Some(Instant::now());
                    p.paused_offset = 0.0;
                } else if p.paused {
                    // resume
                    p.start_time = Some(Instant::now());
                }

                p.playing = true;
                p.paused = false;
                p.stopped = false;
                println!("{}", file_path);
                println!("▶ play");
            }
            "a" => {
                let mut p = playback.lock().unwrap();
                sink.lock().unwrap().pause();

                if !p.paused {
                    p.paused_offset += p.start_time.unwrap().elapsed().as_secs_f32();
                }

                p.paused = true;
                println!("⏸ pause");
            }
            "s" => {
                let mut p = playback.lock().unwrap();
                sink.lock().unwrap().stop();

                p.playing = false;
                p.paused = false;
                p.stopped = true;
                p.paused_offset = 0.0;

                println!("⏹ stop");
            }
            "q" => {
                println!("x quitting...");
                break;
            }
            _ => println!("unknown control"),
        }
    }
}

/// progressbar
fn print_progress(elapsed: f32, total: f32) {
    let width = 30;
    let ratio = (elapsed / total).clamp(0.0, 1.0);

    let filled = (ratio * width as f32) as usize;
    let empty = width - filled;

    let bar = format!(
        "[{}{}]",
        "█".repeat(filled),
        "-".repeat(empty)
    );

    print!("\r{} {:.1}s / {:.1}s", bar, elapsed, total);
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
