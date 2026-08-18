//! Tests for the guard that stops `.github/workflows/release.yml` publishing a
//! version `CHANGELOG.md` does not name.
//!
//! `release.yml` computes the version to publish two ways that no longer
//! agree: `custom_version` names it outright, while `version_type` counts up
//! *from* `Cargo.toml`. Since #170 landed 0.8.0 into `Cargo.toml` as a
//! *prepared* release, `version_type: minor` — the shape every previous
//! release used — computes 0.9.0 and publishes it, skipping a version. This
//! repository has already lost one that way: its tags are `v0.3.0 v0.4.0
//! v0.5.0 v0.5.1 v0.7.0`, with no `v0.6.0`, though `CHANGELOG.md` carries a
//! `## [0.6.0]` heading.
//!
//! `cargo publish` cannot be undone and a published version cannot be reused
//! even after `cargo yank`, so a guard that only exists in an input's
//! description is not a guard. `.github/scripts/verify-release-version.sh` is
//! the real one, and these are the tests for it — nothing in a Rust build runs
//! a workflow, so what is checkable here is that the guard is *wired in
//! correctly* and that the invariant it depends on holds today.
//!
//! They live in the lib rather than `tests/`, for the reason stated at
//! `src/elements.rs`'s `triage_coverage`: `cargo test --lib` is the command
//! that works in a constrained environment. Each parser asserts it matched
//! something before anything else is trusted, for the same reason that module
//! does — a parser that silently found nothing reports success.

use std::path::PathBuf;
use std::process::Command;

const CHANGELOG: &str = include_str!("../CHANGELOG.md");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const RELEASE_YML: &str = include_str!("../.github/workflows/release.yml");

/// The script the workflow calls. Named once so a rename fails in one place.
const SCRIPT: &str = ".github/scripts/verify-release-version.sh";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The topmost `## [x.y.z]` heading's version.
///
/// The digit straight after the bracket is load-bearing: `## [Unreleased]` sits
/// above every versioned heading in this file and must not be read as one.
fn top_versioned_heading() -> String {
    CHANGELOG
        .lines()
        .find_map(|line| {
            let rest = line.strip_prefix("## [")?;
            let version = rest.split(']').next()?;
            version
                .starts_with(|c: char| c.is_ascii_digit())
                .then(|| version.to_string())
        })
        .expect("CHANGELOG.md has no `## [x.y.z]` heading at all")
}

/// The `version = "…"` from `[package]`, which is the first one in the file.
fn cargo_toml_version() -> String {
    CARGO_TOML
        .lines()
        .find_map(|line| {
            let rest = line.trim().strip_prefix("version")?.trim_start();
            let rest = rest.strip_prefix('=')?.trim();
            rest.strip_prefix('"')?
                .split('"')
                .next()
                .map(str::to_string)
        })
        .expect("Cargo.toml declares no version")
}

#[test]
fn the_changelog_parses_at_all() {
    let top = top_versioned_heading();

    assert!(
        CHANGELOG.contains("## [Unreleased]"),
        "the Unreleased heading is gone, so the digit-after-bracket rule this all \
         depends on is no longer being exercised"
    );
    assert_ne!(top, "Unreleased", "`## [Unreleased]` was read as a version");
    assert!(
        top.starts_with(|c: char| c.is_ascii_digit()),
        "the topmost versioned heading parsed as `{top}`"
    );
}

/// #170's convention, enforced on every `cargo test --lib` rather than only at
/// release time: `Cargo.toml` carries the version being *prepared*, and the
/// changelog's topmost heading names it.
///
/// This deliberately moves "the changelog was never updated" from a
/// release-time failure to a PR-time one. The cost — a pull request that bumps
/// one without writing the other goes red — is the feature. Under #170's
/// convention both land in the same pull request.
#[test]
fn changelog_names_the_version_cargo_toml_is_prepared_to_release() {
    assert_eq!(
        cargo_toml_version(),
        top_versioned_heading(),
        "Cargo.toml and CHANGELOG.md's topmost heading disagree. The release guard \
         compares the computed version against that heading, so a release dispatched \
         from this tree would abort. Move both together"
    );
}

#[test]
fn the_workflow_still_calls_the_guard() {
    assert!(
        repo_root().join(SCRIPT).exists(),
        "{SCRIPT} is gone, and release.yml calls it"
    );
    assert!(
        RELEASE_YML.contains(SCRIPT),
        "release.yml no longer calls {SCRIPT}, so nothing checks the version being \
         published against CHANGELOG.md"
    );
}

/// The guard is worth nothing after `Cargo.toml` has been rewritten or a tag
/// pushed, so it has to precede every literal that writes something.
#[test]
fn the_guard_runs_before_anything_is_written() {
    let guard = RELEASE_YML
        .find(SCRIPT)
        .expect("release.yml no longer calls the guard");

    for writer in [
        "cargo set-version",
        "git add Cargo.toml",
        "git tag -a",
        "git push origin",
    ] {
        let at = RELEASE_YML.find(writer).unwrap_or_else(|| {
            panic!(
                "release.yml no longer contains `{writer}`. If the step was renamed, \
                 update this list — otherwise this test passes by finding nothing"
            )
        });
        assert!(
            guard < at,
            "`{writer}` runs before the version guard, so the guard fires after the \
             damage is done"
        );
    }
}

/// A dry run is the run someone does *precisely* to check what would be
/// released. Skipping the guard on it would hide the one answer that run
/// exists to give.
#[test]
fn the_guard_step_is_not_skipped_on_a_dry_run() {
    let start = RELEASE_YML
        .find("      - name: Verify the version is the one being released")
        .expect("the guard step was renamed; update this test with it");
    let rest = &RELEASE_YML[start + 1..];
    let end = rest
        .find("\n      - name: ")
        .expect("the guard step is never closed by another step");
    let step = &rest[..end];

    assert!(
        !step.contains("if:"),
        "the guard step grew an `if:`, so some dispatch no longer runs it:\n{step}"
    );

    // Vacuity check: the detector has to be able to see an `if:` at all, or
    // the assertion above passes against anything.
    let sibling = RELEASE_YML
        .find("      - name: Update Cargo.toml version")
        .expect("the Cargo.toml step was renamed");
    let sibling = &RELEASE_YML[sibling..];
    let sibling_end = sibling[1..]
        .find("\n      - name: ")
        .expect("the Cargo.toml step is never closed");
    assert!(
        sibling[..sibling_end].contains("if:"),
        "the neighbouring step no longer carries an `if:`, so the check above is \
         not looking for anything"
    );
}

/// The script and this file have to agree on which heading is topmost, or one
/// of them is checking a different document. Answered by running the real
/// script rather than by duplicating its parsing.
#[test]
fn the_script_agrees_with_this_file_about_the_top_heading() {
    let script = repo_root().join(SCRIPT);
    let changelog = repo_root().join("CHANGELOG.md");
    let top = top_versioned_heading();

    let run = |version: &str| {
        Command::new("bash")
            .arg(&script)
            .arg(version)
            .arg(&changelog)
            .output()
            .expect("bash is available to run the guard")
    };

    assert!(
        run(&top).status.success(),
        "the script rejects `{top}`, which this file reads as the topmost heading"
    );

    let wrong = format!("{top}-not-a-real-version");
    let rejected = run(&wrong);
    assert!(
        !rejected.status.success(),
        "the script accepted `{wrong}`, so it is not comparing anything"
    );
    let message = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        message.contains(&top) && message.contains(&wrong),
        "the mismatch message names neither value, which is the whole point of \
         printing one:\n{message}"
    );
}
