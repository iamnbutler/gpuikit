#![allow(missing_docs)]
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, size, App, AppContext, Application, Bounds, ClipboardItem, Context, Entity,
    FocusHandle, FontWeight, Hsla, InteractiveElement, IntoElement, Menu, ParentElement, Render,
    Rgba, SharedString, StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds,
    WindowOptions,
};
use gpui_platform;
use gpuikit::a11y::FocusNavigation;
use gpuikit::input::InputState;
use gpuikit::markdown::{preprocessing_available, Markdown, MarkdownElement};
use gpuikit::theme::{ActiveTheme, GlobalTheme, Theme, Themeable};
use gpuikit::{
    elements::{
        accordion::{accordion, accordion_item, AccordionState},
        alert::alert,
        aspect_ratio::{aspect_ratio, aspect_ratio_square, aspect_ratio_video},
        avatar::avatar,
        badge::badge,
        breadcrumb::{breadcrumb, breadcrumb_item, BreadcrumbSeparator},
        button::button,
        button_group::button_group,
        card::card,
        checkbox::{checkbox, Checkbox},
        collapsible::{collapsible, Collapsible},
        context_menu::{context_menu, menu_item},
        dialog::{dialog, DialogState},
        empty::empty,
        field::{field, LabelPosition},
        form::fieldset,
        icon_button::icon_button,
        kbd::{kbd, kbd_combo},
        label::label,
        list::{List, ListEntry},
        loading_indicator::loading_indicator,
        popover::{popover, PopoverState},
        progress::{progress, ProgressVariant},
        radio_group::{radio_group, radio_option, RadioGroup},
        scroll_area::scroll_area,
        select::{select, SelectState},
        separator::separator,
        sidebar::{sidebar, sidebar_trigger, SidebarEdge, SidebarState},
        slider::{slider, Slider},
        splitter::splitter,
        switch::{switch, Switch},
        table::{table, CellAlign, Column, Row, SortDescriptor, SortDirection},
        tabs::{tab, tabs, Tabs},
        text_field::{text_field, Adornment},
        textarea::textarea,
        toast::ToastExt,
        toggle::{toggle, Toggle},
        toggle_group::{toggle_group, toggle_option, ToggleGroup, ToggleGroupMode},
        tooltip::tooltip,
        typography::{blockquote, h1, h2, h3, h4, lead, p, small, text},
    },
    layout::{h_stack, v_stack},
    theme::ControlSize,
    traits::control_sized::ControlSized,
    traits::disableable::Disableable,
    traits::labelable::Labelable,
    traits::orientable::Orientable,
    DefaultIcons,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

/// The Markdown page's document. It doubles as a regression surface: every
/// shape that has broken here recently is in it, so a renderer regression is
/// visible to anyone who opens the showcase rather than only to `cargo test`.
const SAMPLE_MARKDOWN: &str = r#"# Markdown Showcase

This is a **bold** statement and this is *italic*.

## Features

- Bullet lists
- **Bold** and *italic* text
- `inline code`

### Nested Lists

- A parent item keeps its own row…
    - …and a nested item is indented under it
        - three levels deep
- Ordered lists nested in bullets renumber from one:
    1. first
    2. second

### Loose Lists

A list whose items are separated by a blank line is *loose* — CommonMark wraps
each item's content in a paragraph, and it still has to render as a list:

- The first loose item

- The second one, which keeps its marker

  A second block of the same item lines up under the first, and draws no
  second marker.

### Code Blocks

```rust
fn main() {
    println!("Hello, GPUI!");
}
```

### Blockquotes

> This is a blockquote.
> It can span multiple lines.

### Links & More

Visit [GPUI](https://zed.dev) for more info.

---

1. Numbered lists
2. Work too
3. Like this

| Column 1 | Column 2 |
|----------|----------|
| Cell A   | Cell B   |
| Cell C   | Cell D   |
"#;

/// The reply the Markdown page streams, a few characters at a time — the
/// shape an LLM answer has. `examples/markdown_streaming.rs` goes further.
const STREAMED_REPLY: &str = "\
## A streamed reply

Every delta goes in through `Markdown::append`, which extends the source and
re-parses **off the UI thread**. The previous parse keeps rendering until the
new one lands, so the document never blanks.

- Deltas arriving during a parse coalesce into one follow-up parse
- Build with `--features stitch` to close syntax a half-written document
  leaves open, so `**bold` does not flash as literal asterisks

```rust
fn main() {
    let greeting = \"Hello, GPUI!\";
    for word in greeting.split(' ') {
        println!(\"{word}\");
    }
}
```

The fence above stays plain monospace while it is still arriving and gains its
colors the moment it closes: a growing block misses the highlight cache on
every delta.
";

/// Characters per delta, and the gap between them.
const STREAM_CHUNK: usize = 3;
const STREAM_INTERVAL: Duration = Duration::from_millis(24);

/// The buffer the Editor page shows. Only built with the `editor` feature —
/// without it the page renders a placeholder instead.
#[cfg(feature = "editor")]
const EDITOR_SAMPLE: &str = r#"// The editor renders a gutter, line numbers and an active line.
fn main() {
    let greeting = "Hello, GPUI!";
    for word in greeting.split(' ') {
        println!("{word}");
    }
}
"#;

/// Every module in `src/elements/`, and the nav page that shows it.
///
/// Rendered by the Coverage page, so this is live code rather than a constant
/// only a test reads — the list is in front of anyone who opens the showcase.
/// Two tests in `src/elements.rs` cross-check it against the crate: every
/// element module needs a row here, and every page named here has to be one
/// the nav can actually reach. An element that genuinely should not have a
/// page is spelled `("name", "none: <reason>")`.
const ELEMENT_COVERAGE: &[(&str, &str)] = &[
    ("accordion", "collapsible"),
    ("alert", "alert"),
    ("aspect_ratio", "aspect-ratio"),
    ("avatar", "avatar"),
    ("badge", "badge"),
    ("breadcrumb", "breadcrumb"),
    ("button", "button"),
    ("button_group", "button"),
    ("card", "card"),
    ("checkbox", "toggle"),
    ("collapsible", "collapsible"),
    ("context_menu", "context-menu"),
    ("dialog", "dialog"),
    ("empty", "empty"),
    ("field", "text"),
    ("form", "form"),
    ("icon_button", "button"),
    ("input", "text"),
    ("kbd", "badge"),
    ("label", "badge"),
    ("list", "list"),
    ("loading_indicator", "loading"),
    ("popover", "popover"),
    ("progress", "loading"),
    ("radio_group", "selection"),
    ("scroll_area", "scroll-area"),
    ("select", "select"),
    ("separator", "separator"),
    ("sidebar", "sidebar"),
    ("slider", "slider"),
    ("splitter", "splitter"),
    ("switch", "toggle"),
    ("table", "table"),
    ("tabs", "tabs"),
    ("text_field", "text"),
    ("textarea", "text"),
    ("toast", "toast"),
    ("toggle", "toggle"),
    ("toggle_group", "selection"),
    ("tooltip", "tooltip"),
    ("typography", "typography"),
];

/// The sidebar, as data. Every entry is `(page id, label)`, and the page id has
/// to match an arm of `Showcase::render`'s match — `ELEMENT_COVERAGE` and the
/// tests in `src/elements.rs` are checked against those arms, not against this.
///
/// A `const` rather than a `vec!` rebuilt inside `render`: the rows it produces
/// are built once, in `Showcase::new`, because `render` runs on every frame and
/// the sidebar does not change between them.
const NAV_SECTIONS: &[NavSection] = &[
    (
        "Foundations",
        DefaultIcons::ruler_square,
        &[("control-sizes", "Control Sizes")],
    ),
    (
        "Input",
        DefaultIcons::input,
        &[
            ("button", "Button"),
            ("toggle", "Toggle"),
            ("selection", "Selection"),
            ("select", "Select"),
            ("slider", "Slider"),
            ("text", "Text"),
            ("form", "Form"),
            ("tabs", "Tabs"),
        ],
    ),
    (
        "Display",
        DefaultIcons::eye_open,
        &[
            ("avatar", "Avatar"),
            ("badge", "Badge"),
            ("typography", "Typography"),
            ("loading", "Loading"),
            ("alert", "Alert"),
            ("tooltip", "Tooltip"),
            ("card", "Card"),
            ("aspect-ratio", "Aspect Ratio"),
            ("empty", "Empty"),
        ],
    ),
    (
        "Layout",
        DefaultIcons::layout,
        &[
            ("breadcrumb", "Breadcrumb"),
            ("separator", "Separator"),
            ("sidebar", "Sidebar"),
            ("splitter", "Splitter"),
            ("collapsible", "Collapsible"),
            ("scroll-area", "Scroll Area"),
            ("list", "List"),
        ],
    ),
    (
        "Overlay",
        DefaultIcons::stack,
        &[
            ("popover", "Popover"),
            ("dialog", "Dialog"),
            ("context-menu", "Context Menu"),
            ("toast", "Toast"),
        ],
    ),
    ("Data", DefaultIcons::table, &[("table", "Table")]),
    (
        "Content",
        DefaultIcons::file_text,
        &[("markdown", "Markdown"), ("editor", "Editor")],
    ),
    (
        "System",
        DefaultIcons::gear,
        &[("theme", "Theme"), ("coverage", "Coverage")],
    ),
];

/// One nav section: its label, the glyph its rail row draws when the sidebar
/// is collapsed, and its pages.
type NavSection = (
    &'static str,
    fn() -> gpui::Svg,
    &'static [(&'static str, &'static str)],
);

/// Which section a page belongs to, so the collapsed rail can highlight the
/// one the current page is in.
fn section_of(page: &str) -> Option<&'static str> {
    NAV_SECTIONS
        .iter()
        .find_map(|(label, _, items)| items.iter().any(|(id, _)| *id == page).then_some(*label))
}

/// A `Select`'s value as the page prints it. Every select holds an
/// `Option<T>`, including the ones built with `.selected(…)`, so "nothing
/// chosen" is a state the page has to be able to say out loud.
fn described<T: std::fmt::Debug>(value: Option<&T>) -> String {
    value.map_or_else(|| "None".to_string(), |value| format!("{value:?}"))
}

/// A prebuilt sidebar row, and the page it selects — `None` for a section
/// header, which selects nothing.
///
/// `ListEntry` is `Rc`-backed, so cloning one per frame copies two pointers.
/// Building one costs a `format!`, a couple of `SharedString`s and two boxed
/// closures, which is why they are not rebuilt per frame.
struct NavEntry {
    page: Option<SharedString>,
    entry: ListEntry,
}

/// Build the sidebar's rows once. Clicking a row writes its page id into
/// `active_page`, which is the same cell `render` reads.
fn nav_entries(active_page: &Rc<RefCell<SharedString>>) -> Vec<NavEntry> {
    let mut entries = Vec::new();

    for (section_label, _icon, items) in NAV_SECTIONS {
        entries.push(NavEntry {
            page: None,
            entry: ListEntry::header(*section_label),
        });

        for (id, label) in *items {
            let page = SharedString::from(*id);
            let label = SharedString::from(*label);
            let target = page.clone();
            let cell = active_page.clone();

            entries.push(NavEntry {
                page: Some(page),
                entry: ListEntry::item(
                    SharedString::from(format!("nav-{id}")),
                    move |_window, _cx| div().px_2().child(label.clone()).into_any_element(),
                )
                .on_click(move |_, window, _cx| {
                    *cell.borrow_mut() = target.clone();
                    window.refresh();
                }),
            });
        }
    }

    entries
}

#[derive(Clone, PartialEq, Debug)]
enum Size {
    Small,
    Medium,
    Large,
}

#[derive(Clone, PartialEq, Debug)]
enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, PartialEq, Debug)]
enum NotificationPreference {
    All,
    Important,
    None,
}

#[derive(Clone, PartialEq, Debug)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, PartialEq, Debug)]
enum TextStyle {
    Bold,
    Italic,
    Underline,
}

