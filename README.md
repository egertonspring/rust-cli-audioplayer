# rust-cli-audioplayer
is an audio-player that lives in the commandline. Especially written for and tested with MP3! (GenAI helped me!)

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
p = play
a = pause
s = stop
q = quit


## Installing Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
. "$HOME/.cargo/env"
```
 
## Compiling
```
`cd rust-cli-audioplayer
cargo build --release
```

## Running
```
`cd rust-cli-audioplayer
./target/release/cli-audioplayer-rs Your-Song.mp3
```
