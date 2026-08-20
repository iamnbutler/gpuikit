//! Tests for the build settings that keep linking an example inside the memory
//! a small machine has.
//!
//! Linking any example here is a whole-program link of gpui, and the debug info
//! was nearly all of what the linker held. It cost the same for every example,
//! because the bytes are gpui's and not the example's: `popover_demo.rs` (5 KB)
//! and `showcase.rs` (152 KB) peaked within 4% of each other. cargo picks `-j`
//! from the CPU count and knows nothing about memory, so four cores meant four
//! linkers at that peak at once, and on a 4-core / 5.9 GiB / no-swap box
//! `cargo build --all-targets` had three of them killed by the kernel and
//! exited 101 with `ld terminated with signal 9 [Killed]` — a message naming no
//! crate and no symbol, which reads as a compile error and is not one.
//!
//! `[profile.dev.package."*"] debug = 0` is what removes those bytes. The
//! comment above that table already argues dependencies are only ever linked
//! here, never iterated on; the same sentence applies to their debug info.
//! `[profile.dev-debuginfo]` is the hatch for anyone who needs the full thing
//! back, and it exists so the answer is never "edit `[profile.dev]` and
//! remember to put it back" — forgetting to put it back is the bug.
//!
//! `scripts/check.sh` is the other half. A piped build reports the *pipe's*
//! status, which is how a killed link was once reported as a green run, so that
//! script sets `pipefail` and takes each cargo status from `${PIPESTATUS[0]}`.
//!
//! They live in the lib rather than `tests/`, for the reason stated at
//! `src/elements.rs`'s `triage_coverage`: `cargo test --lib` is the command
//! that works in a constrained environment — and pointedly so here, since an
//! integration test would be one more whole-program link of gpui. Each parser
//! asserts it matched something before anything else is trusted, and there is a
//! fixture it is run against, for the same reason `release_version_guard` does
//! — a parser that silently found nothing reports success.

use std::collections::BTreeSet;
use std::path::PathBuf;

const CARGO_TOML: &str = include_str!("../Cargo.toml");

/// The script that runs this repository's documented checks. Named once so a
/// rename fails in one place.
const CHECK_SCRIPT: &str = "scripts/check.sh";

/// Debug-info settings that do *not* hold a linker's worth of type and local
/// descriptions. `0` emits none; `"line-tables-only"` keeps the file and line a
/// backtrace needs and drops the rest. Anything else — `1`, `2`, `true`,
/// `"full"` — is what this module exists to keep out of the dependency tier.
const CHEAP_DEBUG: &[&str] = &["0", "false", "\"line-tables-only\"", "\"none\""];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The value of `key` inside the table headed exactly `header`, as it is
/// written in the file (quotes and all), or `None` if either is absent.
///
/// A hand-rolled reader rather than a toml dependency, for the reason the other
/// guard modules give: the thing under test is the literal text of a file a
/// human edits, and a parser that normalises `debug = 2` into a number would
/// hide the difference between `2` and `"line-tables-only"` that is the whole
/// subject here. It stops at the next `[` in column zero, so a key from a later
/// table is never read as this one's.
fn table_value(toml: &str, header: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == header;
            continue;
        }
        if !inside || trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                return Some(value.split('#').next()?.trim().to_string());
            }
        }
    }
    None
}

/// Every `path = "…"` under an `[[example]]` block, in manifest order.
fn declared_example_paths(toml: &str) -> Vec<String> {
    let mut inside = false;
    let mut paths = Vec::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == "[[example]]";
            continue;
        }
        if !inside || trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("path") {
            if let Some(value) = rest.trim_start().strip_prefix('=') {
                paths.push(value.trim().trim_matches('"').to_string());
            }
        }
    }
    paths
}

/// Every path under `examples/` that cargo would auto-discover as an example if
/// `autoexamples` were left on: a top-level `examples/<name>.rs`, and the
/// directory form `examples/<name>/main.rs`.
///
/// The directory form is the one that matters most once auto-discovery is off.
/// A stray `examples/<name>.rs` at least looks like a target; a
/// `examples/<name>/main.rs` with no `[[example]]` block is built by nothing at
/// all, reports no error anywhere, and is discovered when somebody notices it
/// has rotted.
fn discoverable_example_paths() -> BTreeSet<String> {
    let dir = repo_root().join("examples");
    let mut found = BTreeSet::new();
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .collect::<Result<Vec<_>, _>>()
        .expect("cannot read an entry under examples/");
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        if path.is_file() && name.ends_with(".rs") {
            found.insert(format!("examples/{name}"));
        } else if path.is_dir() && path.join("main.rs").is_file() {
            found.insert(format!("examples/{name}/main.rs"));
        }
    }
    found
}

/// A manifest shaped like this one, with the settings this module rejects, so
/// the parsers above are shown reading a value they are meant to catch rather
/// than only ever agreeing with the file next to them.
const FIXTURE: &str = r#"[package]
name = "fixture"
autoexamples = true

[[example]]
name = "one"
path = "examples/one.rs"

