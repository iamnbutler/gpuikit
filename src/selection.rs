//! One selection value type, and the arithmetic of moving a highlight through a
//! list, in one place.
//!
//! Written the way [`crate::a11y`] is written — a value type first, a trait
//! second — because the alternative is what this module replaces: five
//! hand-rolled selection stores, two byte-identical `Single`/`Multiple` enums,
//! and the same keyboard-wrap arithmetic written twice. See the tracking issue
//! for the roster.
//!
//! # What this is, and what it is not
//!
//! [`SelectionModel`] is *shared interpretation and bookkeeping*, not
//! element-owned state. It keeps to the house rule the rest of the crate keeps:
//! the caller still owns the applied state (a table answers a `SelectRequest`;
//! a list is told what is selected). This type is the piece everyone was
//! re-implementing to interpret those requests — which index a toggle lands on,
//! whether a box should read checked/indeterminate, where a shift-range runs.
//!
//! # The one deliberate fork
//!
//! Moving a highlight wraps at both ends in a chooser popup (a menu, a
//! combobox) and clamps at both ends in a persistent list (a shift-extension
//! must not jump from the last row to the first). That is a *policy* the
//! element owns, so it is a parameter — [`HighlightMotion`] — not a property of
//! the model. The arithmetic itself ([`wrap_index`], [`clamp_index`]) is shared;
//! the choice between them is not.

use crate::elements::checkbox::CheckState;
use std::collections::BTreeSet;

/// Whether a selection holds one item or many.
///
/// The single type behind what were `ToggleGroupMode` and `AccordionMode`.
/// Deliberately not `Default`: single-select and multi-select controls each
/// have their own natural default, so the choice is made at the call site
/// rather than inherited from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// At most one item is selected. Selecting another replaces it.
    Single,
    /// Any number of items may be selected at once.
    Multiple,
}

/// Which end-policy [`SelectionModel::move_highlight`] follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightMotion {
    /// Past the last item comes the first, and vice versa. Right for a chooser
    /// popup, where the list is transient and the ends are the same case.
    Wrap,
    /// The ends hold. Right for a shift-extension in a persistent list, which
    /// must not leap from the bottom to the top.
    Clamp,
}

/// Move `current` by `delta` through `count` items, wrapping at both ends.
///
/// `None` in means nothing is highlighted yet: a forward move enters at the
/// top, a backward move at the bottom. `None` out means there is nothing to
/// highlight (`count == 0`) — not an index no item has.
///
/// Wrapping is `rem_euclid` rather than a pair of bounds checks because the two
/// ends are the same case and a signed remainder would otherwise send `-1` off
/// the front of the list.
pub fn wrap_index(current: Option<usize>, delta: isize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match current {
        Some(current) => (current as isize + delta).rem_euclid(count as isize) as usize,
        None if delta < 0 => count - 1,
        None => 0,
    })
}

/// Move `current` by `delta` through `count` items, clamping at both ends.
///
/// Same entry rule as [`wrap_index`] for `None`, but a move past either end
/// stays on that end.
pub fn clamp_index(current: Option<usize>, delta: isize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match current {
        Some(current) => (current as isize + delta).clamp(0, count as isize - 1) as usize,
        None if delta < 0 => count - 1,
        None => 0,
    })
}

/// A selection: its mode, the selected set, the keyboard cursor, and the anchor
/// a shift-range runs from.
///
/// `highlighted` is the keyboard cursor and is *not* the selection — a list can
/// move its highlight with the arrow keys without selecting, and select what is
/// highlighted only on `Space`/`Enter`. Listbox already draws this distinction;
/// this type keeps it.
#[derive(Debug, Clone)]
pub struct SelectionModel {
    mode: SelectionMode,
    selected: BTreeSet<usize>,
    highlighted: Option<usize>,
    anchor: Option<usize>,
}

impl SelectionModel {
    /// A model in `mode`, with nothing selected or highlighted.
    pub fn new(mode: SelectionMode) -> Self {
        Self {
            mode,
            selected: BTreeSet::new(),
            highlighted: None,
            anchor: None,
        }
    }

    /// A single-select model.
    pub fn single() -> Self {
        Self::new(SelectionMode::Single)
    }

    /// A multi-select model.
    pub fn multiple() -> Self {
        Self::new(SelectionMode::Multiple)
    }

    /// This model's mode.
    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Whether `ix` is selected.
    pub fn is_selected(&self, ix: usize) -> bool {
        self.selected.contains(&ix)
    }

    /// The selected indices, ascending.
    pub fn selected(&self) -> &BTreeSet<usize> {
        &self.selected
    }

