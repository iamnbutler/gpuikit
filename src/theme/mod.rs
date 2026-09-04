//! A trait-based theme system for gpuikit
//!
//! The `Themeable` trait defines the color contract that gpuikit components use,
//! and — since a control's height is as much a design decision as its colour —
//! the shared size scale in [`control`].
//!
//! Consumers can implement this trait for their own theme types and resolve
//! them with [`Theme::from_themeable`], and can add tokens of their own that
//! every theme answers for with [`ThemeExtension`].

pub mod control;

pub use control::{ControlMetrics, ControlScale, ControlSize, TrackMetrics};

use gpui::{App, BoxShadow, Global, Hsla, Pixels, SharedString, hsla, px};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// How thick the ring around a keyboard-focused control is.
///
/// One number, in one place, so every control that grows a ring grows the same
/// one.
pub const FOCUS_RING_WIDTH: Pixels = px(2.);

/// The ring a control draws when the keyboard reaches it.
///
/// A **spread shadow rather than a border**, on purpose: a border changes the
/// control's box, so arriving focus would resize it and reflow its neighbours.
/// A spread shadow is painted outside the bounds and moves nothing.
///
/// Apply it through gpui's `focus_visible`, not `focus` — that is the
/// `:focus-visible` rule, so clicking a control does not leave a ring behind
/// it. `crate::a11y` is what makes the control focusable in the first place;
/// this is only what it looks like.
pub fn focus_ring(color: Hsla) -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(0.), color).spread_radius(FOCUS_RING_WIDTH)]
}

/// Core theme trait that defines the color contract for UI components.
///
/// Implement this trait for your own theme type to customize colors.
/// Only a small set of "primitive" colors are required - everything else
/// has sensible defaults derived from them.
pub trait Themeable {
    // === Required methods (the primitives) ===

    /// Primary foreground/text color
    fn fg(&self) -> Hsla;

    /// Primary background color
    fn bg(&self) -> Hsla;

    /// Surface color for cards, panels, elevated elements
    fn surface(&self) -> Hsla;

    /// Border color for dividers and boundaries
    fn border(&self) -> Hsla;

    /// Accent color for primary actions and focus states
    fn accent(&self) -> Hsla;

    // === Optional methods with defaults ===

    /// Muted foreground for secondary text
    fn fg_muted(&self) -> Hsla {
        self.fg().opacity(0.7)
    }

    /// Disabled foreground color
    fn fg_disabled(&self) -> Hsla {
        self.fg().opacity(0.4)
    }

    /// Secondary surface for nested panels
    fn surface_secondary(&self) -> Hsla {
        self.surface()
    }

    /// Tertiary surface for deeply nested elements
    fn surface_tertiary(&self) -> Hsla {
        self.surface_secondary()
    }

    /// Secondary border for hover states
    fn border_secondary(&self) -> Hsla {
        self.border()
    }

    /// Subtle border for minimal separation
    fn border_subtle(&self) -> Hsla {
        self.border().opacity(0.5)
    }

    /// Focus outline color
    fn outline(&self) -> Hsla {
        self.accent()
    }

    /// Accent background (for tags, badges)
    fn accent_bg(&self) -> Hsla {
        self.accent().opacity(0.15)
    }

    /// Accent background hover state
    fn accent_bg_hover(&self) -> Hsla {
        self.accent().opacity(0.25)
    }

    /// Info color (blue)
    fn info(&self) -> Hsla {
        hsla(210.0 / 360.0, 0.7, 0.5, 1.0)
    }

    /// Success color (green)
    fn success(&self) -> Hsla {
        hsla(142.0 / 360.0, 0.7, 0.4, 1.0)
    }

    /// Warning color (yellow/orange)
    fn warning(&self) -> Hsla {
        hsla(38.0 / 360.0, 0.9, 0.5, 1.0)
    }

    /// Danger/error color
    fn danger(&self) -> Hsla {
        hsla(0.0, 0.7, 0.5, 1.0)
    }

    /// Selection highlight color
    fn selection(&self) -> Hsla {
        self.accent().opacity(0.3)
    }

    /// Placeholder text color
    fn placeholder(&self) -> Hsla {
        self.fg().opacity(0.5)
    }

    /// Overlay/scrim color for modal backdrops.
    ///
    /// Used by Dialog and other modal components to dim the background.
    /// Defaults to a semi-transparent black, which works for both light and dark themes.
    fn overlay(&self) -> Hsla {
        hsla(0.0, 0.0, 0.0, 0.6)
    }

    // === Component-specific defaults ===

    fn button_bg(&self) -> Hsla {
        self.surface()
    }

    fn button_bg_hover(&self) -> Hsla {
        self.surface_secondary()
    }

    fn button_bg_active(&self) -> Hsla {
        self.surface_tertiary()
    }

    fn button_border(&self) -> Hsla {
        self.border()
    }

    /// Fill of a destructive button — one that deletes, discards or revokes.
    ///
    /// Derived from [`danger`](Themeable::danger) rather than introduced as a
    /// new palette entry: a theme that has already said what its error colour
    /// is has said what a destructive action looks like, and two entries that
    /// could disagree is a way for them to.
    fn destructive_bg(&self) -> Hsla {
        self.danger()
    }

    /// Hover fill of a destructive button: the fill, lightened.
    fn destructive_bg_hover(&self) -> Hsla {
        let base = self.destructive_bg();
        hsla(base.h, base.s, (base.l + 0.08).min(1.0), base.a)
    }

    /// Pressed fill of a destructive button: the fill, darkened.
    fn destructive_bg_active(&self) -> Hsla {
        let base = self.destructive_bg();
        hsla(base.h, base.s, (base.l - 0.08).max(0.0), base.a)
    }

    /// Text on a destructive button: black or white, whichever the fill
    /// actually reads better against.
    ///
    /// Not [`fg`](Themeable::fg) — a light theme's foreground is dark, and
    /// dark text on a saturated red is the one combination this has to avoid
    /// — and not the fill's HSL lightness either, which is what this used to
    /// pick by. HSL lightness is not perceived brightness: a saturated red
    /// sits high on the L axis and low on the luminance one, so the rule
    /// `l > 0.6 → black` chose *white* for Gruvbox Dark's `#fb4934`, which is
    /// 3.4:1 — under WCAG AA — where black on the same fill is 6.1:1. It was
    /// the only theme in the crate the old rule got wrong, and it was the one
    /// whose destructive button looked washed out.
    ///
    /// Comparing contrast ratios picks the same answer as before for every
    /// other built-in theme, and it keeps picking the readable one for a
    /// consumer's `danger` colour whatever hue it is.
    fn destructive_fg(&self) -> Hsla {
        let black = hsla(0.0, 0.0, 0.0, 1.0);
        let white = hsla(0.0, 0.0, 1.0, 1.0);
        let bg = self.destructive_bg();

        if contrast_ratio(bg, white) >= contrast_ratio(bg, black) {
            white
        } else {
            black
        }
    }

    fn input_bg(&self) -> Hsla {
        self.surface()
    }

    fn input_border(&self) -> Hsla {
        self.border()
    }

    fn input_border_hover(&self) -> Hsla {
        self.border_secondary()
    }

    fn input_border_focused(&self) -> Hsla {
        self.accent()
    }

    /// Input text color
    fn input_text(&self) -> Hsla {
        self.fg()
    }

    /// Input placeholder text color
    fn input_placeholder(&self) -> Hsla {
        self.placeholder()
    }

    /// Input selection/highlight color
    fn input_selection(&self) -> Hsla {
        self.selection()
    }

