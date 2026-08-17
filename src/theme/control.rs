//! The shared size scale for controls that can sit on one row.
//!
//! Before this existed, every control picked its own height — `Button` was
//! 16px, `Toggle` 20px, `Switch` and `IconButton` 24px — and several never
//! declared one at all, letting padding plus a line box decide. A row of them
//! could not line up, and nothing in the crate would have noticed.
//!
//! This module is the single place a control dimension is defined. A
//! [`ControlSize`] names a rung; the theme resolves it to [`ControlMetrics`],
//! which carries every dimension a control needs to sit on that rung. Change
//! [`ControlScale::default`] and every control moves together — that is the
//! point of putting it here rather than in each element.
//!
//! # What belongs here
//!
//! A dimension shared across controls. A shape specific to one control stays
//! in that control's file, derived from its rung. [`ControlMetrics::track`] is
//! the borderline case that lives here anyway: `Switch` and `Toggle` draw the
//! same sliding track, and left in their own files the two had already drifted
//! to different shapes with nothing holding them together.

use gpui::Rems;

/// The 1px border a bordered control draws, in rems at the 16px root this
/// scale's values are stated at.
///
/// Only used where a rem-valued dimension has to account for a pixel-valued
/// border. gpui lays out border-box, so a border eats into a declared height
/// rather than adding to it.
const BORDER_REMS: f32 = 1.0 / 16.0;

/// A rung on the shared control size scale.
///
/// Heights are 16 / 20 / 24px at a 16px root. `Medium` is the default: it is
/// the rung most of the crate's controls were already closest to.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlSize {
    /// 16px tall. Dense toolbars, inline chrome.
    Small,
    /// 20px tall. The default.
    #[default]
    Medium,
    /// 24px tall. Primary actions, touch-friendlier rows.
    Large,
}

impl ControlSize {
    /// Every rung, smallest first.
    pub const ALL: [ControlSize; 3] = [ControlSize::Small, ControlSize::Medium, ControlSize::Large];

    /// The rung's name, for showcases and debug output.
    pub fn name(&self) -> &'static str {
        match self {
            ControlSize::Small => "Small",
            ControlSize::Medium => "Medium",
            ControlSize::Large => "Large",
        }
    }
}

/// Every dimension a control needs to sit on one rung.
///
/// All lengths are rems, so a consumer that changes the root font size
/// rescales the whole set. Resolve one with
/// [`Themeable::control`](crate::theme::Themeable::control).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlMetrics {
    /// The control's outer height. This is the number the whole scale exists
    /// to make equal across a row.
    pub height: Rems,
    /// Horizontal padding inside the control.
    pub padding_x: Rems,
    /// Spacing between an icon and a label *inside* one control. A trailing
    /// label that belongs to the control but sits outside its box — a
    /// checkbox's or a switch's — wants more room than this; those use
    /// `gap * 2.0`.
    pub gap: Rems,
    /// Corner radius.
    pub radius: Rems,
    /// Font size for text inside the control.
    pub text_size: Rems,
    /// The line box that text sits in. Declared rather than inherited: set
    /// only a height and a large enough font pushes text out of the box it was
    /// supposed to be centred in — the same "emergent size" failure one level
    /// down. Every rung satisfies `line_height + 2px border <= height`.
    pub line_height: Rems,
    /// How much of its box a control's graphic fills — a checkbox's box, an
    /// icon button's glyph, a switch's track height.
    ///
    /// Equal heights are necessary and not sufficient: a control can sit on
    /// its rung and still read heavy next to its neighbours. `ink` is the knob
    /// for that, and it is a judgement rather than a derivation — an element
    /// with a genuine reason may use its own ratio.
    pub ink: Rems,
}

impl ControlMetrics {
    /// Vertical padding that centres the line box in the rung, for a bordered
    /// control that does *not* declare its height.
    ///
    /// A control that declares its height does not need this — `items_center`
    /// does the same job and survives a font that overflows.
    pub fn padding_y(&self) -> Rems {
        Rems(((self.height.0 - self.line_height.0) / 2.0 - BORDER_REMS).max(0.0))
    }

