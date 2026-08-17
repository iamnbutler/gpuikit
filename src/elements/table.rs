//! A table: a column model, a header that stays put over a body that scrolls,
//! and — behind explicit opt-ins — sortable headers and row selection.
//!
//! This is one element, not two. `docs/issues/data-table.md` recommended
//! against shipping a separate "data table": shadcn splits them because its
//! `Table` is unstyled markup and its `DataTable` is a TanStack recipe, which
//! is an artifact of that ecosystem rather than a design. Primer ships one
//! `DataTable`. A second module here would mean a second showcase page, a
//! second `ELEMENT_COVERAGE` row and a permanent question about which one to
//! reach for. Sorting and selection are therefore properties of this element,
//! off unless asked for.
//!
//! # The state lives with the caller
//!
//! The part of TanStack's headless model worth taking is that the component
//! does not own the data view. This element is handed rows that are *already
//! filtered and already sorted*, plus the [`SortDescriptor`] describing how,
//! and it reports [`SortRequest`] / [`SelectRequest`] / [`SelectAllRequest`]
//! back. Sorting inside the element would mean owning comparison for arbitrary
//! cell types, which this crate should not do.
//!
//! The consequence is worth stating plainly: **nothing moves until the caller
//! moves it.** A `sortable()` column with no [`Table::on_sort`] handler is
//! inert on purpose — no pointer, no hover, no click.
//!
//! # Filtering is a `TextField` above the table
//!
//! It is not a table feature and there is no filter input inside this element.
//! A filter is a predicate over the caller's own rows, and the caller already
//! has to re-derive its rows for sorting. `examples/showcase.rs`'s Table page
//! demonstrates the intended shape.
//!
//! ```ignore
//! use gpuikit::elements::table::{column, row, table, SortRequest};
//!
//! table("repositories")
//!     .column(column("Repository", |repo: &Repo, _, _| {
//!         div().child(repo.name.clone()).into_any_element()
//!     }).sortable())
//!     .column(column("Stars", |repo: &Repo, _, _| {
//!         div().child(repo.stars.to_string()).into_any_element()
//!     }).end())
//!     .rows(visible_rows.iter().cloned().map(row))
//!     .sorted_by(current_sort)
//!     .on_sort(move |request, _, _| { /* re-sort your own rows */ })
//!     .max_h(px(320.))
//! ```
//!
//! # Accessibility
//!
//! No roles are reported, because `docs/issues/element-roles-convention.md`
//! has not landed and this element must not invent a mechanism. When it does,
//! this element needs `Grid` with row and column counts, `Row` with
//! `aria_selected`, `ColumnHeader` with its sort direction, and `Cell`. Two
//! findings for that issue to decide about, both discovered here:
//!
//! - **gpui has no `aria_sort`.** `accesskit::Node::set_sort_direction`
//!   exists, but `div`'s builders stop at `aria_selected` / `aria_row_index` /
//!   `aria_column_count`, so a sorted `ColumnHeader` cannot report its
//!   direction without a hand-written `Element`.
//! - **A role needs an id.** `role()` lives on `StatefulInteractiveElement`,
//!   so reporting one turns body cells — which have no id today — into
//!   id-minting sites, which puts them inside `src/element_id.rs`'s duplicate
//!   id trap. Read that module before adding a role to anything here.
//!
//! # Not built, deliberately
//!
//! Row virtualisation (`docs/issues/table.md` puts it out of scope and points
//! at `src/elements/list.rs` for when it is needed), column resizing, column
//! visibility toggles, and multi-column sort.

use std::rc::Rc;

use gpui::{
    div, prelude::*, px, AnyElement, App, Div, ElementId, FontWeight, IntoElement, Length,
    MouseButton, ParentElement, RenderOnce, SharedString, Styled, Window,
};

use crate::element_id;
use crate::elements::checkbox::{checkbox_box, CheckState};
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;

/// How a cell is drawn from the caller's row type.
///
/// Named rather than written inline because it appears in `Column`'s field, in
/// its constructor and in `column()`, and because the shape — the row by
/// reference, a `Window` and an `App` — is the whole contract between the
/// element and the caller.
type CellRenderer<R> = Rc<dyn Fn(&R, &mut Window, &mut App) -> AnyElement>;

/// What activating a row does.
type ActivateHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// What a click on a sortable header reports to.
type SortHandler = Rc<dyn Fn(&SortRequest, &mut Window, &mut App)>;

/// What a click on a row's selection checkbox reports to.
type SelectHandler = Rc<dyn Fn(&SelectRequest, &mut Window, &mut App)>;

/// What a click on the header's select-all checkbox reports to.
type SelectAllHandler = Rc<dyn Fn(&SelectAllRequest, &mut Window, &mut App)>;

