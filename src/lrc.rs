use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use std::convert::TryInto;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::usize;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

fn lines_from_file<P: AsRef<Path>>(filepath: P) -> Result<Vec<String>, String> {
    let file = File::open(filepath).map_err(|e| e.to_string())?;
    io::BufReader::new(file)
        .lines()
        .map(|l| l.map_err(|e| e.to_string()))
        .collect()
}

struct TimedLocation {
    time: Duration,
    source_line_index: usize,
    line_char_from_index: usize, // from this character in line
    line_char_to_index: usize,   // to this character in line
}

impl fmt::Debug for TimedLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TimedLocation {{ time: {}, from: {}, to: {} }}",
            self.time.as_micros(),
            self.line_char_from_index,
            self.line_char_to_index,
        )
    }
}

#[derive(Debug)]
struct TextLine {
    source_line_index: usize,
    text: String,
}

#[derive(Debug)]
enum Tag {
    Time(std::time::Duration),
    Offset(i64), // ms
    Unknown,
}

#[derive(Debug)]
enum LrcElement {
    TextLine(TextLine),
    TimedLocation(TimedLocation),
    Tag(Tag),
}

#[derive(Debug)]
pub struct LrcFile {
    metadata: Vec<(String, String)>,
    lines: Vec<TextLine>,
    timed_texts_lines: Vec<TimedLocation>,
}

fn duration_from_time_string(time_str: &str) -> Result<Duration, String> {
    let minutes_str = &time_str[0..2];
    let minutes = minutes_str
        .parse::<u64>()
        .map_err(|e| format!("Bad minutes format ({}): {}", minutes_str, e.to_string()))?;

    if &time_str[2..3] != ":" {
        return Err("Bad seconds divider".to_owned());
    }
    let seconds_str = &time_str[3..5];
    let seconds = seconds_str
        .parse::<u64>()
        .map_err(|e| format!("Bad seconds format ({}): {}", seconds_str, e.to_string()))?;

    let ms_divider_char = &time_str[5..6];
    if ms_divider_char != "." && ms_divider_char != ":" {
        return Err(format!("Bad milliseconds divider: {}", ms_divider_char));
    }
    let centiseconds_str = &time_str[6..8];
    let centiseconds = centiseconds_str.parse::<u64>().map_err(|e| {
        format!(
            "Bad centiseconds format ({}): {}",
            centiseconds_str,
            e.to_string()
        )
    })?;

    Ok(Duration::from_micros(
        ((((minutes * 60) + seconds) * 100) + centiseconds) * 10000,
    ))
}

fn parse_tag(tag_content: &str) -> Result<Tag, String> {
    trace!("Parsing tag content {}", tag_content);
    let first_char_in_tag_name = tag_content
        .chars()
        .next()
        .ok_or("Tag content must not be empty")?;
    if first_char_in_tag_name.is_ascii_digit() {
        let time = duration_from_time_string(tag_content)?;
        Ok(Tag::Time(time))
    } else {
        let mut parts = tag_content.split(':');
        let tag_first_part = parts
            .next()
            .expect("Should never happen; split always returns at least one element");
        match tag_first_part {
            "offset" => {
                let offset_val_str = parts.next().ok_or_else(|| {
                    format!("Wrong offset tag format (missing ':'): {}", tag_content)
                })?;
                let offset = offset_val_str.parse::<i64>().map_err(|e| {
                    format!("Bad offset format ({}): {}", offset_val_str, e.to_string())
                })?;
                Ok(Tag::Offset(offset))
            }
            _ => Ok(Tag::Unknown),
        }
    }
}

