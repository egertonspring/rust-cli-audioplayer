# rust-cli-audioplayer
is an audio-player that lives in the commandline. Especially written for and tested with local MP3 files! (GenAI helped me!)
The player now also supports HTTP URLs. It simply downloads the track and plays it locally. This is easier than streaming the track from HTTP. It also uses much less RAM.
## macOS
It compiles and runs on macOS 26 immediately.


## Raspberry Pi OS (trixie)
needs libasound2-dev libssl-dev and pkg-config installed
Pulseaudio will help you to connect your bluetooth speakers correctly and play the music when no HDMI screen is connected but a little 3.5 inch touch screen.
The reason is that RPiOS trixie changed from pulseaudio to pipewire.
```
sudo apt-get update
sudo apt-get install libasound2-dev pkg-config libssl-dev -y # to compile the player completely
sudo apt-get install pulseaudio -y # to use bluetooth speakers when no HDMI screen is connected
```

## Ubuntu 25.10
needs libasound2-dev libssl-dev and pkg-config installed
```
sudo apt-get update
sudo apt-get install libasound2-dev pkg-config libssl-dev
```

## controls
```
SPACE = play/pause | q = quit | <- skip backwards 5 seconds | -> skip forward 5 seconds
```

## Installing Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
. "$HOME/.cargo/env"
```
 
## Compiling
```
cd rust-cli-audioplayer
cargo build --release
```

## Running
```
cd rust-cli-audioplayer
./target/release/cli-audioplayer-rs Your-Song.mp3
```