#[derive(Clone, PartialEq, Debug)]
enum ThemeChoice {
    GruvboxDark,
    GruvboxLight,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
}

#[derive(Clone, PartialEq, Debug)]
enum Country {
    US,
    UK,
    CA,
    DE,
    FR,
}

/// The Table page's data. A plain `const` the page owns: the element is handed
/// rows that are already filtered and already sorted, so the data has to live
/// somewhere the page can re-derive it from.
#[derive(Clone, Copy)]
struct Repo {
    id: u32,
    name: &'static str,
    language: &'static str,
    stars: u32,
    status: RepoStatus,
}

#[derive(Clone, Copy, PartialEq)]
enum RepoStatus {
    Active,
    Archived,
    Draft,
}

impl RepoStatus {
    fn label(self) -> &'static str {
        match self {
            RepoStatus::Active => "Active",
            RepoStatus::Archived => "Archived",
            RepoStatus::Draft => "Draft",
        }
    }
}

const REPOSITORIES: &[Repo] = &[
    Repo {
        id: 1,
        name: "gpui",
        language: "Rust",
        stars: 8420,
        status: RepoStatus::Active,
    },
    Repo {
        id: 2,
        name: "gpuikit",
        language: "Rust",
        stars: 312,
        status: RepoStatus::Active,
    },
    Repo {
        id: 3,
        name: "taffy",
        language: "Rust",
        stars: 1904,
        status: RepoStatus::Active,
    },
    Repo {
        id: 4,
        name: "accesskit",
        language: "Rust",
        stars: 1210,
        status: RepoStatus::Active,
    },
    Repo {
        id: 5,
        name: "pulldown-cmark",
        language: "Rust",
        stars: 2180,
        status: RepoStatus::Active,
    },
    Repo {
        id: 6,
        name: "syntect",
        language: "Rust",
        stars: 2036,
        status: RepoStatus::Archived,
    },
    Repo {
        id: 7,
        name: "harfbuzz",
        language: "C++",
        stars: 4100,
        status: RepoStatus::Active,
    },
    Repo {
        id: 8,
        name: "swash",
        language: "Rust",
        stars: 640,
        status: RepoStatus::Draft,
    },
    Repo {
        id: 9,
        name: "cosmic-text",
        language: "Rust",
        stars: 1480,
        status: RepoStatus::Active,
    },
    Repo {
        id: 10,
        name: "wgpu",
        language: "Rust",
        stars: 12800,
        status: RepoStatus::Active,
    },
];

/// The columns the Table page sorts by, by index. Restated here because the
/// comparator is the page's job — the element is told *how* the rows are
/// sorted, never *how to* sort them.
const TABLE_COLUMN_REPOSITORY: usize = 0;
const TABLE_COLUMN_LANGUAGE: usize = 1;
const TABLE_COLUMN_STARS: usize = 2;

struct Showcase {
    focus_handle: FocusHandle,
    active_page: Rc<RefCell<SharedString>>,
    /// The sidebar, built once. `render` clones these and stamps `selected` on
    /// them rather than rebuilding 24 rows per frame.
    nav: Vec<NavEntry>,
    /// The showcase's own navigation panel. The state lives on the app rather
    /// than in the component — that is the point of the design.
    nav_collapsed: bool,
    /// The demo page's own panel, independent of the one on the left.
    demo_collapsed: bool,
    demo_edge: SidebarEdge,
    /// In rems, which is what `Sidebar::width` takes.
    demo_width: f32,
    /// Forces the demo panel to draw as a drawer whatever the window width is,
    /// so the transition can be seen without resizing.
    demo_overlay: bool,
    /// The three splitter demos' ratios. They live here rather than in the
    /// element on purpose — that is the whole design, and the Reset button on
    /// the page is what makes it visible.
    split_side_by_side: f32,
    split_stacked: f32,
    split_rungs: [f32; 3],
    click_count: usize,
    toggled_count: usize,
    size_select: Entity<SelectState<Size>>,
    priority_select: Entity<SelectState<Priority>>,
    theme_select: Entity<SelectState<ThemeChoice>>,
    country_select: Entity<SelectState<Country>>,
    /// Retained, not minted per frame: selection state lives on the entity,
    /// so `markdown()` — which creates a fresh one per call — cannot hold it.
    markdown: Entity<Markdown>,
    /// The document the Streaming section feeds with `append`.
    markdown_stream: Entity<Markdown>,
    /// Bumped on each restart, so a stream still running does not feed a
    /// document that has already been reset.
    stream_generation: usize,
    markdown_copy_status: SharedString,
    slider_volume: Entity<Slider>,
    slider_brightness: Entity<Slider>,
    slider_disabled: Entity<Slider>,
    toggle_bold: Entity<Toggle>,
    toggle_pinned: Entity<Toggle>,
    toggle_disabled: Entity<Toggle>,
    checkbox_agree: Entity<Checkbox>,
    /// The form page's three checkboxes. Two of them are inside a fieldset
    /// disabled at the group and say nothing about `disabled` themselves.
    checkbox_form_consent: Entity<Checkbox>,
    checkbox_form_locked: Entity<Checkbox>,
    checkbox_form_updates: Entity<Checkbox>,
    checkbox_newsletter: Entity<Checkbox>,
    radio_notifications: Entity<RadioGroup<NotificationPreference>>,
    switch_wifi: Entity<Switch>,
    switch_bluetooth: Entity<Switch>,
    switch_airplane: Entity<Switch>,
    collapsible_basic: Entity<Collapsible>,
    collapsible_nested: Entity<Collapsible>,
    accordion: Entity<AccordionState>,
    toggle_group_alignment: Entity<ToggleGroup<Alignment>>,
    toggle_group_text_style: Entity<ToggleGroup<TextStyle>>,
    tabs_example: Entity<Tabs>,
    text_field_plain: Entity<InputState>,
    text_field_icon: Entity<InputState>,
    text_field_affixes: Entity<InputState>,
    text_field_action: Entity<InputState>,
    text_field_composed: Entity<InputState>,
    text_field_disabled: Entity<InputState>,
    text_field_read_only: Entity<InputState>,
    /// One of each stateful control per rung, for the Control Sizes page.
    /// Indexed by `ControlSize::ALL`.
    control_row_checkboxes: [Entity<Checkbox>; 3],
    control_row_switches: [Entity<Switch>; 3],
    control_row_toggles: [Entity<Toggle>; 3],
    control_row_selects: [Entity<SelectState<Size>>; 3],
    control_row_fields: [Entity<InputState>; 3],
    textarea_example: Entity<InputState>,
    /// Its own state: sharing the live example's was both a duplicate element
    /// id and, now that `read_only` writes through to the state, a clobber
    /// hazard.
    textarea_disabled: Entity<InputState>,
    textarea_read_only: Entity<InputState>,
    popover_example: Entity<PopoverState>,
    dialog_example: Entity<DialogState>,
    /// The destructive confirmation: same element, confirm mode.
    destructive_dialog: Entity<DialogState>,
    context_menu_pinned: bool,
    context_menu_status: SharedString,
    /// Whether the Loading page's indicators advance. Pausing them takes the
    /// shared loading clock out of the picture without leaving the page.
    loading_playing: bool,
    /// The Table page's data-view state, all three pieces of it. This is the
    /// division of labour the element exists to demonstrate: the filter, the
    /// sort and the selection are the page's, and the table is handed the
    /// result plus a description of it.
    table_filter: Entity<InputState>,
    table_sort: SortDescriptor,
    table_selected: HashSet<u32>,
    table_status: SharedString,
}

impl Showcase {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let size_select = cx.new(|_cx| {
            SelectState::new(
                select(
                    "size-select",
                    "Size",
                    vec![
                        (Size::Small, "Small"),
                        (Size::Medium, "Medium"),
                        (Size::Large, "Large"),
                    ],
                )
                .selected(Size::Medium),
            )
        });

        let priority_select = cx.new(|_cx| {
            SelectState::new(
                select(
                    "priority-select",
                    "Priority",
                    vec![
                        (Priority::Low, "Low"),
                        (Priority::Normal, "Normal"),
                        (Priority::High, "High"),
                        (Priority::Critical, "Critical"),
                    ],
                )
                .selected(Priority::Normal),
            )
        });

        let theme_select = cx.new(|_cx| {
            SelectState::new(
                select(
                    "theme-select",
                    "Theme",
                    vec![
                        (ThemeChoice::GruvboxDark, "Gruvbox Dark"),
                        (ThemeChoice::GruvboxLight, "Gruvbox Light"),
                        (ThemeChoice::CatppuccinLatte, "Catppuccin Latte"),
                        (ThemeChoice::CatppuccinFrappe, "Catppuccin Frappé"),
                        (ThemeChoice::CatppuccinMacchiato, "Catppuccin Macchiato"),
                        (ThemeChoice::CatppuccinMocha, "Catppuccin Mocha"),
                    ],
                )
                .selected(ThemeChoice::GruvboxDark)
                .full_width(true)
                .on_change(|choice, window, cx| {
                    let theme = match choice {
                        ThemeChoice::GruvboxDark => Theme::gruvbox_dark(),
                        ThemeChoice::GruvboxLight => Theme::gruvbox_light(),
                        ThemeChoice::CatppuccinLatte => Theme::catppuccin_latte(),
                        ThemeChoice::CatppuccinFrappe => Theme::catppuccin_frappe(),
                        ThemeChoice::CatppuccinMacchiato => Theme::catppuccin_macchiato(),
                        ThemeChoice::CatppuccinMocha => Theme::catppuccin_mocha(),
                    };
                    cx.set_global(GlobalTheme(Arc::new(theme)));
                    window.refresh();
                }),
            )
        });

        let country_select = cx.new(|_cx| {
            SelectState::new(
                select(
                    "country-select",
                    "Country",
                    vec![
                        (Country::US, "United States"),
                        (Country::UK, "United Kingdom"),
                        (Country::CA, "Canada"),
                        (Country::DE, "Germany"),
                        (Country::FR, "France"),
                    ],
                )
                .placeholder("Choose a country..."),
            )
        });

        let markdown = cx.new(|cx| Markdown::new(SAMPLE_MARKDOWN, cx));
        let markdown_stream = cx.new(|cx| Markdown::new("", cx));

        let slider_volume = cx.new(|_cx| {
            slider("volume-slider", 0.6, 0.0..=1.0)
                .label("Volume")
                .step(0.05)
        });
        let slider_brightness = cx.new(|_cx| {
            slider("brightness-slider", 40.0, 0.0..=100.0)
                .label("Brightness")
                .step(1.0)
        });
        let slider_disabled = cx.new(|_cx| {
            slider("disabled-slider", 0.25, 0.0..=1.0)
                .label("Disabled")
                .disabled(true)
        });

        let toggle_bold = cx.new(|_cx| toggle("toggle-bold", true).label("Bold"));
        let toggle_pinned = cx.new(|_cx| toggle("toggle-pinned", false).label("Pinned"));
        let toggle_disabled = cx.new(|_cx| {
            toggle("toggle-disabled", false)
                .label("Disabled")
                .disabled(true)
        });

        let checkbox_form_consent = cx.new(|_cx| checkbox("form-consent", false));
        let checkbox_form_locked = cx.new(|_cx| checkbox("form-locked-consent", true));
        let checkbox_form_updates = cx.new(|_cx| checkbox("form-locked-updates", false));

        let checkbox_agree =
            cx.new(|_cx| checkbox("agree-terms", false).label("I agree to the terms"));
        let checkbox_newsletter =
            cx.new(|_cx| checkbox("newsletter", true).label("Subscribe to newsletter"));

        let radio_notifications = cx.new(|_cx| {
            radio_group(
                "notifications",
                vec![
                    radio_option(NotificationPreference::All, "All notifications"),
                    radio_option(NotificationPreference::Important, "Important only"),
                    radio_option(NotificationPreference::None, "None"),
                ],
            )
            .selected(NotificationPreference::Important)
        });