# A comment mentioning debug = 7, which is not a setting.

[[example]]
name = "two"
path = "examples/two/main.rs"

[profile.dev]
opt-level = 0
debug = 2

[profile.dev.package."*"]
opt-level = 2
debug = 2

[profile.expensive]
inherits = "dev"
debug = 2   # trailing comment
"#;

#[test]
fn parsers_read_a_fixture_manifest() {
    assert_eq!(
        table_value(FIXTURE, "[profile.dev.package.\"*\"]", "debug").as_deref(),
        Some("2"),
        "the fixture's dependency tier is written `debug = 2`; the table reader \
         either did not find that table or read a key out of a neighbouring one"
    );
    assert_eq!(
        table_value(FIXTURE, "[profile.expensive]", "debug").as_deref(),
        Some("2"),
        "the fixture's `[profile.expensive]` sets `debug = 2` with a trailing \
         comment; the reader must strip the comment and not the value"
    );
    assert_eq!(
        table_value(FIXTURE, "[package]", "autoexamples").as_deref(),
        Some("true"),
        "the fixture leaves auto-discovery on; the reader did not see it"
    );
    assert_eq!(
        table_value(FIXTURE, "[profile.dev]", "debug-assertions"),
        None,
        "`debug-assertions` is not set in the fixture. A reader that matches it \
         on the `debug` prefix would pass every other assertion here while \
         reading the wrong key in the real manifest"
    );
    assert_eq!(
        declared_example_paths(FIXTURE),
        vec!["examples/one.rs", "examples/two/main.rs"],
        "the fixture declares exactly two examples, one in each form, and a \
         comment that mentions neither"
    );

    // Vacuity: the assertions below are only worth anything if `CHEAP_DEBUG`
    // can reject something. Take the value out of the fixture's expensive
    // profile — parsed by the same reader, not written here as a literal — and
    // prove it is rejected.
    let expensive = table_value(FIXTURE, "[profile.expensive]", "debug")
        .expect("just asserted the fixture's expensive profile has a debug key");
    assert!(
        !CHEAP_DEBUG.contains(&expensive.as_str()),
        "CHEAP_DEBUG accepts `{expensive}`, the very setting this module exists \
         to keep out of the dependency tier. Every assertion about the real \
         manifest below is vacuous until this holds"
    );
}

#[test]
fn dependencies_carry_no_debug_info_in_the_dev_profile() {
    let value = table_value(CARGO_TOML, "[profile.dev.package.\"*\"]", "debug").expect(
        "Cargo.toml has no `debug` under `[profile.dev.package.\"*\"]`. That \
         setting is what keeps `ld` from being OOM-killed while linking an \
         example: debug info for dependencies was nearly all of the 2.7 GB \
         every link held, and nothing in this repository is ever debugged \
         inside gpui. Use `--profile dev-debuginfo` when you need it back",
    );
    assert!(
        CHEAP_DEBUG.contains(&value.as_str()),
        "`[profile.dev.package.\"*\"] debug = {value}` puts full debug info back \
         into every dependency, and dependencies are essentially all of the code \
         volume in this build. That is what took a link to 2.7 GB and got three \
         linkers killed by the kernel on a 4-core / 5.9 GiB box, reported as \
         `ld terminated with signal 9 [Killed]` — a message naming no crate and \
         no symbol. Want it back for a debugging session? \
         `cargo build --profile dev-debuginfo`"
    );
}

#[test]
fn the_debuginfo_escape_hatch_profile_is_intact() {
    assert_eq!(
        table_value(CARGO_TOML, "[profile.dev-debuginfo]", "inherits").as_deref(),
        Some("\"dev\""),
        "`[profile.dev-debuginfo]` must inherit `dev`, or it is a different \
         build rather than the same build with its debug info back"
    );
    assert_eq!(
        table_value(CARGO_TOML, "[profile.dev-debuginfo]", "debug").as_deref(),
        Some("2"),
        "`[profile.dev-debuginfo]` exists to be the answer to \"but I need a \
         debugger\", so that the answer is never \"edit `[profile.dev]` and \
         remember to put it back\". With anything short of `debug = 2` it is not \
         that answer and somebody will edit `[profile.dev]` instead"
    );
    // The package table is restated rather than inherited: cargo does not
    // document whether `inherits` reaches into a package-override table or
    // whether a child's table replaces the parent's. Naming both keys makes the
    // profile correct either way, and `cargo build --profile dev-debuginfo
    // --lib -v` confirms dependencies get `-C opt-level=2 -C debuginfo=2`.
    assert_eq!(
        table_value(CARGO_TOML, "[profile.dev-debuginfo.package.\"*\"]", "debug").as_deref(),
        Some("2"),
        "`[profile.dev-debuginfo.package.\"*\"]` must restate `debug = 2`. \
         Dependencies are where the debug info was cut, so a hatch that does not \
         name them hands back the tier nobody removed"
    );
    assert_eq!(
        table_value(
            CARGO_TOML,
            "[profile.dev-debuginfo.package.\"*\"]",
            "opt-level"
        )
        .as_deref(),
        Some("2"),
        "`[profile.dev-debuginfo.package.\"*\"]` must restate `opt-level = 2` \
         as well. If a child's package table replaces its parent's rather than \
         merging, omitting this drops dependencies to `opt-level = 0` and the \
         showcase stops drawing smoothly — the thing b28732f set it for"
    );
}

