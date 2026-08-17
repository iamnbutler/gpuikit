pub mod accordion;
pub mod alert;
pub mod aspect_ratio;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod button_group;
pub mod card;
pub mod checkbox;
pub mod collapsible;
pub mod context_menu;
pub mod dialog;
pub mod dropdown;
pub mod empty;
pub mod field;
pub mod icon_button;
pub mod input;
pub mod kbd;
pub mod label;
pub mod list;
pub mod loading_indicator;
pub mod popover;
pub mod progress;
pub mod radio_group;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod slider;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod text_field;
pub mod textarea;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;
pub mod typography;

/// Cross-element tests for the shared control size scale — the property no
/// single element can check from inside itself.
#[cfg(test)]
mod control_size_tests;

/// `docs/component-triage.md` is the decision record for #59's deferred
/// components: a verdict per component, with the surviving ones carrying a
/// ready-to-file issue body under `docs/issues/`.
///
/// #59 went stale because nothing connected it to the crate. These tests are
/// that connection. They parse the verdict table and fail the build when the
/// document stops describing what is actually here.
///
/// They live in the lib rather than `tests/`, and read the document with
/// `include_str!`, for the same reason `showcase_coverage` does:
/// `cargo test --lib` is the command that works in a constrained environment.
#[cfg(test)]
mod triage_coverage {
    use std::fs;
    use std::path::PathBuf;

    const TRIAGE: &str = include_str!("../docs/component-triage.md");
    const ELEMENTS: &str = include_str!("elements.rs");

    /// The number of entries on #59's list, as this repository recorded it:
    /// the 21 rows of `todo.md`'s two deferred lists plus the 8 that had
    /// already shipped. The test's job is to stop a row being deleted to avoid
    /// deciding, so this should only change if #59 is re-read and found to have
    /// a different number of entries.
    const ENTRIES_IN_59: usize = 29;

    /// The split the document states in prose. Restated here on purpose:
    /// editing the table without editing the prose fails.
    const EXPECTED: [(&str, usize); 3] = [("Shipped", 10), ("Issue", 8), ("Rejected", 11)];

    /// An issue body shorter than this is a stub, and #146 asked for complete
    /// ones — prior art, references, crate gaps, the a11y answer, sizing and
    /// the showcase requirement do not fit in less.
    const MIN_ISSUE_BYTES: usize = 1500;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// The verdict table, as `(component, verdict, where)`.
    ///
    /// Anchored on an HTML comment rather than on "the first table", because
    /// the document has five other tables and picking the wrong one would make
    /// every assertion below meaningless rather than failing.
    fn verdict_rows() -> Vec<(String, String, String)> {
        let start = TRIAGE
            .find("<!-- verdict-table -->")
            .expect("docs/component-triage.md no longer anchors its verdict table");

        TRIAGE[start..]
            .lines()
            .skip_while(|line| !line.starts_with('|'))
            .take_while(|line| line.starts_with('|'))
            .filter_map(|line| {
                let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
                if cells.len() != 3 || cells[0] == "Component" || cells[0].starts_with("---") {
                    return None;
                }
                Some((
                    cells[0].to_string(),
                    cells[1].to_string(),
                    cells[2].to_string(),
                ))
            })
            .collect()
    }

    /// A `` `path` `` cell, unwrapped.
    fn backticked(cell: &str) -> Option<&str> {
        cell.strip_prefix('`')?.strip_suffix('`')
    }

    #[test]
    fn every_entry_in_59_has_exactly_one_verdict() {
        let rows = verdict_rows();

        assert_eq!(
            rows.len(),
            ENTRIES_IN_59,
            "the verdict table has {} rows but #59 had {ENTRIES_IN_59} entries — a row was \
             added or, more likely, deleted to avoid deciding about it",
            rows.len(),
        );

        for (component, verdict, _) in &rows {
            assert!(
                EXPECTED.iter().any(|(name, _)| name == verdict),
                "`{component}` has verdict `{verdict}`, which is not one of the three. There \
                 is deliberately no fourth verdict — see the document"
            );
        }

        for (verdict, expected) in EXPECTED {
            let actual = rows.iter().filter(|(_, v, _)| v == verdict).count();
            assert_eq!(
                actual, expected,
                "the table has {actual} {verdict} rows but the document says {expected} in \
                 prose. If a verdict legitimately changed, move both"
            );
        }
    }

    #[test]
    fn every_shipped_row_names_a_real_module() {
        for (component, verdict, location) in verdict_rows() {
            if verdict != "Shipped" {
                continue;
            }

            let path = backticked(&location)
                .unwrap_or_else(|| panic!("`{component}` is Shipped but names no module"));
            let module = path
                .strip_prefix("src/elements/")
                .and_then(|name| name.strip_suffix(".rs"))
                .unwrap_or_else(|| {
                    panic!("`{component}` is Shipped but `{path}` is not a src/elements module")
                });

            assert!(
                ELEMENTS.contains(&format!("pub mod {module};")),
                "`{component}` is Shipped as `{path}`, but src/elements.rs declares no \
                 `pub mod {module};`"
            );
        }
    }

    #[test]
    fn surviving_components_have_an_issue_written() {
        for (component, verdict, location) in verdict_rows() {
            if verdict != "Issue" {
                continue;
            }

            let path = backticked(&location)
                .unwrap_or_else(|| panic!("`{component}` has an Issue verdict but names no file"));
            let full = repo_root().join(path);
            let body = fs::read_to_string(&full).unwrap_or_else(|error| {
                panic!("`{component}` names `{path}`, which cannot be read: {error}")
            });

            assert!(
                body.len() >= MIN_ISSUE_BYTES,
                "`{component}`'s issue body at `{path}` is {} bytes — #146 asked for complete \
                 issue bodies, not placeholders",
                body.len(),
            );
        }
    }