        let switch_wifi = cx.new(|_cx| switch("wifi-switch", true).label("Wi-Fi"));
        let switch_bluetooth = cx.new(|_cx| switch("bluetooth-switch", false).label("Bluetooth"));
        let switch_airplane = cx.new(|_cx| {
            switch("airplane-switch", false)
                .label("Airplane Mode")
                .disabled(true)
        });

        let collapsible_basic = cx.new(|_cx| {
            collapsible("collapsible-basic")
                .trigger_label("Click to expand")
                .content(|_window, _cx| {
                    div()
                        .text_sm()
                        .child(
                            "This is the collapsible content. It can contain any elements you want.",
                        )
                        .into_any_element()
                })
                .default_open(false)
        });

        let collapsible_nested = cx.new(|_cx| {
            collapsible("collapsible-nested")
                .trigger_label("Settings")
                .content(|_window, _cx| {
                    v_stack()
                        .gap_2()
                        .child(div().text_sm().child("Configure your preferences below:"))
                        .child(
                            h_stack()
                                .gap_2()
                                .child(badge("Option 1"))
                                .child(badge("Option 2"))
                                .child(badge("Option 3")),
                        )
                        .into_any_element()
                })
                .default_open(true)
        });

        let accordion = cx.new(|_cx| {
            AccordionState::new(
                accordion("showcase-accordion")
                    .item(
                        accordion_item("getting-started", "Getting Started")
                            .content("Welcome to GPUIKit! This library provides a comprehensive set of UI components for building GPUI applications."),
                    )
                    .item(
                        accordion_item("installation", "Installation")
                            .content("Add gpuikit to your Cargo.toml and call gpuikit::init(cx) in your application."),
                    )
                    .item(
                        accordion_item("theming", "Theming")
                            .content("GPUIKit supports theming through the theme module. You can customize colors, fonts, and spacing."),
                    )
                    .item(
                        accordion_item("disabled-section", "Disabled Section")
                            .content("This section is disabled.")
                            .disabled(true),
                    )
                    .default_expanded("getting-started"),
            )
        });

        let toggle_group_alignment = cx.new(|_cx| {
            toggle_group(
                "alignment",
                vec![
                    toggle_option(Alignment::Left, "Left"),
                    toggle_option(Alignment::Center, "Center"),
                    toggle_option(Alignment::Right, "Right"),
                ],
            )
            .selected_value(Alignment::Center)
        });

        let toggle_group_text_style = cx.new(|_cx| {
            toggle_group(
                "text-style",
                vec![
                    toggle_option(TextStyle::Bold, "B"),
                    toggle_option(TextStyle::Italic, "I"),
                    toggle_option(TextStyle::Underline, "U"),
                ],
            )
            .mode(ToggleGroupMode::Multiple)
            .selected(vec![TextStyle::Bold])
        });

        let tabs_example = cx.new(|_cx| {
            tabs("example-tabs")
                .tab(tab("home", "Home"))
                .tab(tab("profile", "Profile"))
                .tab(tab("settings", "Settings"))
                .tab(tab("disabled", "Disabled").disabled(true))
        });

        let text_field_plain = cx.new(|cx| InputState::new_singleline(cx));
        let text_field_icon = cx.new(|cx| InputState::new_singleline(cx));
        let text_field_affixes = cx.new(|cx| InputState::new_singleline(cx));
        let text_field_action = cx.new(|cx| InputState::new_singleline(cx));
        let text_field_composed = cx.new(|cx| InputState::new_singleline(cx));
        let text_field_disabled = cx.new(|cx| InputState::new_singleline(cx));
        let text_field_read_only = cx.new(|cx| {
            let mut state = InputState::new_singleline(cx);
            state.set_content("gpuikit-0.8.0", cx);
            state
        });
        // One of each stateful control per rung. Built here rather than in
        // `render` because `render` runs every frame.
        let control_row_checkboxes = ControlSize::ALL.map(|size| {
            cx.new(|_cx| {
                checkbox(
                    SharedString::from(format!("control-row-checkbox-{}", size.name())),
                    true,
                )
                .control_size(size)
            })
        });
        let control_row_switches = ControlSize::ALL.map(|size| {
            cx.new(|_cx| {
                switch(
                    SharedString::from(format!("control-row-switch-{}", size.name())),
                    true,
                )
                .control_size(size)
            })
        });
        let control_row_toggles = ControlSize::ALL.map(|size| {
            cx.new(|_cx| {
                toggle(
                    SharedString::from(format!("control-row-toggle-{}", size.name())),
                    true,
                )
                .control_size(size)
            })
        });
        let control_row_selects = ControlSize::ALL.map(|size| {
            cx.new(|_cx| {
                SelectState::new(
                    select(
                        SharedString::from(format!("control-row-select-{}", size.name())),
                        "Size",
                        vec![(Size::Small, "Small"), (Size::Medium, "Medium")],
                    )
                    .selected(Size::Medium)
                    .control_size(size),
                )
            })
        });
        let control_row_fields =
            ControlSize::ALL.map(|_| cx.new(|cx| InputState::new_singleline(cx)));
        let textarea_example = cx.new(|cx| InputState::new_multiline(cx));
        let textarea_disabled = cx.new(|cx| InputState::new_multiline(cx));
        let textarea_read_only = cx.new(|cx| {
            let mut state = InputState::new_multiline(cx);
            state.set_content(
                "This one is read-only: select it, copy it, scroll it — but you \
                 cannot change it.",
                cx,
            );
            state
        });

        let popover_example = cx.new(|_cx| {
            PopoverState::new(
                popover("showcase-popover")
                    .trigger(|_window, _cx| {
                        button("popover-trigger", "Open Popover").into_any_element()
                    })
                    .content(|_window, cx| {
                        let theme = cx.theme();
                        v_stack()
                            .p_3()
                            .gap_2()
                            .w(px(200.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Popover Content"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Click outside or press Escape to close."),
                            )
                            .into_any_element()
                    }),
            )
        });

        let dialog_example = cx.new(|_cx| {
            DialogState::new(
                dialog("showcase-dialog")
                    .title("Confirm Action")
                    .description("Are you sure you want to proceed? This action cannot be undone.")
                    .footer(|_window, _cx| {
                        h_stack()
                            .gap_2()
                            .justify_end()
                            .child(button("dialog-cancel", "Cancel"))
                            .child(button("dialog-confirm", "Confirm"))
                            .into_any_element()
                    }),
            )
        });

        let destructive_dialog = cx.new(|_cx| {
            DialogState::new(
                dialog("showcase-destructive-dialog")
                    .confirm(
                        "Delete this project?",
                        "Its 42 tasks are deleted with it. This cannot be undone.",
                    )
                    // Name the verb, not "Confirm".
                    .confirm_label("Delete")
                    .on_confirm(|_window, _cx| {
                        log::info!("the destructive confirmation was confirmed");
                    }),
            )
        });

        let table_filter = cx.new(InputState::new_singleline);
        // Typing in the filter has to re-derive the page's rows, and the rows
        // are derived in `render`, so the page has to hear about the keystroke.
        cx.observe(&table_filter, |_this, _filter, cx| cx.notify())
            .detach();

        let active_page = Rc::new(RefCell::new(SharedString::from("button")));
        let nav = nav_entries(&active_page);

