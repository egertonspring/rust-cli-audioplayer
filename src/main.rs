use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use id3::TagLike;
use rodio::{buffer::SamplesBuffer, DeviceSinkBuilder, Player};

use std::{
    fs::File,
    io::Write,
    num::NonZero,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::DecoderOptions,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia::default::{get_codecs, get_probe};

struct PlaybackState {
    paused: bool,
    start_time: Instant,
    paused_offset: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Arc::new(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "song.mp3".to_string()),
    );

    println!("Using file '{}'\n", file_path);
    read_id3_tags(&file_path);

    let (sample_rate, channels, samples) = decode_mp3(&file_path)?;
    let total_duration = samples.len() as f32 / sample_rate as f32 / channels as f32;

    enable_raw_mode()?;

    // rodio 0.22: DeviceSinkBuilder ersetzt OutputStream, Player ersetzt Sink
    let mut stream_handle = DeviceSinkBuilder::open_default_sink()?;
    stream_handle.log_on_drop(false); // kein Log bei Drop (stört raw-mode TUI)
    let mut player = Player::connect_new(stream_handle.mixer());

    // rodio 0.22: SamplesBuffer nimmt NonZero<u16>/NonZero<u32>, Samples sind f32
    let source = SamplesBuffer::new(
        NonZero::new(channels as u16).expect("channels > 0"),
        NonZero::new(sample_rate as u32).expect("sample_rate > 0"),
        samples.clone(),
    );

    player.append(source);

    let mut playback = PlaybackState {
        paused: false,
        start_time: Instant::now(),
        paused_offset: 0.0,
    };

    println!("controls: SPACE = pause/resume | ←/→ = seek | q = quit");

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Right => {
                        let current = current_time(&playback);
                        let new_pos = (current + 10.0).min(total_duration);

                        player.stop();
                        player = new_player(
                            stream_handle.mixer(),
                            &samples,
                            sample_rate,
                            channels,
                            new_pos,
                        );

                        playback.start_time = Instant::now();
                        playback.paused_offset = new_pos;
                        playback.paused = false;
                    }

                    KeyCode::Left => {
                        let current = current_time(&playback);
                        let new_pos = (current - 10.0).max(0.0);

                        player.stop();
                        player = new_player(
                            stream_handle.mixer(),
                            &samples,
                            sample_rate,
                            channels,
                            new_pos,
                        );

                        playback.start_time = Instant::now();
                        playback.paused_offset = new_pos;
                        playback.paused = false;
                    }

                    KeyCode::Char('q') => break,

                    KeyCode::Char(' ') => {
                        if playback.paused {
                            player.play();
                            playback.start_time = Instant::now();
                            playback.paused = false;
                        } else {
                            player.pause();
                            playback.paused_offset +=
                                playback.start_time.elapsed().as_secs_f32();
                            playback.paused = true;
                        }
                    }

                    _ => {}
                }
            }
        }

        let elapsed = current_time(&playback);
        let status = if playback.paused { "⏸" } else { "▶" };

        print_progress(elapsed, total_duration, status);

        if player.empty() {
            println!("\nSong finished.");
            break;
        }
    }

    disable_raw_mode()?;
    println!();
    Ok(())
}

fn current_time(p: &PlaybackState) -> f32 {
    if p.paused {
        p.paused_offset
    } else {
        p.paused_offset + p.start_time.elapsed().as_secs_f32()
    }
}

fn new_player(
    mixer: &rodio::mixer::Mixer,
    samples: &[f32],
    sample_rate: usize,
    channels: usize,
    start_sec: f32,
) -> Player {
    let start_sample = (start_sec * sample_rate as f32 * channels as f32) as usize;

    let source = SamplesBuffer::new(
        NonZero::new(channels as u16).expect("channels > 0"),
        NonZero::new(sample_rate as u32).expect("sample_rate > 0"),
        samples[start_sample..].to_vec(),
    );

    let player = Player::connect_new(mixer);
    player.append(source);
    player.play();
    player
}

fn decode_mp3(path: &str) -> Result<(usize, usize, Vec<f32>), Box<dyn std::error::Error>> {
    let file = File::open(Path::new(path))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let probed = get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;

    let track = format.default_track().ok_or("No track")?;

    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let sample_rate = track.codec_params.sample_rate.ok_or("No sample rate")?;
    let channels = track.codec_params.channels.ok_or("No channels")?.count();

    let mut samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };

        let decoded = decoder.decode(&packet)?;

        match decoded {
            AudioBufferRef::S16(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..channels {
                        samples.push(buf.chan(ch)[frame] as f32 / i16::MAX as f32);
                    }
                }
            }
            AudioBufferRef::F32(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..channels {
                        samples.push(buf.chan(ch)[frame]);
                    }
                }
            }
            _ => {}
        }
    }

    Ok((sample_rate as usize, channels, samples))
}

fn read_id3_tags(path: &str) {
    if let Ok(tag) = id3::Tag::read_from_path(path) {
        println!("ID3 Tags:");
        if let Some(title) = tag.title() {
            println!("  Title : {}", title);
        }
        if let Some(artist) = tag.artist() {
            println!("  Artist: {}", artist);
        }
        if let Some(album) = tag.album() {
            println!("  Album : {}", album);
        }
        println!();
    }
}

fn format_time(sec: f32) -> String {
    let total = sec.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn print_progress(elapsed: f32, total: f32, status: &str) {
    let width = 30;
    let ratio = (elapsed / total).clamp(0.0, 1.0);
    let filled = (ratio * width as f32) as usize;

    let bar = format!("[{}{}]", "█".repeat(filled), "-".repeat(width - filled));

    print!(
        "\r{} {} / {} {}",
        bar,
        format_time(elapsed),
        format_time(total),
        status
    );

    std::io::stdout().flush().unwrap();
}
