use std::fs;
use std::path::Path;
use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::formatters::format_duration;
use crate::lrc::Lyrics;
use crate::lrc::LyricsTiming;

const DEFAULT_PREDISPLAY_DURATION: Duration = Duration::from_secs(2);

const ASS_CONTENT_PREFIX: &str = r#"[Script Info]
; This is a Sub Station Alpha v4 script.
Title: 
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
Collisions: Normal

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Jap,Arial,20,&H00FFFFFF,&H000088EF,&H00000000,&H00666666,-1,0,0,0,100,100,0,0,1,3,0,8,10,10,10,1
Style: Eng,Arial,20,&H00FFFFFF,&H000088EF,&H00000000,&H00666666,-1,0,0,0,100,100,0,0,1,3,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"#;

#[derive(Debug)]
struct KaraokeLineSegment {
    duration: Duration,
    text: String,
}

#[derive(Debug)]
struct KaraokeLine {
    line_display_start: Duration,
    line_display_end: Duration,
    line_segments: Vec<KaraokeLineSegment>,
}

fn format_text_duration(duration: &Duration, text: &str) -> String {
    let duration_centisec = duration.as_millis() / 10;
    format!("{{\\k{}}}{}", duration_centisec, text)
}

fn generate_ass_file(karaoke_lines: Vec<KaraokeLine>) -> Vec<String> {
    let mut ass_lines = ASS_CONTENT_PREFIX
        .split('\n')
        .map(String::from)
        .collect::<Vec<String>>();
    for karaoke_line in karaoke_lines {
        let mut line_segments_strings = Vec::new();
        let predisplay_duration = std::cmp::min(
            &karaoke_line.line_display_start,
            &DEFAULT_PREDISPLAY_DURATION,
        );
        line_segments_strings.push(format_text_duration(predisplay_duration, ""));

        for line_segment in karaoke_line.line_segments {
            let line_segments_string =
                format_text_duration(&line_segment.duration, &line_segment.text);
            line_segments_strings.push(line_segments_string);
        }
        ass_lines.push(format!(
            "Dialogue: 1,0:{},0:{},Jap,,0,0,0,,{}",
            format_duration(&karaoke_line.line_display_start),
            format_duration(&karaoke_line.line_display_end),
            line_segments_strings.join("")
        ));
    }
    ass_lines
}

fn generate_karaoke_line(timings_for_line: &[LyricsTiming], line: &str) -> KaraokeLine {
    let mut karaoke_line_segments = Vec::new();
    let line_start = timings_for_line.first().unwrap().time;
    let line_end = timings_for_line.last().unwrap().time;

    for timing in timings_for_line {
        trace!("timing = {:?}", timing);
        let karaoke_segment = KaraokeLineSegment {
            duration: timing.duration,
            text: line
                .get(timing.line_char_from_index..timing.line_char_to_index)
                .unwrap()
                .to_owned(),
        };
        trace!("karaoke_segment = {:?}", karaoke_segment);
        karaoke_line_segments.push(karaoke_segment);
    }
    let line_display_start = line_start
        .checked_sub(Duration::from_secs_f32(2.0))
        .unwrap_or(Duration::ZERO);
    KaraokeLine {
        line_display_start,
        line_display_end: line_end,
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

pub fn f(lrc: &Lyrics, out: &Path) -> Result<(), String> {
    let ass_lines = generate_ass_file(generate_karaoke_lines(lrc));
    fs::write(out, ass_lines.join("\n"))
        .map_err(|e| format!("Cannot write to {:?}: {}", out, e))?;
    Ok(())
}