    /// The line height for text that is allowed to wrap — a textarea, a
    /// multi-line input. The rung's own `line_height` is tuned to fit *inside*
    /// the rung, which is too tight once there is a second line.
    pub fn multiline_line_height(&self) -> Rems {
        self.text_size * 1.5
    }

    /// The sliding track shape `Switch` and `Toggle` both draw.
    ///
    /// Derived from `ink` so the two cannot drift apart again: track height is
    /// the ink, the track is a little under twice as wide as it is tall, and
    /// the thumb is what actually fits inside the track's 1px border plus a
    /// 1px inset. Deriving the thumb rather than naming it is the fix for the
    /// old shape, whose 16px thumb overflowed its 20px track by 2px —
    /// absolute insets are relative to the padding box, so the border was not
    /// subtracted anywhere.
    pub fn track(&self) -> TrackMetrics {
        let margin = Rems(BORDER_REMS);
        let inset = (margin + Rems(BORDER_REMS)) * 2.0;

        TrackMetrics {
            width: self.ink * 2.0 - Rems(0.25),
            height: self.ink,
            thumb: self.ink - inset,
            thumb_margin: margin,
        }
    }
}

/// The shape of the sliding track drawn by `Switch` and `Toggle`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackMetrics {
    /// Overall track width.
    pub width: Rems,
    /// Overall track height. Equal to the rung's `ink`.
    pub height: Rems,
    /// Diameter of the thumb. Sized to fit exactly inside the track's border
    /// and inset.
    pub thumb: Rems,
    /// Inset of the thumb from the track's padding box on every side.
    pub thumb_margin: Rems,
}

/// The three rungs, as a theme-owned value.
///
/// A theme overrides
/// [`Themeable::control_scale`](crate::theme::Themeable::control_scale) to
/// rescale every control at once.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlScale {
    /// The `Small` rung.
    pub small: ControlMetrics,
    /// The `Medium` rung.
    pub medium: ControlMetrics,
    /// The `Large` rung.
    pub large: ControlMetrics,
}

impl ControlScale {
    /// The metrics for one rung.
    pub fn metrics(&self, size: ControlSize) -> ControlMetrics {
        match size {
            ControlSize::Small => self.small,
            ControlSize::Medium => self.medium,
            ControlSize::Large => self.large,
        }
    }
}

