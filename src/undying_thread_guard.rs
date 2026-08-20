//! Tests for the rule that this crate creates no thread it cannot join.
//!
//! `cargo test --lib` aborted with `SIGABRT` roughly one run in three, during
//! teardown, after every test had already passed (#190). The backtrace named a
//! thread called `async-io`, under `std::thread::lifecycle::spawn_unchecked`,
//! in no test at all: `panic in a destructor during cleanup`, a second panic
//! while unwinding, abort. The crate in that name appears nowhere in this
//! repository, which is why the thread read as gpui's. It was not.
//!
//! `smol::Timer` *is* `async_io::Timer`. Constructing one reaches
//! `async_io::driver::init`, which spawns the process-global OS thread named
//! `async-io`; that thread's `main_loop` is an infinite loop with no exit path
//! by design, so there is no clean-shutdown call to add. It was still inside
//! the reactor when the test binary exited and lost the race against process
//! teardown. Two `smol::Timer::after` calls — the cursor blink interval and
//! the toast auto-dismiss — were the only reason it existed. Removing them
//! removes the thread; `blocking`'s never-joining pool (`blocking-*`) leaves
//! with the same dependency for free.
//!
//! Delays now go through gpui's own `BackgroundExecutor::timer`. That is not a
//! pure refactor: under `#[gpui::test]` these timers run on the deterministic
//! clock instead of a real wall-clock thread, so a blink that never fired
//! inside a test may now fire once that test advances the clock. That is the
//! correct behaviour — the old arrangement raced the deterministic scheduler —
//! but it is worth knowing before writing a blink or toast timing test.
//!
//! Three tempting non-fixes, for the next person to arrive at this from a red
//! run rather than from the issue: `--test-threads=1`, a `std::process::exit`
//! after the harness, and retry-on-abort all hide it, and all three leave a
//! live thread racing teardown in every consumer of the library rather than
//! only in the test binary. The thread is the bug. It is fixed by never
//! creating it.
//!
//! The load-bearing change is in `Cargo.toml`: with no `smol` dependency,
//! `use smol::Timer` does not compile, so the invariant is a build error and
//! not a convention. These tests cover what a build error cannot — a
//! re-added manifest entry, and the two call sites still being scheduled on
//! the executor rather than on something new.
//!
//! Like `release_version_guard` and `build_profile_guard`, this lives in the
//! lib rather than in `tests/`, because `cargo test --lib` is the command that
//! works in a constrained environment (`examples/README.md`, `.tasks/verify`)
//! — and it is the command that aborted. Each scan asserts it actually looked
//! at something before its "found nothing" is trusted.
//!
//! **Platform note.** On macOS this removes `async-io` from the dependency
//! graph outright. On Linux, `gpui-platform` depends on `smol` itself, so the
//! crate stays in the graph — `cargo tree -i smol` shows exactly one path, and
//! the direct edge is gone. That residue is outside this crate's control,
//! which is why `scripts/run-tests.sh` is worth keeping rather than deleting
//! as now-dead.

use std::path::{Path, PathBuf};

const CARGO_TOML: &str = include_str!("../Cargo.toml");

/// The crates whose `Timer` spawns the undying thread. `smol` re-exports
/// `async-io`'s, so both names are forbidden; the underscore spellings are how
/// they read in Rust source.
const FORBIDDEN_CRATES: [&str; 2] = ["smol", "async-io"];

/// This module names every forbidden string it is looking for, in string
/// literals that `code_only` does not strip, so it would flag itself. It is
/// the one file the source scan skips, and the scan asserts the count so the
/// exemption cannot silently widen to a second file.
const SELF: &str = "undying_thread_guard.rs";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `source` with `//` line comments and `/* … */` blocks removed. String
/// literals are left alone — this is a scan for imports and calls, not a
/// parser, and the one file whose literals would trip it is skipped by name.
fn code_only(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut in_block = false;

    while let Some(c) = chars.next() {
        if in_block {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block = false;
            }
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    for c in chars.by_ref() {
                        if c == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    in_block = true;
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }

    out
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

/// The keys of a `[dependencies]`-shaped table in `Cargo.toml`: everything
/// left of the first `=` on a line that is not blank, not a comment, and not
/// a continuation of a multi-line value.
fn dependency_keys(toml: &str, header: &str) -> Vec<String> {
    let start = toml
        .find(&format!("\n{header}\n"))
        .unwrap_or_else(|| panic!("Cargo.toml declares no `{header}` table"))
        + 1
        + header.len()
        + 1;
    let rest = &toml[start..];
    let end = rest
        .match_indices("\n[")
        .next()
        .map(|(i, _)| i + 1)
        .unwrap_or(rest.len());

    rest[..end]
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, _)| key.trim().trim_matches('"').to_string())
        .collect()
}

#[test]
fn the_manifest_declares_no_async_runtime() {
    for table in ["[dependencies]", "[dev-dependencies]"] {
        let keys = dependency_keys(CARGO_TOML, table);
        assert!(
            keys.len() > 1,
            "read {} dependencies out of `{table}` — the parser found nothing to check",
            keys.len(),
        );

        for forbidden in FORBIDDEN_CRATES {
            let underscored = forbidden.replace('-', "_");
            assert!(
                !keys
                    .iter()
                    .any(|key| key == forbidden || *key == underscored),
                "`{table}` declares `{forbidden}`. Constructing its `Timer` spawns the \
                 process-global `async-io` thread, whose `main_loop` never returns, and it \
                 aborts the test binary at exit (#190). Use \
                 `cx.background_executor().timer(duration)` — see this module's docs.",
            );
        }
    }
}

#[test]
fn no_source_file_reaches_for_one() {
    let src = repo_root().join("src");
    let files = rust_files(&src);
    assert!(
        files.len() > 20,
        "walked src/ and found {} Rust files — the walker is broken",
        files.len(),
    );

    let mut scanned = 0usize;
    for file in &files {
        if file.file_name().is_some_and(|name| name == SELF) {
            continue;
        }
        scanned += 1;

        let code = code_only(&std::fs::read_to_string(file).expect("source file is readable"));
        for forbidden in FORBIDDEN_CRATES {
            let path = format!("{}::", forbidden.replace('-', "_"));
            assert!(
                !code.contains(&path),
                "{} uses `{path}` outside a comment. That crate's `Timer` spawns the \
                 `async-io` thread, which never exits and aborts the test binary at \
                 teardown (#190). Use `cx.background_executor().timer(duration)`.",
                file.display(),
            );
        }
    }

    assert_eq!(
        scanned,
        files.len() - 1,
        "exactly one file — `{SELF}`, which names the forbidden strings in its own literals \
         — is exempt from this scan. {} were skipped.",
        files.len() - scanned,
    );
}

#[test]
fn the_two_delays_are_scheduled_on_the_executor() {
    for (relative, expected) in [("src/input/blink.rs", 2), ("src/elements/toast.rs", 1)] {
        let path = repo_root().join(relative);
        let code = code_only(&std::fs::read_to_string(&path).expect("source file is readable"));
        let found = code.matches("background_executor().timer(").count();
        assert_eq!(
            found, expected,
            "{relative} schedules {found} delay(s) on `background_executor().timer(`, \
             expected {expected}. A delay that goes anywhere else is how the `async-io` \
             thread came back (#190) — if the call site legitimately moved, move this \
             expectation with it.",
        );
    }
}
