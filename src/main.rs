mod formatters;
mod lrc;
mod subtitle_gen;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use structopt::StructOpt;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::formatters::format_duration;
use crate::lrc::{parse_lrc_file, Lyrics, LyricsTiming};
use crate::subtitle_gen::f;

/// Show lyrics
#[derive(StructOpt, Debug)]
#[structopt(name = "lrc-to-sub-rs")]
struct Opt {
    /// Lyrics file to use.
    #[structopt(short = "l", long, parse(from_os_str))]
    lyrics: PathBuf,
}

fn run(lrc_filepath: PathBuf) -> Result<(), String> {
    let lrc_file = parse_lrc_file(&lrc_filepath)
        .map_err(|e| format!("Parsing lrc file {:?} failed: {}", lrc_filepath, e))?;
    let lyrics = Lyrics::new(lrc_file);
    // println!("{:?}", lyrics);
    f(&lyrics);
    Ok(())
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .write_style(env_logger::WriteStyle::Auto)
        .format_module_path(false)
        .format_timestamp_nanos()
        .init();

    let opt = Opt::from_args();
    let lyrics_filepath = opt.lyrics;
    if !lyrics_filepath.is_file() {
        error!("Lyrics path must be a file");
        return;
    }
    if let Err(s) = run(lyrics_filepath) {
        error!("{}", s);
    }
}