impl Default for ControlScale {
    /// The crate's default scale. Values are given in rems with their pixel
    /// equivalent at a 16px root in the comment.
    ///
    /// | rung   | height | pad_x | gap    | radius | text | line | ink |
    /// |--------|--------|-------|--------|--------|------|------|-----|
    /// | Small  | 16     | 6     | 3      | 3      | 11   | 14   | 12  |
    /// | Medium | 20     | 8     | 4      | 4      | 12   | 16   | 14  |
    /// | Large  | 24     | 10    | 6      | 5      | 13   | 18   | 16  |
    fn default() -> Self {
        Self {
            small: ControlMetrics {
                height: Rems(1.0),        // 16
                padding_x: Rems(0.375),   // 6
                gap: Rems(0.1875),        // 3
                radius: Rems(0.1875),     // 3
                text_size: Rems(0.6875),  // 11
                line_height: Rems(0.875), // 14
                ink: Rems(0.75),          // 12
            },
            medium: ControlMetrics {
                height: Rems(1.25),     // 20
                padding_x: Rems(0.5),   // 8
                gap: Rems(0.25),        // 4
                radius: Rems(0.25),     // 4
                text_size: Rems(0.75),  // 12
                line_height: Rems(1.0), // 16
                ink: Rems(0.875),       // 14
            },
            large: ControlMetrics {
                height: Rems(1.5),        // 24
                padding_x: Rems(0.625),   // 10
                gap: Rems(0.375),         // 6
                radius: Rems(0.3125),     // 5
                text_size: Rems(0.8125),  // 13
                line_height: Rems(1.125), // 18
                ink: Rems(1.0),           // 16
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Rems` has no `Eq` and a hand-written `Debug`, so tests compare `.0`.
    fn px(value: Rems) -> f32 {
        value.0 * 16.0
    }

    #[test]
    fn the_rungs_are_the_documented_pixel_sizes() {
        let scale = ControlScale::default();
        assert_eq!(px(scale.small.height), 16.0);
        assert_eq!(px(scale.medium.height), 20.0);
        assert_eq!(px(scale.large.height), 24.0);
    }

    #[test]
    fn every_dimension_grows_with_the_rung() {
        let scale = ControlScale::default();
        for (smaller, larger) in [(scale.small, scale.medium), (scale.medium, scale.large)] {
            assert!(smaller.height.0 < larger.height.0, "height");
            assert!(smaller.padding_x.0 < larger.padding_x.0, "padding_x");
            assert!(smaller.gap.0 < larger.gap.0, "gap");
            assert!(smaller.radius.0 < larger.radius.0, "radius");
            assert!(smaller.text_size.0 < larger.text_size.0, "text_size");
            assert!(smaller.line_height.0 < larger.line_height.0, "line_height");
            assert!(smaller.ink.0 < larger.ink.0, "ink");
        }
    }

    /// The invariant that makes a declared height safe: the line box plus the
    /// control's 1px border on each side still fits inside the rung. Break it
    /// and text overflows the box it was meant to be centred in.
    #[test]
    fn the_line_box_fits_inside_the_rung() {
        let scale = ControlScale::default();
        for size in ControlSize::ALL {
            let metrics = scale.metrics(size);
            assert!(
                px(metrics.line_height) + 2.0 <= px(metrics.height),
                "{}: a {}px line box plus 2px of border does not fit in {}px",
                size.name(),
                px(metrics.line_height),
                px(metrics.height),
            );
        }
    }

    /// Ink is what a control *fills*, so it has to be less than the box it
    /// fills — a graphic exactly the height of the row reads as an overflow.
    #[test]
    fn ink_is_smaller_than_the_box_it_fills() {
        let scale = ControlScale::default();
        for size in ControlSize::ALL {
            let metrics = scale.metrics(size);
            assert!(
                metrics.ink.0 < metrics.height.0,
                "{}: ink {}px is not smaller than the {}px box",
                size.name(),
                px(metrics.ink),
                px(metrics.height),
            );
        }
    }

    /// The thumb has to fit inside the track's border and inset, which is the
    /// bug the derivation replaced: the old switch's 16px thumb overflowed its
    /// 20px track by 2px.
    #[test]
    fn the_thumb_fits_inside_its_track() {
        let scale = ControlScale::default();
        for size in ControlSize::ALL {
            let track = scale.metrics(size).track();
            let occupied = px(track.thumb) + 2.0 * (px(track.thumb_margin) + 1.0);
            assert!(
                occupied <= px(track.height),
                "{}: a {}px thumb inset {}px inside a 1px border does not fit a \
                 {}px track",
                size.name(),
                px(track.thumb),
                px(track.thumb_margin),
                px(track.height),
            );
            assert!(
                track.width.0 > track.height.0,
                "{}: a track that is not wider than it is tall has nowhere to \
                 slide the thumb",
                size.name(),
            );
        }
    }

    #[test]
    fn multiline_text_gets_a_looser_line_box() {
        let scale = ControlScale::default();
        for size in ControlSize::ALL {
            let metrics = scale.metrics(size);
            assert!(
                metrics.multiline_line_height().0 > metrics.text_size.0,
                "{}: wrapped lines would collide",
                size.name(),
            );
        }
    }

    #[test]
    fn medium_is_the_default_rung() {
        assert_eq!(ControlSize::default(), ControlSize::Medium);
        assert_eq!(ControlSize::ALL.len(), 3);
    }
}
