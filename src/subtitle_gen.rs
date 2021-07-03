use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::formatters::format_duration;
use crate::lrc::Lyrics;

struct KaraokeLineSegment {
    duration: Duration,
    text: String,
}

struct KaraokeLine {
    line_display_start: Duration,
    line_display_end: Duration,
    line_segments: Vec<KaraokeLineSegment>,
}

fn generate_ass_file(karaoke_lines: Vec<KaraokeLine>) -> Vec<String> {
    let prefix = r#"[Script Info]
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
    let mut ass_lines = prefix
        .split('\n')
        .map(String::from)
        .collect::<Vec<String>>();
    for karaoke_line in karaoke_lines {
        let mut line_segments_strings = Vec::new();
        for line_segment in karaoke_line.line_segments {
            let duration_centisec = line_segment.duration.as_millis() / 10;
            let line_segments_string = format!(
                "{{\\k{}}}{}",
                duration_centisec,
                line_segment.text
            );
            line_segments_strings.push(line_segments_string);
        }
        ass_lines.push(format!(
            "Dialogue: 1,0:{},0:{},Jap,,0,0,0,,{{\\k200}}{}",
            format_duration(&karaoke_line.line_display_start),
            format_duration(&karaoke_line.line_display_end),
            line_segments_strings.join("")
        ));
    }
    ass_lines
}

fn generate_karaoke_lines(lrc: &Lyrics) -> Vec<KaraokeLine> {
    let mut current_line = lrc.timings[1].line_index;
    let mut line_start = lrc.timings[1].time;
    let mut line_end = lrc.timings[1].time;
    let mut karaoke_line_segments = Vec::new();
    let mut karaoke_lines = Vec::new();

    for timing_pairs in lrc.timings.windows(2) {
        let timing = &timing_pairs[0];
        let timing_next = &timing_pairs[1];

        if timing.line_index > current_line {
            let line_display_start = line_start
                .checked_sub(Duration::from_secs_f32(2.0))
                .unwrap_or(Duration::ZERO);
            karaoke_lines.push(KaraokeLine {
                line_display_start,
                line_display_end: line_end,
                line_segments: karaoke_line_segments.split_off(0),
            });
            // Now set up next line
            karaoke_line_segments.clear();
            line_start = timing.time;
            current_line = timing.line_index;
            // println!("{}", &lrc.lines[current_line]);
        }
        line_end = timing.time;
        let duration = timing_next.time - line_end;
        let line = &lrc.lines[current_line];
        // println!("{:?}", timing);
        if timing.line_char_from_index != timing.line_char_to_index {
            let karaoke_segment = KaraokeLineSegment {
                duration,
                text: line.get(timing.line_char_from_index..timing.line_char_to_index)
                    .unwrap().to_owned()
            };
            // println!("{}", karaoke_segment);
            karaoke_line_segments.push(karaoke_segment);
        }
    }
    karaoke_lines
}

pub fn f(lrc: &Lyrics, out: &Path) -> Result<(), String> {
    let ass_lines = generate_ass_file(generate_karaoke_lines(lrc));
    fs::write(out, ass_lines.join("\n"))
        .map_err(|e| format!("Cannot write to {:?}: {}", out, e))?;
    Ok(())
}