    #[test]
    fn every_written_issue_is_reachable_from_the_triage() {
        let dir = repo_root().join("docs/issues");
        let mut found = 0;

        for entry in fs::read_dir(&dir).expect("docs/issues is readable") {
            let path = entry.expect("directory entry is readable").path();
            if path.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            found += 1;

            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("issue file has a name");
            assert!(
                TRIAGE.contains(&format!("docs/issues/{name}")),
                "`docs/issues/{name}` is not reachable from docs/component-triage.md — an \
                 issue nothing points at is the shape of thing #59 was"
            );
        }

        // A scan that finds nothing reports no orphans, which is
        // indistinguishable from a clean tree. Ten surviving components plus
        // three prerequisites; the floor only has to be non-trivial.
        assert!(
            found >= 10,
            "only {found} issue bodies found under {} — check how the tree is being located \
             before trusting a green result",
            dir.display(),
        );
    }

    #[test]
    fn every_rejection_is_argued_in_prose() {
        let start = TRIAGE
            .find("## The rejections, argued")
            .expect("the triage no longer argues its rejections");
        let rejections = &TRIAGE[start..];
        let end = rejections
            .find("\n## Prerequisites")
            .expect("the rejections section is never closed");
        let rejections = &rejections[..end];

        for (component, verdict, _) in verdict_rows() {
            if verdict != "Rejected" {
                continue;
            }

            assert!(
                rejections.contains(&format!("**{component}.**")),
                "`{component}` is Rejected in the table but has no paragraph in the \
                 rejections section. A rejection asserted in a table cell is a deferral \
                 with better manners"
            );
        }
    }
}

/// The showcase is where a component is looked at, so an element with no page
/// in it is an element nobody sees. These two tests cross-check this file
/// against `examples/showcase.rs`, so a new element module fails the build
/// until it is either shown or explicitly excused.
///
/// They live in the lib rather than `tests/` because `cargo test --lib` is the
/// command that works in a constrained environment — an integration test would
/// be another full link of gpui.
#[cfg(test)]
mod showcase_coverage {
    /// This file, so the module list is read from the source of truth rather
    /// than restated.
    const ELEMENTS: &str = include_str!("elements.rs");
    const SHOWCASE: &str = include_str!("../examples/showcase.rs");

    /// Every `pub mod` declared here.
    fn element_modules() -> Vec<String> {
        ELEMENTS
            .lines()
            .filter_map(|line| {
                line.trim()
                    .strip_prefix("pub mod ")?
                    .strip_suffix(';')
                    .map(str::to_string)
            })
            .collect()
    }

    /// The showcase's `ELEMENT_COVERAGE` table, as `(module, page)` pairs.
    ///
    /// This parses source, so the table's shape matters: one row per line. A
    /// row collapsed onto another line is not silently dropped — the module it
    /// named then has no row and `showcase_covers_every_element` fails — but
    /// the count check below says so directly.
    fn coverage_rows() -> Vec<(String, String)> {
        let start = SHOWCASE
            .find("const ELEMENT_COVERAGE")
            .expect("examples/showcase.rs no longer declares ELEMENT_COVERAGE");
        let table = &SHOWCASE[start..];
        let end = table
            .find("];")
            .expect("ELEMENT_COVERAGE is never closed with `];`");
        let table = &table[..end];

        let rows: Vec<(String, String)> = table
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if !line.starts_with("(\"") {
                    return None;
                }
                let mut quoted = line.split('"');
                let module = quoted.nth(1)?;
                let page = quoted.nth(1)?;
                Some((module.to_string(), page.to_string()))
            })
            .collect();

        assert_eq!(
            rows.len(),
            table.matches("(\"").count(),
            "ELEMENT_COVERAGE is no longer one row per line, so this parser is \
             reading fewer rows than the table has"
        );
        rows
    }

    #[test]
    fn showcase_covers_every_element() {
        let modules = element_modules();
        let rows = coverage_rows();

        assert!(!modules.is_empty(), "no `pub mod` found in src/elements.rs");

        for module in &modules {
            assert!(
                rows.iter().any(|(name, _)| name == module),
                "`{module}` has no row in ELEMENT_COVERAGE: give it a showcase page and \
                 name the page there, or record why it has none with \
                 (\"{module}\", \"none: <reason>\")"
            );
        }

        for (name, _) in &rows {
            assert!(
                modules.contains(name),
                "ELEMENT_COVERAGE names `{name}`, which is not a module in src/elements.rs"
            );
        }
    }

    #[test]
    fn every_covered_page_is_reachable() {
        for (module, page) in coverage_rows() {
            if let Some(reason) = page.strip_prefix("none:") {
                assert!(
                    !reason.trim().is_empty(),
                    "`{module}` opts out of a page without saying why"
                );
                continue;
            }

            // The page id has to be an arm of the showcase's render match, or
            // the table claims coverage that clicking never reaches.
            let arm = format!("\"{page}\" => ");
            assert!(
                SHOWCASE.contains(&arm),
                "`{module}` is listed as shown on the `{page}` page, but no `{arm}` arm \
                 renders it"
            );
        }
    }
}
