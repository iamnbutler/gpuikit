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