    /// Input cursor/caret color
    fn input_cursor(&self) -> Hsla {
        self.accent()
    }

    // === Badge palette ===
    // Colors for small colored indicators: icon badges, status dots, category tags.
    // Defaults derive from semantic colors but at appropriate saturation for badge fills.

    fn badge_blue(&self) -> Hsla {
        self.info()
    }

    fn badge_gold(&self) -> Hsla {
        self.warning()
    }

    fn badge_red(&self) -> Hsla {
        self.danger()
    }

    fn badge_green(&self) -> Hsla {
        self.success()
    }

    fn badge_teal(&self) -> Hsla {
        hsla(180.0 / 360.0, 0.55, 0.45, 1.0)
    }

    fn badge_amber(&self) -> Hsla {
        hsla(30.0 / 360.0, 0.7, 0.50, 1.0)
    }

    fn badge_gray(&self) -> Hsla {
        self.fg_muted()
    }

    // === Control sizing ===
    // Sits beside the colors deliberately: a control's height is a design
    // decision of the same kind, and putting it anywhere else is how the crate
    // ended up with five different control heights and no way to change them
    // together.

    /// The three rungs of the shared control size scale.
    ///
    /// Override this to rescale every control at once.
    fn control_scale(&self) -> ControlScale {
        ControlScale::default()
    }

    /// The metrics for one rung — height, padding, gap, radius, text size,
    /// line box and ink.
    ///
    /// This is what an element calls; `control_scale` is what a theme
    /// overrides.
    fn control(&self, size: ControlSize) -> ControlMetrics {
        self.control_scale().metrics(size)
    }
}

/// The WCAG 2.1 contrast ratio between two opaque colours, 1.0 to 21.0.
///
/// Alpha is ignored: both callers pass fills, and compositing a translucent
/// one correctly needs the colour behind it, which a theme accessor does not
/// have. A ratio computed against the fill as if it were opaque is the right
/// answer for the fills the crate actually draws, and wrong in the same
/// direction as simply reading the declared colour, which is what the code
/// this replaced did.
fn contrast_ratio(a: Hsla, b: Hsla) -> f32 {
    let lighter = relative_luminance(a).max(relative_luminance(b));
    let darker = relative_luminance(a).min(relative_luminance(b));
    (lighter + 0.05) / (darker + 0.05)
}

/// WCAG relative luminance: sRGB channels linearised, then weighted by how
/// much the eye gets from each.
///
/// This is the quantity HSL's `l` is often mistaken for and is not. `l` is the
/// midpoint of the largest and smallest channel, so it says a fully saturated
/// red and a fully saturated cyan are equally bright; luminance says the cyan
/// is nearly four times brighter, which is what a reader sees.
fn relative_luminance(color: Hsla) -> f32 {
    let rgba = gpui::Rgba::from(color);
    let channel = |c: f32| {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(rgba.r) + 0.7152 * channel(rgba.g) + 0.0722 * channel(rgba.b)
}

/// A cluster of theme tokens that gpuikit does not define.
///
/// The crate's own vocabulary is fixed: [`Themeable`] names about forty
/// colours, and a crate built *on* gpuikit invariably needs a few more — a
/// diff view wants an added and a removed fill, a terminal wants sixteen ANSI
/// colours, a graph wants a series palette. Before this trait the only places
/// to put them were a second global that no theme could see, or a fork.
///
/// An extension is a plain struct of whatever a component needs, plus one
/// function saying what it looks like when nobody has said otherwise:
///
/// ```
/// use gpui::Hsla;
/// use gpuikit::theme::{Theme, ThemeExtension, Themeable};
///
/// #[derive(Clone)]
/// struct DiffColors {
///     added: Hsla,
///     removed: Hsla,
/// }
///
/// impl ThemeExtension for DiffColors {
///     fn derive(theme: &Theme) -> Self {
///         DiffColors {
///             added: theme.success().opacity(0.18),
///             removed: theme.danger().opacity(0.18),
///         }
///     }
/// }
///
/// // Every theme now answers for it, including ones written before
/// // `DiffColors` existed.
/// let theme = Theme::gruvbox_dark();
/// let diff = theme.extension::<DiffColors>();
/// assert_eq!(diff.added, theme.success().opacity(0.18));
/// ```
///
/// [`derive`](ThemeExtension::derive) is what makes this worth having. It is
/// the same bargain the colour tokens strike: a theme is complete the moment
/// its five primitives exist, and an author only writes down the parts they
/// disagree with. A theme that has never heard of an extension still has
/// coherent values for it, because they are computed from that theme's own
/// colours.
///
/// A theme author who *does* disagree says so with
/// [`Theme::with_extension`]:
///
/// ```
/// # use gpui::{Hsla, hsla};
/// # use gpuikit::theme::{Theme, ThemeExtension, Themeable};
/// # #[derive(Clone)]
/// # struct DiffColors { added: Hsla, removed: Hsla }
/// # impl ThemeExtension for DiffColors {
/// #     fn derive(theme: &Theme) -> Self {
/// #         DiffColors { added: theme.success(), removed: theme.danger() }
/// #     }
/// # }
/// let theme = Theme::gruvbox_dark().with_extension(DiffColors {
///     added: hsla(0.33, 0.6, 0.2, 1.0),
///     removed: hsla(0.0, 0.6, 0.2, 1.0),
/// });
/// assert_eq!(theme.extension::<DiffColors>().added, hsla(0.33, 0.6, 0.2, 1.0));
/// ```
///
/// # Cost
///
/// [`Theme::extension`] returns by value, and derives on every read when the
/// theme does not carry an explicit value. That is deliberate: caching would
/// need interior mutability on a type that is otherwise plainly `Clone`, and
/// the derivation is a few multiplications on colours already in cache. Read
/// it once at the top of a render function, the way you would `cx.theme()`
/// itself, rather than per row of a list.
pub trait ThemeExtension: Clone + Send + Sync + 'static {
    /// What this extension looks like under `theme` when the theme does not
    /// name it.
    ///
    /// Write it in terms of `theme`'s own accessors — `success()`, `accent()`,
    /// `surface()` — rather than fixed colours, or the extension will look
    /// wrong on every theme but the one it was written against.
    fn derive(theme: &Theme) -> Self;
}

/// The explicit extension values a [`Theme`] carries.
///
/// Opaque, and rarely named: build one through [`Theme::with_extension`] and
/// read it through [`Theme::extension`]. It is a public field on `Theme` only
/// because every other field is, so a struct literal is still possible.
#[derive(Clone, Default)]
pub struct ThemeExtensions(HashMap<TypeId, Arc<dyn Any + Send + Sync>>);

impl ThemeExtensions {
    /// The value stored for `T`, if a theme author set one.
    fn get<T: ThemeExtension>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    /// Store `value` for `T`, replacing any previous one.
    fn insert<T: ThemeExtension>(&mut self, value: T) {
        self.0.insert(TypeId::of::<T>(), Arc::new(value));
    }
}

/// Says how many extensions are set, not what they are: the values are
/// `dyn Any`, so there is nothing to print. `Theme`'s derived `Debug` needs
/// this to exist.
impl fmt::Debug for ThemeExtensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ThemeExtensions({} set)", self.0.len())
    }
}