    /// How many indices are selected.
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// The sole selected index, for single-select consumers. `None` when
    /// nothing is selected; the smallest index when (against a single-select
    /// model's contract) more than one is.
    pub fn single_selection(&self) -> Option<usize> {
        self.selected.iter().next().copied()
    }

    /// The keyboard cursor.
    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    /// Move the keyboard cursor.
    pub fn set_highlighted(&mut self, highlighted: Option<usize>) {
        self.highlighted = highlighted;
    }

    /// The shift-range anchor.
    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    /// Set the shift-range anchor.
    pub fn set_anchor(&mut self, anchor: Option<usize>) {
        self.anchor = anchor;
    }

    /// Select `ix`. In [`Single`](SelectionMode::Single) this replaces the
    /// selection; in [`Multiple`](SelectionMode::Multiple) it adds to it. Either
    /// way `ix` becomes the anchor.
    pub fn select(&mut self, ix: usize) {
        if self.mode == SelectionMode::Single {
            self.selected.clear();
        }
        self.selected.insert(ix);
        self.anchor = Some(ix);
    }

    /// Deselect `ix`.
    pub fn deselect(&mut self, ix: usize) {
        self.selected.remove(&ix);
    }

    /// Clear the selection. Leaves the highlight and anchor alone.
    pub fn clear(&mut self) {
        self.selected.clear();
    }

    /// Toggle `ix`, following the mode.
    ///
    /// [`Single`](SelectionMode::Single): selecting the already-selected item is
    /// a no-op — a single-select control does not deselect by re-clicking, it
    /// waits to be given a different item (the `ToggleGroup` rule).
    /// [`Multiple`](SelectionMode::Multiple): membership flips. Returns whether
    /// the selection changed.
    pub fn toggle(&mut self, ix: usize) -> bool {
        match self.mode {
            SelectionMode::Single => {
                if self.is_selected(ix) {
                    false
                } else {
                    self.select(ix);
                    true
                }
            }
            SelectionMode::Multiple => {
                if self.selected.remove(&ix) {
                    true
                } else {
                    self.select(ix);
                    true
                }
            }
        }
    }

    /// Select the inclusive range from the anchor to `ix` (a shift-click or
    /// shift-arrow). In [`Multiple`](SelectionMode::Multiple) this replaces the
    /// selection with that whole run; with no anchor yet it behaves like
    /// [`select`](Self::select) and sets the anchor. In
    /// [`Single`](SelectionMode::Single) there is no range, so it is a plain
    /// [`select`](Self::select).
    pub fn extend_to(&mut self, ix: usize) {
        if self.mode == SelectionMode::Single {
            self.select(ix);
            return;
        }
        let Some(anchor) = self.anchor else {
            self.select(ix);
            return;
        };
        let (lo, hi) = if anchor <= ix {
            (anchor, ix)
        } else {
            (ix, anchor)
        };
        self.selected = (lo..=hi).collect();
        // The anchor stays put: successive shift-arrows all measure from it.
    }

    /// Move the keyboard cursor `delta` through `len` items under `motion`,
    /// store the result, and return it.
    pub fn move_highlight(
        &mut self,
        delta: isize,
        len: usize,
        motion: HighlightMotion,
    ) -> Option<usize> {
        let next = match motion {
            HighlightMotion::Wrap => wrap_index(self.highlighted, delta, len),
            HighlightMotion::Clamp => clamp_index(self.highlighted, delta, len),
        };
        self.highlighted = next;
        next
    }

    /// How a box standing for `total` items, of which this model has some
    /// selected, should be drawn: checked, unchecked, or indeterminate. Reuses
    /// [`CheckState::from_count`]; indices at or beyond `total` (left over from a
    /// list that shrank) do not count.
    pub fn check_state(&self, total: usize) -> CheckState {
        let in_range = self.selected.iter().filter(|&&ix| ix < total).count();
        CheckState::from_count(in_range, total)
    }
}

/// An element that owns a [`SelectionModel`], for tests and scroll-to-reveal
/// glue that need at the selection without knowing the element.
pub trait HasSelection {
    /// The model, to read.
    fn selection(&self) -> &SelectionModel;
    /// The model, to change.
    fn selection_mut(&mut self) -> &mut SelectionModel;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_index_moves_and_wraps_at_both_ends() {
        assert_eq!(wrap_index(Some(0), 1, 3), Some(1));
        assert_eq!(
            wrap_index(Some(2), 1, 3),
            Some(0),
            "past the end comes the front"
        );
        assert_eq!(
            wrap_index(Some(0), -1, 3),
            Some(2),
            "before the front comes the end"
        );
    }

