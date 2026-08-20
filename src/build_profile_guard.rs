//! Tests for the build configuration that keeps `ld` from being OOM-killed
//! while linking this crate's examples.
//!
//! Eight example binaries live in `examples/`, and each one is a full link of
//! gpui. `cargo build --all-targets` and a bare `cargo test` link all eight;
//! cargo sizes `-j` from the CPU count with no knowledge of a memory limit, so
//! several `ld` processes run at once, each holding that binary's debug info,
//! and the kernel kills one. The message is `ld terminated with signal 9
//! [Killed]` — it names no crate and no symbol, so it reads as a compile error
//! that does not exist, and three separate runs in one week were lost hunting
//! a type error that was never there.
//!
//! Three pieces of configuration answer it, and this module is what holds them
//! in place:
//!
//! 1. `[profile.dev] debug = "line-tables-only"` in `Cargo.toml`. Full DWARF
//!    for this crate *and* for gpui beneath it is the bulk of what the linker
//!    holds. `[profile.dev.package."*"]` must not raise it back, which is the
//!    reason for a second test rather than a second clause in the first.
//! 2. `-Csplit-debuginfo=unpacked`, for Linux targets only, in
//!    `.cargo/config.toml`. On Linux the dev default is `off`, meaning every
//!    byte of DWARF is copied through `ld` into the linked image. This cannot
//!    live in `[profile.dev]`: cargo profiles take no `cfg`, and the value is a
//!    hard error on `windows-msvc`.
//! 3. `required-features = ["examples"]` on every `[[example]]`, against a
//!    feature that enables nothing. This is what removes the eight links from
//!    `cargo test` and `cargo {build,check} --all-targets` outright.
//!
//! **The gate does not cover `--all-features`.** Cargo offers no way to hold a
//! feature back from that flag, so `cargo test --all-features` — the command
//! that produced the kill on `markdown_streaming` — still builds all eight.
//! That case rests entirely on 1 and 2. If it still dies, the next lever is
//! `[profile.dev.package."*"] debug = 0`, which is why the override test below
//! rejects only a *raised* value.
//!
//! Autodiscovery is the hole in the gate: a top-level `examples/*.rs` or an
//! `examples/*/main.rs` becomes a target with no manifest entry, and an
//! autodiscovered target cannot carry `required-features`. `context_menu.rs`
//! was exactly that before this change. One new undeclared file restores the
//! whole bug, so the test for it reads the directory rather than trusting the
//! manifest.
//!
//! These live in the lib rather than `tests/`, for the reason stated at
//! `src/elements.rs`'s `triage_coverage` and repeated in
//! `src/release_version_guard.rs`: `cargo test --lib` is the command that works
//! in a constrained environment — which is precisely the subject here. Each
//! parser asserts it matched something before anything is trusted, for the same
//! reason those modules do: a parser that silently found nothing reports
//! success.

use std::path::PathBuf;

const CARGO_TOML: &str = include_str!("../Cargo.toml");
const CARGO_CONFIG: &str = include_str!("../.cargo/config.toml");
const EXAMPLES_README: &str = include_str!("../examples/README.md");

/// The feature every example is gated behind. Named once so a rename fails in
/// one place rather than eight.
const GATE: &str = "examples";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The body of a `[header]` table: every line after it, up to the next line
/// that opens a table of its own. Comment and blank lines come through as they
/// are; callers read keys, not lines.
fn table<'a>(toml: &'a str, header: &str) -> Option<&'a str> {
    let start = toml.find(&format!("\n{header}\n"))? + 1 + header.len() + 1;
    let rest = &toml[start..];
    let end = rest
        .match_indices("\n[")
        .next()
        .map(|(i, _)| i + 1)
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

/// The right-hand side of `key = …` in a table body, trimmed, with any
/// trailing `# comment` removed. `None` when the table does not set the key.
fn value<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    body.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix('=')?.trim();
        Some(rest.split('#').next().unwrap_or(rest).trim())
    })
}

/// One `[[example]]` entry, as the manifest declares it.
struct Example {
    name: String,
    path: String,
    required_features: Option<String>,
}