pub fn init(cx: &mut App) {
    cx.set_global(GlobalTheme::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    #[default]
    Dark,
    Light,
}

/// A concrete theme implementation with stored color values.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: SharedString,
    pub variant: ThemeVariant,

    // Primitives
    pub fg_color: Hsla,
    pub bg_color: Hsla,
    pub surface_color: Hsla,
    pub border_color: Hsla,
    pub accent_color: Hsla,

    // Overrides (None = use default from trait)
    pub fg_muted_color: Option<Hsla>,
    pub fg_disabled_color: Option<Hsla>,
    pub surface_secondary_color: Option<Hsla>,
    pub surface_tertiary_color: Option<Hsla>,
    pub border_secondary_color: Option<Hsla>,
    pub border_subtle_color: Option<Hsla>,
    pub outline_color: Option<Hsla>,
    pub accent_bg_color: Option<Hsla>,
    pub accent_bg_hover_color: Option<Hsla>,
    pub info_color: Option<Hsla>,
    pub success_color: Option<Hsla>,
    pub warning_color: Option<Hsla>,
    pub danger_color: Option<Hsla>,
    pub selection_color: Option<Hsla>,
    pub placeholder_color: Option<Hsla>,
    pub destructive_bg_color: Option<Hsla>,
    pub destructive_bg_hover_color: Option<Hsla>,
    pub destructive_bg_active_color: Option<Hsla>,
    pub destructive_fg_color: Option<Hsla>,
    pub button_bg_color: Option<Hsla>,
    pub button_bg_hover_color: Option<Hsla>,
    pub button_bg_active_color: Option<Hsla>,
    pub button_border_color: Option<Hsla>,
    pub input_bg_color: Option<Hsla>,
    pub input_border_color: Option<Hsla>,
    pub input_border_hover_color: Option<Hsla>,
    pub input_border_focused_color: Option<Hsla>,
    pub input_text_color: Option<Hsla>,
    pub input_placeholder_color: Option<Hsla>,
    pub input_selection_color: Option<Hsla>,
    pub input_cursor_color: Option<Hsla>,
    pub overlay_color: Option<Hsla>,
    pub badge_blue_color: Option<Hsla>,
    pub badge_gold_color: Option<Hsla>,
    pub badge_red_color: Option<Hsla>,
    pub badge_green_color: Option<Hsla>,
    pub badge_teal_color: Option<Hsla>,
    pub badge_amber_color: Option<Hsla>,
    pub badge_gray_color: Option<Hsla>,

    /// The shared control size scale. Not an `Option`: every theme has one,
    /// and `ControlScale::default()` is the crate's scale rather than an
    /// absence of one.
    pub controls: ControlScale,

    /// Values a theme author set for extensions defined outside this crate.
    /// Anything absent is derived on read — see [`ThemeExtension`].
    pub extensions: ThemeExtensions,
}

impl Themeable for Theme {
    fn fg(&self) -> Hsla {
        self.fg_color
    }
    fn bg(&self) -> Hsla {
        self.bg_color
    }
    fn surface(&self) -> Hsla {
        self.surface_color
    }
    fn border(&self) -> Hsla {
        self.border_color
    }
    fn accent(&self) -> Hsla {
        self.accent_color
    }

    fn fg_muted(&self) -> Hsla {
        self.fg_muted_color
            .unwrap_or_else(|| self.fg().opacity(0.7))
    }
    fn fg_disabled(&self) -> Hsla {
        self.fg_disabled_color
            .unwrap_or_else(|| self.fg().opacity(0.4))
    }
    fn surface_secondary(&self) -> Hsla {
        self.surface_secondary_color
            .unwrap_or_else(|| self.surface())
    }
    fn surface_tertiary(&self) -> Hsla {
        self.surface_tertiary_color
            .unwrap_or_else(|| self.surface_secondary())
    }
    fn border_secondary(&self) -> Hsla {
        self.border_secondary_color.unwrap_or_else(|| self.border())
    }
    fn border_subtle(&self) -> Hsla {
        self.border_subtle_color
            .unwrap_or_else(|| self.border().opacity(0.5))
    }
    fn outline(&self) -> Hsla {
        self.outline_color.unwrap_or_else(|| self.accent())
    }
    fn accent_bg(&self) -> Hsla {
        self.accent_bg_color
            .unwrap_or_else(|| self.accent().opacity(0.15))
    }
    fn accent_bg_hover(&self) -> Hsla {
        self.accent_bg_hover_color
            .unwrap_or_else(|| self.accent().opacity(0.25))
    }
    fn info(&self) -> Hsla {
        self.info_color
            .unwrap_or_else(|| hsla(210.0 / 360.0, 0.7, 0.5, 1.0))
    }
    fn success(&self) -> Hsla {
        self.success_color
            .unwrap_or_else(|| hsla(142.0 / 360.0, 0.7, 0.4, 1.0))
    }
    fn warning(&self) -> Hsla {
        self.warning_color
            .unwrap_or_else(|| hsla(38.0 / 360.0, 0.9, 0.5, 1.0))
    }
    fn danger(&self) -> Hsla {
        self.danger_color
            .unwrap_or_else(|| hsla(0.0, 0.7, 0.5, 1.0))
    }
    fn selection(&self) -> Hsla {
        self.selection_color
            .unwrap_or_else(|| self.accent().opacity(0.3))
    }
    fn placeholder(&self) -> Hsla {
        self.placeholder_color
            .unwrap_or_else(|| self.fg().opacity(0.5))
    }
    // The four destructive tokens repeat their trait defaults rather than
    // calling them: an impl cannot reach the default it is overriding, and
    // they have to be overridable for `from_themeable` to be lossless.
    fn destructive_bg(&self) -> Hsla {
        self.destructive_bg_color.unwrap_or_else(|| self.danger())
    }
    fn destructive_bg_hover(&self) -> Hsla {
        self.destructive_bg_hover_color.unwrap_or_else(|| {
            let base = self.destructive_bg();
            hsla(base.h, base.s, (base.l + 0.08).min(1.0), base.a)
        })
    }
    fn destructive_bg_active(&self) -> Hsla {
        self.destructive_bg_active_color.unwrap_or_else(|| {
            let base = self.destructive_bg();
            hsla(base.h, base.s, (base.l - 0.08).max(0.0), base.a)
        })
    }
    fn destructive_fg(&self) -> Hsla {
        self.destructive_fg_color.unwrap_or_else(|| {
            let black = hsla(0.0, 0.0, 0.0, 1.0);
            let white = hsla(0.0, 0.0, 1.0, 1.0);
            let bg = self.destructive_bg();
            if contrast_ratio(bg, white) >= contrast_ratio(bg, black) {
                white
            } else {
                black
            }
        })
    }
    fn button_bg(&self) -> Hsla {
        self.button_bg_color.unwrap_or_else(|| self.surface())
    }
    fn button_bg_hover(&self) -> Hsla {
        self.button_bg_hover_color
            .unwrap_or_else(|| self.surface_secondary())
    }
    fn button_bg_active(&self) -> Hsla {
        self.button_bg_active_color
            .unwrap_or_else(|| self.surface_tertiary())
    }
    fn button_border(&self) -> Hsla {
        self.button_border_color.unwrap_or_else(|| self.border())
    }
    fn input_bg(&self) -> Hsla {
        self.input_bg_color.unwrap_or_else(|| self.surface())
    }
    fn input_border(&self) -> Hsla {
        self.input_border_color.unwrap_or_else(|| self.border())
    }
    fn input_border_hover(&self) -> Hsla {
        self.input_border_hover_color
            .unwrap_or_else(|| self.border_secondary())
    }
    fn input_border_focused(&self) -> Hsla {
        self.input_border_focused_color
            .unwrap_or_else(|| self.accent())
    }
    fn input_text(&self) -> Hsla {
        self.input_text_color.unwrap_or_else(|| self.fg())
    }
    fn input_placeholder(&self) -> Hsla {
        self.input_placeholder_color
            .unwrap_or_else(|| self.placeholder())
    }
    fn input_selection(&self) -> Hsla {
        self.input_selection_color
            .unwrap_or_else(|| self.selection())
    }
    fn input_cursor(&self) -> Hsla {
        self.input_cursor_color.unwrap_or_else(|| self.accent())
    }
    fn overlay(&self) -> Hsla {
        self.overlay_color
            .unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.6))
    }
    fn badge_blue(&self) -> Hsla {
        self.badge_blue_color.unwrap_or_else(|| self.info())
    }
    fn badge_gold(&self) -> Hsla {
        self.badge_gold_color.unwrap_or_else(|| self.warning())
    }
    fn badge_red(&self) -> Hsla {
        self.badge_red_color.unwrap_or_else(|| self.danger())
    }
    fn badge_green(&self) -> Hsla {
        self.badge_green_color.unwrap_or_else(|| self.success())
    }
    fn badge_teal(&self) -> Hsla {
        self.badge_teal_color
            .unwrap_or_else(|| hsla(180.0 / 360.0, 0.55, 0.45, 1.0))
    }
    fn badge_amber(&self) -> Hsla {
        self.badge_amber_color
            .unwrap_or_else(|| hsla(30.0 / 360.0, 0.7, 0.50, 1.0))
    }
    fn badge_gray(&self) -> Hsla {
        self.badge_gray_color.unwrap_or_else(|| self.fg_muted())
    }
    fn control_scale(&self) -> ControlScale {
        self.controls
    }
}

