use gpui::{hsla, svg, Hsla, Styled};

pub enum IconName {
    Check,
    QuestionMark,
}

impl IconName {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Check => "icons/lucide/check.svg",
            Self::QuestionMark => "icons/lucide/help-circle.svg",
        }
    }
}

// todo!("Icon::new likely returns an Icon in the long run, not an svg")
pub struct Icon {}

impl Icon {
    pub fn new(icon: IconName, color: impl Into<Option<Hsla>>) -> gpui::Svg {
        let color = color.into().unwrap_or(hsla(0.0, 0.0, 0.0, 1.0));
        svg().size_4().text_color(color).path(icon.path())
    }
}
