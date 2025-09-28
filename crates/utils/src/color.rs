use gpui::Hsla;

/// Parse a hex color into an Hsla
pub fn parse_hex_color(value: &str) -> Option<Hsla> {
    if let Ok(rgba) = gpui::Rgba::try_from(value) {
        return Some(rgba.into());
    }

    let value = value.trim();

    let hex = if value.starts_with('#') {
        &value[1..]
    } else if !value.starts_with("rgb") && !value.starts_with("hsl") {
        value
    } else {
        return None;
    };

    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()? as f32 / 255.0;

        let rgba = gpui::Rgba { r, g, b, a: 1.0 };
        return Some(rgba.into());
    } else if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

        let rgba = gpui::Rgba { r, g, b, a: 1.0 };
        return Some(rgba.into());
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;

        let rgba = gpui::Rgba { r, g, b, a };
        return Some(rgba.into());
    }

    None
}
