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
    lrc-to-sub-rs [OPTIONS] --lyrics <lyrics> --out <out>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --long-text-secondary-color <long-text-secondary-color>
            Secondary color for long text, in AARRGGBB hex format [default: 00BF6C00]

    -l, --lyrics <lyrics>                                          Lyrics file to use
    -o, --out <out>                                                Subtitle file path to write output to
        --primary-color <primary-color>
            Primary color, in AARRGGBB hex format [default: 00FFFFFF]

        --secondary-color <secondary-color>
            Secondary color, in AARRGGBB hex format [default: 00EF8800]
```

## Examples
```
lrc-to-sub-rs --lyrics '/home/user/Rick Astley - Never Gonna Give You Up.lrc' --out sub.ass
```
