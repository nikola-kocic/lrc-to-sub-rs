use std::time::Duration;

use crate::formatters::format_duration;
use crate::lrc::Lyrics;

pub fn f(lrc: &Lyrics) {
    println!(r#"[Script Info]
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
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"#);

    let mut current_line = lrc.timings[1].line_index;
    let mut line_start = lrc.timings[1].time;
    let mut line_end = lrc.timings[1].time;
    let mut karaoke_text = Vec::new();

    for timing_pairs in lrc.timings.windows(2) {
        let timing = &timing_pairs[0];
        let timing_next = &timing_pairs[1];

        if timing.line_index > current_line {
            let line_display_start = line_start
                .checked_sub(Duration::from_secs_f32(2.0))
                .unwrap_or(Duration::ZERO);
            // if line_display_start.
            let s = format!(
                "Dialogue: 1,0:{},0:{},Jap,,0,0,0,,{{\\k200}}{}",
                format_duration(&line_display_start),
                format_duration(&line_end),
                karaoke_text.join("")//lrc.lines[current_line as usize]
            );
            println!("{}", s);
            // Now set up next line
            karaoke_text.clear();
            line_start = timing.time;
            current_line = timing.line_index;
            // println!("{}", &lrc.lines[current_line]);
        }
        line_end = timing.time;
        let duration = timing_next.time - line_end;
        let duration_centisec = duration.as_millis() / 10;
        let line = &lrc.lines[current_line];
        // println!("{:?}", timing);
        if timing.line_char_from_index != timing.line_char_to_index {
            let karaoke_segment = format!("{{\\k{}}}{}", duration_centisec, line.get(timing.line_char_from_index..timing.line_char_to_index).unwrap());
            // println!("{}", karaoke_segment);
            karaoke_text.push(karaoke_segment);
        }
    }
}
