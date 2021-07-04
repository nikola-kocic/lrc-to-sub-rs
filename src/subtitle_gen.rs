use std::fs;
use std::path::Path;
use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::formatters::format_duration;
use crate::lrc::Lyrics;
use crate::lrc::LyricsTiming;
use crate::subtitle_style::{AssSubtitleStyle, SubtitleStyle};

// TODO: Fix line disappearing too soon when there is no timestamp tag after the last segment in line

const DEFAULT_PREDISPLAY_DURATION: Duration = Duration::from_secs(2);
const LONG_KARAOKE_SEGMENT: Duration = Duration::from_millis(700);

// For colors, hexadecimal byte order of this value is AABBGGRR.
// Karaoke effects go from SecondaryColour to PrimaryColour.

const ASS_SCRIPT_INFO_HEADER: &str = "[Script Info]
; This is a Sub Station Alpha v4 script.
Title: 
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
Collisions: Normal";

const ASS_EVENTS_HEADER: &str = "[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";

#[derive(Debug)]
struct KaraokeLineSegment {
    duration: Duration,
    text: String,
}

#[derive(Debug)]
struct KaraokeLine {
    line_start: Duration,
    line_end: Duration,
    line_segments: Vec<KaraokeLineSegment>,
}

fn get_ass_styles_header(style: &AssSubtitleStyle) -> String {
    format!("[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style:   Jap,    Arial,       20,  &H{default_primary_color},    &H{default_secondary_color},    &H00000000, &H00666666,   -1,      0,         0,         0,    100,    100,       0,     0,           1,       3,      0,         8,      10,      10,      10,        1
Style:   Eng,    Arial,       20,  &H{default_primary_color},    &H{default_secondary_color},    &H00000000, &H00666666,   -1,      0,         0,         0,    100,    100,       0,     0,           1,       3,      0,         2,      10,      10,      10,        1
", default_primary_color=style.primary_color, default_secondary_color=style.secondary_color)
}

fn format_text_duration(duration: &Duration, text: &str, style: &AssSubtitleStyle) -> String {
    let duration_centisec = duration.as_millis() / 10;
    if *duration < LONG_KARAOKE_SEGMENT {
        format!(
            "{{\\k{duration}}}{text}",
            duration = duration_centisec,
            text = text
        )
    } else {
        format!(
            "{{\\2c{override_secondary_color}\\K{duration}}}{text}{{\\2c}}",
            duration = duration_centisec,
            text = text,
            override_secondary_color = style.long_text_secondary_color,
        )
    }
}

fn generate_ass_file(karaoke_lines: Vec<KaraokeLine>, style: &SubtitleStyle) -> Vec<String> {
    let style = AssSubtitleStyle::new(style);
    let mut ass_lines = vec![
        ASS_SCRIPT_INFO_HEADER.to_owned(),
        get_ass_styles_header(&style),
        ASS_EVENTS_HEADER.to_owned(),
    ];
    for (i, karaoke_line) in karaoke_lines.iter().enumerate() {
        trace!("writing ASS file from = {:?}", karaoke_line);
        let mut line_segments_strings = Vec::new();
        let predisplay_duration = {
            let first_min = if i < 2
                || karaoke_line.line_start - DEFAULT_PREDISPLAY_DURATION
                    > karaoke_lines[i - 2].line_end
            {
                karaoke_line.line_start
            } else {
                karaoke_line.line_start - karaoke_lines[i - 2].line_end
            };
            std::cmp::min(first_min, DEFAULT_PREDISPLAY_DURATION)
        };
        line_segments_strings.push(format_text_duration(&predisplay_duration, "— ", &style));

        for line_segment in &karaoke_line.line_segments {
            let line_segments_string =
                format_text_duration(&line_segment.duration, &line_segment.text, &style);
            line_segments_strings.push(line_segments_string);
        }
        ass_lines.push(format!(
            "Dialogue: 1,0:{},0:{},Jap,,0,0,0,,{}",
            format_duration(&(karaoke_line.line_start - predisplay_duration)),
            format_duration(&karaoke_line.line_end),
            line_segments_strings.join("")
        ));
    }
    ass_lines
}

fn generate_karaoke_line(timings_for_line: &[LyricsTiming], line: &str) -> KaraokeLine {
    let mut karaoke_line_segments = Vec::new();
    let line_start = timings_for_line.first().unwrap().time;
    let line_end = timings_for_line.last().unwrap().time;

    for (i, timing) in timings_for_line.iter().enumerate() {
        trace!("timing = {:?}", timing);
        let karaoke_segment = KaraokeLineSegment {
            duration: timing.duration,
            text: line
                .get(timing.line_char_from_index..timing.line_char_to_index)
                .unwrap()
                .to_owned(),
        };

        // Don't output unneeded karaoke tag at the end of the line
        if i < timings_for_line.len() - 1 || !karaoke_segment.text.is_empty() {
            trace!("karaoke_segment = {:?}", karaoke_segment);
            karaoke_line_segments.push(karaoke_segment);
        }
    }
    KaraokeLine {
        line_start,
        line_end,
        line_segments: karaoke_line_segments,
    }
}

fn generate_karaoke_lines(lrc: &Lyrics) -> Vec<KaraokeLine> {
    let mut karaoke_lines = Vec::new();
    let mut current_line_index = 0;
    let mut timings_index_start = 0;

    while current_line_index < lrc.lines.len() {
        let slice = &lrc.timings[timings_index_start..];
        let pp = slice.partition_point(|t| t.line_index == current_line_index);
        karaoke_lines.push(generate_karaoke_line(
            &slice[..pp],
            &lrc.lines[current_line_index],
        ));
        timings_index_start += pp;
        current_line_index += 1;
    }

    karaoke_lines
}

pub fn f(lrc: &Lyrics, out: &Path, style: &SubtitleStyle) -> Result<(), String> {
    let ass_lines = generate_ass_file(generate_karaoke_lines(lrc), style);
    fs::write(out, ass_lines.join("\n"))
        .map_err(|e| format!("Cannot write to {:?}: {}", out, e))?;
    Ok(())
}
