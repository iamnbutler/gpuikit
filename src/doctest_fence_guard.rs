//! Tests for the rule that a rustdoc example which is not checked does not
//! exist.
//!
//! `src/` used to hold 56 fenced examples, 55 of them `` ```ignore ``. rustdoc
//! does not compile an `ignore`d block, so CI's doctest job reported
//! `1 passed; 55 ignored` and went green while the examples on docs.rs
//! drifted: `Application::new()` had not existed for some time, two examples
//! named crates this workspace does not have, and one used a method that lives
//! on a trait it never imported. Every one of them *looked* like Rust.
//!
//! This is the same rule [`showcase_coverage`](crate::elements) enforces for
//! pages, applied to examples: the fence a block carries decides whether
//! rustdoc will ever compile it, so the fence is what this guards.
//!
//! | Fence | Rule |
//! |---|---|
//! | bare, `rust` | allowed — an example that compiles *and runs* |
//! | `no_run` | allowed only with an entry in [`NO_RUN`] |
//! | `ignore` | never allowed, with no escape hatch |
//! | `text` | allowed — not Rust, and it claims not to be |
//! | `compile_fail` | allowed only with an entry in [`COMPILE_FAIL`] |
//! | anything else | rejected by name |
//!
//! `ignore` has no allowlist on purpose. Every other fence here states
//! something true about the block — `no_run` says "this compiles but must not
//! run", `text` says "this is not Rust". `ignore` states nothing; it only
//! asks rustdoc to look away, which is exactly the outcome this file exists to
//! prevent. An example that cannot be made to compile is an example whose
//! prose was doing the work all along, and the prose can stay without it.
//!
//! `compile_fail` is allowlisted for a different reason than `no_run`, and not
//! a truth-related one: rustdoc *does* check a `compile_fail` block, but it
//! can never fold one into the merged doctest binary, so each one is its own
//! whole-program link of gpui. That is the cost gpuikit#180 was about. There
//! are none today, and the empty list is how it stays deliberate.
//!
//! # Writing an example that runs
//!
//! Most of these need a live `App`, which a doctest gets from
//! `gpui::TestAppContext::single()` — `test-support` is a dev-dependency, and
//! doctests link dev-dependencies. Two shapes cover the crate:
//!
//! An element is rendered by a hidden view, because `VisualTestContext::draw`
//! cannot draw one: any element carrying an `.id()` reaches
//! `Window::current_view`, which unwraps an empty entity stack and panics.
//! `add_window_view` draws a real frame, so the example is *rendered*, not
//! merely constructed — omit the `gpuikit::init` line and the doctest panics,
//! which is the proof that it runs.
//!
//! ```text
//! /// # use gpui::{Context, IntoElement, Render, Window, prelude::*};
//! /// use gpuikit::elements::button::button;
//! /// # struct D;
//! /// # impl Render for D { fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! /// button("save", "Save")
//! /// # }}
//! /// # let mut tcx = gpui::TestAppContext::single();
//! /// # tcx.update(gpuikit::init);
//! /// # let _ = tcx.add_window_view(|_, _| D);
//! ```
//!
//! Something that only needs an `App` — binding keys, building state — skips
//! the view and uses `tcx.update(|cx| { … })` instead.
//!
//! Anything that spawns background work must drive it. The executor under
//! `TestAppContext` is deterministic and runs nothing on its own, so a
//! doctest that calls, say, `Markdown::append` and then returns leaves a
//! parse queued and the binary never exits — a hang, not a failure. End it
//! with `tcx.run_until_parked()`; `markdown::Markdown` is the worked example.
//!
//! Like the other guards this lives in the lib rather than `tests/`, so that
//! `cargo test --lib` — the command that works in a constrained environment —
//! is enough to run it. It reads fences out of the source rather than asking
//! rustdoc, so it costs nothing and needs no doctest run of its own.

use std::path::{Path, PathBuf};

/// This module names every fence in its own prose and tables, so it would flag
/// itself. It is the one file the scan skips, and the count is asserted so the
/// exemption cannot silently widen.
const SELF: &str = "doctest_fence_guard.rs";

/// (path relative to `src/`, why running it is not an option). Every entry is
/// asserted to still match a `no_run` fence in that file, so an example that
/// stops needing the exemption fails the scan until its entry goes too.
const NO_RUN: &[(&str, &str)] = &[
    (
        "lib.rs",
        "calls `Application::run`, which takes over the thread and opens a real window",
    ),
    (
        "elements/dialog.rs",
        "calls `Application::run`, which takes over the thread and opens a real window",
    ),
    (
        "elements/toast.rs",
        "calls `Application::run`, which takes over the thread and opens a real window",
    ),
    (
        "markdown/code_highlight.rs",
        "calls `Application::run`, which takes over the thread and opens a real window",
    ),
];

/// (path relative to `src/`, why the link is worth paying for). Empty, and
/// meant to stay that way: a `compile_fail` block cannot join the merged
/// doctest binary, so each one links gpui by itself.
const COMPILE_FAIL: &[(&str, &str)] = &[];

