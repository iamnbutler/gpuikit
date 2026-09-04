//! Cross-element tests for the shared control size scale.
//!
//! These live in their own file rather than in any one element's because the
//! property they assert is not any one element's: a row of controls has to be
//! one height, and no element can check that from inside itself. They render
//! a row of controls that are on the scale into a real test window and read
//! the laid-out height of each box back.
//!
//! "On the scale" is narrower than "in the crate", and deliberately so — see
//! [`CONTROLS`] and [`every_sized_control_on_a_row_is_the_same_height`] for
//! which elements are left out and why.

use gpui::{Context, Entity, Pixels, Render, TestAppContext, Window, div, prelude::*, px};

use crate::elements::badge::badge;
use crate::elements::button::button;
use crate::elements::checkbox::{Checkbox, checkbox};
use crate::elements::icon_button::icon_button;
use crate::elements::kbd::kbd;
use crate::elements::radio_group::{RadioGroup, radio_group, radio_option};
use crate::elements::select::{SelectState, select};
use crate::elements::slider::{Slider, slider};
use crate::elements::switch::{Switch, switch};
use crate::elements::tabs::{Tabs, tab, tabs};
use crate::elements::text_field::text_field;
use crate::elements::toggle::{Toggle, toggle};
use crate::elements::toggle_group::{ToggleGroup, toggle_group, toggle_option};
use crate::icons::Icons;
use crate::input::InputState;
use crate::layout::h_stack;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::orientable::{Orientable, Orientation};

/// The controls the row draws, in the order it draws them. Each name is also
/// the `debug_selector` its box is measured through.
///
/// Thirteen of the crate's twenty-one `ControlSized` implementors. The other
/// eight, and the two elements that are not on the scale at all, are
/// enumerated on [`every_sized_control_on_a_row_is_the_same_height`].
const CONTROLS: &[&str] = &[
    "button",
    "icon-button",
    "badge",
    "kbd",
    "checkbox",
    "switch",
    "toggle",
    "select",
    "text-field",
    "tabs",
    "toggle-group",
    "radio-group",
    "slider",
];

/// A toolbar: one of every control *on the scale* that can share a row, on
/// one row. Not one of every control in the crate — see [`CONTROLS`].
struct Toolbar {
    size: ControlSize,
    checkbox: Entity<Checkbox>,
    switch: Entity<Switch>,
    toggle: Entity<Toggle>,
    select: Entity<SelectState<u8>>,
    field: Entity<InputState>,
    tabs: Entity<Tabs>,
    toggle_group: Entity<ToggleGroup<u8>>,
    radio_group: Entity<RadioGroup<u8>>,
    slider: Entity<Slider>,
}

impl Toolbar {
    fn new(size: ControlSize, cx: &mut Context<Self>) -> Self {
        Self {
            size,
            checkbox: cx.new(|_cx| checkbox("cb", true).control_size(size)),
            switch: cx.new(|_cx| switch("sw", true).control_size(size)),
            toggle: cx.new(|_cx| toggle("tg", true).control_size(size)),
            select: cx.new(|_cx| {
                SelectState::new(
                    select("sel", "Number", vec![(0u8, "Zero"), (1u8, "One")])
                        .selected(0u8)
                        .control_size(size),
                )
            }),
            field: cx.new(InputState::new_singleline),
            tabs: cx.new(|_cx| {
                tabs("tabs")
                    .tab(tab("one", "One"))
                    .tab(tab("two", "Two"))
                    .control_size(size)
            }),
            toggle_group: cx.new(|_cx| {
                toggle_group(
                    "tg-group",
                    vec![toggle_option(0u8, "Left"), toggle_option(1u8, "Right")],
                )
                .selected_value(0u8)
                .control_size(size)
            }),
            // Horizontal, because that is the form that shares a row: a
            // `RadioGroup` defaults to `Orientation::Vertical`, which is a
            // stack of rungs rather than one, and has no single height to
            // agree with — the same reason `Textarea` is not in this row.
            radio_group: cx.new(|_cx| {
                radio_group(
                    "rg",
                    vec![radio_option(0u8, "Zero"), radio_option(1u8, "One")],
                )
                .selected(0u8)
                .orientation(Orientation::Horizontal)
                .control_size(size)
            }),
            // No label and no value, so the slider is the one track row a
            // toolbar would put it on. With either it is two rows and has no
            // single height to agree with, the same as `Textarea`.
            slider: cx.new(|_cx| {
                slider("slider", 50., 0.0..=100.)
                    .show_value(false)
                    .control_size(size)
            }),
        }
    }
}