impl Theme {
    /// Create a new theme with just the required primitives.
    /// All other colors will use sensible defaults.
    pub fn new(
        name: impl Into<SharedString>,
        variant: ThemeVariant,
        fg: Hsla,
        bg: Hsla,
        surface: Hsla,
        border: Hsla,
        accent: Hsla,
    ) -> Self {
        Theme {
            name: name.into(),
            variant,
            fg_color: fg,
            bg_color: bg,
            surface_color: surface,
            border_color: border,
            accent_color: accent,
            fg_muted_color: None,
            fg_disabled_color: None,
            surface_secondary_color: None,
            surface_tertiary_color: None,
            border_secondary_color: None,
            border_subtle_color: None,
            outline_color: None,
            accent_bg_color: None,
            accent_bg_hover_color: None,
            info_color: None,
            success_color: None,
            warning_color: None,
            danger_color: None,
            selection_color: None,
            placeholder_color: None,
            destructive_bg_color: None,
            destructive_bg_hover_color: None,
            destructive_bg_active_color: None,
            destructive_fg_color: None,
            button_bg_color: None,
            button_bg_hover_color: None,
            button_bg_active_color: None,
            button_border_color: None,
            input_bg_color: None,
            input_border_color: None,
            input_border_hover_color: None,
            input_border_focused_color: None,
            input_text_color: None,
            input_placeholder_color: None,
            input_selection_color: None,
            input_cursor_color: None,
            overlay_color: None,
            badge_blue_color: None,
            badge_gold_color: None,
            badge_red_color: None,
            badge_green_color: None,
            badge_teal_color: None,
            badge_amber_color: None,
            badge_gray_color: None,
            controls: ControlScale::default(),
            extensions: ThemeExtensions::default(),
        }
    }