        Self {
            focus_handle: cx.focus_handle(),
            active_page,
            nav,
            nav_collapsed: false,
            demo_collapsed: false,
            demo_edge: SidebarEdge::Left,
            demo_width: 13.75,
            demo_overlay: false,
            split_side_by_side: 0.4,
            split_stacked: 0.35,
            split_rungs: [0.5; 3],
            click_count: 0,
            toggled_count: 0,
            size_select,
            priority_select,
            theme_select,
            country_select,
            markdown,
            markdown_stream,
            stream_generation: 0,
            markdown_copy_status: "Nothing copied yet.".into(),
            slider_volume,
            slider_brightness,
            slider_disabled,
            toggle_bold,
            toggle_pinned,
            toggle_disabled,
            checkbox_agree,
            checkbox_form_consent,
            checkbox_form_locked,
            checkbox_form_updates,
            checkbox_newsletter,
            radio_notifications,
            switch_wifi,
            switch_bluetooth,
            switch_airplane,
            collapsible_basic,
            collapsible_nested,
            accordion,
            toggle_group_alignment,
            toggle_group_text_style,
            tabs_example,
            text_field_plain,
            text_field_icon,
            text_field_affixes,
            text_field_action,
            text_field_composed,
            text_field_disabled,
            text_field_read_only,
            control_row_checkboxes,
            control_row_switches,
            control_row_toggles,
            control_row_selects,
            control_row_fields,
            textarea_example,
            textarea_disabled,
            textarea_read_only,
            popover_example,
            dialog_example,
            destructive_dialog,
            context_menu_pinned: true,
            context_menu_status: "No action chosen yet.".into(),
            loading_playing: true,
            table_filter,
            table_sort: SortDescriptor::new(TABLE_COLUMN_STARS, SortDirection::Descending),
            table_selected: HashSet::new(),
            table_status: "No repository opened yet.".into(),
        }
    }

    fn render_button_page(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Button"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .child(button("click-me", "Click Me").on_click(cx.listener(
                        |showcase, _event, _window, cx| {
                            showcase.click_count += 1;
                            cx.notify();
                        },
                    )))
                    .child(button("disabled-btn", "Disabled Button").disabled(true))
                    .child(button("reset-btn", "Reset Counter").on_click(cx.listener(
                        |showcase, _event, _window, cx| {
                            showcase.click_count = 0;
                            cx.notify();
                        },
                    ))),
            )
            .child(
                h_stack()
                    .items_center()
                    .gap_2()
                    .mt_2()
                    .child("Click count:")
                    .child(
                        div()
                            .text_color(theme.accent())
                            .font_weight(FontWeight::BOLD)
                            .child(format!("{}", self.click_count)),
                    ),
            )
    }

    fn render_button_group_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("ButtonGroup"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .child(
                        button_group("btn-group-1")
                            .child(button("group-1-a", "Left"))
                            .child(button("group-1-b", "Center"))
                            .child(button("group-1-c", "Right")),
                    )
                    .child(
                        button_group("btn-group-2")
                            .vertical()
                            .child(button("group-2-a", "Top"))
                            .child(button("group-2-b", "Middle"))
                            .child(button("group-2-c", "Bottom")),
                    ),
            )
            .child(
                h_stack().gap_2().items_center().mt_2().child(
                    div()
                        .text_color(theme.fg_muted())
                        .child("(horizontal / vertical)"),
                ),
            )
    }

    fn render_icon_button_page(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Icon Button"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(icon_button("icon-star", DefaultIcons::star()))
                    .child(icon_button("icon-heart", DefaultIcons::heart()))
                    .child(icon_button("icon-gear", DefaultIcons::gear()))
                    .child(icon_button("icon-bell", DefaultIcons::bell()))
                    .child(icon_button("icon-home", DefaultIcons::home()))
                    .child(icon_button("icon-search", DefaultIcons::magnifying_glass()))
                    .child(icon_button("icon-plus", DefaultIcons::plus()))
                    .child(icon_button("icon-trash", DefaultIcons::trash())),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(
                        icon_button("icon-selected", DefaultIcons::check_circled()).selected(true),
                    )
                    .child(icon_button("icon-disabled", DefaultIcons::lock_closed()).disabled(true))
                    .child(
                        div()
                            .text_color(theme.fg_muted())
                            .child("(selected / disabled)"),
                    ),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(
                        icon_button("toggle-star", DefaultIcons::star())
                            .use_state()
                            .on_toggle(cx.listener(|showcase, toggled, _window, cx| {
                                if *toggled {
                                    showcase.toggled_count += 1;
                                } else {
                                    showcase.toggled_count =
                                        showcase.toggled_count.saturating_sub(1);
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        icon_button("toggle-heart", DefaultIcons::heart())
                            .use_state()
                            .on_toggle(cx.listener(|showcase, toggled, _window, cx| {
                                if *toggled {
                                    showcase.toggled_count += 1;
                                } else {
                                    showcase.toggled_count =
                                        showcase.toggled_count.saturating_sub(1);
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        icon_button("toggle-bell", DefaultIcons::bell())
                            .use_state()
                            .on_toggle(cx.listener(|showcase, toggled, _window, cx| {
                                if *toggled {
                                    showcase.toggled_count += 1;
                                } else {
                                    showcase.toggled_count =
                                        showcase.toggled_count.saturating_sub(1);
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .text_color(theme.fg_muted())
                            .child("Toggled:")
                            .child(
                                div()
                                    .text_color(theme.accent())
                                    .font_weight(FontWeight::BOLD)
                                    .child(format!("{}", self.toggled_count)),
                            ),
                    ),
            )
    }

    fn render_checkbox_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Checkbox"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(self.checkbox_agree.clone())
                    .child(self.checkbox_newsletter.clone()),
            )
    }

    fn render_switch_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Switch"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(self.switch_wifi.clone())
                    .child(self.switch_bluetooth.clone())
                    .child(self.switch_airplane.clone()),
            )
    }

    fn render_toggle_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Toggle"),
            )
            .child(
                div().text_sm().text_color(theme.fg_muted()).child(
                    "A button that stays pressed — distinct from Switch, which is a setting.",
                ),
            )
            .child(
                h_stack()
                    .gap_2()
                    .child(self.toggle_bold.clone())
                    .child(self.toggle_pinned.clone())
                    .child(self.toggle_disabled.clone()),
            )
    }

    fn render_radio_group_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("RadioGroup"),
            )
            .child(self.radio_notifications.clone())
    }

    fn render_toggle_group_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("ToggleGroup"),
            )
            .child(
                v_stack()
                    .gap_3()
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .w_20()
                                    .child("Single:"),
                            )
                            .child(self.toggle_group_alignment.clone()),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .w_20()
                                    .child("Multiple:"),
                            )
                            .child(self.toggle_group_text_style.clone()),
                    ),
            )
    }

    fn render_slider_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Slider"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .child("Drag the handle, or click anywhere on the track."),
            )
            .child(
                v_stack()
                    .gap_4()
                    .max_w(px(360.))
                    .child(self.slider_volume.clone())
                    .child(self.slider_brightness.clone())
                    .child(self.slider_disabled.clone()),
            )
    }

    fn render_tabs_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Tabs"),
            )
            .child(self.tabs_example.clone())
    }

    /// One page for one chooser. `Dropdown` was `Select` under a second name
    /// and is gone; what is left is the two *shapes* a select comes in, which
    /// is what the second name was really about. A select built with
    /// `.selected(…)` always has a value — that is the old `Dropdown` — and one
    /// built without shows its placeholder until something is chosen, and can
    /// be put back into that state with `clear()`.
    fn render_select_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let country_select = self.country_select.clone();

        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Select"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .child("With a value, from `.selected(…)`"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_start()
                    .child(
                        v_stack()
                            .gap_1()
                            .child(div().text_xs().text_color(theme.fg_muted()).child("Size"))
                            .child(self.size_select.clone()),
                    )
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Priority"),
                            )
                            .child(self.priority_select.clone()),
                    ),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .mt_2()
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .text_color(theme.fg_muted())
                            .child("Selected size:")
                            .child(
                                div()
                                    .text_color(theme.accent())
                                    .font_weight(FontWeight::BOLD)
                                    .child(described(self.size_select.read(cx).selected.as_ref())),
                            ),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .text_color(theme.fg_muted())
                            .child("Selected priority:")
                            .child(
                                div()
                                    .text_color(theme.accent())
                                    .font_weight(FontWeight::BOLD)
                                    .child(described(
                                        self.priority_select.read(cx).selected.as_ref(),
                                    )),
                            ),
                    ),
            )
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .child("Without one, showing its placeholder"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_end()
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Country"),
                            )
                            .child(self.country_select.clone()),
                    )
                    .child(
                        button("select-clear", "Clear").on_click(move |_, _window, cx| {
                            country_select.update(cx, |state, cx| state.clear(cx));
                        }),
                    ),
            )
            .child(
                h_stack().gap_4().items_start().child(
                    h_stack()
                        .gap_2()
                        .items_center()
                        .text_color(theme.fg_muted())
                        .child("Selected country:")
                        .child(
                            div()
                                .text_color(theme.accent())
                                .font_weight(FontWeight::BOLD)
                                .child(described(self.country_select.read(cx).selected.as_ref())),
                        ),
                ),
            )
    }

    fn render_field_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Field"),
            )
            .child(
                v_stack()
                    .gap_4()
                    .child(
                        field("username")
                            .label("Username")
                            .description("Enter your preferred username")
                            .required(true)
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(theme.border())
                                    .rounded(gpui::px(4.0))
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("(input placeholder)"),
                            ),
                    )
                    .child(
                        field("email")
                            .label("Email")
                            .error("Please enter a valid email address")
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(theme.danger())
                                    .rounded(gpui::px(4.0))
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("invalid@"),
                            ),
                    )
                    .child(
                        field("department")
                            .label("Department")
                            .label_position(LabelPosition::Beside)
                            .description("Select your department")
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(theme.border())
                                    .rounded(gpui::px(4.0))
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("(horizontal layout)"),
                            ),
                    ),
            )
    }

    /// Grouping and label association — the two things #164 asked for, and
    /// nothing about form state.
    ///
    /// The second fieldset is the whole argument: it says `disabled(true)`
    /// once, and neither field nor either checkbox inside it says anything
    /// about `disabled` at all.
    fn render_form_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Form"),
            )
            .child(
                fieldset("form-showcase-billing")
                    .legend("Billing address")
                    .description("A fieldset groups related controls and names the group.")
                    .error("This address could not be verified")
                    .child(
                        field("form-showcase-street")
                            .label("Street")
                            .description("Click the label to focus the control it names")
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(theme.border())
                                    .rounded(gpui::px(4.0))
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("(input placeholder)"),
                            ),
                    )
                    .child(
                        field("form-showcase-consent")
                            .label("Consent")
                            .child(self.checkbox_form_consent.clone()),
                    ),
            )
            .child(
                fieldset("form-showcase-locked")
                    .legend("Disabled at the group")
                    .description("Neither field nor checkbox below says `disabled`.")
                    .disabled(true)
                    .child(
                        field("form-showcase-locked-consent")
                            .label("Consent")
                            .child(self.checkbox_form_locked.clone()),
                    )
                    .child(
                        field("form-showcase-locked-updates")
                            .label("Updates")
                            .child(self.checkbox_form_updates.clone()),
                    ),
            )
    }

    fn render_text_field_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        fn row(
            label: &'static str,
            field: impl IntoElement,
            theme: &std::sync::Arc<gpuikit::theme::Theme>,
        ) -> impl IntoElement {
            h_stack()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .w(gpui::rems(6.0))
                        .text_sm()
                        .text_color(theme.fg_muted())
                        .child(label),
                )
                .child(field)
        }

        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("TextField"),
            )
            .child(
                v_stack()
                    .gap_3()
                    .child(row(
                        "Plain:",
                        text_field(&self.text_field_plain, cx).placeholder("Your name"),
                        theme,
                    ))
                    .child(row(
                        "Icon:",
                        text_field(&self.text_field_icon, cx)
                            .placeholder("Search")
                            .prefix(Adornment::icon(DefaultIcons::magnifying_glass())),
                        theme,
                    ))
                    .child(row(
                        "Affixes:",
                        text_field(&self.text_field_affixes, cx)
                            .placeholder("example")
                            .prefix(Adornment::text("https://"))
                            .suffix(Adornment::text(".com")),
                        theme,
                    ))
                    .child(row(
                        "Inline:",
                        // An action *inside* the field is an adornment. It has
                        // to fit the box, so it takes the same rung.
                        text_field(&self.text_field_action, cx)
                            .placeholder("Type to filter")
                            .suffix(Adornment::element(
                                icon_button("text-field-clear", DefaultIcons::cross_1()).small(),
                            )),
                        theme,
                    ))
                    .child(row(
                        "Beside:",
                        // A button that is its own box beside the field is
                        // composition, not a field feature — which is the
                        // three-boxes-pretending-to-be-one shape InputGroup was.
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(text_field(&self.text_field_composed, cx).placeholder("Query"))
                            .child(button("text-field-go", "Go")),
                        theme,
                    ))
                    .child(row(
                        "Disabled:",
                        // Actually inert: a disabled field renders its value as
                        // static text rather than a dimmed live input.
                        text_field(&self.text_field_disabled, cx)
                            .placeholder("Unavailable")
                            .disabled(true),
                        theme,
                    ))
                    .child(row(
                        "Read-only:",
                        // Still focusable and selectable; every edit path is
                        // refused by `InputState`.
                        text_field(&self.text_field_read_only, cx).read_only(true),
                        theme,
                    )),
            )
    }

    fn render_textarea_page(&self, cx: &Context<Self>) -> impl IntoElement {
        card()
            .title("Textarea")
            .description("Multi-line text input for longer content")
            .body(
                v_stack()
                    .gap_4()
                    .child(
                        field("message")
                            .label("Message")
                            .description("Tell us what's on your mind")
                            .child(
                                textarea(&self.textarea_example, cx)
                                    .placeholder("Type your message here...")
                                    .rows(4),
                            ),
                    )
                    .child(
                        // Disabled all the way down: the field dims its own
                        // label, and the textarea paints static text with no
                        // live element at all, so it takes neither focus nor
                        // keystrokes. It clips a long value rather than
                        // scrolling it — that is the trade for being inert.
                        field("disabled-message").label("Disabled").disabled(true).child(
                            textarea(&self.textarea_disabled, cx)
                                .placeholder("This is disabled...")
                                .rows(2)
                                .disabled(true),
                        ),
                    )
                    .child(
                        // Read-only is the other half: still focusable, still
                        // selectable, still scrollable, and every edit path —
                        // typing, IME, paste, delete, tab, undo — refused by
                        // `InputState`.
                        field("read-only-message")
                            .label("Read-only")
                            .description("Focus it, select it, copy it — it will not change")
                            .child(
                                textarea(&self.textarea_read_only, cx)
                                    .rows(2)
                                    .read_only(true),
                            ),
                    ),
            )
    }

    fn render_avatar_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Avatar"),
            )
            .child(
                h_stack().gap_2().child(
                    avatar("https://avatars.githubusercontent.com/u/1714999?v=4").size(px(32.)),
                ),
            )
    }

    fn render_badge_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Badge"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(badge("Default"))
                    .child(badge("Secondary").secondary())
                    .child(badge("Outline").outline())
                    .child(badge("Destructive").destructive()),
            )
    }

    fn render_kbd_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Kbd"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(kbd("Esc"))
                    .child(kbd("Enter"))
                    .child(kbd("Tab"))
                    .child(kbd_combo(&["Ctrl", "C"]))
                    .child(kbd_combo(&["Cmd", "Shift", "P"])),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .mt_2()
                    .child(kbd("S").small())
                    .child(kbd("M"))
                    .child(kbd("L").large())
                    .child(
                        div()
                            .text_color(theme.fg_muted())
                            .child("(small / medium / large)"),
                    ),
            )
    }

    fn render_label_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Label"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .child(label("Basic Label"))
                    .child(label("Required Field").required(true))
                    .child(label("Disabled Label").disabled(true)),
            )
    }

    /// All seven variants at once — a gallery should show them, and they now
    /// share one clock rather than each pinning the window at the display
    /// refresh rate. Pause stops that clock without leaving the page, so the
    /// cost of the indicators can be told apart from the cost of everything
    /// else here.
    fn render_loading_indicator_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let fg_muted = theme.fg_muted();
        let playing = self.loading_playing;

        v_stack()
            .gap_2()
            .child(
                h_stack()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(fg_muted)
                            .child("LoadingIndicator"),
                    )
                    .child(
                        button("loading-play-pause", if playing { "Pause" } else { "Play" })
                            .on_click(cx.listener(|showcase, _event, _window, cx| {
                                showcase.loading_playing = !showcase.loading_playing;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .child(loading_indicator().dots().playing(playing))
                    .child(loading_indicator().ellipsis().playing(playing))
                    .child(loading_indicator().dash().playing(playing))
                    .child(loading_indicator().star().playing(playing))
                    .child(loading_indicator().triangle().playing(playing))
                    .child(loading_indicator().braille().playing(playing))
                    .child(loading_indicator().braille_extended().playing(playing)),
            )
            .child(div().text_sm().text_color(fg_muted).child(
                "One shared clock wakes only when a glyph changes, and only the views \
                         showing an indicator. Paused, it stops entirely.",
            ))
    }

    fn render_progress_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Progress"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(progress(0.25))
                    .child(progress(0.5))
                    .child(progress(0.75))
                    .child(progress(1.0).variant(ProgressVariant::Danger)),
            )
    }

    fn render_alert_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Alert"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(alert("This is a default alert message."))
                    .child(alert("Informational: Your session will expire in 5 minutes.").info())
                    .child(alert("Success! Your changes have been saved.").success())
                    .child(alert("Warning: This action cannot be undone.").warning())
                    .child(alert("Error: Failed to connect to server.").destructive())
                    .child(
                        alert("New feature available!")
                            .info()
                            .title("Heads up!")
                            .id("dismissible-alert")
                            .dismissible(true),
                    ),
            )
    }

    fn render_tooltip_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Tooltip"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .child(
                        button("tooltip-btn-1", "Hover me").tooltip(tooltip("This is a tooltip")),
                    )
                    .child(
                        icon_button("tooltip-icon", DefaultIcons::info_circled())
                            .tooltip(tooltip("More information")),
                    )
                    .child(
                        button("tooltip-btn-2", "Another one")
                            .tooltip(tooltip("Tooltips work on any element with an id")),
                    ),
            )
    }

    fn render_card_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Card"),
            )
            .child(
                card()
                    .title("Card Title")
                    .description("A short description of the card content.")
                    .footer(
                        h_stack()
                            .gap_2()
                            .child(button("card-save", "Save"))
                            .child(button("card-cancel", "Cancel").disabled(true)),
                    ),
            )
    }

    fn render_aspect_ratio_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("AspectRatio"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_start()
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("1:1 Square"),
                            )
                            .child(
                                aspect_ratio_square().width(px(80.0)).child(
                                    div()
                                        .size_full()
                                        .bg(theme.accent())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_xs()
                                        .text_color(theme.bg())
                                        .child("1:1"),
                                ),
                            ),
                    )
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("16:9 Video"),
                            )
                            .child(
                                aspect_ratio_video().width(px(160.0)).child(
                                    div()
                                        .size_full()
                                        .bg(theme.surface_secondary())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_xs()
                                        .text_color(theme.fg())
                                        .child("16:9"),
                                ),
                            ),
                    )
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("4:3 Photo"),
                            )
                            .child(
                                aspect_ratio(4.0 / 3.0).width(px(120.0)).child(
                                    div()
                                        .size_full()
                                        .bg(theme.accent_bg())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_xs()
                                        .text_color(theme.accent())
                                        .child("4:3"),
                                ),
                            ),
                    ),
            )
    }

    fn render_breadcrumb_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Breadcrumb"),
            )
            .child(
                v_stack()
                    .gap_3()
                    .child(
                        breadcrumb("breadcrumb-1")
                            .item(breadcrumb_item("Home"))
                            .item(breadcrumb_item("Documents"))
                            .item(breadcrumb_item("Projects")),
                    )
                    .child(
                        breadcrumb("breadcrumb-2")
                            .separator(BreadcrumbSeparator::Chevron)
                            .item(breadcrumb_item("Settings"))
                            .item(breadcrumb_item("Account"))
                            .item(breadcrumb_item("Profile")),
                    )
                    .child(
                        breadcrumb("breadcrumb-3")
                            .separator(BreadcrumbSeparator::Arrow)
                            .item(breadcrumb_item("Level 1"))
                            .item(breadcrumb_item("Level 2"))
                            .item(breadcrumb_item("Current")),
                    ),
            )
    }

    fn render_separator_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Separator"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(div().text_sm().child("Content above"))
                    .child(separator())
                    .child(div().text_sm().child("Content below")),
            )
    }

    fn render_splitter_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let border = theme.border();
        let fg_muted = theme.fg_muted();
        let surface = theme.surface_secondary();

        // A pane, so the three demos below are about the divider rather than
        // about what is on either side of it.
        let filled = move |label: &'static str, tint: Hsla| {
            div()
                .size_full()
                .bg(tint)
                .p_2()
                .text_sm()
                .text_color(fg_muted)
                .child(label)
        };

        let boxed = move |height: f32, child: gpui::AnyElement| {
            div()
                .h(px(height))
                .w_full()
                .border_1()
                .border_color(border)
                .rounded_md()
                .overflow_hidden()
                .child(child)
        };

        v_stack()
            .gap_6()
            .child(
                v_stack()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Splitter"),
                    )
                    .child(div().text_sm().text_color(fg_muted).child(
                        "One divider, two panes, a floor under each side. The ratio is the \
                         caller's — this page keeps all three of them, which is why Reset \
                         can exist at all.",
                    )),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Side by side"),
                            )
                            .child(div().text_xs().text_color(fg_muted).child(format!(
                                "{:.0}% / {:.0}%",
                                self.split_side_by_side * 100.,
                                (1. - self.split_side_by_side) * 100.,
                            )))
                            .child(button("splitter-reset", "Reset").on_click(cx.listener(
                                |this, _, _window, cx| {
                                    this.split_side_by_side = 0.4;
                                    cx.notify();
                                },
                            ))),
                    )
                    .child(boxed(
                        220.,
                        splitter("splitter-demo", "Files and editor", self.split_side_by_side)
                            .min_start(px(120.))
                            .min_end(px(160.))
                            .start(filled("Files", surface))
                            .end(filled("Editor", theme.surface()))
                            .on_resize(cx.listener(|this, ratio: &f32, _window, cx| {
                                this.split_side_by_side = *ratio;
                                cx.notify();
                            }))
                            .into_any_element(),
                    )),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Stacked"),
                    )
                    .child(boxed(
                        220.,
                        splitter("splitter-stacked", "Output and console", self.split_stacked)
                            .horizontal()
                            .min_start(px(48.))
                            .min_end(px(48.))
                            .start(filled("Output", surface))
                            .end(filled("Console", theme.surface()))
                            .on_resize(cx.listener(|this, ratio: &f32, _window, cx| {
                                this.split_stacked = *ratio;
                                cx.notify();
                            }))
                            .into_any_element(),
                    )),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("One per rung"),
                            )
                            .child(div().text_xs().text_color(fg_muted).child(
                                "The band you can grab is 6 / 8 / 12px — twice the rung's \
                                 gap. The line it draws is the same 1px hairline either way.",
                            )),
                    )
                    .children(
                        ControlSize::ALL
                            .into_iter()
                            .enumerate()
                            .map(|(index, size)| {
                                v_stack()
                                    .gap_1()
                                    .child(div().text_xs().text_color(fg_muted).child(size.name()))
                                    .child(boxed(
                                        72.,
                                        splitter(
                                            SharedString::from(format!("splitter-rung-{index}")),
                                            format!("{} splitter", size.name()),
                                            self.split_rungs[index],
                                        )
                                        .control_size(size)
                                        .start(filled("Start", surface))
                                        .end(filled("End", theme.surface()))
                                        .on_resize(cx.listener(
                                            move |this, ratio: &f32, _window, cx| {
                                                this.split_rungs[index] = *ratio;
                                                cx.notify();
                                            },
                                        ))
                                        .into_any_element(),
                                    ))
                            }),
                    ),
            )
    }

    fn render_sidebar_page(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Colors resolved up front: `List::render` takes `cx` mutably.
        let theme = cx.theme();
        let border = theme.border();
        let fg_muted = theme.fg_muted();
        let state = SidebarState::from(!self.demo_collapsed);
        let edge = self.demo_edge;

        // The contents are `List` + `Separator` + `Button` and nothing else —
        // no menu/group/header sub-components. That composition is the
        // argument for the component's small scope.
        let entries = vec![
            ListEntry::header("Project"),
            ListEntry::item("sidebar-demo-overview", |_w, _cx| {
                div().px_2().child("Overview").into_any_element()
            }),
            ListEntry::item("sidebar-demo-activity", |_w, _cx| {
                div().px_2().child("Activity").into_any_element()
            }),
            ListEntry::header("Settings"),
            ListEntry::item("sidebar-demo-members", |_w, _cx| {
                div().px_2().child("Members").into_any_element()
            }),
        ];

        let rail = v_stack()
            .gap_1()
            .child(
                icon_button("sidebar-demo-rail-overview", DefaultIcons::dashboard())
                    .tooltip(tooltip("Overview")),
            )
            .child(
                icon_button("sidebar-demo-rail-activity", DefaultIcons::activity_log())
                    .tooltip(tooltip("Activity")),
            )
            .child(
                icon_button("sidebar-demo-rail-members", DefaultIcons::person())
                    .tooltip(tooltip("Members")),
            );

        let panel = sidebar("sidebar-demo")
            .label("Project navigation")
            .edge(edge)
            .state(state)
            .width(gpui::rems(self.demo_width))
            // The demo box is 320px tall inside a much wider window, so the
            // window-width breakpoint would never fire here. Forcing it is
            // what makes the drawer visible without resizing the window — and
            // it also shows, on purpose, that the drawer is positioned in
            // *window* coordinates rather than inside this box.
            .map(|panel| {
                if self.demo_overlay {
                    panel.overlay_below(px(100_000.))
                } else {
                    panel.never_overlay()
                }
            })
            .on_dismiss(cx.listener(|this, _, _window, cx| {
                this.demo_overlay = false;
                cx.notify();
            }))
            .rail(rail)
            .child(
                h_stack().items_center().justify_between().child(
                    sidebar_trigger("sidebar-demo-trigger", state)
                        .edge(edge)
                        .label("Toggle project navigation")
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.demo_collapsed = !this.demo_collapsed;
                            cx.notify();
                        })),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .child(List::new("sidebar-demo-list", entries).render(window, cx)),
            )
            .child(separator())
            .child(button("sidebar-demo-action", "New project"));

        let body = div().flex_1().p_4().text_sm().text_color(fg_muted).child(
            "The panel beside this text is a Sidebar. Collapse it and it becomes a rail \
                 of icons rather than disappearing; make it overlay and it becomes a \
                 dismissible drawer with a scrim.",
        );

        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(fg_muted)
                    .child("Sidebar"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(
                        button(
                            "sidebar-demo-collapse",
                            if self.demo_collapsed {
                                "Expand"
                            } else {
                                "Collapse"
                            },
                        )
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.demo_collapsed = !this.demo_collapsed;
                            cx.notify();
                        })),
                    )
                    .child(
                        button(
                            "sidebar-demo-edge",
                            if edge == SidebarEdge::Left {
                                "Dock right"
                            } else {
                                "Dock left"
                            },
                        )
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.demo_edge = if this.demo_edge == SidebarEdge::Left {
                                SidebarEdge::Right
                            } else {
                                SidebarEdge::Left
                            };
                            cx.notify();
                        })),
                    )
                    .child(button("sidebar-demo-wider", "Wider").on_click(cx.listener(
                        |this, _, _window, cx| {
                            this.demo_width = (this.demo_width + 2.5).min(25.);
                            cx.notify();
                        },
                    )))
                    .child(
                        button("sidebar-demo-narrower", "Narrower").on_click(cx.listener(
                            |this, _, _window, cx| {
                                this.demo_width = (this.demo_width - 2.5).max(7.5);
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        button(
                            "sidebar-demo-overlay",
                            if self.demo_overlay {
                                "Push instead"
                            } else {
                                "Show as overlay"
                            },
                        )
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.demo_overlay = !this.demo_overlay;
                            cx.notify();
                        })),
                    ),
            )
            .child({
                let frame = h_stack()
                    .h(px(320.))
                    .w_full()
                    .border_1()
                    .border_color(border)
                    .rounded_md()
                    .overflow_hidden();

                // The panel is a flex child on the docked side, which is the
                // whole of what "push" means.
                if edge == SidebarEdge::Left {
                    frame.child(panel).child(body)
                } else {
                    frame.child(body).child(panel)
                }
            })
    }

    fn render_collapsible_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Collapsible"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(self.collapsible_basic.clone())
                    .child(self.collapsible_nested.clone()),
            )
    }

    fn render_accordion_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Accordion"),
            )
            .child(self.accordion.clone())
    }

    fn render_scroll_area_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("ScrollArea"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Vertical scroll:"),
                            )
                            .child(
                                scroll_area("vertical-scroll-demo")
                                    .max_h(px(120.))
                                    .vertical()
                                    .child(
                                        v_stack()
                                            .gap_2()
                                            .p_2()
                                            .bg(theme.surface())
                                            .border_1()
                                            .border_color(theme.border())
                                            .rounded_sm()
                                            .children((1..=15).map(|i| {
                                                div().text_xs().child(format!("Item {}", i))
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Horizontal scroll:"),
                            )
                            .child(
                                scroll_area("horizontal-scroll-demo")
                                    .max_w(px(150.))
                                    .horizontal()
                                    .child(
                                        h_stack()
                                            .gap_2()
                                            .p_2()
                                            .bg(theme.surface())
                                            .border_1()
                                            .border_color(theme.border())
                                            .rounded_sm()
                                            .children((1..=10).map(|i| {
                                                div()
                                                    .px_3()
                                                    .py_1()
                                                    .bg(theme.accent_bg())
                                                    .rounded_sm()
                                                    .text_xs()
                                                    .child(format!("Tag {}", i))
                                            })),
                                    ),
                            ),
                    ),
            )
    }

    fn render_list_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("List"),
            )
            .child(
                div()
                    .h(px(250.))
                    .border_1()
                    .border_color(theme.border())
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        List::new(
                            "showcase-list",
                            vec![
                                ListEntry::header("Conflicts"),
                                ListEntry::item("f-1", |_w, _cx| {
                                    div().px_2().child("src/services.rs").into_any_element()
                                }),
                                ListEntry::header("Changes"),
                                ListEntry::item("f-2", |_w, _cx| {
                                    div().px_2().child("src/main.rs").into_any_element()
                                }),
                                ListEntry::item("f-3", |_w, _cx| {
                                    div()
                                        .px_2()
                                        .child("src/services/auth.rs")
                                        .into_any_element()
                                }),
                                ListEntry::item("f-4", |_w, _cx| {
                                    div().px_2().child("src/ui/auth.rs").into_any_element()
                                }),
                                ListEntry::item("f-5", |_w, _cx| {
                                    div()
                                        .px_2()
                                        .child("src/utils/helpers.rs")
                                        .into_any_element()
                                }),
                                ListEntry::header("New"),
                                ListEntry::item("f-6", |_w, _cx| {
                                    div().px_2().child("build.rs").into_any_element()
                                }),
                                ListEntry::item("f-7", |_w, _cx| {
                                    div().px_2().child("Cargo.toml").into_any_element()
                                }),
                                ListEntry::item("f-8", |_w, _cx| {
                                    div().px_2().child("src/lib.rs").into_any_element()
                                }),
                            ],
                        )
                        .render(window, cx),
                    ),
            )
    }

    fn render_popover_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Popover"),
            )
            .child(self.popover_example.clone())
    }

    fn render_dialog_page(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Dialog"),
            )
            .child(button("open-dialog", "Open Dialog").on_click(cx.listener(
                |showcase, _, window, cx| {
                    showcase.dialog_example.update(cx, |dialog, cx| {
                        dialog.open(window, cx);
                    });
                },
            )))
            .child(div().text_sm().text_color(theme.fg_muted()).child(
                "A confirmation is the same element in confirm mode: one question, \
                            two answers, focus on the safe one, and Role::AlertDialog.",
            ))
            .child(
                button("open-destructive-dialog", "Delete Project")
                    .destructive()
                    .on_click(cx.listener(|showcase, _, window, cx| {
                        showcase.destructive_dialog.update(cx, |dialog, cx| {
                            dialog.open(window, cx);
                        });
                    })),
            )
    }

    fn render_context_menu_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let this = cx.entity();
        let pinned = self.context_menu_pinned;

        // The trigger is an ordinary element. The menu attaches to what the
        // view already renders instead of taking over how it is built.
        let target = div()
            .px_8()
            .py_6()
            .rounded_md()
            .border_1()
            .border_color(theme.border())
            .bg(theme.surface())
            .text_sm()
            .text_color(theme.fg_muted())
            .child("Right-click here");

        v_stack()
            .gap_4()
            .items_start()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Context Menu"),
            )
            .child(
                context_menu("showcase-context-menu", target).menu(move |menu, _window, _cx| {
                    let choose = |label: &'static str| {
                        let this = this.clone();
                        move |_window: &mut Window, cx: &mut App| {
                            this.update(cx, |showcase: &mut Showcase, cx| {
                                showcase.context_menu_status = format!("Chose “{label}”").into();
                                cx.notify();
                            });
                        }
                    };

                    menu.header("Edit")
                        .item(
                            menu_item("Cut")
                                .icon(DefaultIcons::scissors)
                                .kbd("⌘X")
                                .on_click(choose("Cut")),
                        )
                        .item(
                            menu_item("Copy")
                                .icon(DefaultIcons::copy)
                                .kbd("⌘C")
                                .on_click(choose("Copy")),
                        )
                        .item(
                            menu_item("Paste")
                                .icon(DefaultIcons::clipboard)
                                .kbd("⌘V")
                                // Nothing to paste: shown, but not choosable.
                                .disabled(true),
                        )
                        .separator()
                        .item(menu_item("Pinned").toggled(pinned).on_click({
                            let this = this.clone();
                            move |_window, cx| {
                                this.update(cx, |showcase: &mut Showcase, cx| {
                                    showcase.context_menu_pinned = !showcase.context_menu_pinned;
                                    showcase.context_menu_status = if showcase.context_menu_pinned {
                                        "Pinned".into()
                                    } else {
                                        "Unpinned".into()
                                    };
                                    cx.notify();
                                });
                            }
                        }))
                        .separator()
                        .item(
                            menu_item("Delete")
                                .icon(DefaultIcons::trash)
                                .destructive()
                                .on_click(choose("Delete")),
                        )
                }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.fg_muted())
                    .child(self.context_menu_status.clone()),
            )
    }

    fn render_toast_page(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Toast"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .child(button("toast-default", "Default").on_click(cx.listener(
                        |_, _, window, cx| {
                            cx.toast("This is a default toast").show(window, cx);
                        },
                    )))
                    .child(button("toast-success", "Success").on_click(cx.listener(
                        |_, _, window, cx| {
                            cx.toast("Changes saved successfully")
                                .success()
                                .show(window, cx);
                        },
                    )))
                    .child(button("toast-warning", "Warning").on_click(cx.listener(
                        |_, _, window, cx| {
                            cx.toast("Please check your input")
                                .warning()
                                .show(window, cx);
                        },
                    ))),
            )
    }

    fn render_typography_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Typography"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .child("Prose primitives for text a component owns. For a whole document, use Markdown."),
            )
            .child(
                v_stack()
                    .gap_3()
                    .max_w(px(520.))
                    .child(h1("Heading one"))
                    .child(h2("Heading two"))
                    .child(h3("Heading three"))
                    .child(h4("Heading four"))
                    .child(lead(
                        "A lead paragraph introduces the section it opens, one size up from body text.",
                    ))
                    .child(p(
                        "A paragraph of body text. It wraps, it takes the theme's foreground colour, and it is the default for prose.",
                    ))
                    .child(p("A muted paragraph, for secondary detail.").muted())
                    .child(p("A destructive paragraph, for something that went wrong.").destructive())
                    .child(blockquote("A block quote, set off by a rule down its left side."))
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(text("Inline text,"))
                            .child(text("bold,").bold())
                            .child(text("code,").code())
                            .child(text("accent.").accent()),
                    )
                    .child(small("Small print, for a footnote or a caption.")),
            )
    }

    fn render_empty_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Empty"),
            )
            .child(div().text_sm().text_color(theme.fg_muted()).child(
                "The placeholder a list, table or search result shows when it has nothing in it.",
            ))
            .child(
                v_stack()
                    .gap_4()
                    .child(
                        div()
                            .border_1()
                            .border_color(theme.border())
                            .rounded_md()
                            .child(
                                empty()
                                    .icon(DefaultIcons::magnifying_glass())
                                    .title("No results")
                                    .description(
                                        "Nothing matched that search. Try a shorter query.",
                                    )
                                    .action(button("empty-clear", "Clear search")),
                            ),
                    )
                    .child(
                        div()
                            .border_1()
                            .border_color(theme.border())
                            .rounded_md()
                            .child(empty().title("Nothing here yet")),
                    ),
            )
    }

    /// Restart the Streaming section's document, feeding it one delta at a
    /// time on a background timer — the same `append` path a real stream uses.
    fn stream_reply(&mut self, cx: &mut Context<Self>) {
        self.stream_generation = self.stream_generation.wrapping_add(1);
        let generation = self.stream_generation;
        self.markdown_stream
            .update(cx, |markdown, cx| markdown.set_source("", cx));

        cx.spawn(async move |this, cx| {
            let mut sent = 0;
            while sent < STREAMED_REPLY.len() {
                cx.background_executor().timer(STREAM_INTERVAL).await;

                let mut end = (sent + STREAM_CHUNK).min(STREAMED_REPLY.len());
                while !STREAMED_REPLY.is_char_boundary(end) {
                    end += 1;
                }
                let delta = &STREAMED_REPLY[sent..end];
                sent = end;

                let still_current = this.update(cx, |this: &mut Self, cx| {
                    if this.stream_generation != generation {
                        return false;
                    }
                    this.markdown_stream
                        .update(cx, |markdown, cx| markdown.append(delta, cx));
                    true
                });

                match still_current {
                    Ok(true) => {}
                    // Restarted, or the showcase is gone.
                    Ok(false) | Err(_) => break,
                }
            }
        })
        .detach();

        cx.notify();
    }

    /// The rows the table is handed: filtered by the field above it, then
    /// sorted by whatever the last header click asked for.
    ///
    /// Both halves are the page's, not the element's. Filtering is a
    /// `TextField` above the table by design, and sorting inside the element
    /// would mean it owning comparison for arbitrary cell types.
    fn visible_repositories(&self, cx: &App) -> Vec<Repo> {
        let needle = self.table_filter.read(cx).content().to_lowercase();

        let mut rows: Vec<Repo> = REPOSITORIES
            .iter()
            .copied()
            .filter(|repo| {
                needle.is_empty()
                    || repo.name.to_lowercase().contains(&needle)
                    || repo.language.to_lowercase().contains(&needle)
            })
            .collect();

        let sort = self.table_sort;
        rows.sort_by(|left, right| {
            let ordering = match sort.column {
                TABLE_COLUMN_REPOSITORY => left.name.cmp(right.name),
                TABLE_COLUMN_LANGUAGE => left
                    .language
                    .cmp(right.language)
                    .then_with(|| left.name.cmp(right.name)),
                TABLE_COLUMN_STARS => left.stars.cmp(&right.stars),
                // The Status column is not sortable, so nothing asks for this.
                _ => std::cmp::Ordering::Equal,
            };

            match sort.direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });

        rows
    }

    fn render_table_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        // The handlers below run with `&mut App`, not with this view's
        // `Context`, so they reach the page through its own entity.
        let view = cx.entity();

        let rows = self.visible_repositories(cx);
        let selected = self.table_selected.clone();
        let selected_count = rows
            .iter()
            .filter(|repo| self.table_selected.contains(&repo.id))
            .count();

        let data_table = table("showcase-repositories")
            .column(
                Column::new("Repository", |repo: &Repo, _window, _cx| {
                    div().child(repo.name).into_any_element()
                })
                .sortable()
                .min_width(px(160.)),
            )
            .column(
                Column::new("Language", |repo: &Repo, _window, _cx| {
                    div().child(repo.language).into_any_element()
                })
                .sortable()
                .fixed(px(140.)),
            )
            .column(
                Column::new("Stars", |repo: &Repo, _window, _cx| {
                    div().child(repo.stars.to_string()).into_any_element()
                })
                .sortable()
                .fixed(px(110.))
                .end(),
            )
            .column(
                // A cell renderer returns any element, so a column can hold a
                // control rather than text.
                Column::new("Status", |repo: &Repo, _window, _cx| {
                    match repo.status {
                        RepoStatus::Active => badge(repo.status.label()),
                        RepoStatus::Archived => badge(repo.status.label()).secondary(),
                        RepoStatus::Draft => badge(repo.status.label()).outline(),
                    }
                    .into_any_element()
                })
                .fixed(px(130.))
                .align(CellAlign::Center),
            )
            .rows(rows.iter().map(|repo| {
                let repo = *repo;
                let view = view.clone();
                Row::new(repo)
                    .selected(selected.contains(&repo.id))
                    // Activation, which is a different act from selection —
                    // clicking the checkbox does not open the row.
                    .on_click(move |_window, cx| {
                        view.update(cx, |this, cx| {
                            this.table_status = format!("Opened {}", repo.name).into();
                            cx.notify();
                        });
                    })
            }))
            .sorted_by(self.table_sort)
            .on_sort({
                let view = view.clone();
                move |request, _window, cx| {
                    view.update(cx, |this, cx| {
                        // `suggested()` is the conventional toggle. A page that
                        // wanted Stars to start descending would ignore it and
                        // build its own descriptor here.
                        this.table_sort = request.suggested();
                        cx.notify();
                    });
                }
            })
            .on_select_row({
                let view = view.clone();
                move |request, _window, cx| {
                    view.update(cx, |this, cx| {
                        // The request carries a row *index*, in the order this
                        // page handed the rows over, so it is resolved against
                        // the same derivation before an id is stored. That
                        // round trip is the demonstration.
                        let rows = this.visible_repositories(cx);
                        if let Some(repo) = rows.get(request.row) {
                            if request.selected {
                                this.table_selected.insert(repo.id);
                            } else {
                                this.table_selected.remove(&repo.id);
                            }
                        }
                        cx.notify();
                    });
                }
            })
            .on_select_all({
                let view = view.clone();
                move |request, _window, cx| {
                    view.update(cx, |this, cx| {
                        // "All" means the rows currently on screen, because
                        // those are the rows this page gave the table.
                        let rows = this.visible_repositories(cx);
                        for repo in rows {
                            if request.selected {
                                this.table_selected.insert(repo.id);
                            } else {
                                this.table_selected.remove(&repo.id);
                            }
                        }
                        cx.notify();
                    });
                }
            })
            .max_h(px(320.))
            .empty("No repositories match this filter.");

        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Table"),
            )
            .child(div().text_sm().text_color(theme.fg_muted()).child(
                "Rows arrive already filtered and already sorted; the table reports \
                         clicks back. The filter is a TextField above the table, not a table \
                         feature.",
            ))
            .child(
                h_stack()
                    .gap_3()
                    .items_center()
                    .child(
                        div().w(px(260.)).child(
                            text_field(&self.table_filter, cx)
                                .placeholder("Filter repositories")
                                .prefix(Adornment::icon(DefaultIcons::magnifying_glass())),
                        ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.fg_muted())
                            .child(format!("{selected_count} of {} selected", rows.len())),
                    ),
            )
            .child(data_table)
            .child(
                div()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .child(self.table_status.clone()),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted())
                            .child("Sizes"),
                    )
                    .children(ControlSize::ALL.map(|size| {
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child(size.name()),
                            )
                            .child(
                                table(SharedString::from(format!("table-size-{}", size.name())))
                                    .control_size(size)
                                    .column(Column::new("Repository", |repo: &Repo, _, _| {
                                        div().child(repo.name).into_any_element()
                                    }))
                                    .column(
                                        Column::new("Stars", |repo: &Repo, _, _| {
                                            div().child(repo.stars.to_string()).into_any_element()
                                        })
                                        .fixed(px(110.))
                                        .end(),
                                    )
                                    .rows(REPOSITORIES.iter().take(2).copied().map(Row::new)),
                            )
                    })),
            )
    }

    fn render_markdown_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        // `init_code_highlighting` is itself feature-gated, so the default
        // build's ```rust fence is plain monospace by design. Say which build
        // this is, so that is not read as a bug.
        let highlighting = if cfg!(feature = "editor") {
            "Code fences are syntax highlighted (built with --features editor)."
        } else {
            "Code fences are plain monospace — rebuild with --features editor to highlight them."
        };
        let stitching = if preprocessing_available() {
            "Partial syntax is closed before parsing (built with --features stitch)."
        } else {
            "Partial syntax is left as written — build with --features stitch to close it."
        };

        let selected = self.markdown.read(cx).selected_text();
        let selection_readout = match &selected {
            Some(text) => {
                let mut summary: String = text.chars().take(80).collect();
                if text.chars().count() > 80 {
                    summary.push('…');
                }
                format!("Selected {} characters: {summary}", text.len())
            }
            None => "Nothing selected — drag across the document above.".to_string(),
        };

        let note = |label: &str, body: String| {
            v_stack()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(label.to_string()),
                )
                .child(div().text_xs().text_color(theme.fg_muted()).child(body))
        };

        v_stack()
            .gap_6()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Markdown"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(note("This build", format!("{highlighting} {stitching}")))
                    .child(note(
                        "Accessibility",
                        "Every block is announced with a role — heading (with its level), \
                         paragraph, quote, list item, code — under one document node."
                            .to_string(),
                    )),
            )
            .child(
                div()
                    .border_1()
                    .border_color(theme.border())
                    .rounded_md()
                    .p_4()
                    .child(MarkdownElement::new(self.markdown.clone())),
            )
            .child(separator())
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Selection"),
                    )
                    .child(div().text_xs().text_color(theme.fg_muted()).child(
                        "Drag across the blocks above — the selection flows between them. \
                                 Double-click a word, triple-click a block. \
                                 examples/markdown_selection.rs binds this to cmd-c.",
                    ))
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                button("markdown-copy", "Copy selection").on_click(cx.listener(
                                    |this, _, _window, cx| {
                                        this.markdown_copy_status =
                                            match this.markdown.read(cx).selected_text() {
                                                Some(text) => {
                                                    let len = text.len();
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(text),
                                                    );
                                                    format!(
                                                        "Copied {len} characters to the clipboard."
                                                    )
                                                    .into()
                                                }
                                                None => "Nothing selected to copy.".into(),
                                            };
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child(self.markdown_copy_status.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.fg_muted())
                            .child(selection_readout),
                    ),
            )
            .child(separator())
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Streaming"),
                    )
                    .child(div().text_xs().text_color(theme.fg_muted()).child(
                        "`Markdown::append` extends the source and re-parses off the UI \
                             thread, so the previous parse keeps rendering until the new one \
                             lands. examples/markdown_streaming.rs streams at frame rate. \
                             Watch the code fence: it draws plain until its closing ``` \
                             arrives, then highlights once and stays cached.",
                    ))
                    .child(
                        button("markdown-stream", "Stream a reply")
                            .on_click(cx.listener(|this, _, _window, cx| this.stream_reply(cx))),
                    )
                    .child(
                        div()
                            .min_h(px(120.))
                            .border_1()
                            .border_color(theme.border())
                            .rounded_md()
                            .p_4()
                            .child(MarkdownElement::new(self.markdown_stream.clone())),
                    ),
            )
    }

    fn render_editor_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // The page exists in both builds: requiring `--features editor` for
        // the showcase would make every other page pay for syntect, and
        // dropping the page is what let the editor go undemonstrated.
        #[cfg(feature = "editor")]
        let demo = {
            use gpuikit::editor::{Editor, EditorElement};

            let lines: Vec<String> = EDITOR_SAMPLE.lines().map(str::to_string).collect();
            let mut editor = Editor::new("showcase-editor", lines);
            editor.set_language("rust".to_string());

            div()
                .h(px(220.))
                .border_1()
                .border_color(theme.border())
                .rounded_md()
                .overflow_hidden()
                .child(EditorElement::new(editor))
                .into_any_element()
        };
        #[cfg(not(feature = "editor"))]
        let demo = empty()
            .title("Built without the editor feature")
            .description("Run `cargo run --example showcase --features editor` for a live buffer.")
            .into_any_element();

        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Editor"),
            )
            .child(div().text_sm().text_color(theme.fg_muted()).child(
                "A gutter, line numbers, an active line and syntect highlighting. \
                     Display only here: `EditorElement` has no keyboard handling of its own, \
                     so an interactive page waits on an `EditorView`.",
            ))
            .child(demo)
    }

    /// Every control that can share a row, on one row, once per rung.
    ///
    /// Each row sits on a tinted stripe exactly the rung's height, so a control
    /// that is off its rung is visible immediately rather than only in a test.
    fn render_control_sizes_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_stack()
            .gap_6()
            .child(
                v_stack()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Control Sizes"),
                    )
                    .child(div().text_sm().text_color(theme.fg_muted()).child(
                        "One rung per row. The stripe behind each row is exactly the \
                         rung's height — a control that overhangs it is off its rung.",
                    )),
            )
            .children(
                ControlSize::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(index, size)| {
                        let metrics = theme.control(size);

                        v_stack()
                            .gap_2()
                            .child(
                                h_stack()
                                    .gap_2()
                                    .items_baseline()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(size.name()),
                                    )
                                    .child(div().text_xs().text_color(theme.fg_muted()).child(
                                        format!(
                                            "{}px tall · {}px text",
                                            metrics.height.0 * 16.0,
                                            metrics.text_size.0 * 16.0,
                                        ),
                                    )),
                            )
                            .child(
                                div()
                                    .relative()
                                    .child(
                                        // The stripe is the rung, drawn behind the row.
                                        div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .right_0()
                                            .h(metrics.height)
                                            .bg(theme.accent().opacity(0.12)),
                                    )
                                    .child(
                                        h_stack()
                                            .gap_3()
                                            // Flex defaults to stretch, which would
                                            // give every control the row's height and
                                            // make this page prove nothing.
                                            .items_start()
                                            .flex_wrap()
                                            .child(
                                                button(
                                                    SharedString::from(format!(
                                                        "control-row-button-{}",
                                                        size.name()
                                                    )),
                                                    "Button",
                                                )
                                                .control_size(size),
                                            )
                                            .child(
                                                icon_button(
                                                    SharedString::from(format!(
                                                        "control-row-icon-{}",
                                                        size.name()
                                                    )),
                                                    DefaultIcons::star(),
                                                )
                                                .control_size(size),
                                            )
                                            .child(badge("Badge").control_size(size))
                                            .child(kbd("K").control_size(size))
                                            .child(self.control_row_checkboxes[index].clone())
                                            .child(self.control_row_switches[index].clone())
                                            .child(self.control_row_toggles[index].clone())
                                            .child(self.control_row_selects[index].clone())
                                            .child(
                                                text_field(&self.control_row_fields[index], cx)
                                                    .placeholder("Field")
                                                    .control_size(size),
                                            ),
                                    ),
                            )
                    }),
            )
    }

    fn render_coverage_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        let mut table = v_stack().gap_0().child(
            h_stack()
                .gap_4()
                .py_1()
                .border_b_1()
                .border_color(theme.border())
                .child(
                    div()
                        .w(px(220.))
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("src/elements/"),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Shown on"),
                ),
        );

        for (module, page) in ELEMENT_COVERAGE {
            table = table.child(
                h_stack()
                    .gap_4()
                    .py_1()
                    .child(div().w(px(220.)).text_sm().child(format!("{module}.rs")))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.fg_muted())
                            .child(page.to_string()),
                    ),
            );
        }

        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Coverage"),
            )
            .child(div().text_sm().text_color(theme.fg_muted()).child(format!(
                "{} element modules, each mapped to the page that shows it. Two tests in \
                     src/elements.rs fail the build if a module gains no page, or if a page \
                     named here is not reachable from the nav.",
                ELEMENT_COVERAGE.len()
            )))
            .child(table)
    }

    fn render_theme_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        let sections: Vec<(&str, Vec<(&str, Hsla)>)> = vec![
            (
                "Primitives",
                vec![
                    ("fg", theme.fg()),
                    ("bg", theme.bg()),
                    ("surface", theme.surface()),
                    ("border", theme.border()),
                    ("accent", theme.accent()),
                ],
            ),
            (
                "Foreground",
                vec![
                    ("fg_muted", theme.fg_muted()),
                    ("fg_disabled", theme.fg_disabled()),
                    ("placeholder", theme.placeholder()),
                ],
            ),
            (
                "Surface & Border",
                vec![
                    ("surface_secondary", theme.surface_secondary()),
                    ("surface_tertiary", theme.surface_tertiary()),
                    ("border_secondary", theme.border_secondary()),
                    ("border_subtle", theme.border_subtle()),
                    ("outline", theme.outline()),
                ],
            ),
            (
                "Accent",
                vec![
                    ("accent_bg", theme.accent_bg()),
                    ("accent_bg_hover", theme.accent_bg_hover()),
                    ("selection", theme.selection()),
                ],
            ),
            (
                "Semantic",
                vec![
                    ("info", theme.info()),
                    ("success", theme.success()),
                    ("warning", theme.warning()),
                    ("danger", theme.danger()),
                ],
            ),
            ("Overlay", vec![("overlay", theme.overlay())]),
            (
                "Button",
                vec![
                    ("button_bg", theme.button_bg()),
                    ("button_bg_hover", theme.button_bg_hover()),
                    ("button_bg_active", theme.button_bg_active()),
                    ("button_border", theme.button_border()),
                ],
            ),
            (
                "Input",
                vec![
                    ("input_bg", theme.input_bg()),
                    ("input_border", theme.input_border()),
                    ("input_border_hover", theme.input_border_hover()),
                    ("input_border_focused", theme.input_border_focused()),
                    ("input_text", theme.input_text()),
                    ("input_placeholder", theme.input_placeholder()),
                    ("input_selection", theme.input_selection()),
                    ("input_cursor", theme.input_cursor()),
                ],
            ),
            (
                "Badge",
                vec![
                    ("badge_blue", theme.badge_blue()),
                    ("badge_gold", theme.badge_gold()),
                    ("badge_red", theme.badge_red()),
                    ("badge_green", theme.badge_green()),
                    ("badge_teal", theme.badge_teal()),
                    ("badge_amber", theme.badge_amber()),
                    ("badge_gray", theme.badge_gray()),
                ],
            ),
        ];

        let mut root = v_stack().gap_6().child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.fg_muted())
                .child(format!("Theme — {}", theme.name)),
        );

        for (section_name, rows) in sections {
            let mut section = v_stack().gap_1().child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .pb_1()
                    .border_b_1()
                    .border_color(theme.border_subtle())
                    .child(section_name.to_string()),
            );
            for (name, color) in rows {
                section = section.child(color_row(name, color, &theme));
            }
            root = root.child(section);
        }

        root
    }
}