/// Fences that need no justification. `text` is prose in a box — it does not
/// claim to be Rust, and rustdoc never compiles it.
const ALWAYS_ALLOWED: &[&str] = &["", "rust", "text"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("source directory is readable") {
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

/// The opening fence of every doc-comment code block in `source`, as
/// `(line number, fence)` — `("", …)` for a bare ` ``` `. Only `//!` and `///`
/// lines are read, so a fence inside ordinary code or a `//` comment is not a
/// doc example and is not this file's business.
///
/// Openers and closers alternate, which is how a closing ` ``` ` is told from
/// the next opener without parsing the block between them.
fn doc_fences(source: &str) -> Vec<(usize, String)> {
    let mut fences = Vec::new();
    let mut open = false;

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed
            .strip_prefix("//!")
            .or_else(|| trimmed.strip_prefix("///"))
        else {
            continue;
        };
        let Some(after) = rest.trim_start().strip_prefix("```") else {
            continue;
        };

        if open {
            open = false;
            continue;
        }
        open = true;
        fences.push((index + 1, after.trim().to_string()));
    }

    fences
}

#[test]
fn every_doc_example_is_one_rustdoc_will_compile() {
    let src = repo_root().join("src");
    let files = rust_files(&src);
    assert!(
        files.len() > 20,
        "walked src/ and found {} Rust files — the walker is broken",
        files.len(),
    );

    let mut scanned = 0usize;
    let mut fences_seen = 0usize;
    let mut no_run_seen: Vec<String> = Vec::new();
    let mut compile_fail_seen: Vec<String> = Vec::new();

    for file in &files {
        if file.file_name().is_some_and(|name| name == SELF) {
            continue;
        }
        scanned += 1;

        let relative = file
            .strip_prefix(&src)
            .expect("file is under src/")
            .to_str()
            .expect("source paths are UTF-8")
            .to_string();
        let source = std::fs::read_to_string(file).expect("source file is readable");

        for (line, fence) in doc_fences(&source) {
            fences_seen += 1;
            let at = format!("{relative}:{line}");

            assert_ne!(
                fence, "ignore",
                "{at} is a ```ignore example. rustdoc never compiles one, so nothing \
                 keeps it true — this is the fence that let `Application::new()` sit \
                 in the crate's own Quick Start after it stopped existing. There is no \
                 allowlist for it. Make the example compile (see this module's docs for \
                 the hidden prelude), mark it `no_run` and add an entry to `NO_RUN` in \
                 {SELF} if running it opens a window, or delete it — an example whose \
                 prose was doing the work does not lose anything by going."
            );

            if ALWAYS_ALLOWED.contains(&fence.as_str()) {
                continue;
            }

            if fence == "no_run" {
                assert!(
                    NO_RUN.iter().any(|(path, _)| *path == relative),
                    "{at} is a ```no_run example, but {relative} has no entry in `NO_RUN` \
                     in {SELF}. `no_run` costs the one thing a doctest is for — running \
                     — so add an entry saying why running it is not an option, or make \
                     it run."
                );
                no_run_seen.push(relative.clone());
                continue;
            }

            if fence == "compile_fail" {
                assert!(
                    COMPILE_FAIL.iter().any(|(path, _)| *path == relative),
                    "{at} is a ```compile_fail example, but {relative} has no entry in \
                     `COMPILE_FAIL` in {SELF}. rustdoc does check these, but it cannot \
                     merge one into the shared doctest binary, so each is a whole-program \
                     link of gpui — see gpuikit#180. Add an entry saying the link is \
                     worth it."
                );
                compile_fail_seen.push(relative.clone());
                continue;
            }

            panic!(
                "{at} carries the fence ```{fence}, which this crate has no rule for. \
                 Either it is a typo, or it is a rustdoc attribute worth a deliberate \
                 decision — add it to `ALWAYS_ALLOWED` or give it an allowlist in {SELF}."
            );
        }
    }

    assert_eq!(
        scanned,
        files.len() - 1,
        "exactly one file — `{SELF}`, which names every fence in its own prose — is \
         exempt from this scan. {} were skipped.",
        files.len() - scanned,
    );
    assert!(
        fences_seen > 30,
        "found only {fences_seen} doc-comment fences in src/ — the scanner is reading \
         nothing, and its 'no ```ignore anywhere' verdict means nothing either",
    );

    // Every allowlist entry must still be earning its place, so an example
    // that stops needing the exemption cannot leave a stale one behind for the
    // next one to inherit.
    for (path, justification) in NO_RUN {
        assert!(
            no_run_seen.iter().any(|seen| seen == path),
            "`NO_RUN` in {SELF} still allows `{path}` ({justification}), but that file \
             has no ```no_run example any more. Delete the entry.",
        );
    }
    for (path, justification) in COMPILE_FAIL {
        assert!(
            compile_fail_seen.iter().any(|seen| seen == path),
            "`COMPILE_FAIL` in {SELF} still allows `{path}` ({justification}), but that \
             file has no ```compile_fail example any more. Delete the entry.",
        );
    }
}

/// The scanner reads fences out of doc comments only, and tells an opener from
/// a closer. Without this, "found no `ignore` anywhere" could just mean the
/// scan never saw a fence.
#[test]
fn the_scanner_reads_doc_fences_and_only_doc_fences() {
    let source = r#"
//! ```
//! let module_level = 1;
//! ```
//!
//! ```text
//! not rust
//! ```

// ``` a plain comment fence is not a doc example
/// ```no_run
/// let item_level = 2;
/// ```
fn documented() {
    let s = "``` a fence inside a string literal";
}
"#;

    assert_eq!(
        doc_fences(source),
        vec![
            (2, String::new()),
            (6, "text".to_string()),
            (11, "no_run".to_string()),
        ],
        "the scanner should report three openers — bare, text, no_run — and neither \
         the closers, the `//` comment, nor the string literal",
    );
}