/// Wrap a control in a measurable box that adds nothing of its own.
fn measured(name: &'static str, control: impl IntoElement) -> impl IntoElement {
    div()
        .flex_none()
        .debug_selector(move || name.to_string())
        .child(control)
}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = self.size;

        h_stack()
            .gap_2()
            // Flex defaults to `align-items: stretch`, which would give every
            // control the row's height and make every assertion below pass
            // against anything. This is the same trap as eyeballing a real
            // toolbar: a row can *look* aligned because flex stretched it.
            .items_start()
            .child(measured("button", button("b", "Button").control_size(size)))
            .child(measured(
                "icon-button",
                icon_button("ib", Icons::star()).control_size(size),
            ))
            .child(measured("badge", badge("Badge").control_size(size)))
            .child(measured("kbd", kbd("K").control_size(size)))
            .child(measured("checkbox", self.checkbox.clone()))
            .child(measured("switch", self.switch.clone()))
            .child(measured("toggle", self.toggle.clone()))
            .child(measured("select", self.select.clone()))
            .child(measured(
                "text-field",
                text_field(&self.field, cx)
                    .placeholder("Field")
                    .control_size(size),
            ))
            .child(measured("tabs", self.tabs.clone()))
            .child(measured("toggle-group", self.toggle_group.clone()))
            .child(measured("radio-group", self.radio_group.clone()))
            // A slider is `w_full`, and `measured` is `flex_none`, so without
            // a width it collapses and the thumb has nowhere to sit. The width
            // is arbitrary; only the height is asserted.
            .child(measured(
                "slider",
                div().w(px(120.)).child(self.slider.clone()),
            ))
    }
}

/// Draw the toolbar at `size` and report the laid-out height of every control.
fn measure_row(cx: &mut TestAppContext, size: ControlSize) -> Vec<(&'static str, Pixels)> {
    cx.update(crate::theme::init);
    let (_view, cx) = cx.add_window_view(move |_window, cx| Toolbar::new(size, cx));

    CONTROLS
        .iter()
        .map(|name| {
            let bounds = cx
                .debug_bounds(name)
                .unwrap_or_else(|| panic!("`{name}` was never laid out"));
            (*name, bounds.size.height)
        })
        .collect()
}