/// How a column takes its width.
///
/// There is deliberately **no content-sized arm**, which is the one thing
/// `docs/issues/table.md` asked for that this element does not do. The header
/// sits outside the scrolled body, so the two are separate flex containers,
/// and worse, every row is its own flex container as well. A cell sized to its
/// own content is therefore measured per row: column two would be one width in
/// row one and another in row two, and the header would agree with neither.
/// Sizing a column to the widest cell *in the column* needs a measurement pass
/// across rows, which flex cannot do and which gpui's grid — uniform
/// `repeat(n, minmax(_, 1fr))` tracks, no per-column template — cannot express
/// either. A hand-written `Element` that lays cells out at `MinContent` /
/// `MaxContent`, takes the per-column maximum and re-lays could; that is real
/// work and belongs to the Table issue rather than being faked here.
/// [`Column::min_width`] recovers most of the use soundly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnWidth {
    /// Share the leftover space in proportion to this factor.
    ///
    /// The basis is zero rather than the content's own size — see the note on
    /// this type for why it has to be.
    Flex(f32),
    /// Exactly this wide, in whatever unit `Length` carries.
    Fixed(Length),
}

impl Default for ColumnWidth {
    fn default() -> Self {
        ColumnWidth::Flex(1.0)
    }
}

/// Horizontal alignment of a column's cells, header included.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CellAlign {
    /// Against the leading edge. The default.
    #[default]
    Start,
    /// Centred.
    Center,
    /// Against the trailing edge — numbers, usually.
    End,
}

/// One column: a header, a width, an alignment, and how to draw a cell.
///
/// The render closure is handed the caller's row type and returns any element,
/// so a cell can be a `Badge`, a `Button`, an avatar and a name — anything.
/// The table never looks inside a row.
pub struct Column<R> {
    header: SharedString,
    width: ColumnWidth,
    min_width: Option<Length>,
    align: CellAlign,
    sortable: bool,
    render: CellRenderer<R>,
}

impl<R> Clone for Column<R> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            width: self.width,
            min_width: self.min_width,
            align: self.align,
            sortable: self.sortable,
            render: self.render.clone(),
        }
    }
}

impl<R> Column<R> {
    /// A column with the given header, drawing its cells with `render`.
    pub fn new(
        header: impl Into<SharedString>,
        render: impl Fn(&R, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            header: header.into(),
            width: ColumnWidth::default(),
            min_width: None,
            align: CellAlign::default(),
            sortable: false,
            render: Rc::new(render),
        }
    }

    /// Sets how this column takes its width.
    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    /// Takes leftover space in proportion to `grow`.
    pub fn flex(self, grow: f32) -> Self {
        self.width(ColumnWidth::Flex(grow))
    }

    /// Exactly this wide.
    pub fn fixed(self, width: impl Into<Length>) -> Self {
        self.width(ColumnWidth::Fixed(width.into()))
    }

    /// A floor under the column's width. The usable half of the missing
    /// content-sized arm: a flexible column that will not collapse.
    pub fn min_width(mut self, min_width: impl Into<Length>) -> Self {
        self.min_width = Some(min_width.into());
        self
    }

    /// Sets the horizontal alignment of this column's cells and header.
    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }

    /// Centres this column.
    pub fn center(self) -> Self {
        self.align(CellAlign::Center)
    }

    /// Aligns this column to its trailing edge.
    pub fn end(self) -> Self {
        self.align(CellAlign::End)
    }

    /// Marks this column's header clickable, *provided* the table has an
    /// [`Table::on_sort`] handler. Without one the header stays inert: a
    /// header that looks clickable and sorts nothing is worse than a plain
    /// one.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Whether this column asked to be sortable.
    pub fn is_sortable(&self) -> bool {
        self.sortable
    }
}

/// Convenience function to create a column.
pub fn column<R>(
    header: impl Into<SharedString>,
    render: impl Fn(&R, &mut Window, &mut App) -> AnyElement + 'static,
) -> Column<R> {
    Column::new(header, render)
}

/// One row: the caller's data, whether the caller considers it selected, and
/// optionally what activating it does.
///
/// Activation ("open this row") is a different act from selection ("include
/// this row in the next operation"), which is why they are different hooks.
pub struct Row<R> {
    data: R,
    selected: bool,
    on_click: Option<ActivateHandler>,
}

impl<R> Row<R> {
    /// A row holding `data`, unselected and inert.
    pub fn new(data: R) -> Self {
        Self {
            data,
            selected: false,
            on_click: None,
        }
    }

    /// Marks the row selected. The table draws it; the caller owns it.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Activates the row on click — opening it, usually. Independent of
    /// selection: a click on the selection checkbox does not activate.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Whether the caller marked this row selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

/// Convenience function to create a row.
pub fn row<R>(data: R) -> Row<R> {
    Row::new(data)
}

/// Which way a sorted column is sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Smallest first.
    Ascending,
    /// Largest first.
    Descending,
}

impl SortDirection {
    /// The other one.
    pub fn reversed(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    /// The arrow drawn beside a sorted header.
    pub fn indicator(self) -> &'static str {
        match self {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    }
}

/// How the rows handed to the table are already sorted.
///
/// One column, because nothing here needs more; `sorted_by` takes an `Option`,
/// so widening this later does not change the call shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortDescriptor {
    /// Index into the table's columns.
    pub column: usize,
    /// Which way that column is sorted.
    pub direction: SortDirection,
}