    /// Create a Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        let mut theme = Theme::new(
            "Gruvbox Dark",
            ThemeVariant::Dark,
            parse_hex("#ebdbb2"), // fg
            parse_hex("#282828"), // bg
            parse_hex("#3c3836"), // surface
            parse_hex("#504945"), // border
            parse_hex("#8ec07c"), // accent
        );
        theme.fg_muted_color = Some(parse_hex("#a89984"));
        theme.fg_disabled_color = Some(parse_hex("#7c6f64"));
        theme.surface_secondary_color = Some(parse_hex("#504945"));
        theme.surface_tertiary_color = Some(parse_hex("#665c54"));
        theme.border_secondary_color = Some(parse_hex("#7c6f64"));
        theme.border_subtle_color = Some(parse_hex("#3c3836"));
        theme.outline_color = Some(parse_hex("#458588"));
        theme.info_color = Some(parse_hex("#458588"));
        theme.success_color = Some(parse_hex("#b8bb26"));
        theme.warning_color = Some(parse_hex("#fabd2f"));
        theme.danger_color = Some(parse_hex("#fb4934"));
        theme.selection_color = Some(hsla(55.0 / 360.0, 0.56, 0.64, 0.25));
        theme.button_bg_color = Some(parse_hex("#504945"));
        theme.button_bg_hover_color = Some(parse_hex("#665c54"));
        theme.button_bg_active_color = Some(parse_hex("#7c6f64"));
        theme.button_border_color = Some(parse_hex("#7c6f64"));
        theme.input_border_hover_color = Some(parse_hex("#665c54"));
        theme
    }

    /// Create a Gruvbox Light theme
    pub fn gruvbox_light() -> Self {
        let mut theme = Theme::new(
            "Gruvbox Light",
            ThemeVariant::Light,
            parse_hex("#3c3836"), // fg
            parse_hex("#fbf1c7"), // bg
            parse_hex("#ebdbb2"), // surface
            parse_hex("#d5c4a1"), // border
            parse_hex("#427b58"), // accent
        );
        theme.fg_muted_color = Some(parse_hex("#665c54"));
        theme.fg_disabled_color = Some(parse_hex("#a89984"));
        theme.surface_secondary_color = Some(parse_hex("#d5c4a1"));
        theme.surface_tertiary_color = Some(parse_hex("#bdae93"));
        theme.border_secondary_color = Some(parse_hex("#a89984"));
        theme.border_subtle_color = Some(parse_hex("#ebdbb2"));
        theme.outline_color = Some(parse_hex("#076678"));
        theme.info_color = Some(parse_hex("#076678"));
        theme.success_color = Some(parse_hex("#79740e"));
        theme.warning_color = Some(parse_hex("#b57614"));
        theme.danger_color = Some(parse_hex("#cc241d"));
        theme.selection_color = Some(hsla(48.0 / 360.0, 0.87, 0.61, 0.15));
        theme.button_bg_color = Some(parse_hex("#ebdbb2"));
        theme.button_bg_hover_color = Some(parse_hex("#d5c4a1"));
        theme.button_bg_active_color = Some(parse_hex("#bdae93"));
        theme.button_border_color = Some(parse_hex("#a89984"));
        theme.input_border_hover_color = Some(parse_hex("#bdae93"));
        theme
    }

    /// Create a Catppuccin Latte (light) theme
    pub fn catppuccin_latte() -> Self {
        let mut theme = Theme::new(
            "Catppuccin Latte",
            ThemeVariant::Light,
            parse_hex("#4c4f69"), // text
            parse_hex("#eff1f5"), // base
            parse_hex("#e6e9ef"), // mantle
            parse_hex("#ccd0da"), // surface0
            parse_hex("#8839ef"), // mauve
        );
        theme.fg_muted_color = Some(parse_hex("#6c6f85"));
        theme.fg_disabled_color = Some(parse_hex("#9ca0b0"));
        theme.surface_secondary_color = Some(parse_hex("#ccd0da"));
        theme.surface_tertiary_color = Some(parse_hex("#bcc0cc"));
        theme.border_secondary_color = Some(parse_hex("#acb0be"));
        theme.border_subtle_color = Some(parse_hex("#e6e9ef"));
        theme.info_color = Some(parse_hex("#1e66f5"));
        theme.success_color = Some(parse_hex("#40a02b"));
        theme.warning_color = Some(parse_hex("#df8e1d"));
        theme.danger_color = Some(parse_hex("#d20f39"));
        theme.button_bg_color = Some(parse_hex("#ccd0da"));
        theme.button_bg_hover_color = Some(parse_hex("#bcc0cc"));
        theme.button_bg_active_color = Some(parse_hex("#acb0be"));
        theme.button_border_color = Some(parse_hex("#acb0be"));
        theme.input_border_hover_color = Some(parse_hex("#acb0be"));
        theme.badge_blue_color = Some(parse_hex("#1e66f5"));
        theme.badge_gold_color = Some(parse_hex("#df8e1d"));
        theme.badge_red_color = Some(parse_hex("#d20f39"));
        theme.badge_green_color = Some(parse_hex("#40a02b"));
        theme.badge_teal_color = Some(parse_hex("#179299"));
        theme.badge_amber_color = Some(parse_hex("#fe640b"));
        theme.badge_gray_color = Some(parse_hex("#6c6f85"));
        theme
    }

    /// Create a Catppuccin Frappé theme
    pub fn catppuccin_frappe() -> Self {
        let mut theme = Theme::new(
            "Catppuccin Frappé",
            ThemeVariant::Dark,
            parse_hex("#c6d0f5"), // text
            parse_hex("#303446"), // base
            parse_hex("#292c3c"), // mantle
            parse_hex("#414559"), // surface0
            parse_hex("#ca9ee6"), // mauve
        );
        theme.fg_muted_color = Some(parse_hex("#a5adce"));
        theme.fg_disabled_color = Some(parse_hex("#838ba7"));
        theme.surface_secondary_color = Some(parse_hex("#51576d"));
        theme.surface_tertiary_color = Some(parse_hex("#626880"));
        theme.border_secondary_color = Some(parse_hex("#626880"));
        theme.border_subtle_color = Some(parse_hex("#292c3c"));
        theme.info_color = Some(parse_hex("#8caaee"));
        theme.success_color = Some(parse_hex("#a6d189"));
        theme.warning_color = Some(parse_hex("#e5c890"));
        theme.danger_color = Some(parse_hex("#e78284"));
        theme.button_bg_color = Some(parse_hex("#414559"));
        theme.button_bg_hover_color = Some(parse_hex("#51576d"));
        theme.button_bg_active_color = Some(parse_hex("#626880"));
        theme.button_border_color = Some(parse_hex("#626880"));
        theme.input_border_hover_color = Some(parse_hex("#51576d"));
        theme.badge_blue_color = Some(parse_hex("#8caaee"));
        theme.badge_gold_color = Some(parse_hex("#e5c890"));
        theme.badge_red_color = Some(parse_hex("#e78284"));
        theme.badge_green_color = Some(parse_hex("#a6d189"));
        theme.badge_teal_color = Some(parse_hex("#81c8be"));
        theme.badge_amber_color = Some(parse_hex("#ef9f76"));
        theme.badge_gray_color = Some(parse_hex("#a5adce"));
        theme
    }

    /// Create a Catppuccin Macchiato theme
    pub fn catppuccin_macchiato() -> Self {
        let mut theme = Theme::new(
            "Catppuccin Macchiato",
            ThemeVariant::Dark,
            parse_hex("#cad3f5"), // text
            parse_hex("#24273a"), // base
            parse_hex("#1e2030"), // mantle
            parse_hex("#363a4f"), // surface0
            parse_hex("#c6a0f6"), // mauve
        );
        theme.fg_muted_color = Some(parse_hex("#a5adcb"));
        theme.fg_disabled_color = Some(parse_hex("#8087a2"));
        theme.surface_secondary_color = Some(parse_hex("#494d64"));
        theme.surface_tertiary_color = Some(parse_hex("#5b6078"));
        theme.border_secondary_color = Some(parse_hex("#5b6078"));
        theme.border_subtle_color = Some(parse_hex("#1e2030"));
        theme.info_color = Some(parse_hex("#8aadf4"));
        theme.success_color = Some(parse_hex("#a6da95"));
        theme.warning_color = Some(parse_hex("#eed49f"));
        theme.danger_color = Some(parse_hex("#ed8796"));
        theme.button_bg_color = Some(parse_hex("#363a4f"));
        theme.button_bg_hover_color = Some(parse_hex("#494d64"));
        theme.button_bg_active_color = Some(parse_hex("#5b6078"));
        theme.button_border_color = Some(parse_hex("#5b6078"));
        theme.input_border_hover_color = Some(parse_hex("#494d64"));
        theme.badge_blue_color = Some(parse_hex("#8aadf4"));
        theme.badge_gold_color = Some(parse_hex("#eed49f"));
        theme.badge_red_color = Some(parse_hex("#ed8796"));
        theme.badge_green_color = Some(parse_hex("#a6da95"));
        theme.badge_teal_color = Some(parse_hex("#8bd5ca"));
        theme.badge_amber_color = Some(parse_hex("#f5a97f"));
        theme.badge_gray_color = Some(parse_hex("#a5adcb"));
        theme
    }

    /// Create a Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        let mut theme = Theme::new(
            "Catppuccin Mocha",
            ThemeVariant::Dark,
            parse_hex("#cdd6f4"), // text
            parse_hex("#1e1e2e"), // base
            parse_hex("#181825"), // mantle
            parse_hex("#313244"), // surface0
            parse_hex("#cba6f7"), // mauve
        );
        theme.fg_muted_color = Some(parse_hex("#a6adc8"));
        theme.fg_disabled_color = Some(parse_hex("#7f849c"));
        theme.surface_secondary_color = Some(parse_hex("#45475a"));
        theme.surface_tertiary_color = Some(parse_hex("#585b70"));
        theme.border_secondary_color = Some(parse_hex("#585b70"));
        theme.border_subtle_color = Some(parse_hex("#181825"));
        theme.info_color = Some(parse_hex("#89b4fa"));
        theme.success_color = Some(parse_hex("#a6e3a1"));
        theme.warning_color = Some(parse_hex("#f9e2af"));
        theme.danger_color = Some(parse_hex("#f38ba8"));
        theme.button_bg_color = Some(parse_hex("#313244"));
        theme.button_bg_hover_color = Some(parse_hex("#45475a"));
        theme.button_bg_active_color = Some(parse_hex("#585b70"));
        theme.button_border_color = Some(parse_hex("#585b70"));
        theme.input_border_hover_color = Some(parse_hex("#45475a"));
        theme.badge_blue_color = Some(parse_hex("#89b4fa"));
        theme.badge_gold_color = Some(parse_hex("#f9e2af"));
        theme.badge_red_color = Some(parse_hex("#f38ba8"));
        theme.badge_green_color = Some(parse_hex("#a6e3a1"));
        theme.badge_teal_color = Some(parse_hex("#94e2d5"));
        theme.badge_amber_color = Some(parse_hex("#fab387"));
        theme.badge_gray_color = Some(parse_hex("#a6adc8"));
        theme
    }

    /// This theme's values for the extension `T`.
    ///
    /// Returns what a theme author set with [`with_extension`](Theme::with_extension),
    /// or [`ThemeExtension::derive`] applied to this theme when they set
    /// nothing — so this never fails and never needs an `Option` at the call
    /// site. See [`ThemeExtension`] for the cost of calling it.
    pub fn extension<T: ThemeExtension>(&self) -> T {
        self.extensions
            .get::<T>()
            .cloned()
            .unwrap_or_else(|| T::derive(self))
    }

    /// This theme, carrying `value` for the extension `T`.
    ///
    /// Chainable, so a theme and its extensions read as one expression.
    pub fn with_extension<T: ThemeExtension>(mut self, value: T) -> Self {
        self.extensions.insert(value);
        self
    }

    /// Whether a value for `T` was set explicitly, rather than derived.
    ///
    /// Only a theme editor needs this — a component should call
    /// [`extension`](Theme::extension) and not care which it got.
    pub fn has_extension<T: ThemeExtension>(&self) -> bool {
        self.extensions.get::<T>().is_some()
    }

    /// Resolve any [`Themeable`] into a concrete `Theme`.
    ///
    /// This is how a type of your own becomes the active theme. Implement
    /// `Themeable` on it, snapshot it here, and install the result the usual
    /// way:
    ///
    /// ```
    /// # use gpui::{Hsla, hsla};
    /// use gpuikit::theme::{Theme, ThemeVariant, Themeable};
    ///
    /// struct BrandTheme;
    ///
    /// impl Themeable for BrandTheme {
    ///     fn fg(&self) -> Hsla { hsla(0.0, 0.0, 0.9, 1.0) }
    ///     fn bg(&self) -> Hsla { hsla(0.6, 0.2, 0.1, 1.0) }
    ///     fn surface(&self) -> Hsla { hsla(0.6, 0.2, 0.16, 1.0) }
    ///     fn border(&self) -> Hsla { hsla(0.6, 0.2, 0.3, 1.0) }
    ///     fn accent(&self) -> Hsla { hsla(0.9, 0.7, 0.6, 1.0) }
    /// }
    ///
    /// let theme = Theme::from_themeable("Brand", ThemeVariant::Dark, &BrandTheme);
    /// assert_eq!(theme.accent(), BrandTheme.accent());
    /// ```
    ///
    /// # Why a snapshot, and not `Arc<dyn Themeable>`
    ///
    /// A theme changes when someone picks a new one. It is *read* several
    /// hundred times per frame, by every element that draws. Making the global
    /// a trait object would put a vtable dispatch on each of those reads to buy
    /// flexibility at the one moment it is not needed. Resolving every token
    /// once, here, pays that cost a single time and leaves the hot path a field
    /// read.
    ///
    /// The snapshot is lossless: `Theme` carries an override field for every
    /// token `Themeable` defines, and this sets all of them, so the result
    /// answers identically to its source. `control_scale` comes across too.
    /// What does *not* come across is behaviour — a `Themeable` whose
    /// `danger()` varies with the time of day is a fixed colour once
    /// snapshotted, and should be re-snapshotted when it changes.
    pub fn from_themeable(
        name: impl Into<SharedString>,
        variant: ThemeVariant,
        source: &dyn Themeable,
    ) -> Self {
        Theme {
            name: name.into(),
            variant,
            fg_color: source.fg(),
            bg_color: source.bg(),
            surface_color: source.surface(),
            border_color: source.border(),
            accent_color: source.accent(),
            fg_muted_color: Some(source.fg_muted()),
            fg_disabled_color: Some(source.fg_disabled()),
            surface_secondary_color: Some(source.surface_secondary()),
            surface_tertiary_color: Some(source.surface_tertiary()),
            border_secondary_color: Some(source.border_secondary()),
            border_subtle_color: Some(source.border_subtle()),
            outline_color: Some(source.outline()),
            accent_bg_color: Some(source.accent_bg()),
            accent_bg_hover_color: Some(source.accent_bg_hover()),
            info_color: Some(source.info()),
            success_color: Some(source.success()),
            warning_color: Some(source.warning()),
            danger_color: Some(source.danger()),
            selection_color: Some(source.selection()),
            placeholder_color: Some(source.placeholder()),
            destructive_bg_color: Some(source.destructive_bg()),
            destructive_bg_hover_color: Some(source.destructive_bg_hover()),
            destructive_bg_active_color: Some(source.destructive_bg_active()),
            destructive_fg_color: Some(source.destructive_fg()),
            button_bg_color: Some(source.button_bg()),
            button_bg_hover_color: Some(source.button_bg_hover()),
            button_bg_active_color: Some(source.button_bg_active()),
            button_border_color: Some(source.button_border()),
            input_bg_color: Some(source.input_bg()),
            input_border_color: Some(source.input_border()),
            input_border_hover_color: Some(source.input_border_hover()),
            input_border_focused_color: Some(source.input_border_focused()),
            input_text_color: Some(source.input_text()),
            input_placeholder_color: Some(source.input_placeholder()),
            input_selection_color: Some(source.input_selection()),
            input_cursor_color: Some(source.input_cursor()),
            overlay_color: Some(source.overlay()),
            badge_blue_color: Some(source.badge_blue()),
            badge_gold_color: Some(source.badge_gold()),
            badge_red_color: Some(source.badge_red()),
            badge_green_color: Some(source.badge_green()),
            badge_teal_color: Some(source.badge_teal()),
            badge_amber_color: Some(source.badge_amber()),
            badge_gray_color: Some(source.badge_gray()),
            controls: source.control_scale(),
            extensions: ThemeExtensions::default(),
        }
    }

    pub fn get_global(cx: &App) -> &Arc<Theme> {
        &cx.global::<GlobalTheme>().0
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::gruvbox_dark()
    }
}

