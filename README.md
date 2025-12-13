# rust-cli-audioplayer
is an audio-player that lives in the commandline. Especially written for and tested local MP3 files! (GenAI helped me!)
The player now also supports HTTP URLs. It simply downloads the track and plays it locally. This is easier than streaming the track from HTTP. It also uses much less RAM.
## macOS
It compiles and runs on macOS 26 immediately.


## Raspberry Pi OS
needs libasound2-dev
```
sudo apt-get update
sudo apt-get install libasound2-dev pkg-config
```

## Ubuntu 25.10
needs libasound2-dev
```
sudo apt-get update
sudo apt-get install libasound2-dev pkg-config
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
