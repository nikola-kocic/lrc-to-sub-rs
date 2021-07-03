# lrc-to-sub-rs
Convers the lyrics of the song to the karaoke subtitle.

Only [Enhanced LRC format](https://en.wikipedia.org/wiki/LRC_(file_format)#Enhanced_format) for lyrics is supported.

## Installation
Project builds with the Rust stable version, using the Cargo build system.

`cargo build --release`

Resulting binary is at `./target/release/lrc-to-sub-rs`

## Usage
```
USAGE:
    lrc-to-sub-rs --lyrics <lyrics>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --lyrics <lyrics>    Lyrics file to use
```

## Examples
```
lrc-to-sub-rs --lyrics '/home/user/Rick Astley - Never Gonna Give You Up.lrc'
```