impl SortDescriptor {
    /// A descriptor for `column`, sorted `direction`.
    pub fn new(column: usize, direction: SortDirection) -> Self {
        Self { column, direction }
    }
}

/// "The user clicked this column's header."
///
/// The table asks; the caller decides. [`SortRequest::suggested`] is the
/// conventional answer and nothing more — a caller with a column that should
/// start descending, or that should cycle back to unsorted, is expected to
/// ignore it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortRequest {
    /// Index of the column whose header was clicked.
    pub column: usize,
    /// What the table was sorted by when it was clicked.
    pub current: Option<SortDescriptor>,
}

impl SortRequest {
    /// The conventional toggle: reverse the column that is already sorted,
    /// start any other one ascending.
    pub fn next(current: Option<SortDescriptor>, column: usize) -> SortDescriptor {
        match current {
            Some(descriptor) if descriptor.column == column => {
                SortDescriptor::new(column, descriptor.direction.reversed())
            }
            _ => SortDescriptor::new(column, SortDirection::Ascending),
        }
    }

    /// [`SortRequest::next`] applied to this request. A suggestion.
    pub fn suggested(&self) -> SortDescriptor {
        Self::next(self.current, self.column)
    }
}

/// "The user asked for this row's selection to change."
///
/// The row is an **index** into the rows the caller handed in, in the caller's
/// order — the element cannot key an arbitrary `R`, so resolving the index
/// against your own data and storing your own id is the caller's half of the
/// round trip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectRequest {
    /// Index of the row, in the order the rows were handed to the table.
    pub row: usize,
    /// The state being asked for.
    pub selected: bool,
}

/// "The user asked for every row to be selected, or none of them."
///
/// Only sent by a table that was given an [`Table::on_select_all`] handler:
/// "all" is only meaningful where the caller's table has all of the rows, and
/// only the caller knows whether it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectAllRequest {
    /// The state being asked for.
    pub selected: bool,
}

/// A table. See the [module docs](self).
pub struct Table<R> {
    id: ElementId,
    columns: Vec<Column<R>>,
    rows: Vec<Row<R>>,
    sorted_by: Option<SortDescriptor>,
    on_sort: Option<SortHandler>,
    on_select_row: Option<SelectHandler>,
    on_select_all: Option<SelectAllHandler>,
    max_height: Option<Length>,
    empty_message: Option<SharedString>,
    size: ControlSize,
}

impl<R: 'static> IntoElement for Table<R> {
    type Element = gpui::ViewElement<Self>;

    #[track_caller]
    fn into_element(self) -> Self::Element {
        gpui::ViewElement::new(self)
    }
}

/// Creates a new table.
///
/// The id has to be unique among everything drawn in a frame — see
/// `src/element_id.rs`. Every part of the table hangs its own id off this one.
pub fn table<R>(id: impl Into<ElementId>) -> Table<R> {
    Table::new(id)
}

