//! The rule for minting [`ElementId`]s, and the two helpers that implement it.
//!
//! # The rule
//!
//! **An element's id must be unique among everything drawn in a frame, not
//! just among that element's own parts.** Concretely:
//!
//! 1. An element that stands on its own derives its id from something unique
//!    per instance — the entity backing it ([`for_entity`]), or an id its
//!    caller supplied.
//! 2. A *part* of such an element hangs its id off the instance's id
//!    ([`scoped`]) rather than naming itself. `"dismiss"` is not an id;
//!    `scoped(&alert_id, "dismiss")` is.
//!
//! A constant — `div().id("textarea")` — satisfies neither. It is the same id
//! for every textarea on screen, and it is only ever correct by accident of an
//! ancestor the element does not control.
//!
//! # Why it matters
//!
//! gpui keys two separate things on an element's *whole* id path (its
//! `GlobalElementId`):
//!
//! - **Element state.** `Window::with_element_state` — and everything built on
//!   it, `use_keyed_state` included — stores per-element state under that path.
//!   Two elements with the same path share one slot, silently: hover state,
//!   scroll offsets and open/closed flags bleed between them.
//! - **Accessibility node ids.** gpui hashes the path into an
//!   `accesskit::NodeId`. A duplicate id trips a `debug_assert!` in debug
//!   builds and is silently dropped from the a11y tree in release ones — the
//!   element simply does not exist for a screen reader.
//!
//! The a11y half only bites once an element also reports a role: gpui builds a
//! node only for an element that has *both* an id and an
//! `Element::a11y_role`. That is why an unscoped id can sit in a codebase
//! looking harmless — it is one `Role::Button` away from a process abort.
//! **Scope the ids first, add the roles second.**
//!
//! # What does and does not scope an id
//!
//! The path is the stack of ids of the *elements* between the window root and
//! this one — not the stack of Rust types. So:
//!
//! - An `Entity<V: Render>` rendered as a child **does** scope: gpui's
//!   `ViewElement::id()` returns `Some(ElementId::View(entity_id))`, so
//!   everything the view renders sits under a per-entity segment.
//! - A `RenderOnce` struct **does not**. It is inlined into its parent's
//!   element tree and pushes nothing of its own.
//! - A bare `div()` **does not**. Only `div().id(…)` pushes a segment.
//! - `deferred()` / `anchored()` **neither scope nor unscope**.
//!   `Window::defer_draw` clones the ambient `element_id_stack` and restores it
//!   when the deferred draw runs, so a popup keeps the path it was built
//!   under. That cuts both ways: it will not orphan a properly scoped panel,
//!   and it will not rescue an unscoped one.
//!
//! Because the first two differ, "is this id unique?" cannot be answered from
//! the element's own source. It needs a walk up the ancestor chain — which is
//! exactly why deriving the id instead is the rule.
//!
//! # Sibling ids
//!
//! `ElementId::NamedInteger("tab".into(), index)` for a row in a list is fine
//! **provided its parent satisfies clause 1** — the index disambiguates the
//! siblings and the parent disambiguates the lists. That relationship is not
//! checkable locally either: if you restructure such a parent, re-check its
//! children by hand.

use std::sync::Arc;

use gpui::{ElementId, EntityId, SharedString};

/// An id for an element derived from the entity that backs it.
///
/// Unique per instance — two of the element on screen are two entities — and
/// stable across frames, which accessibility needs: assistive technology reads
/// a changed node id as a different element, so an id that moves every frame
/// reads as the element being replaced every frame.
///
/// ```
/// # use gpui::{Context, IntoElement, Render, Window, div, prelude::*};
/// use gpuikit::element_id;
/// # struct D;
/// # impl Render for D { fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// div().id(element_id::for_entity("select-listbox", cx.entity_id()))
/// # }}
/// # let mut tcx = gpui::TestAppContext::single();
/// # tcx.update(gpuikit::init);
/// # let _ = tcx.add_window_view(|_, _| D);
/// ```
pub fn for_entity(name: impl Into<SharedString>, entity_id: EntityId) -> ElementId {
    ElementId::NamedInteger(name.into(), entity_id.as_u64())
}