/// The whole point of the scale, as one assertion — over the thirteen
/// controls [`CONTROLS`] names, which is **not** every control in the crate.
///
/// The name says "sized" rather than "control" because both halves of that
/// gap are real, and neither is visible from the row itself. `grep "impl
/// ControlSized"` finds twenty-one implementors; this row measures thirteen.
/// Someone who adds a fourteenth needs to be able to tell unfinished coverage
/// from a broken scale, and this comment is the only thing that tells them.
///
/// **Two elements are not on the scale at all**, and would fail here for a
/// reason that is not a bug in the scale. Neither mentions `ControlSized` or
/// `control_size` anywhere in its module, and both are deliberate:
///
/// - `elements::alert` — a banner, not a row control. Its height is a
///   `rems(0.75)` padding plus however many lines of message it was given, and
///   there is no rung a two-line alert could sit on.
/// - `elements::progress` — a `px(8.)` bar. Called out in iamnbutler/gpuikit#230
///   as legitimately local under `crate::theme::control`'s "what belongs here"
///   note: the bar is a graphic with no text and no interaction, and the
///   element is being superseded by Gauge (iamnbutler/gpuikit#222) anyway.
///
/// `Tabs`, `ToggleGroup`, `RadioGroup` and `Slider` used to be on that list —
/// they are the four #230 named as the ones that genuinely sit beside a
/// `Button`, and they are in the row above now.
///
/// **Eight more do implement `ControlSized` and are still not measured here**,
/// which is the half that is easy to miss:
///
/// - `Field`, `Input` — reachable through the `text-field` entry above, which
///   is `TextField` wrapping an `Input`; measuring them again would measure
///   the same box twice.
/// - `CheckboxBox` — an internal sub-part, reached through `Checkbox`.
/// - `Textarea` — multi-line by definition, so it has no single row height to
///   agree with; its rung sets its text metrics, not its box. A `Slider` with
///   a label or a value readout is the same shape, which is why the one in the
///   row above has neither.
/// - `Table`, `Sidebar`, `SidebarTrigger` — containers rather than row
///   controls. `SidebarTrigger` is the arguable one: it is an `IconButton` in
///   all but name and could join the row.
/// - `Dialog` — a modal surface, not a row control. Its rung sizes the two
///   action buttons of a confirmation and the gap between them; the panel's
///   own padding is component-specific.
///
/// What is left of iamnbutler/gpuikit#230 after this row grew: `list`'s px
/// constants, `context_menu`'s `px(180.)`/`px(420.)`/`px(14.)` literals, and
/// the icon-slot literals repeated across `context_menu`, `accordion` and
/// `collapsible`. None of the four is a control that shares a row, so none of
/// them can be held by a test shaped like this one; #230's `metrics_coverage`
/// guard is what would.
#[gpui::test]
fn every_sized_control_on_a_row_is_the_same_height(cx: &mut TestAppContext) {
    for size in ControlSize::ALL {
        let measured = measure_row(cx, size);
        let expected = cx.update(|cx| cx.theme().control(size).height.to_pixels(px(16.)));

        let off_rung: Vec<String> = measured
            .iter()
            .filter(|(_, height)| (f32::from(*height) - f32::from(expected)).abs() > 0.5)
            .map(|(name, height)| format!("  {name}: {height:?}"))
            .collect();

        assert!(
            off_rung.is_empty(),
            "on the {} rung ({expected:?}), these controls are not:\n{}\n\nthe whole row \
             measured:\n{}",
            size.name(),
            off_rung.join("\n"),
            measured
                .iter()
                .map(|(name, height)| format!("  {name}: {height:?}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
}

/// The rungs have to actually differ, or the assertion above is vacuous — a
/// scale whose three rungs are the same height would satisfy it perfectly.
#[gpui::test]
fn the_rungs_are_16_20_and_24_pixels(cx: &mut TestAppContext) {
    let heights: Vec<Pixels> = ControlSize::ALL
        .into_iter()
        .map(|size| {
            measure_row(cx, size)
                .first()
                .expect("the row draws at least one control")
                .1
        })
        .collect();

    assert_eq!(heights, vec![px(16.), px(20.), px(24.)]);
}

/// iamnbutler/tasks#919: a single-line `input()` has no intrinsic size, so an
/// `Auto` height used to resolve to zero and the field was invisible until
/// something outside it happened to set a height.
#[gpui::test]
fn a_single_line_input_is_never_zero_height(cx: &mut TestAppContext) {
    struct BareInput {
        state: Entity<InputState>,
    }

    impl Render for BareInput {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            // A column, so the input keeps its own height rather than being
            // stretched to the window's.
            div().flex().flex_col().child(
                div()
                    .debug_selector(|| "bare-input".to_string())
                    .child(crate::elements::input::input(&self.state, cx)),
            )
        }
    }

    cx.update(crate::theme::init);
    let (_view, cx) = cx.add_window_view(|_window, cx| BareInput {
        state: cx.new(InputState::new_singleline),
    });

    let height = cx
        .debug_bounds("bare-input")
        .expect("the input was never laid out")
        .size
        .height;

    assert_eq!(height, px(20.), "a single-line input collapsed to nothing");
}

/// A height assertion cannot see this one: `ink` is how much of its box a
/// control's graphic fills, and a track exactly the height of the row reads as
/// an overflow however well the row lines up.
#[gpui::test]
fn a_switch_track_is_shorter_than_its_box(cx: &mut TestAppContext) {
    struct TrackOnly {
        switch: Entity<Switch>,
    }

    impl Render for TrackOnly {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            h_stack().items_start().child(
                div()
                    .debug_selector(|| "switch-box".to_string())
                    .child(self.switch.clone()),
            )
        }
    }

    cx.update(crate::theme::init);
    let expected_track = cx.update(|cx| {
        cx.theme()
            .control(ControlSize::Medium)
            .track()
            .height
            .to_pixels(px(16.))
    });
    let (_view, cx) = cx.add_window_view(|_window, cx| TrackOnly {
        switch: cx.new(|_cx| switch("sw", true)),
    });

    let box_height = cx
        .debug_bounds("switch-box")
        .expect("the switch was never laid out")
        .size
        .height;

    assert!(
        expected_track < box_height,
        "the switch's {expected_track:?} track is not shorter than its {box_height:?} box"
    );
}