/// Every `[[example]]` block in the manifest, in file order.
///
/// Matched on whole lines: the manifest's own comments name `[[example]]` in
/// prose, and a substring split would read one of those as a ninth target.
fn declared_examples() -> Vec<Example> {
    let mut examples = Vec::new();
    let mut lines = CARGO_TOML.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim() != "[[example]]" {
            continue;
        }

        let mut body = String::new();
        while let Some(next) = lines.peek() {
            if next.trim_start().starts_with('[') {
                break;
            }
            body.push_str(lines.next().expect("peeked"));
            body.push('\n');
        }

        examples.push(Example {
            name: value(&body, "name")
                .unwrap_or_else(|| panic!("an [[example]] declares no name:\n{body}"))
                .trim_matches('"')
                .to_string(),
            path: value(&body, "path")
                .unwrap_or_else(|| panic!("an [[example]] declares no path:\n{body}"))
                .trim_matches('"')
                .to_string(),
            required_features: value(&body, "required-features").map(str::to_string),
        });
    }

    assert!(
        !examples.is_empty(),
        "no [[example]] blocks parsed out of Cargo.toml at all — the parser has \
         stopped matching, not the manifest stopped declaring"
    );
    examples
}

/// Files under `examples/` that cargo turns into targets by itself: top-level
/// `examples/*.rs`, and `examples/*/main.rs`. Nothing else — `examples/input/
/// fixtures.rs` is reached by `mod fixtures;` from `sandbox.rs`, not built on
/// its own.
fn autodiscoverable_paths() -> Vec<String> {
    let dir = repo_root().join("examples");
    let mut found = Vec::new();

    for entry in std::fs::read_dir(&dir).expect("examples/ is not readable") {
        let entry = entry.expect("examples/ entry is not readable");
        let file_type = entry.file_type().expect("examples/ entry has no file type");
        let name = entry.file_name().to_string_lossy().into_owned();

        if file_type.is_file() && name.ends_with(".rs") {
            found.push(format!("examples/{name}"));
        } else if file_type.is_dir() && entry.path().join("main.rs").is_file() {
            found.push(format!("examples/{name}/main.rs"));
        }
    }

    assert!(
        !found.is_empty(),
        "no example sources found under examples/ — the scan is broken, since \
         the manifest declares some"
    );
    found.sort();
    found
}

#[test]
fn dev_builds_carry_line_tables_only() {
    let dev = table(CARGO_TOML, "[profile.dev]").expect("Cargo.toml has no [profile.dev] table");
    let debug = value(dev, "debug").expect("[profile.dev] sets no debug level");

    assert_eq!(
        debug, "\"line-tables-only\"",
        "[profile.dev] debug is `{debug}`. Full debug info for this crate and \
         for gpui beneath it is the bulk of what `ld` holds, and several links \
         run at once under cargo's default -j; raising this is what got them \
         OOM-killed. A debugging session asks for the detail on that build \
         alone with RUSTFLAGS=\"-Cdebuginfo=2\"."
    );
}

#[test]
fn the_wildcard_package_override_does_not_restore_debug_info() {
    let over = table(CARGO_TOML, "[profile.dev.package.\"*\"]").expect(
        "Cargo.toml has no [profile.dev.package.\"*\"] table — if it was removed on \
                 purpose, remove this test with it",
    );

    // Absent is the state that ships, and `0` is the next lever if
    // `--all-features` still dies; only a raised value undoes the fix.
    match value(over, "debug") {
        None => {}
        Some("0") | Some("false") | Some("\"none\"") => {}
        Some(other) => panic!(
            "[profile.dev.package.\"*\"] sets debug = {other}, which puts \
             dependency debug info — gpui's above all — back in front of the \
             linker and undoes [profile.dev] debug = \"line-tables-only\""
        ),
    }
}

#[test]
fn linux_targets_keep_debug_info_out_of_the_linked_image() {
    let targets = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"];

    for target in targets {
        let body = table(CARGO_CONFIG, &format!("[target.{target}]"))
            .unwrap_or_else(|| panic!(".cargo/config.toml has no [target.{target}] table"));
        let flags = value(body, "rustflags")
            .unwrap_or_else(|| panic!("[target.{target}] sets no rustflags"));

        assert!(
            flags.contains("split-debuginfo=unpacked"),
            "[target.{target}] rustflags is {flags}, without \
             -Csplit-debuginfo=unpacked. On Linux the dev default is `off`, so \
             every byte of DWARF is copied through `ld` into the linked image."
        );
    }
}