#[test]
fn every_discoverable_example_is_declared_in_the_manifest() {
    assert_eq!(
        table_value(CARGO_TOML, "[package]", "autoexamples").as_deref(),
        Some("false"),
        "`[package] autoexamples = false` is what makes the `[[example]]` blocks \
         the whole list. Each example is another full link of gpui, so adding \
         one should be a line in the manifest next to the comment that says so, \
         not a file drop"
    );

    let declared: BTreeSet<String> = declared_example_paths(CARGO_TOML).into_iter().collect();
    assert!(
        !declared.is_empty(),
        "the manifest parser found no `[[example]]` block at all. It found \
         nothing rather than disagreeing with anything, so nothing below this \
         line means what it says"
    );

    for path in &declared {
        assert!(
            repo_root().join(path).is_file(),
            "`[[example]] path = \"{path}\"` names a file that does not exist. \
             With `autoexamples = false` the manifest is the only list, so a \
             stale entry here is a build error rather than a missing target"
        );
    }

    let discoverable = discoverable_example_paths();
    let orphans: Vec<&String> = discoverable.difference(&declared).collect();
    assert!(
        orphans.is_empty(),
        "{orphans:?} would have been auto-discovered as examples, but \
         `autoexamples = false` means nothing builds them: no target, no error, \
         no output. They rot until somebody notices. Add an `[[example]]` block \
         for each in Cargo.toml, or move the file out of examples/ if it is a \
         module rather than a binary. Both forms cargo discovers are checked \
         here: `examples/<name>.rs` and `examples/<name>/main.rs`. A module a \
         real example includes belongs in a subdirectory without a `main.rs` \
         (as `examples/input/fixtures.rs` does), which is invisible to both"
    );
}

#[test]
fn the_check_script_takes_its_status_from_cargo_and_not_from_the_pipe() {
    let path = repo_root().join(CHECK_SCRIPT);
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {CHECK_SCRIPT}: {e}. It is the script that runs this \
             repository's documented checks without a pipe swallowing a failure"
        )
    });

    // Comment lines are dropped before anything is looked for. That script's
    // own header explains `pipefail` and `${PIPESTATUS[0]}` in prose, so a
    // search over the whole text finds the explanation and passes whether or
    // not the code below it still does the thing. Caught by mutation: replacing
    // the real `set -euo pipefail` with `set -eu` left this test green.
    let script: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !script.trim().is_empty(),
        "{CHECK_SCRIPT} is nothing but comments once they are stripped, so \
         every assertion below would be searching an empty string"
    );
    assert!(
        script
            .lines()
            .any(|line| line.trim() == "set -euo pipefail"),
        "{CHECK_SCRIPT} does not `set -euo pipefail`. Without `pipefail` a \
         pipeline reports the *last* command's status, so `cargo build | tee \
         log` exits 0 while the build was OOM-killed — which is how a killed \
         link was once reported as a green run"
    );
    assert!(
        script.contains("PIPESTATUS[0]"),
        "{CHECK_SCRIPT} pipes cargo somewhere but never reads \
         `${{PIPESTATUS[0]}}`. `pipefail` alone makes the pipeline fail, but the \
         script still has to name cargo's own status to say which check failed \
         rather than reporting `tee`'s"
    );
    assert!(
        script.contains("signal 9"),
        "{CHECK_SCRIPT} does not recognise `signal 9`. `ld terminated with \
         signal 9 [Killed]` names no crate and no symbol, so it reads as a \
         compile error; the script exists partly to say what it actually is and \
         how to get past it (`CARGO_BUILD_JOBS=1`)"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)
            .expect("just read the script")
            .permissions()
            .mode();
        assert!(
            mode & 0o111 != 0,
            "{CHECK_SCRIPT} is not executable ({mode:o}). The README tells \
             contributors to run it by path"
        );
    }
}

#[test]
fn the_dev_profile_still_optimizes_dependencies() {
    // Not a memory setting, and it is here because this change is the most
    // likely thing to knock it out: `[profile.dev.package."*"]` grew a `debug`
    // key, and the next person to edit that table is editing the line above
    // `opt-level`. b28732f set it, and it is the difference between a debug
    // showcase that draws smoothly and one that does not.
    assert_eq!(
        table_value(CARGO_TOML, "[profile.dev.package.\"*\"]", "opt-level").as_deref(),
        Some("2"),
        "`[profile.dev.package.\"*\"] opt-level = 2` is gone. Cutting the \
         dependency tier's debug info was not supposed to touch its \
         optimization: without this the debug showcase does not draw smoothly, \
         which is what b28732f set it for"
    );
}