/// An id for a named part of an element, hung off that element's own id.
///
/// Composes into a *single* `ElementId` rather than relying on the part being
/// nested under an element that carries `parent`. That matters where the part
/// is not actually a descendant — a deferred panel, an element whose root
/// carries no id — and it is harmless where it is.
///
/// ```
/// # use gpui::{Context, ElementId, IntoElement, Render, Window, div, prelude::*};
/// use gpuikit::element_id;
/// # struct D { id: ElementId }
/// # impl Render for D { fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// div().id(element_id::scoped(&self.id, "dismiss"))
/// # }}
/// # let mut tcx = gpui::TestAppContext::single();
/// # tcx.update(gpuikit::init);
/// # let _ = tcx.add_window_view(|_, _| D { id: "panel".into() });
/// ```
pub fn scoped(parent: &ElementId, part: impl Into<SharedString>) -> ElementId {
    ElementId::NamedChild(Arc::new(parent.clone()), part.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn for_entity_separates_two_instances() {
        let one = for_entity("textarea", EntityId::from(1u64));
        let two = for_entity("textarea", EntityId::from(2u64));

        assert_ne!(one, two);
        assert_eq!(one, for_entity("textarea", EntityId::from(1u64)));
    }

    #[test]
    fn scoped_parts_follow_their_parent() {
        let left = ElementId::Name("left".into());
        let right = ElementId::Name("right".into());

        assert_ne!(scoped(&left, "dismiss"), scoped(&right, "dismiss"));
        assert_ne!(scoped(&left, "dismiss"), scoped(&left, "close"));
        assert_eq!(scoped(&left, "dismiss"), scoped(&left, "dismiss"));
        // The part is a child of the parent id, not a replacement for it.
        assert_ne!(scoped(&left, "dismiss"), left);
    }

    /// The rule in this module, made enforceable.
    ///
    /// Scans the crate's own source for elements minting a constant id and
    /// fails with a file:line for each. It cannot see whether an
    /// `(name, index)` sibling id has a properly scoped parent — that part of
    /// the rule stays a review question — but it does catch the shape that
    /// produced every collision this module was written for.
    #[test]
    fn no_element_mints_a_constant_id() {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();

        let files = rust_files(&src);

        // A scan that finds nothing to scan reports no offenders, which is
        // indistinguishable from a clean crate. `constant_ids` is unit-tested
        // above, so the matcher cannot silently stop matching; this is the
        // other half — the corpus cannot silently become empty. The floor is
        // far below the real count and only has to be non-trivial.
        assert!(
            files.len() > 20,
            "the scan found only {} source file(s) under {}, so it is not \
             guarding anything — check how the source tree is being located \
             before trusting a green result here",
            files.len(),
            src.display()
        );

        for file in files {
            let source = fs::read_to_string(&file).expect("source file is readable");
            let relative = file
                .strip_prefix(&src)
                .unwrap_or(&file)
                .display()
                .to_string();

            for (line, text) in constant_ids(&source) {
                offenders.push(format!("  src/{relative}:{line}: {text}"));
            }
        }

        assert!(
            offenders.is_empty(),
            "these elements mint an id that is the same for every instance of them:\n{}\n\n\
             Derive it instead — `element_id::for_entity(name, entity_id)` for an element \
             backed by an entity, `element_id::scoped(&parent_id, part)` for a named part of \
             one. See the `element_id` module docs for why.",
            offenders.join("\n")
        );
    }

    fn rust_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut stack = vec![dir.to_path_buf()];

        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).expect("source directory is readable") {
                let path = entry.expect("directory entry is readable").path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    files.push(path);
                }
            }
        }

        files.sort();
        files
    }

    /// The `.id("…")` calls in `source`, as `(line number, trimmed line)`.
    ///
    /// Top-level `#[cfg(test)] mod …` blocks are skipped: a *caller* is
    /// entitled to a constant. `MarkdownElement::new(doc).id("left")` in a test
    /// is correct use of an override API, not an element naming itself. Those
    /// blocks are found by column — an attribute in column 0 opens one, and a
    /// `}` in column 0 closes it — which holds for rustfmt-formatted source.
    fn constant_ids(source: &str) -> Vec<(usize, String)> {
        let mut hits = Vec::new();
        let mut in_test_module = false;

        for (index, line) in source.lines().enumerate() {
            if in_test_module {
                if line == "}" {
                    in_test_module = false;
                }
                continue;
            }
            if line.starts_with("#[cfg(test)]") {
                in_test_module = true;
                continue;
            }

            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains(".id(\"") || trimmed.contains(".id(ElementId::Name(\"") {
                hits.push((index + 1, trimmed.to_string()));
            }
        }

        hits
    }

    #[test]
    fn the_scan_reads_constants_and_not_derived_ids() {
        let source = r#"
fn render(&self) -> impl IntoElement {
    div().id("textarea").child(
        div().id(ElementId::Name("nested".into())),
    )
}

fn fine(&self) -> impl IntoElement {
    // .id("in-a-comment")
    div()
        .id(self.id.clone())
        .id(element_id::scoped(&self.id, "track"))
        .id(ElementId::NamedInteger("tab".into(), index))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_caller_may_name_its_own_element() {
        div().id("left");
    }
}
"#;

        assert_eq!(
            constant_ids(source)
                .into_iter()
                .map(|(line, _)| line)
                .collect::<Vec<_>>(),
            vec![3, 4]
        );
    }
}