fn fmt_hex(color: Hsla) -> String {
    let rgba: Rgba = color.into();
    let r = (rgba.r * 255.0).round() as u8;
    let g = (rgba.g * 255.0).round() as u8;
    let b = (rgba.b * 255.0).round() as u8;
    if rgba.a < 0.999 {
        let a = (rgba.a * 255.0).round() as u8;
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

fn color_row(name: &str, color: Hsla, theme: &gpuikit::theme::Theme) -> gpui::Div {
    h_stack()
        .items_center()
        .gap_3()
        .py_1()
        .child(
            div()
                .w(px(18.))
                .h(px(18.))
                .rounded_full()
                .bg(color)
                .border_1()
                .border_color(theme.border_subtle()),
        )
        .child(div().w(px(220.)).text_sm().child(name.to_string()))
        .child(
            div()
                .text_sm()
                .text_color(theme.fg_muted())
                .child(fmt_hex(color)),
        )
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_page: SharedString = self.active_page.borrow().clone();

        // Captured before any mutable borrow. The panel owns its own surface
        // and border now, so only the window's own two colors are needed here.
        let bg = cx.theme().bg();
        let fg = cx.theme().fg();

        // The sidebar was built once in `Showcase::new`; a frame only decides
        // which row is highlighted.
        let entries: Vec<ListEntry> = self
            .nav
            .iter()
            .map(|nav| {
                nav.entry
                    .clone()
                    .selected(nav.page.as_ref() == Some(&current_page))
            })
            .collect();

        // The acceptance test from the Sidebar issue: the showcase's own
        // hand-rolled `div`-with-a-border sidebar is now the component, with a
        // rail and a drawer. Nothing here is a sub-component — the contents
        // are `List`, the theme `Select`, and `IconButton`s.
        let nav_state = SidebarState::from(!self.nav_collapsed);
        let current_section = section_of(&current_page);

        let rail = v_stack()
            .gap_1()
            .children(NAV_SECTIONS.iter().map(|(label, icon, items)| {
                let first = items.first().map(|(id, _)| SharedString::from(*id));
                let cell = self.active_page.clone();

                icon_button(SharedString::from(format!("nav-rail-{label}")), icon())
                    .selected(current_section == Some(*label))
                    .tooltip(tooltip(*label))
                    .on_click(move |_, window, _cx| {
                        if let Some(page) = first.clone() {
                            *cell.borrow_mut() = page;
                            window.refresh();
                        }
                    })
            }));

        let sidebar_panel = sidebar("showcase-nav")
            .label("Showcase navigation")
            .state(nav_state)
            .width(gpui::rems(12.5))
            .rail(rail)
            .on_dismiss(cx.listener(|this, _, _window, cx| {
                this.nav_collapsed = true;
                cx.notify();
            }))
            .child(
                h_stack().items_center().justify_between().child(
                    sidebar_trigger("showcase-nav-trigger", nav_state)
                        .label("Toggle navigation")
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.nav_collapsed = !this.nav_collapsed;
                            cx.notify();
                        })),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .child(List::new("nav-list", entries).render(window, cx)),
            )
            .child(self.theme_select.clone());

        let content = match current_page.as_ref() {
            "button" => v_stack()
                .gap_8()
                .child(self.render_button_page(window, cx))
                .child(self.render_icon_button_page(window, cx))
                .child(self.render_button_group_page(cx))
                .into_any_element(),
            "toggle" => v_stack()
                .gap_8()
                .child(self.render_checkbox_page(cx))
                .child(self.render_switch_page(cx))
                .child(self.render_toggle_page(cx))
                .into_any_element(),
            "selection" => v_stack()
                .gap_8()
                .child(self.render_radio_group_page(cx))
                .child(self.render_toggle_group_page(cx))
                .into_any_element(),
            "select" => self.render_select_page(cx).into_any_element(),
            "control-sizes" => self.render_control_sizes_page(cx).into_any_element(),
            "text" => v_stack()
                .gap_8()
                .child(self.render_field_page(cx))
                .child(self.render_text_field_page(cx))
                .child(self.render_textarea_page(cx))
                .into_any_element(),
            "form" => self.render_form_page(cx).into_any_element(),
            "slider" => self.render_slider_page(cx).into_any_element(),
            "tabs" => self.render_tabs_page(cx).into_any_element(),
            "avatar" => self.render_avatar_page(cx).into_any_element(),
            "typography" => self.render_typography_page(cx).into_any_element(),
            "empty" => self.render_empty_page(cx).into_any_element(),
            "badge" => v_stack()
                .gap_8()
                .child(self.render_badge_page(cx))
                .child(self.render_label_page(cx))
                .child(self.render_kbd_page(cx))
                .into_any_element(),
            "loading" => v_stack()
                .gap_8()
                .child(self.render_loading_indicator_page(cx))
                .child(self.render_progress_page(cx))
                .into_any_element(),
            "alert" => self.render_alert_page(cx).into_any_element(),
            "tooltip" => self.render_tooltip_page(cx).into_any_element(),
            "card" => self.render_card_page(cx).into_any_element(),
            "aspect-ratio" => self.render_aspect_ratio_page(cx).into_any_element(),
            "breadcrumb" => self.render_breadcrumb_page(cx).into_any_element(),
            "separator" => self.render_separator_page(cx).into_any_element(),
            "sidebar" => self.render_sidebar_page(window, cx).into_any_element(),
            "splitter" => self.render_splitter_page(cx).into_any_element(),
            "collapsible" => v_stack()
                .gap_8()
                .child(self.render_collapsible_page(cx))
                .child(self.render_accordion_page(cx))
                .into_any_element(),
            "scroll-area" => self.render_scroll_area_page(cx).into_any_element(),
            "list" => self.render_list_page(window, cx).into_any_element(),
            "popover" => self.render_popover_page(cx).into_any_element(),
            "dialog" => self.render_dialog_page(window, cx).into_any_element(),
            "context-menu" => self.render_context_menu_page(cx).into_any_element(),
            "toast" => self.render_toast_page(window, cx).into_any_element(),
            "table" => self.render_table_page(cx).into_any_element(),
            "markdown" => self.render_markdown_page(cx).into_any_element(),
            "editor" => self.render_editor_page(cx).into_any_element(),
            "theme" => self.render_theme_page(cx).into_any_element(),
            "coverage" => self.render_coverage_page(cx).into_any_element(),
            _ => div().child("Unknown page").into_any_element(),
        };

        h_stack()
            // The cold-start case, worked. `gpuikit::init` binds Tab, and
            // `a11y::announce` puts the listener on every control it makes
            // focusable — but with *nothing* focused gpui dispatches to the
            // node belonging to its own wrapper around this view, above this
            // element, so the very first Tab would reach no listener at all.
            // Tracking the handle `main` focuses at startup and answering Tab
            // here is what makes it work. See `gpuikit::a11y`, section 4.
            .id("showcase-root")
            .track_focus(&self.focus_handle)
            .moves_focus_on_tab()
            .bg(bg)
            .text_color(fg)
            .size_full()
            .overflow_hidden()
            .child(sidebar_panel)
            .child(
                div()
                    .id("content-area")
                    .flex_1()
                    .overflow_y_scroll()
                    .min_h_full()
                    .p_8()
                    .child(content),
            )
            .child(self.dialog_example.clone())
            .child(self.destructive_dialog.clone())
            .child(cx.toast_manager().clone())
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);

            // Syntax highlighting for the Markdown page's ```rust fence.
            // Opt-in, and itself gated on the feature that pulls in syntect,
            // so this cannot be called unconditionally.
            #[cfg(feature = "editor")]
            gpuikit::markdown::init_code_highlighting(cx);

            cx.set_menus(vec![Menu {
                name: "GPUIKit Showcase".into(),
                items: vec![],
                disabled: false,
            }]);

            let window = cx
                .open_window(
                    WindowOptions {
                        titlebar: Some(TitlebarOptions {
                            title: Some("GPUIKit Component Showcase".into()),
                            ..Default::default()
                        }),
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Default::default(),
                            size: size(px(1200.0), px(680.0)),
                        })),
                        ..Default::default()
                    },
                    |window, cx| cx.new(|cx| Showcase::new(window, cx)),
                )
                .unwrap();

            window
                .update(cx, |showcase, window, cx| {
                    window.focus(&showcase.focus_handle, cx);
                    cx.activate(true);
                })
                .unwrap();
        });
}