fn parse_lrc_line(line: &str, source_line_index: usize) -> Result<Vec<LrcElement>, String> {
    let mut line_elements = Vec::new();
    trace!("Parsing line {}", line);
    match line.chars().next() {
        None => {}
        Some('[') => {
            let mut current_text_index_in_line = 0;
            let parts = line.split('[');
            let mut texts = Vec::new();
            for part in parts.skip(1) {
                let mut subparts = part.split(']');
                let tag_content = subparts
                    .next()
                    .expect("Should never happen; split always returns at least one element");
                let mut text_len: usize = 0;

                if let Some(text) = subparts.next() {
                    text_len = text.bytes().len();
                    texts.push(text);
                }

                match parse_tag(tag_content)? {
                    Tag::Time(time) => {
                        let location = TimedLocation {
                            time,
                            source_line_index,
                            line_char_from_index: current_text_index_in_line,
                            line_char_to_index: current_text_index_in_line + text_len,
                        };
                        line_elements.push(LrcElement::TimedLocation(location));
                        current_text_index_in_line += text_len;
                    }
                    tag => line_elements.push(LrcElement::Tag(tag)),
                }
            }
            if !texts.is_empty() {
                let text = texts.join("");
                line_elements.push(LrcElement::TextLine(TextLine {
                    source_line_index,
                    text,
                }))
            }
        }
        Some(c) => {
            let mut buf = [0; 10];
            return Err(format!(
                "Invalid lrc file format. First character in line: \"{}\" (hex bytes: {:x?})",
                c,
                c.encode_utf8(&mut buf).as_bytes()
            ));
        }
    }
    Ok(line_elements)
}

pub fn parse_lrc_file<P: AsRef<Path>>(filepath: P) -> Result<LrcFile, String> {
    let text_lines = lines_from_file(filepath)?;
    let mut timed_texts_lines = Vec::new();
    let mut offset_ms = 0i64;
    let mut lrc_lines = Vec::new();
    for (source_line_index, line) in text_lines.iter().enumerate() {
        let line_elements = parse_lrc_line(line, source_line_index)?;
        for line_element in line_elements {
            match line_element {
                LrcElement::TextLine(t) => lrc_lines.push(t),
                LrcElement::TimedLocation(mut l) => {
                    if offset_ms != 0 {
                        let prev_time_ms: i64 = l.time.as_millis().try_into().unwrap();
                        l.time = Duration::from_millis(
                            (prev_time_ms + offset_ms).try_into().map_err(|_| {
                                format!(
                                    "Cannot apply offset {} to value {}",
                                    offset_ms, prev_time_ms
                                )
                            })?,
                        );
                    }
                    timed_texts_lines.push(l);
                }
                LrcElement::Tag(Tag::Offset(v)) => {
                    offset_ms = v;
                    debug!("Applying offset {}", offset_ms);
                }
                _ => {}
            }
        }
    }
    Ok(LrcFile {
        metadata: Vec::new(),
        lines: lrc_lines,
        timed_texts_lines,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LyricsTiming {
    pub time: Duration,              // time in song at which this occurs
    pub duration: Duration,          // how long should this be displayed
    pub line_index: usize,           // index of line
    pub line_char_from_index: usize, // from this character in line
    pub line_char_to_index: usize,   // to this character in line
}

#[derive(Debug)]
pub struct Lyrics {
    pub lines: Vec<String>,
    pub timings: Vec<LyricsTiming>,
}

impl Lyrics {
    pub fn new(lrc_file: LrcFile) -> Self {
        let mut timings = Vec::new();
        let mut src_lines_with_timings = HashSet::new();
        let mut lrc_lines = Vec::new();

        if !lrc_file.timed_texts_lines.is_empty() {
            timings.push(LyricsTiming {
                time: Duration::ZERO,
                duration: Duration::ZERO,
                line_index: 0,
                line_char_from_index: 0,
                line_char_to_index: 0,
            });

            for timing in lrc_file.timed_texts_lines {
                if let Some(prev_timing) = timings.last_mut() {
                    prev_timing.duration = timing.time - prev_timing.time;
                }
                src_lines_with_timings.insert(timing.source_line_index);

                timings.push(LyricsTiming {
                    time: timing.time,
                    duration: Duration::ZERO,
                    line_index: src_lines_with_timings.len() - 1,
                    line_char_from_index: timing.line_char_from_index,
                    line_char_to_index: timing.line_char_to_index,
                });
            }

            for line in lrc_file.lines {
                if src_lines_with_timings.contains(&line.source_line_index) {
                    lrc_lines.push(line.text);
                }
            }
        }

        Lyrics {
            lines: lrc_lines,
            timings,
        }
    }
}
