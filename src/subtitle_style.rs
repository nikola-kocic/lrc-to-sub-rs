#[derive(Debug)]
pub struct SubtitleStyle {
    pub primary_color: String,             // in [AA]RRGGBB format
    pub secondary_color: String,           // in [AA]RRGGBB format
    pub long_text_secondary_color: String, // in [AA]RRGGBB format
}

pub struct AssSubtitleStyle {
    pub primary_color: String,             // in AABBGGRR format
    pub secondary_color: String,           // in AABBGGRR format
    pub long_text_secondary_color: String, // in AARRGGBB format
}

// Convert color from [AA]RRGGBB to AABBGGRR
fn convert_color_to_ass_standard(color: &str) -> String {
    let (aa, index_after_aa) = if color.len() == 6 {
        ("00", 0)
    } else {
        (color.get(0..2).unwrap(), 2)
    };
    format!(
        "{}{}{}{}",
        aa,
        color.get(index_after_aa + 4..).unwrap(),
        color.get(index_after_aa + 2..index_after_aa + 4).unwrap(),
        color.get(index_after_aa..index_after_aa + 2).unwrap()
    )
    .to_ascii_uppercase()
}

impl AssSubtitleStyle {
    pub fn new(style: &SubtitleStyle) -> Self {
        Self {
            primary_color: convert_color_to_ass_standard(&style.primary_color),
            secondary_color: convert_color_to_ass_standard(&style.secondary_color),
            long_text_secondary_color: convert_color_to_ass_standard(
                &style.long_text_secondary_color,
            ),
        }
    }
}