impl<R> Table<R> {
    /// An empty table with no columns, no rows and nothing turned on.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            rows: Vec::new(),
            sorted_by: None,
            on_sort: None,
            on_select_row: None,
            on_select_all: None,
            max_height: None,
            empty_message: None,
            size: ControlSize::default(),
        }
    }

    /// Appends a column.
    pub fn column(mut self, column: Column<R>) -> Self {
        self.columns.push(column);
        self
    }

    /// Appends several columns.
    pub fn columns(mut self, columns: impl IntoIterator<Item = Column<R>>) -> Self {
        self.columns.extend(columns);
        self
    }

    /// Appends a row. Rows arrive already filtered and already sorted.
    pub fn row(mut self, row: Row<R>) -> Self {
        self.rows.push(row);
        self
    }

    /// Appends several rows.
    pub fn rows(mut self, rows: impl IntoIterator<Item = Row<R>>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// States how the rows handed in are already sorted, which is what draws
    /// the indicator on that column's header.
    pub fn sorted_by(mut self, sorted_by: impl Into<Option<SortDescriptor>>) -> Self {
        self.sorted_by = sorted_by.into();
        self
    }

    /// Called when a sortable header is clicked. Without this, `sortable()`
    /// columns are inert.
    pub fn on_sort(
        mut self,
        handler: impl Fn(&SortRequest, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort = Some(Rc::new(handler));
        self
    }

    /// Turns on the leading selection column, and is called when one of its
    /// checkboxes is clicked.
    pub fn on_select_row(
        mut self,
        handler: impl Fn(&SelectRequest, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_row = Some(Rc::new(handler));
        self
    }

    /// Adds the header checkbox — the select-all one, with the indeterminate
    /// middle state — and is called when it is clicked. Has no effect without
    /// [`Table::on_select_row`], since that is what draws the column it sits
    /// in.
    pub fn on_select_all(
        mut self,
        handler: impl Fn(&SelectAllRequest, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_all = Some(Rc::new(handler));
        self
    }

    /// Caps the body's height and turns the scroller on, leaving the header
    /// above it. An uncapped table gets no scroller: a header stuck to nothing
    /// is worse than an honest tall table.
    pub fn max_h(mut self, max_height: impl Into<Length>) -> Self {
        self.max_height = Some(max_height.into());
        self
    }

    /// What to show instead of rows when there are none — the state a filter
    /// produces every time it matches nothing.
    pub fn empty(mut self, message: impl Into<SharedString>) -> Self {
        self.empty_message = Some(message.into());
        self
    }

    /// Whether the leading selection column is drawn.
    pub fn has_selection_column(&self) -> bool {
        self.on_select_row.is_some()
    }

    /// Whether the header carries the select-all checkbox.
    pub fn has_select_all(&self) -> bool {
        self.on_select_row.is_some() && self.on_select_all.is_some()
    }

    /// The state the header checkbox is in, derived from the rows.
    pub fn select_all_state(&self) -> CheckState {
        let selected = self.rows.iter().filter(|row| row.selected).count();
        CheckState::from_count(selected, self.rows.len())
    }
}

impl<R> ControlSized for Table<R> {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl<R: 'static> RenderOnce for Table<R> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `cx.theme()` returns a reference that borrows `App`, and a cell's
        // render closure below needs `&mut App`. Clone it out first. No other
        // element in the crate has to do this because none of them call user
        // code mid-render.
        let theme = cx.theme().clone();
        let metrics = theme.control(self.size);

        let id = self.id;
        let columns = self.columns;
        let rows = self.rows;
        let sorted_by = self.sorted_by;
        let on_sort = self.on_sort;
        let on_select_row = self.on_select_row;
        let on_select_all = self.on_select_all;

        let row_count = rows.len();
        let selected_count = rows.iter().filter(|row| row.selected).count();
        // The one shape named in this file rather than taken from the rung,
        // and it is still derived from it: the box plus the padding a cell
        // would have had on either side.
        let selection_width = metrics.ink + metrics.padding_x * 2.0;
        // Cells wrap, and the rung's own line height is tuned to fit *inside*
        // the rung, which is too tight once there is a second line.
        let line_height = metrics.multiline_line_height();

        // The shell every cell shares. A closure rather than a function so it
        // can close over the rung instead of taking six arguments.
        let cell = |width: &ColumnWidth, min_width: Option<Length>, align: CellAlign| -> Div {
            let base = div()
                .flex()
                .items_start()
                .gap(metrics.gap)
                .px(metrics.padding_x)
                .py(metrics.padding_y())
                // A flex item's automatic minimum size is one unbroken line.
                // The cell is a flex row *and* a flex item, so it needs this
                // here and its content holder needs it again inside.
                .min_w_0()
                .map(|this| match align {
                    CellAlign::Start => this.justify_start(),
                    CellAlign::Center => this.justify_center(),
                    CellAlign::End => this.justify_end(),
                });

            let base = match width {
                ColumnWidth::Flex(grow) => base
                    .flex_grow(*grow)
                    .flex_shrink(1.0)
                    // Basis zero, not auto: with an auto basis a column's
                    // width depends on its own content, and the header and
                    // every row are separate flex containers, so they would
                    // each reach a different answer.
                    .flex_basis(px(0.)),
                ColumnWidth::Fixed(length) => base.flex_none().w(*length),
            };

            base.when_some(min_width, |this, min_width| this.min_w(min_width))
        };

        // The other half of the wrapping fix — see `min_w_0` above.
        let cell_content = |content: AnyElement| div().min_w_0().child(content);

        // ---- Header -------------------------------------------------------
        //
        // A *sibling* of the body rather than its first child, which is the
        // whole of how it stays put while the body scrolls.
        let mut header = div()
            .flex()
            .w_full()
            .flex_none()
            .bg(theme.surface_secondary())
            .border_b_1()
            .border_color(theme.border());

        if on_select_row.is_some() {
            let state = CheckState::from_count(selected_count, row_count);

            let mut select_all_cell = div()
                .id(element_id::scoped(&id, "select-all"))
                .flex()
                .flex_none()
                .w(selection_width)
                .justify_center()
                .py(metrics.padding_y())
                .child(
                    div()
                        .flex()
                        .h(line_height)
                        .items_center()
                        .child(checkbox_box(state).control_size(self.size)),
                );

            if let Some(handler) = on_select_all.clone() {
                select_all_cell =
                    select_all_cell
                        .cursor_pointer()
                        .on_click(move |_, window, cx| {
                            handler(
                                &SelectAllRequest {
                                    selected: state.toggled().is_checked(),
                                },
                                window,
                                cx,
                            );
                        });
            }

            // `debug_selector` compiles to a no-op that never calls its
            // closure unless gpui's `test-support` is on (gpui `div.rs`), so a
            // consumer pays nothing for it. This crate's own examples do build
            // with dev-dependencies, so the showcase pays one `format!` per
            // cell per frame — small next to what that page already does, but
            // not free. It is what makes the header/body column alignment
            // assertable at all.
            let select_all_cell =
                select_all_cell.debug_selector(|| "gpuikit-table-select-all".into());

            header = header.child(select_all_cell);
        }

        for (index, column) in columns.iter().enumerate() {
            let is_sorted = sorted_by.is_some_and(|sort| sort.column == index);
            let sortable = column.sortable && on_sort.is_some();

            let mut header_cell = cell(&column.width, column.min_width, column.align)
                .id(element_id::scoped(&id, format!("header-{index}")))
                .items_center()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(if is_sorted {
                    theme.fg()
                } else {
                    theme.fg_muted()
                })
                .child(div().min_w_0().child(column.header.clone()))
                .when_some(sorted_by.filter(|_| is_sorted), |this, sort| {
                    this.child(
                        div()
                            .flex_none()
                            .text_color(theme.fg_muted())
                            .child(sort.direction.indicator()),
                    )
                });

            if sortable {
                let handler = on_sort.clone().expect("checked just above");
                header_cell = header_cell
                    .cursor_pointer()
                    .hover(|style| style.bg(theme.surface_tertiary()))
                    .on_click(move |_, window, cx| {
                        handler(
                            &SortRequest {
                                column: index,
                                current: sorted_by,
                            },
                            window,
                            cx,
                        );
                    });
            }

            let header_cell =
                header_cell.debug_selector(move || format!("gpuikit-table-header-{index}"));

            header = header.child(header_cell);
        }

        // ---- Body ---------------------------------------------------------
        let mut body = div()
            .id(element_id::scoped(&id, "body"))
            .flex()
            .flex_col()
            .w_full()
            .when_some(self.max_height, |this, max_height| {
                this.max_h(max_height).overflow_y_scroll()
            })
            .debug_selector(|| "gpuikit-table-body".into());

        if rows.is_empty() {
            if let Some(message) = self.empty_message.clone() {
                let empty = div()
                    .w_full()
                    .px(metrics.padding_x)
                    .py(metrics.padding_y() * 4.0)
                    .text_color(theme.fg_muted())
                    .child(message);

                let empty = empty.debug_selector(|| "gpuikit-table-empty".into());

                body = body.child(empty);
            }
        }

        for (row_index, table_row) in rows.into_iter().enumerate() {
            let selected = table_row.selected;

            let mut row_element = div()
                .id(element_id::scoped(&id, format!("row-{row_index}")))
                .flex()
                .w_full()
                .when(selected, |this| this.bg(theme.accent_bg()))
                .when(row_index + 1 < row_count, |this| {
                    this.border_b_1().border_color(theme.border_subtle())
                });

            if let Some(activate) = table_row.on_click.clone() {
                row_element = row_element
                    .cursor_pointer()
                    .hover(|style| style.bg(theme.surface_secondary()))
                    .on_click(move |_, window, cx| activate(window, cx));
            }

            if let Some(handler) = on_select_row.clone() {
                let select_cell = div()
                    .id(element_id::scoped(&id, format!("select-{row_index}")))
                    .flex()
                    .flex_none()
                    .w(selection_width)
                    .justify_center()
                    .py(metrics.padding_y())
                    .cursor_pointer()
                    // A checkbox inside a clickable row fires twice otherwise:
                    // the click has to be stopped here, and the mouse-down as
                    // well, because that is what the row's own click detection
                    // pairs up.
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        handler(
                            &SelectRequest {
                                row: row_index,
                                selected: !selected,
                            },
                            window,
                            cx,
                        );
                    })
                    .child(
                        div().flex().h(line_height).items_center().child(
                            checkbox_box(if selected {
                                CheckState::Checked
                            } else {
                                CheckState::Unchecked
                            })
                            .control_size(self.size),
                        ),
                    );

                let select_cell =
                    select_cell.debug_selector(move || format!("gpuikit-table-select-{row_index}"));

                row_element = row_element.child(select_cell);
            }

            for (column_index, column) in columns.iter().enumerate() {
                // The call into the caller's code. This is why the theme had
                // to be cloned above.
                let content = (column.render)(&table_row.data, window, cx);

                let body_cell = cell(&column.width, column.min_width, column.align)
                    .child(cell_content(content));

                let body_cell = body_cell.debug_selector(move || {
                    format!("gpuikit-table-cell-{row_index}-{column_index}")
                });

                row_element = row_element.child(body_cell);
            }

            body = body.child(row_element);
        }

        div()
            .id(id)
            .flex()
            .flex_col()
            .w_full()
            .overflow_hidden()
            .bg(theme.surface())
            .border_1()
            .border_color(theme.border())
            .rounded(metrics.radius)
            .text_size(metrics.text_size)
            .line_height(line_height)
            .text_color(theme.fg())
            .child(header)
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Bounds, Context, Modifiers, Pixels, Render, TestAppContext, VisualTestContext};
    use std::cell::RefCell;

    #[derive(Clone)]
    struct TestRow {
        name: &'static str,
        stars: u32,
    }

    fn test_rows() -> Vec<TestRow> {
        vec![
            TestRow {
                name: "gpui",
                stars: 3,
            },
            TestRow {
                name: "zed",
                stars: 1,
            },
            TestRow {
                name: "taffy",
                stars: 2,
            },
        ]
    }

    fn name_column() -> Column<TestRow> {
        column("Repository", |row: &TestRow, _, _| {
            div().child(row.name).into_any_element()
        })
    }

    fn stars_column() -> Column<TestRow> {
        column("Stars", |row: &TestRow, _, _| {
            div().child(row.stars.to_string()).into_any_element()
        })
        .end()
    }

    /// What a test's handlers wrote down, in order.
    #[derive(Clone, Default)]
    struct Log(Rc<RefCell<Vec<String>>>);

    impl Log {
        fn push(&self, entry: impl Into<String>) {
            self.0.borrow_mut().push(entry.into());
        }

        fn entries(&self) -> Vec<String> {
            self.0.borrow().clone()
        }
    }

    /// Renders whatever the test's closure builds, every frame.
    struct TableView {
        build: Box<dyn Fn() -> Table<TestRow>>,
    }

    impl Render for TableView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.build)()
        }
    }

    /// Draws a table in a real window. The theme has to be bound first or
    /// `cx.theme()` panics on the unbound global.
    fn draw(
        cx: &mut TestAppContext,
        build: impl Fn() -> Table<TestRow> + 'static,
    ) -> &mut VisualTestContext {
        cx.update(crate::theme::init);
        let (_view, cx) = cx.add_window_view(move |_window, _cx| TableView {
            build: Box::new(build),
        });
        cx.run_until_parked();
        cx
    }

    fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
        cx.debug_bounds(selector)
            .unwrap_or_else(|| panic!("`{selector}` was never laid out"))
    }

    fn click(cx: &mut VisualTestContext, selector: &'static str) {
        let target = bounds(cx, selector).center();
        cx.simulate_click(target, Modifiers::default());
    }

    // ============================================================
    // BUILDER / PURE LOGIC
    // ============================================================

    #[test]
    fn a_column_defaults_to_flexible_leading_and_inert() {
        let column = name_column();

        assert_eq!(column.width, ColumnWidth::Flex(1.0));
        assert_eq!(column.align, CellAlign::Start);
        assert!(!column.is_sortable());
        assert_eq!(column.min_width, None);
    }

    #[test]
    fn the_sort_toggle_reverses_one_column_and_starts_the_others_ascending() {
        let ascending = SortDescriptor::new(1, SortDirection::Ascending);

        // The sorted column reverses.
        assert_eq!(
            SortRequest::next(Some(ascending), 1),
            SortDescriptor::new(1, SortDirection::Descending)
        );
        assert_eq!(
            SortRequest::next(Some(SortDescriptor::new(1, SortDirection::Descending)), 1),
            ascending
        );
        // Any other column starts ascending, whatever the old one was doing.
        assert_eq!(
            SortRequest::next(Some(SortDescriptor::new(1, SortDirection::Descending)), 0),
            SortDescriptor::new(0, SortDirection::Ascending)
        );
        assert_eq!(
            SortRequest::next(None, 2),
            SortDescriptor::new(2, SortDirection::Ascending)
        );
    }

    #[test]
    fn the_selection_column_appears_only_when_a_handler_asks_for_it() {
        let plain: Table<TestRow> = table("t").column(name_column());
        assert!(!plain.has_selection_column());
        assert!(!plain.has_select_all());

        let selectable: Table<TestRow> =
            table("t").column(name_column()).on_select_row(|_, _, _| {});
        assert!(selectable.has_selection_column());
        // The header checkbox is a second opt-in: "all" is only meaningful
        // where the caller's table has all of the rows.
        assert!(!selectable.has_select_all());

        let select_all: Table<TestRow> = table("t")
            .column(name_column())
            .on_select_row(|_, _, _| {})
            .on_select_all(|_, _, _| {});
        assert!(select_all.has_select_all());

        // Without the row handler there is no column for the header checkbox
        // to sit in, so it is not drawn.
        let orphan: Table<TestRow> = table("t").column(name_column()).on_select_all(|_, _, _| {});
        assert!(!orphan.has_select_all());
    }

    #[test]
    fn the_header_checkbox_has_three_states() {
        let build = |selected: &[bool]| -> Table<TestRow> {
            table("t").column(name_column()).rows(
                test_rows()
                    .into_iter()
                    .zip(selected.iter())
                    .map(|(data, selected)| row(data).selected(*selected)),
            )
        };

        assert_eq!(
            build(&[false, false, false]).select_all_state(),
            CheckState::Unchecked
        );
        assert_eq!(
            build(&[true, false, false]).select_all_state(),
            CheckState::Indeterminate
        );
        assert_eq!(
            build(&[true, true, true]).select_all_state(),
            CheckState::Checked
        );
        // A table with no rows is not "all selected".
        let empty: Table<TestRow> = table("t").column(name_column());
        assert_eq!(empty.select_all_state(), CheckState::Unchecked);
    }

    // ============================================================
    // LAYOUT
    // ============================================================

    /// The property the whole column model exists for, and the one a header
    /// outside the scrolled body puts at risk.
    #[gpui::test]
    fn every_column_lines_up_in_the_header_and_in_every_row(cx: &mut TestAppContext) {
        let cx = draw(cx, || {
            table("repos")
                .column(name_column())
                .column(stars_column().fixed(px(120.)))
                .rows(test_rows().into_iter().map(row))
        });

        for (column_index, header) in ["gpuikit-table-header-0", "gpuikit-table-header-1"]
            .into_iter()
            .enumerate()
        {
            let header = bounds(cx, header);
            for (row_index, cell) in [
                ["gpuikit-table-cell-0-0", "gpuikit-table-cell-0-1"],
                ["gpuikit-table-cell-1-0", "gpuikit-table-cell-1-1"],
                ["gpuikit-table-cell-2-0", "gpuikit-table-cell-2-1"],
            ]
            .into_iter()
            .enumerate()
            {
                let cell = bounds(cx, cell[column_index]);
                assert_eq!(
                    cell.origin.x, header.origin.x,
                    "row {row_index} column {column_index} does not start where its header does"
                );
                assert_eq!(
                    cell.size.width, header.size.width,
                    "row {row_index} column {column_index} is not as wide as its header"
                );
            }
        }
    }

    /// The same property once the body is capped and scrolling, which is when
    /// the header and the body stop being the same flex container.
    #[gpui::test]
    fn the_columns_still_line_up_when_the_body_scrolls(cx: &mut TestAppContext) {
        let cx = draw(cx, || {
            table("repos")
                .column(name_column())
                .column(stars_column())
                .rows(test_rows().into_iter().map(row))
                .max_h(px(48.))
        });

        for (header, cell) in [
            ("gpuikit-table-header-0", "gpuikit-table-cell-0-0"),
            ("gpuikit-table-header-1", "gpuikit-table-cell-0-1"),
        ] {
            let header = bounds(cx, header);
            let cell = bounds(cx, cell);
            assert_eq!(cell.origin.x, header.origin.x);
            assert_eq!(cell.size.width, header.size.width);
        }
    }

    /// `max_h` caps the body, and the header is above it rather than inside
    /// it — the two together are what "the header stays put" means here.
    #[gpui::test]
    fn max_h_caps_the_body_and_leaves_the_header_above_it(cx: &mut TestAppContext) {
        let capped = px(48.);
        let cx = draw(cx, move || {
            table("repos")
                .column(name_column())
                .rows(test_rows().into_iter().map(row))
                .max_h(capped)
        });

        let header = bounds(cx, "gpuikit-table-header-0");
        let body = bounds(cx, "gpuikit-table-body");
        let first = bounds(cx, "gpuikit-table-cell-0-0");
        let last = bounds(cx, "gpuikit-table-cell-2-0");

        // The header is a sibling above the body, not its first child.
        assert!(
            header.origin.y + header.size.height <= body.origin.y,
            "the header overlaps the body it is supposed to sit above"
        );
        assert!(
            body.size.height <= capped,
            "the body is {} tall, so `max_h` did not cap it",
            body.size.height
        );
        // Three rows do not fit in the cap, so the body is genuinely scrolled
        // rather than merely short.
        let content = last.origin.y + last.size.height - first.origin.y;
        assert!(
            content > capped,
            "the rows total {content}, which fits inside the cap, so this is not \
             testing a scrolled body"
        );
    }

    #[gpui::test]
    fn a_long_cell_wraps_instead_of_running_off_the_edge(cx: &mut TestAppContext) {
        let long = "a repository name far too long to fit inside one hundred and twenty pixels";
        let cx = draw(cx, move || {
            table("repos")
                .column(
                    column("Repository", move |row: &TestRow, _, _| {
                        div().child(row.name).into_any_element()
                    })
                    .fixed(px(120.)),
                )
                .row(row(TestRow {
                    name: "gpui",
                    stars: 1,
                }))
                .row(row(TestRow {
                    name: long,
                    stars: 1,
                }))
        });

        let short = bounds(cx, "gpuikit-table-cell-0-0");
        let wrapped = bounds(cx, "gpuikit-table-cell-1-0");

        assert_eq!(
            wrapped.size.width, short.size.width,
            "the long cell widened its column instead of wrapping"
        );
        assert!(
            wrapped.size.height > short.size.height,
            "the long cell is one line tall, so it did not wrap"
        );
    }

    #[gpui::test]
    fn the_empty_message_stands_in_for_the_rows(cx: &mut TestAppContext) {
        let cx = draw(cx, || {
            table("repos")
                .column(name_column())
                .empty("No repositories match this filter")
        });

        assert!(cx.debug_bounds("gpuikit-table-empty").is_some());
        assert!(
            cx.debug_bounds("gpuikit-table-cell-0-0").is_none(),
            "there are no rows, so nothing should have drawn a cell"
        );
    }

    // ============================================================
    // INTERACTION
    // ============================================================

    #[gpui::test]
    fn clicking_a_sortable_header_asks_for_a_sort(cx: &mut TestAppContext) {
        let log = Log::default();
        let cx = draw(cx, {
            let log = log.clone();
            move || {
                let log = log.clone();
                table("repos")
                    .column(name_column().sortable())
                    .column(stars_column().sortable())
                    .rows(test_rows().into_iter().map(row))
                    .sorted_by(SortDescriptor::new(0, SortDirection::Ascending))
                    .on_sort(move |request, _, _| {
                        let suggested = request.suggested();
                        log.push(format!("{} -> {:?}", request.column, suggested.direction));
                    })
            }
        });

        // The sorted column reverses; the other one starts ascending.
        click(cx, "gpuikit-table-header-0");
        click(cx, "gpuikit-table-header-1");

        assert_eq!(
            log.entries(),
            vec!["0 -> Descending".to_string(), "1 -> Ascending".to_string()]
        );
    }

    /// Two ways a header does nothing, and they are different: a column that
    /// never asked to be sortable, and a table with no handler to ask.
    #[gpui::test]
    fn a_header_with_nothing_to_ask_is_inert(cx: &mut TestAppContext) {
        let log = Log::default();
        let cx = draw(cx, {
            let log = log.clone();
            move || {
                let log = log.clone();
                table("repos")
                    .column(name_column().sortable())
                    .column(stars_column())
                    .rows(test_rows().into_iter().map(row))
                    .on_sort(move |request, _, _| log.push(request.column.to_string()))
            }
        });

        click(cx, "gpuikit-table-header-1");
        assert!(
            log.entries().is_empty(),
            "an unsortable header asked for a sort"
        );

        // The sortable one in the same table does work, so the click itself
        // landed where it was aimed.
        click(cx, "gpuikit-table-header-0");
        assert_eq!(log.entries(), vec!["0".to_string()]);
    }

    #[gpui::test]
    fn a_sortable_column_without_a_handler_stays_inert(cx: &mut TestAppContext) {
        let cx = draw(cx, || {
            table("repos")
                .column(name_column().sortable())
                .rows(test_rows().into_iter().map(row))
        });

        // Nothing to assert against but the absence of a panic and of a
        // handler; the point is that `sortable()` alone does not wire a click.
        click(cx, "gpuikit-table-header-0");
    }

    #[gpui::test]
    fn clicking_a_row_checkbox_asks_for_that_row(cx: &mut TestAppContext) {
        let log = Log::default();
        let cx = draw(cx, {
            let log = log.clone();
            move || {
                let log = log.clone();
                table("repos")
                    .column(name_column())
                    .rows(
                        test_rows()
                            .into_iter()
                            .enumerate()
                            .map(|(index, data)| row(data).selected(index == 1)),
                    )
                    .on_select_row(move |request, _, _| {
                        log.push(format!("{}:{}", request.row, request.selected));
                    })
            }
        });

        click(cx, "gpuikit-table-select-0");
        // Row 1 is already selected, so its checkbox asks to deselect.
        click(cx, "gpuikit-table-select-1");

        assert_eq!(
            log.entries(),
            vec!["0:true".to_string(), "1:false".to_string()]
        );
    }

    #[gpui::test]
    fn the_header_checkbox_asks_for_all_or_none_from_each_of_its_states(cx: &mut TestAppContext) {
        // (how many rows start selected, what the header checkbox then asks for)
        for (selected, expected) in [(0usize, true), (1, true), (3, false)] {
            let log = Log::default();
            let cx = draw(cx, {
                let log = log.clone();
                move || {
                    let log = log.clone();
                    table("repos")
                        .column(name_column())
                        .rows(
                            test_rows()
                                .into_iter()
                                .enumerate()
                                .map(|(index, data)| row(data).selected(index < selected)),
                        )
                        .on_select_row(|_, _, _| {})
                        .on_select_all(move |request, _, _| log.push(request.selected.to_string()))
                }
            });

            click(cx, "gpuikit-table-select-all");
            assert_eq!(
                log.entries(),
                vec![expected.to_string()],
                "with {selected} of 3 rows selected"
            );
        }
    }

    /// Activation and selection are different acts. Both directions are
    /// asserted, because the "checkbox does not activate" half alone would
    /// pass just as well with row activation broken outright.
    #[gpui::test]
    fn a_checkbox_click_selects_without_activating_the_row(cx: &mut TestAppContext) {
        let log = Log::default();
        let cx = draw(cx, {
            let log = log.clone();
            move || {
                let log = log.clone();
                let activated = log.clone();
                table("repos")
                    .column(name_column())
                    .rows(
                        test_rows()
                            .into_iter()
                            .enumerate()
                            .map(move |(index, data)| {
                                let activated = activated.clone();
                                row(data)
                                    .on_click(move |_, _| activated.push(format!("open {index}")))
                            }),
                    )
                    .on_select_row(move |request, _, _| log.push(format!("select {}", request.row)))
            }
        });

        click(cx, "gpuikit-table-select-0");
        assert_eq!(log.entries(), vec!["select 0".to_string()]);

        // ...and the rest of the row still activates.
        click(cx, "gpuikit-table-cell-1-0");
        assert_eq!(
            log.entries(),
            vec!["select 0".to_string(), "open 1".to_string()]
        );
    }
}
