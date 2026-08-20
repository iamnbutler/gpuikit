//! Calendar
//!
//! A six-by-seven month grid of selectable days. Single selection, month
//! navigation, muted leading and trailing days so the grid never changes
//! height, a `today` marker the caller passes in, and a disabled-day
//! *predicate* rather than a list.
//!
//! The date type is [`crate::date::Date`] — three integers and the arithmetic a
//! grid needs — rather than `chrono`, `time` or `jiff`. See that module for the
//! argument.
//!
//! An entity rather than a `RenderOnce` struct, because the grid holds three
//! things across frames a caller should not have to: the visible month, the
//! keyboard-focused day, and its [`FocusHandle`]. The visible month is
//! symmetric — [`Calendar::set_visible_month`] in, [`CalendarEvent::MonthChanged`]
//! out — because a calendar that keeps it privately cannot be told to follow a
//! date typed into a field beside it, which is the one requirement a date
//! picker built on this would impose.
//!
//! # Accessibility
//!
//! [`Role::Grid`], named by the visible month, holding [`Role::Row`]s of
//! [`Role::ColumnHeader`] and [`Role::GridCell`]. **The grid is the one tab
//! stop**: it takes focus, and the focused day claims `active_descendant`.
//!
//! That arrangement is the *valid* one, and the distinction is invisible in a
//! diff so it is written down here: [`A11y::active_descendant`] is honoured
//! only while a focused **ancestor** of the claiming item is on the node stack.
//! The grid is the day cell's ancestor and holds real focus, so this claim
//! lands. The arrangement it is easy to confuse this with — focus on a field,
//! claim on a row in a popup *beside* it — is a sibling relationship, and gpui
//! drops that claim in silence. `src/elements/combobox.rs` and
//! `src/elements/command.rs` are both in the second case and both decline it.
//!
//! [`A11y`] models no grid-index fields — gpui's `div` has `aria_row_index` and
//! friends, but `a11y::tests::no_element_calls_gpuis_a11y_builders_directly`
//! fails the build for an element that reaches past the convention — so a cell
//! carries its full date as its name instead. Adding the four fields to `A11y`
//! is the follow-up.
//!
//! # The keyboard
//!
//! **Actions, not an `on_key_down`.** gpui dispatches bound actions before
//! key-down listeners, so a raw handler loses any key an enclosing element has
//! bound — a calendar in a dialog would give Escape, and could give any of
//! these, to the dialog. `docs/menus-and-listboxes.md` §3 says a new popup in
//! either family copies the listbox; this copies it.
//!
//! | Key | What it does |
//! | --- | --- |
//! | Left / Right | Move the focused day by one day |
//! | Up / Down | Move the focused day by one week |
//! | Home / End | The first / last day of that week |
//! | PageUp / PageDown | The same day one month back / on |
//! | Shift-PageUp / Shift-PageDown | The same day one year back / on |
//! | Enter, Space | Select the focused day |
//!
//! Moving off the visible month brings that month into view rather than
//! refusing, and emits exactly one [`CalendarEvent::MonthChanged`].
//!
//! # What range selection would change
//!
//! Not built, and not stubbed. `selected: Option<Date>` would become an enum of
//! a day and a half-open interval, `CalendarEvent::Selected` would carry it,
//! and every cell's fill would have three states rather than two. That is a
//! breaking change to make deliberately, not a field to bolt on.