#[derive(Clone, Debug)]
pub struct GlobalTheme(pub Arc<Theme>);

impl Global for GlobalTheme {}

impl Default for GlobalTheme {
    fn default() -> Self {
        GlobalTheme(Arc::new(Theme::default()))
    }
}

/// Trait for accessing the current theme from an App context
pub trait ActiveTheme {
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        &self.global::<GlobalTheme>().0
    }
}

pub fn parse_hex(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let lightness = (max + min) / 2.0;

    let saturation = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * lightness - 1.0).abs())
    };

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    gpui::hsla(hue / 360.0, saturation, lightness, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every token `Themeable` defines, read off a theme. The round-trip test
    /// below compares two themes through this, so a token missing here is a
    /// token the test silently stops checking — keep it in step with the
    /// trait.
    fn every_token(theme: &dyn Themeable) -> Vec<(&'static str, Hsla)> {
        vec![
            ("fg", theme.fg()),
            ("bg", theme.bg()),
            ("surface", theme.surface()),
            ("border", theme.border()),
            ("accent", theme.accent()),
            ("fg_muted", theme.fg_muted()),
            ("fg_disabled", theme.fg_disabled()),
            ("surface_secondary", theme.surface_secondary()),
            ("surface_tertiary", theme.surface_tertiary()),
            ("border_secondary", theme.border_secondary()),
            ("border_subtle", theme.border_subtle()),
            ("outline", theme.outline()),
            ("accent_bg", theme.accent_bg()),
            ("accent_bg_hover", theme.accent_bg_hover()),
            ("info", theme.info()),
            ("success", theme.success()),
            ("warning", theme.warning()),
            ("danger", theme.danger()),
            ("selection", theme.selection()),
            ("placeholder", theme.placeholder()),
            ("overlay", theme.overlay()),
            ("button_bg", theme.button_bg()),
            ("button_bg_hover", theme.button_bg_hover()),
            ("button_bg_active", theme.button_bg_active()),
            ("button_border", theme.button_border()),
            ("destructive_bg", theme.destructive_bg()),
            ("destructive_bg_hover", theme.destructive_bg_hover()),
            ("destructive_bg_active", theme.destructive_bg_active()),
            ("destructive_fg", theme.destructive_fg()),
            ("input_bg", theme.input_bg()),
            ("input_border", theme.input_border()),
            ("input_border_hover", theme.input_border_hover()),
            ("input_border_focused", theme.input_border_focused()),
            ("input_text", theme.input_text()),
            ("input_placeholder", theme.input_placeholder()),
            ("input_selection", theme.input_selection()),
            ("input_cursor", theme.input_cursor()),
            ("badge_blue", theme.badge_blue()),
            ("badge_gold", theme.badge_gold()),
            ("badge_red", theme.badge_red()),
            ("badge_green", theme.badge_green()),
            ("badge_teal", theme.badge_teal()),
            ("badge_amber", theme.badge_amber()),
            ("badge_gray", theme.badge_gray()),
        ]
    }

    /// A `Themeable` that is not `Theme`, with a value for every token that
    /// differs from what the defaults would derive — so a snapshot that
    /// dropped any one of them would be caught.
    struct LoudTheme;

    impl Themeable for LoudTheme {
        fn fg(&self) -> Hsla {
            hsla(0.11, 0.11, 0.11, 1.0)
        }
        fn bg(&self) -> Hsla {
            hsla(0.12, 0.12, 0.12, 1.0)
        }
        fn surface(&self) -> Hsla {
            hsla(0.13, 0.13, 0.13, 1.0)
        }
        fn border(&self) -> Hsla {
            hsla(0.14, 0.14, 0.14, 1.0)
        }
        fn accent(&self) -> Hsla {
            hsla(0.15, 0.15, 0.15, 1.0)
        }
        // Every remaining token, said explicitly and distinctly.
        fn fg_muted(&self) -> Hsla {
            hsla(0.21, 0.5, 0.5, 1.0)
        }
        fn fg_disabled(&self) -> Hsla {
            hsla(0.22, 0.5, 0.5, 1.0)
        }
        fn surface_secondary(&self) -> Hsla {
            hsla(0.23, 0.5, 0.5, 1.0)
        }
        fn surface_tertiary(&self) -> Hsla {
            hsla(0.24, 0.5, 0.5, 1.0)
        }
        fn border_secondary(&self) -> Hsla {
            hsla(0.25, 0.5, 0.5, 1.0)
        }
        fn border_subtle(&self) -> Hsla {
            hsla(0.26, 0.5, 0.5, 1.0)
        }
        fn outline(&self) -> Hsla {
            hsla(0.27, 0.5, 0.5, 1.0)
        }
        fn accent_bg(&self) -> Hsla {
            hsla(0.28, 0.5, 0.5, 1.0)
        }
        fn accent_bg_hover(&self) -> Hsla {
            hsla(0.29, 0.5, 0.5, 1.0)
        }
        fn info(&self) -> Hsla {
            hsla(0.30, 0.5, 0.5, 1.0)
        }
        fn success(&self) -> Hsla {
            hsla(0.31, 0.5, 0.5, 1.0)
        }
        fn warning(&self) -> Hsla {
            hsla(0.32, 0.5, 0.5, 1.0)
        }
        fn danger(&self) -> Hsla {
            hsla(0.33, 0.5, 0.5, 1.0)
        }
        fn selection(&self) -> Hsla {
            hsla(0.34, 0.5, 0.5, 1.0)
        }
        fn placeholder(&self) -> Hsla {
            hsla(0.35, 0.5, 0.5, 1.0)
        }
        fn overlay(&self) -> Hsla {
            hsla(0.36, 0.5, 0.5, 1.0)
        }
        fn button_bg(&self) -> Hsla {
            hsla(0.37, 0.5, 0.5, 1.0)
        }
        fn button_bg_hover(&self) -> Hsla {
            hsla(0.38, 0.5, 0.5, 1.0)
        }
        fn button_bg_active(&self) -> Hsla {
            hsla(0.39, 0.5, 0.5, 1.0)
        }
        fn button_border(&self) -> Hsla {
            hsla(0.40, 0.5, 0.5, 1.0)
        }
        fn destructive_bg(&self) -> Hsla {
            hsla(0.41, 0.5, 0.5, 1.0)
        }
        fn destructive_bg_hover(&self) -> Hsla {
            hsla(0.42, 0.5, 0.5, 1.0)
        }
        fn destructive_bg_active(&self) -> Hsla {
            hsla(0.43, 0.5, 0.5, 1.0)
        }
        fn destructive_fg(&self) -> Hsla {
            hsla(0.44, 0.5, 0.5, 1.0)
        }
        fn input_bg(&self) -> Hsla {
            hsla(0.45, 0.5, 0.5, 1.0)
        }
        fn input_border(&self) -> Hsla {
            hsla(0.46, 0.5, 0.5, 1.0)
        }
        fn input_border_hover(&self) -> Hsla {
            hsla(0.47, 0.5, 0.5, 1.0)
        }
        fn input_border_focused(&self) -> Hsla {
            hsla(0.48, 0.5, 0.5, 1.0)
        }
        fn input_text(&self) -> Hsla {
            hsla(0.49, 0.5, 0.5, 1.0)
        }
        fn input_placeholder(&self) -> Hsla {
            hsla(0.50, 0.5, 0.5, 1.0)
        }
        fn input_selection(&self) -> Hsla {
            hsla(0.51, 0.5, 0.5, 1.0)
        }
        fn input_cursor(&self) -> Hsla {
            hsla(0.52, 0.5, 0.5, 1.0)
        }
        fn badge_blue(&self) -> Hsla {
            hsla(0.53, 0.5, 0.5, 1.0)
        }
        fn badge_gold(&self) -> Hsla {
            hsla(0.54, 0.5, 0.5, 1.0)
        }
        fn badge_red(&self) -> Hsla {
            hsla(0.55, 0.5, 0.5, 1.0)
        }
        fn badge_green(&self) -> Hsla {
            hsla(0.56, 0.5, 0.5, 1.0)
        }
        fn badge_teal(&self) -> Hsla {
            hsla(0.57, 0.5, 0.5, 1.0)
        }
        fn badge_amber(&self) -> Hsla {
            hsla(0.58, 0.5, 0.5, 1.0)
        }
        fn badge_gray(&self) -> Hsla {
            hsla(0.59, 0.5, 0.5, 1.0)
        }
    }

    /// The claim `from_themeable`'s docs make: the snapshot answers exactly
    /// what its source did, for every token, including the ones a `Theme`
    /// gained a field for only so that this could be true.
    #[test]
    fn from_themeable_is_lossless() {
        let snapshot = Theme::from_themeable("Loud", ThemeVariant::Dark, &LoudTheme);

        for ((name, source), (_, resolved)) in every_token(&LoudTheme)
            .into_iter()
            .zip(every_token(&snapshot))
        {
            assert_eq!(
                source, resolved,
                "`{name}` did not survive from_themeable: Theme has no override field for it, \
                 so the snapshot fell back to deriving it"
            );
        }
    }

    /// The same, for the bundled themes: snapshotting one has to be a no-op.
    #[test]
    fn snapshotting_a_theme_changes_nothing() {
        for theme in [
            Theme::gruvbox_dark(),
            Theme::gruvbox_light(),
            Theme::catppuccin_latte(),
            Theme::catppuccin_mocha(),
        ] {
            let snapshot = Theme::from_themeable(theme.name.clone(), theme.variant, &theme);
            for ((name, before), (_, after)) in
                every_token(&theme).into_iter().zip(every_token(&snapshot))
            {
                assert_eq!(
                    before, after,
                    "`{name}` changed when {} was snapshotted",
                    theme.name
                );
            }
            assert_eq!(theme.control_scale(), snapshot.control_scale());
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct DiffColors {
        added: Hsla,
        removed: Hsla,
    }

    impl ThemeExtension for DiffColors {
        fn derive(theme: &Theme) -> Self {
            DiffColors {
                added: theme.success().opacity(0.18),
                removed: theme.danger().opacity(0.18),
            }
        }
    }

    /// A theme that has never heard of an extension still answers for it, in
    /// its own colours. This is the whole point of `derive`.
    #[test]
    fn an_unset_extension_derives_from_the_theme() {
        let dark = Theme::gruvbox_dark();
        let light = Theme::gruvbox_light();

        assert_eq!(
            dark.extension::<DiffColors>(),
            DiffColors {
                added: dark.success().opacity(0.18),
                removed: dark.danger().opacity(0.18),
            }
        );
        // And it follows the theme, rather than being one fixed pair.
        assert_ne!(
            dark.extension::<DiffColors>(),
            light.extension::<DiffColors>()
        );
        assert!(!dark.has_extension::<DiffColors>());
    }

    #[test]
    fn an_explicit_extension_wins_over_derivation() {
        let loud = DiffColors {
            added: hsla(0.1, 1.0, 0.5, 1.0),
            removed: hsla(0.9, 1.0, 0.5, 1.0),
        };
        let theme = Theme::gruvbox_dark().with_extension(loud.clone());

        assert!(theme.has_extension::<DiffColors>());
        assert_eq!(theme.extension::<DiffColors>(), loud);
    }

    /// Two extensions do not collide, and setting one leaves the other
    /// deriving.
    #[test]
    fn extensions_are_keyed_by_type() {
        #[derive(Clone, Debug, PartialEq)]
        struct Series(Hsla);
        impl ThemeExtension for Series {
            fn derive(theme: &Theme) -> Self {
                Series(theme.accent())
            }
        }

        let theme = Theme::gruvbox_dark().with_extension(Series(hsla(0.5, 1.0, 0.5, 1.0)));

        assert_eq!(
            theme.extension::<Series>(),
            Series(hsla(0.5, 1.0, 0.5, 1.0))
        );
        assert!(!theme.has_extension::<DiffColors>());
        assert_eq!(
            theme.extension::<DiffColors>(),
            DiffColors::derive(&theme),
            "setting one extension must not disturb another"
        );
    }

    /// Extensions ride along with the theme they were set on.
    #[test]
    fn extensions_survive_a_clone() {
        let theme = Theme::gruvbox_dark().with_extension(DiffColors {
            added: hsla(0.1, 1.0, 0.5, 1.0),
            removed: hsla(0.9, 1.0, 0.5, 1.0),
        });
        let copy = theme.clone();
        assert_eq!(
            copy.extension::<DiffColors>(),
            theme.extension::<DiffColors>()
        );
    }

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, SharedString::from("Gruvbox Dark"));
        assert_eq!(theme.variant, ThemeVariant::Dark);
    }

    #[test]
    fn test_minimal_theme() {
        // Create a theme with just primitives - all else uses defaults
        let theme = Theme::new(
            "Minimal",
            ThemeVariant::Dark,
            parse_hex("#ffffff"),
            parse_hex("#000000"),
            parse_hex("#111111"),
            parse_hex("#333333"),
            parse_hex("#0066cc"),
        );

        // Derived values should work
        assert_eq!(theme.button_bg(), theme.surface());
        assert_eq!(theme.input_border_focused(), theme.accent());
    }

    /// A theme's scale has to actually reach the elements. `Themeable::control`
    /// Every built-in theme, by name, so a new one has to be added here and
    /// is then held to the contrast assertion below.
    fn built_in_themes() -> Vec<(&'static str, Theme)> {
        vec![
            ("default", Theme::default()),
            ("gruvbox_dark", Theme::gruvbox_dark()),
            ("gruvbox_light", Theme::gruvbox_light()),
            ("catppuccin_latte", Theme::catppuccin_latte()),
            ("catppuccin_frappe", Theme::catppuccin_frappe()),
            ("catppuccin_macchiato", Theme::catppuccin_macchiato()),
            ("catppuccin_mocha", Theme::catppuccin_mocha()),
        ]
    }

    /// The defect this replaced: `destructive_fg` picked black or white off
    /// the fill's HSL lightness, and Gruvbox Dark's `#fb4934` sits at `l =
    /// 0.594` — just inside the "dark enough for white text" side of a 0.6
    /// threshold — while its *luminance* puts black nearly twice as far
    /// ahead. The button was legible, badly, and looked washed out.
    ///
    /// 4.5:1 is WCAG AA for body text. Every built-in theme clears it on the
    /// better of the two, so this is a floor the crate already meets rather
    /// than an aspiration.
    #[test]
    fn destructive_text_clears_wcag_aa_on_every_built_in_theme() {
        for (name, theme) in built_in_themes() {
            let ratio = contrast_ratio(theme.destructive_bg(), theme.destructive_fg());
            assert!(
                ratio >= 4.5,
                "{name}: destructive text is {ratio:.2}:1 on its own fill, under WCAG AA"
            );
        }
    }

    /// The rule has to pick the *better* of black and white, not merely one
    /// that happens to pass. A theme whose fill clears 4.5:1 both ways would
    /// satisfy the assertion above with the worse choice.
    #[test]
    fn destructive_text_is_the_better_of_black_and_white() {
        let black = hsla(0.0, 0.0, 0.0, 1.0);
        let white = hsla(0.0, 0.0, 1.0, 1.0);

        for (name, theme) in built_in_themes() {
            let bg = theme.destructive_bg();
            let chosen = contrast_ratio(bg, theme.destructive_fg());
            let best = contrast_ratio(bg, black).max(contrast_ratio(bg, white));
            assert!(
                (chosen - best).abs() < 0.001,
                "{name}: chose {chosen:.2}:1 when {best:.2}:1 was available"
            );
        }
    }

    /// Relative luminance is not HSL lightness, which is the whole reason
    /// `destructive_fg` changed. Saturated red and saturated cyan share an
    /// `l` of 0.5 and are nowhere near each other in luminance.
    #[test]
    fn luminance_and_hsl_lightness_are_different_quantities() {
        let red = hsla(0.0, 1.0, 0.5, 1.0);
        let cyan = hsla(180.0 / 360.0, 1.0, 0.5, 1.0);

        assert_eq!(red.l, cyan.l, "the premise: equal HSL lightness");
        assert!(
            relative_luminance(cyan) > relative_luminance(red) * 3.0,
            "cyan {:.3} is not far brighter than red {:.3}",
            relative_luminance(cyan),
            relative_luminance(red),
        );
    }

    /// has a default impl, so this is the wire between `Theme::controls` and
    /// what a control reads at render time.
    #[test]
    fn a_theme_can_rescale_every_control_at_once() {
        let mut theme = Theme::default();
        assert_eq!(
            theme.control(ControlSize::Medium).height.0,
            ControlScale::default().medium.height.0
        );

        theme.controls.medium.height = gpui::Rems(2.0);
        assert_eq!(theme.control(ControlSize::Medium).height.0, 2.0);
        // The other rungs are untouched by one override.
        assert_eq!(
            theme.control(ControlSize::Small).height.0,
            ControlScale::default().small.height.0
        );
    }

    #[test]
    fn test_hex_parsing() {
        let color = parse_hex("#ffffff");
        assert_eq!(color.l, 1.0);

        let color = parse_hex("#000000");
        assert_eq!(color.l, 0.0);
    }
}
