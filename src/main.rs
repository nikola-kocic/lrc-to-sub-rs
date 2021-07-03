mod formatters;
mod lrc;
mod subtitle_gen;

use std::path::{Path, PathBuf};

use structopt::StructOpt;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::lrc::{parse_lrc_file, Lyrics};
use crate::subtitle_gen::f;

/// Show lyrics
#[derive(StructOpt, Debug)]
#[structopt(name = "lrc-to-sub-rs")]
struct Opt {
    /// Lyrics file to use.
    #[structopt(short = "l", long, parse(from_os_str))]
    lyrics: PathBuf,

    /// Subtitle file path to write output to.
    #[structopt(short = "o", long, parse(from_os_str))]
    out: PathBuf,
}

fn run(lrc_filepath: &Path, out: &Path) -> Result<(), String> {
    let lrc_file = parse_lrc_file(&lrc_filepath)
        .map_err(|e| format!("Parsing lrc file {:?} failed: {}", lrc_filepath, e))?;
    let lyrics = Lyrics::new(lrc_file);
    // println!("{:?}", lyrics);
    f(&lyrics, &out)?;
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
    if let Err(s) = run(&lyrics_filepath, &opt.out) {
        error!("{}", s);
    }
}
