use std::time::Duration;

use crate::formatters::format_duration;
use crate::lrc::Lyrics;

pub fn f(lrc: &Lyrics) {
    let mut current_line = lrc.timings[1].line_index;
    let mut line_start = lrc.timings[1].time;
    let mut line_end = lrc.timings[1].time;

    for timing in lrc.timings.iter().skip(2) {
        if timing.line_index > current_line {
            let mut line_display_start = line_start
                .checked_sub(Duration::from_secs_f32(2.0))
                .unwrap_or(Duration::ZERO);
            // if line_display_start.
            let s = format!(
                "Dialogue: 1,0:{},0:{},Jap,,0,0,0,,{{\\k200}}{}",
                format_duration(&line_display_start),
                format_duration(&line_end),
                lrc.lines[current_line as usize]
            );
            println!("{}", s);
            // Now set up next line
            line_start = timing.time;
            current_line = timing.line_index;
        }
        line_end = timing.time;
    }
}
