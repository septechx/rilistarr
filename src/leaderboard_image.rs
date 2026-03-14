use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use image::Rgba;
use imageproc::drawing::draw_text_mut;
use std::io::Cursor;

use crate::leaderboard::LeaderboardEntry;

const TEMPLATE_PATH: &str = "assets/template.png";
const FONT_PATH: &str = "assets/Inter-Bold.ttf";
const NAME_X: f32 = 240.0;
const TROPHY_X: f32 = 1750.0;
const FIRST_ENTRY_Y: f32 = 108.0;
const ENTRY_HEIGHT: f32 = 74.0;
const ENTRY_SPACING: f32 = 39.0;
const MAX_ENTRIES: usize = 8;
const NAME_FONT_SIZE: f32 = 50.0;
const TROPHY_FONT_SIZE: f32 = 50.0;
const TITLE_FONT_SIZE: f32 = 72.0;
const TIMESTAMP_FONT_SIZE: f32 = 28.0;
const ENTRY_VERTICAL_OFFSET: f32 = 12.0;

fn load_font() -> Result<FontVec, Box<dyn std::error::Error>> {
    let font_data = std::fs::read(FONT_PATH)?;
    let font_vec = FontVec::try_from_vec(font_data)?;
    Ok(font_vec)
}

fn rgb_to_rgba(color: (u8, u8, u8)) -> Rgba<u8> {
    Rgba([color.0, color.1, color.2, 255])
}

fn measure_text_width(font: &FontVec, text: &str, scale: PxScale) -> f32 {
    let scaled_font = font.as_scaled(scale);
    let mut width = 0.0f32;
    for c in text.chars() {
        let glyph_id = scaled_font.glyph_id(c);
        width += scaled_font.h_advance(glyph_id);
    }
    width
}

pub fn render_leaderboard_image(
    entries: &[LeaderboardEntry],
    title: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let font = load_font()?;

    let mut img = image::open(TEMPLATE_PATH)?.to_rgba8();

    let text_color = rgb_to_rgba((24, 24, 24));
    let header_color = rgb_to_rgba((237, 237, 237));

    let title_scale = PxScale::from(TITLE_FONT_SIZE);
    let title_width = measure_text_width(&font, title, title_scale);
    let title_x = (1920.0 - title_width) / 2.0;
    draw_text_mut(
        &mut img,
        header_color,
        title_x as i32,
        30,
        title_scale,
        &font,
        title,
    );

    for (i, entry) in entries.iter().take(MAX_ENTRIES).enumerate() {
        let y = FIRST_ENTRY_Y + (i as f32 * (ENTRY_HEIGHT + ENTRY_SPACING)) + ENTRY_VERTICAL_OFFSET;

        let name_text = if let Some(count) = entry.member_count {
            format!("{} ({} members)", entry.name, count)
        } else {
            entry.name.clone()
        };

        let name_scale = PxScale::from(NAME_FONT_SIZE);
        draw_text_mut(
            &mut img,
            text_color,
            NAME_X as i32,
            y as i32,
            name_scale,
            &font,
            &name_text,
        );

        let trophy_text = entry.trophies.to_string();
        let trophy_scale = PxScale::from(TROPHY_FONT_SIZE);
        let trophy_width = measure_text_width(&font, &trophy_text, trophy_scale);
        let trophy_x = TROPHY_X as i32 - trophy_width as i32;
        draw_text_mut(
            &mut img,
            text_color,
            trophy_x,
            y as i32,
            trophy_scale,
            &font,
            &trophy_text,
        );
    }

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let timestamp_scale = PxScale::from(TIMESTAMP_FONT_SIZE);
    let timestamp_width = measure_text_width(&font, &timestamp, timestamp_scale);
    let timestamp_x = (1920.0 - timestamp_width) / 2.0;
    draw_text_mut(
        &mut img,
        header_color,
        timestamp_x as i32,
        1020,
        timestamp_scale,
        &font,
        &timestamp,
    );

    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(bytes)
}