    #[test]
    fn wrap_index_enters_from_the_near_end_and_is_empty_on_nothing() {
        assert_eq!(wrap_index(None, 1, 3), Some(0), "forward enters at the top");
        assert_eq!(
            wrap_index(None, -1, 3),
            Some(2),
            "backward enters at the bottom"
        );
        assert_eq!(
            wrap_index(None, 1, 0),
            None,
            "an empty list highlights nothing"
        );
        assert_eq!(wrap_index(Some(0), 1, 0), None);
    }

    #[test]
    fn clamp_index_holds_at_both_ends() {
        assert_eq!(clamp_index(Some(1), 1, 3), Some(2));
        assert_eq!(clamp_index(Some(2), 1, 3), Some(2), "the last item holds");
        assert_eq!(clamp_index(Some(0), -1, 3), Some(0), "the first item holds");
        assert_eq!(clamp_index(None, 1, 0), None);
    }

    #[test]
    fn single_select_replaces() {
        let mut model = SelectionModel::single();
        model.select(0);
        model.select(2);
        assert!(!model.is_selected(0), "single-select replaces, not adds");
        assert!(model.is_selected(2));
        assert_eq!(model.single_selection(), Some(2));
    }

    #[test]
    fn single_toggle_does_not_deselect_by_reclicking() {
        let mut model = SelectionModel::single();
        assert!(model.toggle(1), "first toggle selects and reports a change");
        assert!(!model.toggle(1), "re-toggling the selected item is a no-op");
        assert!(model.is_selected(1), "and leaves it selected");
    }

    #[test]
    fn multiple_toggle_flips_membership() {
        let mut model = SelectionModel::multiple();
        model.toggle(0);
        model.toggle(2);
        assert!(model.is_selected(0) && model.is_selected(2));
        assert!(model.toggle(0), "toggling a selected item removes it");
        assert!(!model.is_selected(0));
        assert!(model.is_selected(2), "and leaves the others alone");
    }

    #[test]
    fn extend_to_selects_the_inclusive_run_from_the_anchor() {
        let mut model = SelectionModel::multiple();
        model.select(2); // anchor = 2
        model.extend_to(5);
        assert_eq!(
            model.selected().iter().copied().collect::<Vec<_>>(),
            vec![2, 3, 4, 5]
        );
    }

    #[test]
    fn extend_to_runs_backwards_from_the_anchor_too() {
        let mut model = SelectionModel::multiple();
        model.select(5); // anchor = 5
        model.extend_to(2);
        assert_eq!(
            model.selected().iter().copied().collect::<Vec<_>>(),
            vec![2, 3, 4, 5]
        );
    }

    #[test]
    fn extend_to_measures_every_step_from_the_same_anchor() {
        let mut model = SelectionModel::multiple();
        model.select(2);
        model.extend_to(5);
        model.extend_to(3); // shrinks the run rather than adding to 5..3
        assert_eq!(
            model.selected().iter().copied().collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn extend_to_with_no_anchor_is_a_plain_select() {
        let mut model = SelectionModel::multiple();
        model.extend_to(4);
        assert_eq!(
            model.selected().iter().copied().collect::<Vec<_>>(),
            vec![4]
        );
        assert_eq!(model.anchor(), Some(4));
    }

    #[test]
    fn move_highlight_wraps_or_clamps_by_policy() {
        let mut model = SelectionModel::single();
        model.set_highlighted(Some(2));
        assert_eq!(model.move_highlight(1, 3, HighlightMotion::Wrap), Some(0));
        assert_eq!(model.highlighted(), Some(0));

        model.set_highlighted(Some(2));
        assert_eq!(model.move_highlight(1, 3, HighlightMotion::Clamp), Some(2));
    }

    #[test]
    fn check_state_aggregates_over_a_total() {
        let mut model = SelectionModel::multiple();
        assert_eq!(model.check_state(3), CheckState::Unchecked);
        model.toggle(0);
        assert_eq!(model.check_state(3), CheckState::Indeterminate);
        model.toggle(1);
        model.toggle(2);
        assert_eq!(model.check_state(3), CheckState::Checked);
    }

    #[test]
    fn check_state_ignores_indices_a_shrunk_list_left_behind() {
        let mut model = SelectionModel::multiple();
        model.toggle(0);
        model.toggle(5); // list later shrank to 3
        assert_eq!(
            model.check_state(3),
            CheckState::Indeterminate,
            "the stale index 5 does not count toward 'all of 3'"
        );
    }
}