use crate::a11y::{A11y, Announce};
use crate::date::{month_name, Date, Weekday};
use crate::element_id::scoped;
use crate::icons::Icons;
use crate::theme::{focus_ring, ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use gpui::{
    actions, div, prelude::*, App, Context, ElementId, EventEmitter, FocusHandle, Focusable,
    IntoElement, KeyBinding, ParentElement, Rems, Render, Role, SharedString, Styled, Window,
};
use std::rc::Rc;

actions!(
    calendar,
    [
        /// Move the keyboard focus one day later.
        FocusNextDay,
        /// Move the keyboard focus one day earlier.
        FocusPreviousDay,
        /// Move the keyboard focus one week later.
        FocusNextWeek,
        /// Move the keyboard focus one week earlier.
        FocusPreviousWeek,
        /// Move the keyboard focus to the first day of its week.
        FocusWeekStart,
        /// Move the keyboard focus to the last day of its week.
        FocusWeekEnd,
        /// Move the keyboard focus one month on.
        FocusNextMonth,
        /// Move the keyboard focus one month back.
        FocusPreviousMonth,
        /// Move the keyboard focus one year on.
        FocusNextYear,
        /// Move the keyboard focus one year back.
        FocusPreviousYear,
        /// Select the day the keyboard is on.
        SelectFocusedDay,
    ]
);

/// The key context the grid declares, and the one [`bind_calendar_keys`]
/// scopes its bindings to.
///
/// Public because the bindings are: an app assembling its own keymap needs both
/// halves.
pub const CALENDAR_CONTEXT: &str = "Calendar";

/// Bind the calendar grid's keys.
///
/// [`crate::init`] calls this, so an app that calls `gpuikit::init` gets the
/// keyboard model for free. Every binding is scoped to [`CALENDAR_CONTEXT`], so
/// none of them is reachable while no calendar has focus — which is also what
/// lets a calendar inside a `Dialog` keep these keys for itself, the way
/// `bind_select_keys` does for the listbox.
pub fn bind_calendar_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("right", FocusNextDay, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("left", FocusPreviousDay, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("down", FocusNextWeek, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("up", FocusPreviousWeek, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("home", FocusWeekStart, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("end", FocusWeekEnd, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("pagedown", FocusNextMonth, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("pageup", FocusPreviousMonth, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("shift-pagedown", FocusNextYear, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("shift-pageup", FocusPreviousYear, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("enter", SelectFocusedDay, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("space", SelectFocusedDay, Some(CALENDAR_CONTEXT)),
    ]);
}

/// A day cell is square, and its side is the control rung's height plus a
/// little — the one number that is this component's own shape rather than the
/// scale's. Keyed off the rung so the three sizes stay proportional; see the
/// "what belongs here" note at the top of `src/theme/control.rs`.
const CELL_RATIO: f32 = 1.15;

/// Six rows of seven, always. A month that fits in five still draws six, so the
/// grid does not change height as the user pages through the year.
const WEEKS: usize = 6;

/// Days in a week, and columns in the grid.
const DAYS_IN_WEEK: usize = 7;

/// What a calendar tells its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarEvent {
    /// A day was chosen.
    Selected(Date),
    /// The visible month changed, carrying that month's first day.
    MonthChanged(Date),
}

/// A month grid of selectable days.
///
/// Built with [`Calendar::new`] inside `cx.new`, because it owns focus and the
/// month it is showing:
///
/// ```ignore
/// let calendar = cx.new(|cx| {
///     Calendar::new("calendar", Date::new(2026, 8, 1).unwrap(), cx)
///         .today(Date::new(2026, 8, 20).unwrap())
///         .selected(Date::new(2026, 8, 20))
/// });
/// ```
pub struct Calendar {
    id: ElementId,
    /// The first day of the month on screen. A `Date` rather than a
    /// `(year, month)` pair so that every movement is the same arithmetic.
    visible_month: Date,
    /// The chosen day, if any.
    selected: Option<Date>,
    /// The day the caller says is today, so this component never reads a clock.
    today: Option<Date>,
    /// Where the keyboard has got to. Kept as a date and re-derived, rather
    /// than paged back and forth through `add_months`, which clamps and does
    /// not round-trip.
    focused_day: Date,
    first_day_of_week: Weekday,
    disabled_days: Option<Rc<dyn Fn(Date) -> bool>>,
    month_labels: Option<[SharedString; 12]>,
    weekday_labels: Option<[SharedString; 7]>,
    disabled: bool,
    size: ControlSize,
    focus_handle: FocusHandle,
}

impl EventEmitter<CalendarEvent> for Calendar {}

impl Focusable for Calendar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Calendar {
    /// A calendar showing `anchor`'s month, with nothing chosen.
    ///
    /// `anchor` is also where the keyboard starts. There is no `Date::today`,
    /// so the caller decides what month a fresh calendar opens on — see
    /// `crate::date`.
    pub fn new(id: impl Into<ElementId>, anchor: Date, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            visible_month: anchor.first_of_month(),
            selected: None,
            today: None,
            focused_day: anchor,
            first_day_of_week: Weekday::Sunday,
            disabled_days: None,
            month_labels: None,
            weekday_labels: None,
            disabled: false,
            size: ControlSize::default(),
            focus_handle: cx.focus_handle(),
        }
    }

    /// Set the chosen day. The grid moves to it and the keyboard follows.
    pub fn selected(mut self, selected: Option<Date>) -> Self {
        if let Some(date) = selected {
            self.visible_month = date.first_of_month();
            self.focused_day = date;
        }
        self.selected = selected;
        self
    }

    /// Set which day is marked as today.
    ///
    /// Passed in rather than read from a clock: a UI toolkit that reads the
    /// system clock cannot be tested and cannot be told about the user's zone.
    pub fn today(mut self, today: Date) -> Self {
        self.today = Some(today);
        self
    }

    /// A predicate deciding which days cannot be chosen.
    ///
    /// A predicate rather than a list, because "weekends", "before today" and
    /// "not in this quarter" are all one line and none of them enumerates.
    pub fn disabled_days(mut self, predicate: impl Fn(Date) -> bool + 'static) -> Self {
        self.disabled_days = Some(Rc::new(predicate));
        self
    }

    /// Which weekday the grid's first column is.
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.first_day_of_week = day;
        self
    }

    /// Month names, January first — the localisation parameter.
    pub fn month_labels(mut self, labels: [impl Into<SharedString>; 12]) -> Self {
        self.month_labels = Some(labels.map(Into::into));
        self
    }

    /// Weekday headings, Sunday first, whatever `first_day_of_week` is: the
    /// grid rotates them itself, so a caller writes them down once.
    pub fn weekday_labels(mut self, labels: [impl Into<SharedString>; 7]) -> Self {
        self.weekday_labels = Some(labels.map(Into::into));
        self
    }

    /// The chosen day, if any.
    pub fn selection(&self) -> Option<Date> {
        self.selected
    }

    /// Set the chosen day from outside, emitting [`CalendarEvent::Selected`]
    /// only when a day was actually given.
    pub fn set_selected(&mut self, selected: Option<Date>, cx: &mut Context<Self>) {
        self.selected = selected;
        if let Some(date) = selected {
            self.show_month(date, cx);
            self.focused_day = date;
        }
        cx.notify();
    }

    /// The first day of the month on screen.
    pub fn visible_month(&self) -> Date {
        self.visible_month
    }

    /// Show the month `date` falls in.
    ///
    /// The in-direction of [`CalendarEvent::MonthChanged`], and the reason it
    /// exists: an owner that has a date from somewhere else — a field the user
    /// typed into — has to be able to bring the grid to it. Emits at most one
    /// `MonthChanged`, and none at all when the month is already showing.
    pub fn set_visible_month(&mut self, date: Date, cx: &mut Context<Self>) {
        self.show_month(date, cx);
        cx.notify();
    }

    fn show_month(&mut self, date: Date, cx: &mut Context<Self>) {
        let first = date.first_of_month();
        if first == self.visible_month {
            return;
        }
        self.visible_month = first;
        cx.emit(CalendarEvent::MonthChanged(first));
    }

    /// The 42 days the grid draws, in order.
    ///
    /// Always 42 — six rows of seven — starting on the [`first_day_of_week`]
    /// on or before the first of the visible month, so the grid never changes
    /// height and a date's column is its weekday's offset.
    ///
    /// [`first_day_of_week`]: Calendar::first_day_of_week
    pub fn days(&self) -> Vec<Date> {
        let first = self.visible_month;
        let lead = first.weekday().days_from(self.first_day_of_week) as i64;
        let start = first.add_days(-lead);
        (0..(WEEKS * DAYS_IN_WEEK) as i64)
            .map(|offset| start.add_days(offset))
            .collect()
    }

    /// Whether `date` is a day this calendar refuses.
    pub fn is_day_disabled(&self, date: Date) -> bool {
        self.disabled
            || self
                .disabled_days
                .as_ref()
                .is_some_and(|predicate| predicate(date))
    }

    /// The grid's own accessible name: the month it is showing.
    fn title(&self) -> SharedString {
        let month = match &self.month_labels {
            Some(labels) => labels[self.visible_month.month() as usize - 1].clone(),
            None => month_name(self.visible_month.month()).into(),
        };
        format!("{month} {}", self.visible_month.year()).into()
    }

    /// The heading for column `column`.
    fn weekday_label(&self, column: usize) -> SharedString {
        let weekday = self.first_day_of_week.week_from()[column];
        match &self.weekday_labels {
            Some(labels) => labels[weekday.index()].clone(),
            None => weekday.min_name().into(),
        }
    }

    /// What one day cell announces.
    ///
    /// A method rather than a call inlined into `render` so a test can read the
    /// same value the element reports: `active_descendant` is applied at paint
    /// time behind gpui's `a11y.is_active()`, which no test here can switch on,
    /// so the declaration is the only thing there is to hold.
    pub(crate) fn day_a11y(&self, date: Date) -> A11y {
        // The full date, not the day number: `A11y` has no grid-index fields,
        // so "20" alone would announce a number out of nowhere.
        let name: SharedString = if date.is_same_month(self.visible_month) {
            format!("{date}").into()
        } else {
            format!("{date}, outside {}", self.title()).into()
        };

        A11y::new(Role::GridCell)
            .name(name)
            .selected(self.selected == Some(date))
            .active_descendant(self.focused_day == date)
    }

    /// Move the keyboard's day, bringing its month into view.
    ///
    /// The single place the focused day moves, so `show_month` cannot be
    /// forgotten on one path and remembered on the others.
    fn focus_day(&mut self, date: Date, cx: &mut Context<Self>) {
        self.focused_day = date;
        self.show_month(date, cx);
        cx.notify();
    }

    fn select(&mut self, date: Date, cx: &mut Context<Self>) {
        if self.is_day_disabled(date) {
            return;
        }
        self.selected = Some(date);
        self.focus_day(date, cx);
        cx.emit(CalendarEvent::Selected(date));
    }

    fn select_focused(&mut self, cx: &mut Context<Self>) {
        let date = self.focused_day;
        self.select(date, cx);
    }

    /// Home and End: the ends of the focused day's week, which depends on where
    /// the week starts.
    fn focus_week_edge(&mut self, last: bool, cx: &mut Context<Self>) {
        let offset = self.focused_day.weekday().days_from(self.first_day_of_week) as i64;
        let target = if last {
            self.focused_day.add_days(6 - offset)
        } else {
            self.focused_day.add_days(-offset)
        };
        self.focus_day(target, cx);
    }

    fn month_button(
        &self,
        part: &'static str,
        name: &'static str,
        delta: i32,
        icon: gpui::Svg,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let fg = theme.fg_muted();
        let hover_bg = theme.button_bg_hover();
        let side = metrics.height;

        div()
            .id(scoped(&self.id, part))
            // Not this crate's `Button`: a `Button`'s label *is* its accessible
            // name (`a11y` §2) and a chevron is not a name. So the name is
            // spelled out, and the focus decision is explicit — silence there
            // is a `debug_assert!`, because `Role::Button` is a control a
            // keyboard operates.
            .announce(
                A11y::new(Role::Button).name(name).not_focusable(
                    "the grid owns the one tab stop; PageUp and PageDown reach this from it",
                ),
            )
            .flex()
            .items_center()
            .justify_center()
            .size(side)
            .rounded(metrics.radius)
            .text_color(fg)
            .cursor_pointer()
            .hover(move |style| style.bg(hover_bg))
            .on_click(cx.listener(move |this, _, _window, cx| {
                let target = this.visible_month.add_months(delta);
                this.show_month(target, cx);
                // The keyboard follows the month rather than being left behind
                // in one nobody is looking at.
                this.focused_day = this.focused_day.add_months(delta);
                cx.notify();
            }))
            .child(icon.size(metrics.text_size))
    }
}

/// The grid is the one tab stop, and the focused day claims the active
/// descendant from inside it — which is the arrangement gpui honours, because
/// the claiming cell's focused *ancestor* is on the node stack. See the module
/// docs.
impl Accessible for Calendar {
    fn a11y(&self) -> A11y {
        A11y::new(Role::Grid)
            .name(self.title())
            .focus_handle(self.focus_handle.clone())
    }
}

impl Disableable for Calendar {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for Calendar {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Render for Calendar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Every colour into a local before `cx.listener` needs `cx` back:
        // `cx.theme()` hands out a borrow of the app, and a `.hover(…)` closure
        // that captured the theme would keep it alive across that borrow.
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let fg = theme.fg();
        let fg_muted = theme.fg_muted();
        let fg_disabled = theme.fg_disabled();
        let accent = theme.accent();
        let on_accent = theme.bg();
        let surface_hover = theme.button_bg_hover();
        let border = theme.border();
        let cell: Rems = metrics.height * CELL_RATIO;

        let a11y = self.a11y();
        let title = self.title();
        let days = self.days();
        let visible_month = self.visible_month;
        let selected = self.selected;
        let today = self.today;
        let focused_day = self.focused_day;
        let focus_handle = self.focus_handle.clone();

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(metrics.gap)
            .child(self.month_button(
                "previous-month",
                "Previous month",
                -1,
                Icons::chevron_left(),
                cx,
            ))
            .child(
                div()
                    .flex_1()
                    .text_size(metrics.text_size)
                    .line_height(metrics.line_height)
                    .text_color(fg)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_center()
                    .child(title.clone()),
            )
            .child(self.month_button(
                "next-month",
                "Next month",
                1,
                Icons::chevron_right(),
                cx,
            ));

        let headings = div()
            .id(scoped(&self.id, "weekdays"))
            .announce(A11y::new(Role::Row))
            .flex()
            .children((0..DAYS_IN_WEEK).map(|column| {
                let label = self.weekday_label(column);
                div()
                    .id(scoped(&self.id, format!("weekday-{column}")))
                    .announce(A11y::new(Role::ColumnHeader).name(label.clone()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(cell)
                    .h(metrics.height)
                    .text_size(metrics.text_size)
                    .text_color(fg_muted)
                    .child(label)
            }));

        let rows = (0..WEEKS).map(|week| {
            let cells = (0..DAYS_IN_WEEK).map(|column| {
                let date = days[week * DAYS_IN_WEEK + column];
                let in_month = date.is_same_month(visible_month);
                let is_selected = selected == Some(date);
                let is_today = today == Some(date);
                let is_focused = focused_day == date;
                let is_disabled = self.is_day_disabled(date);
                let a11y = self.day_a11y(date);

                let color = if is_selected {
                    on_accent
                } else if is_disabled {
                    fg_disabled
                } else if in_month {
                    fg
                } else {
                    fg_muted
                };

                let day_cell = div()
                    .id(scoped(&self.id, format!("day-{}", date)))
                    .announce(a11y)
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(cell)
                    .rounded(metrics.radius)
                    .text_size(metrics.text_size)
                    .line_height(metrics.line_height)
                    .text_color(color)
                    .when(is_selected, |this| this.bg(accent))
                    .when(!is_selected && is_focused, |this| {
                        this.border_1().border_color(accent)
                    })
                    // Today is an underline rather than a second fill: a fill
                    // would be indistinguishable from the selection.
                    .when(is_today && !is_selected, |this| {
                        this.font_weight(gpui::FontWeight::BOLD)
                    })
                    .when(is_disabled, |this| this.cursor_not_allowed())
                    .when(!is_disabled, |this| {
                        this.cursor_pointer()
                            .when(!is_selected, |this| {
                                this.hover(move |style| style.bg(surface_hover))
                            })
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.select(date, cx);
                            }))
                    })
                    .child(format!("{}", date.day()));

                #[cfg(test)]
                let day_cell = day_cell.debug_selector(move || format!("gpuikit-calendar-{date}"));

                day_cell
            });

            div()
                .id(scoped(&self.id, format!("week-{week}")))
                .announce(A11y::new(Role::Row))
                .flex()
                .children(cells)
        });

        div()
            .id(self.id.clone())
            .announce(a11y)
            .key_context(CALENDAR_CONTEXT)
            .track_focus(&focus_handle)
            .focus_visible(|style| style.shadow(focus_ring(accent)))
            .on_action(cx.listener(|this, _: &FocusNextDay, _window, cx| {
                let target = this.focused_day.add_days(1);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusPreviousDay, _window, cx| {
                let target = this.focused_day.add_days(-1);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusNextWeek, _window, cx| {
                let target = this.focused_day.add_days(7);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusPreviousWeek, _window, cx| {
                let target = this.focused_day.add_days(-7);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusWeekStart, _window, cx| {
                this.focus_week_edge(false, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusWeekEnd, _window, cx| {
                this.focus_week_edge(true, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusNextMonth, _window, cx| {
                let target = this.focused_day.add_months(1);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusPreviousMonth, _window, cx| {
                let target = this.focused_day.add_months(-1);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusNextYear, _window, cx| {
                let target = this.focused_day.add_months(12);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusPreviousYear, _window, cx| {
                let target = this.focused_day.add_months(-12);
                this.focus_day(target, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectFocusedDay, _window, cx| {
                this.select_focused(cx);
            }))
            .flex()
            .flex_col()
            .gap(metrics.gap)
            .p(metrics.padding_x)
            .rounded(metrics.radius)
            .border_1()
            .border_color(border)
            .when(self.disabled, |this| this.opacity(0.65))
            .child(header)
            .child(headings)
            .children(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{px, size, Entity, TestAppContext, VisualTestContext};
    use std::cell::RefCell;
    use std::ops::Deref;

    fn date(year: i32, month: u32, day: u32) -> Date {
        Date::new(year, month, day).expect("a date the test wrote by hand exists")
    }

    fn calendar(cx: &mut TestAppContext) -> gpui::Entity<Calendar> {
        cx.update(|cx| cx.new(|cx| Calendar::new("calendar", date(2026, 8, 20), cx)))
    }

    #[gpui::test]
    fn the_grid_is_always_forty_two_days_starting_on_the_first_day_of_week(
        cx: &mut TestAppContext,
    ) {
        let calendar = calendar(cx);

        calendar.update(cx, |this, _cx| {
            let days = this.days();
            assert_eq!(days.len(), 42);
            assert_eq!(days[0].weekday(), Weekday::Sunday);
            // August 2026 starts on a Saturday, so a Sunday-first grid leads
            // with six days of July.
            assert_eq!(days[0], date(2026, 7, 26));
            assert!(days.contains(&date(2026, 8, 1)));
            assert!(days.contains(&date(2026, 8, 31)));
        });

        calendar.update(cx, |this, _cx| {
            this.first_day_of_week = Weekday::Monday;
            let days = this.days();
            assert_eq!(days.len(), 42);
            assert_eq!(days[0].weekday(), Weekday::Monday);
            for (index, day) in days.iter().enumerate() {
                assert_eq!(*day, days[0].add_days(index as i64));
            }
        });
    }

    #[gpui::test]
    fn paging_from_the_thirty_first_clamps_and_changes_the_month_once(cx: &mut TestAppContext) {
        let calendar =
            cx.update(|cx| cx.new(|cx| Calendar::new("calendar", date(2026, 1, 31), cx)));

        let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        {
            let events = events.clone();
            cx.update(|cx| {
                cx.subscribe(&calendar, move |_, event: &CalendarEvent, _| {
                    events.borrow_mut().push(*event);
                })
                .detach();
            });
        }

        calendar.update(cx, |this, cx| {
            let target = this.focused_day.add_months(1);
            this.focus_day(target, cx);
        });
        cx.run_until_parked();

        calendar.update(cx, |this, _cx| {
            assert_eq!(this.focused_day, date(2026, 2, 28));
            assert_eq!(this.visible_month(), date(2026, 2, 1));
        });
        assert_eq!(
            *events.borrow(),
            vec![CalendarEvent::MonthChanged(date(2026, 2, 1))],
            "paging a month emits exactly one MonthChanged"
        );
    }

    #[gpui::test]
    fn a_day_cell_announces_a_grid_cell_with_the_two_states_on_the_right_days(
        cx: &mut TestAppContext,
    ) {
        let calendar = calendar(cx);

        calendar.update(cx, |this, cx| {
            this.set_selected(Some(date(2026, 8, 12)), cx);
            let target = date(2026, 8, 19);
            this.focus_day(target, cx);
        });

        calendar.update(cx, |this, _cx| {
            let chosen = this.day_a11y(date(2026, 8, 12));
            assert_eq!(chosen.role(), Role::GridCell);
            assert_eq!(
                chosen.accessible_name().map(SharedString::to_string),
                Some("2026-08-12".to_string())
            );
            assert!(!chosen.is_active_descendant());

            let focused = this.day_a11y(date(2026, 8, 19));
            assert!(focused.is_active_descendant());

            // Exactly one cell claims it, which is what keeps gpui's
            // two-claims-in-one-frame `debug_assert!` unreachable.
            let claims = this
                .days()
                .into_iter()
                .filter(|date| this.day_a11y(*date).is_active_descendant())
                .count();
            assert_eq!(claims, 1);
        });
    }

    #[gpui::test]
    fn the_month_can_be_set_from_outside_and_says_so_once(cx: &mut TestAppContext) {
        let calendar = calendar(cx);
        let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        {
            let events = events.clone();
            cx.update(|cx| {
                cx.subscribe(&calendar, move |_, event: &CalendarEvent, _| {
                    events.borrow_mut().push(*event);
                })
                .detach();
            });
        }

        calendar.update(cx, |this, cx| {
            this.set_visible_month(date(2026, 11, 4), cx);
            // Already showing: no second event.
            this.set_visible_month(date(2026, 11, 30), cx);
        });
        cx.run_until_parked();

        calendar.update(cx, |this, _cx| {
            assert_eq!(this.visible_month(), date(2026, 11, 1));
        });
        assert_eq!(
            *events.borrow(),
            vec![CalendarEvent::MonthChanged(date(2026, 11, 1))]
        );
    }

    /// A window whose root view is the grid, so a keystroke walks the real
    /// dispatch tree.
    ///
    /// This is the half of the keyboard a unit test cannot reach: a binding
    /// registered in the wrong key context and one registered correctly look
    /// identical until a keystroke has to find it. Same shape as
    /// `elements::select`'s harness, and `crate::init` rather than
    /// `theme::init` for the same reason — `bind_calendar_keys` lives in
    /// `init`, so a test that only initialised the theme would be testing a
    /// grid that answers nothing.
    struct GridView {
        calendar: Entity<Calendar>,
    }

    impl Render for GridView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(self.calendar.clone())
        }
    }

    /// One drawn, focused calendar and the events it has emitted since.
    struct Drawn {
        calendar: Entity<Calendar>,
        cx: &'static mut VisualTestContext,
        events: Rc<RefCell<Vec<CalendarEvent>>>,
    }

    fn draw(cx: &mut TestAppContext, anchor: Date) -> Drawn {
        cx.update(crate::init);

        let window = cx.open_window(size(px(480.), px(480.)), move |_window, cx| {
            let calendar = cx.new(|cx| Calendar::new("calendar", anchor, cx).today(anchor));
            GridView { calendar }
        });

        let calendar = window
            .read_with(cx, |view, _cx| view.calendar.clone())
            .expect("the window's root view is the grid view");

        let events = Rc::new(RefCell::new(Vec::new()));
        {
            let events = events.clone();
            cx.update(|cx| {
                cx.subscribe(&calendar, move |_, event: &CalendarEvent, _| {
                    events.borrow_mut().push(*event);
                })
                .detach();
            });
        }

        let cx = VisualTestContext::from_window(*window.deref(), cx).into_mut();
        cx.run_until_parked();

        // The grid is the one tab stop, and nothing else in this window is
        // focusable, so focus it the way an app would.
        let handle = calendar.read_with(cx, |this, _cx| this.focus_handle.clone());
        cx.update(|window, cx| window.focus(&handle, cx));
        cx.run_until_parked();

        Drawn {
            calendar,
            cx,
            events,
        }
    }

    impl Drawn {
        fn press(&mut self, keys: &str) {
            self.cx.simulate_keystrokes(keys);
            self.cx.run_until_parked();
        }

        fn focused_day(&mut self) -> Date {
            self.calendar.read_with(self.cx, |this, _cx| this.focused_day)
        }

        fn visible_month(&mut self) -> Date {
            self.calendar
                .read_with(self.cx, |this, _cx| this.visible_month())
        }

        fn selection(&mut self) -> Option<Date> {
            self.calendar.read_with(self.cx, |this, _cx| this.selection())
        }
    }

    #[gpui::test]
    fn the_bound_actions_move_the_focused_day_and_select_it(cx: &mut TestAppContext) {
        // Thursday 2026-08-20, in a Sunday-first grid.
        let mut drawn = draw(cx, date(2026, 8, 20));

        drawn.press("right");
        assert_eq!(drawn.focused_day(), date(2026, 8, 21));

        drawn.press("left left");
        assert_eq!(drawn.focused_day(), date(2026, 8, 19));

        drawn.press("down");
        assert_eq!(drawn.focused_day(), date(2026, 8, 26));

        drawn.press("up up");
        assert_eq!(drawn.focused_day(), date(2026, 8, 12));

        // Home and End are the ends of *that* week, which for a Sunday-first
        // grid holding Wednesday the 12th is the 9th and the 15th.
        drawn.press("home");
        assert_eq!(drawn.focused_day(), date(2026, 8, 9));
        drawn.press("end");
        assert_eq!(drawn.focused_day(), date(2026, 8, 15));

        assert_eq!(drawn.selection(), None, "moving does not choose");
        drawn.press("enter");
        assert_eq!(drawn.selection(), Some(date(2026, 8, 15)));

        drawn.press("right space");
        assert_eq!(drawn.selection(), Some(date(2026, 8, 16)));

        assert!(
            drawn
                .events
                .borrow()
                .iter()
                .all(|event| !matches!(event, CalendarEvent::MonthChanged(_))),
            "none of that left August"
        );
    }

    #[gpui::test]
    fn pagedown_from_the_thirty_first_clamps_and_brings_the_month_into_view(
        cx: &mut TestAppContext,
    ) {
        let mut drawn = draw(cx, date(2026, 1, 31));

        drawn.press("pagedown");

        assert_eq!(drawn.focused_day(), date(2026, 2, 28), "February clamps");
        assert_eq!(drawn.visible_month(), date(2026, 2, 1));
        assert_eq!(
            *drawn.events.borrow(),
            vec![CalendarEvent::MonthChanged(date(2026, 2, 1))],
            "one keystroke, one MonthChanged"
        );

        // Shift pages by a year, and the focused day is kept rather than paged
        // back and forth through `add_months`, which clamps and does not round
        // trip.
        drawn.press("shift-pagedown");
        assert_eq!(drawn.focused_day(), date(2027, 2, 28));
        assert_eq!(drawn.visible_month(), date(2027, 2, 1));

        drawn.press("pageup");
        assert_eq!(drawn.focused_day(), date(2027, 1, 28));
    }

    #[gpui::test]
    fn a_disabled_day_cannot_be_chosen(cx: &mut TestAppContext) {
        let calendar = cx.update(|cx| {
            cx.new(|cx| {
                Calendar::new("calendar", date(2026, 8, 20), cx)
                    .disabled_days(|date| date.weekday() == Weekday::Sunday)
            })
        });

        calendar.update(cx, |this, cx| {
            assert!(this.is_day_disabled(date(2026, 8, 16)));
            this.select(date(2026, 8, 16), cx);
            assert_eq!(this.selection(), None);
            this.select(date(2026, 8, 17), cx);
            assert_eq!(this.selection(), Some(date(2026, 8, 17)));
        });
    }
}