#[test]
fn split_debuginfo_stays_scoped_to_the_targets_that_can_take_it() {
    // `unpacked` is a hard error on windows-msvc, and cargo profiles take no
    // cfg — so this must not migrate into [profile.dev] or a [build] table.
    assert!(
        !CARGO_TOML.contains("split-debuginfo"),
        "Cargo.toml sets split-debuginfo. A profile cannot be cfg-gated and \
         `unpacked` is a hard error on windows-msvc: it belongs in \
         .cargo/config.toml under the Linux targets, which is where the rest \
         of this module expects it."
    );

    for (i, line) in CARGO_CONFIG.lines().enumerate() {
        let line = line.trim();
        if line.starts_with('[') {
            assert!(
                line.starts_with("[target."),
                ".cargo/config.toml line {} opens `{line}`. Everything in this \
                 file is target-scoped on purpose; a [build] rustflags table \
                 would apply -Csplit-debuginfo=unpacked to windows-msvc too, \
                 where it is a hard error.",
                i + 1
            );
        }
    }
}

#[test]
fn every_example_is_gated_behind_the_examples_feature() {
    let expected = format!("[\"{GATE}\"]");

    for example in declared_examples() {
        let Example {
            name,
            required_features,
            ..
        } = &example;
        let actual = required_features.as_deref().unwrap_or("nothing");

        assert_eq!(
            actual, expected,
            "example `{name}` requires {actual}, not {expected}. An ungated \
             example is linked by `cargo test` and by \
             `cargo {{build,check}} --all-targets`, and each link is a full \
             link of gpui — eight at cargo's default -j is what the kernel \
             killed."
        );
    }
}

#[test]
fn the_gate_feature_exists_and_enables_nothing() {
    let features = table(CARGO_TOML, "[features]").expect("Cargo.toml has no [features] table");
    let gate = value(features, GATE).unwrap_or_else(|| {
        panic!(
            "[features] declares no `{GATE}`, so every example's \
             required-features names a feature that does not exist and cargo \
             refuses the manifest"
        )
    });

    assert_eq!(
        gate, "[]",
        "the `{GATE}` feature enables {gate}. It exists only to be required by \
         the example targets; giving it dependencies would make \
         `--features {GATE}` mean something else as well."
    );
}

#[test]
fn no_example_is_left_to_autodiscovery() {
    let declared: Vec<String> = declared_examples().into_iter().map(|e| e.path).collect();

    for path in autodiscoverable_paths() {
        assert!(
            declared.contains(&path),
            "{path} is an example target that no [[example]] block declares, so \
             cargo autodiscovers it — and an autodiscovered target cannot carry \
             required-features. It is linked by `cargo test` and \
             `--all-targets` no matter what the other seven do. Declare it in \
             Cargo.toml with required-features = [\"{GATE}\"]."
        );
    }
}

#[test]
fn every_declared_example_path_exists() {
    for Example { name, path, .. } in declared_examples() {
        assert!(
            repo_root().join(&path).is_file(),
            "example `{name}` points at {path}, which is not a file. A \
             [[example]] with a dead path fails the whole manifest, so this \
             fails first and says which one."
        );
    }
}

#[test]
fn the_examples_readme_runs_examples_through_the_gate() {
    // Deliberately about the commands and nothing else: prose goes stale on
    // rewording, and a test that greps an explanation only teaches people to
    // stop editing the explanation.
    let commands: Vec<&str> = EXAMPLES_README
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("cargo run --example") || line.starts_with("cargo build --example")
        })
        .collect();

    assert!(
        !commands.is_empty(),
        "examples/README.md documents no `cargo run --example` command at all — \
         either the scan broke or the one place that says how to run these is gone"
    );

    for command in commands {
        assert!(
            command.contains(&format!("--features {GATE}"))
                || command.contains(&format!("--features \"{GATE}"))
                || command.contains(&format!("{GATE},"))
                || command.contains(&format!(",{GATE}")),
            "examples/README.md documents `{command}`, which does not pass \
             --features {GATE}. Since the gate landed, that command fails with \
             `no example target named ...` — a reader following the README \
             gets an error, not a window."
        );
    }
}
