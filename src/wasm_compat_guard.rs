//! Tests for the rule that runtime code stays compilable *and runnable* on
//! `wasm32-unknown-unknown`.
//!
//! gpuikit runs in the browser on gpui's web platform: the showcase is hosted
//! at <https://nate.rip/gpuikit/>, built by `examples/showcase-web/`. The
//! failure mode this guards against is quiet: several std APIs *compile* for
//! wasm and then fail at runtime — `std::time::Instant::now()` panics
//! outright, and `std::fs` returns errors on a filesystem that does not
//! exist. The first symptom is a crashed or broken page in a browser, on a
//! target no local `cargo test` exercises.
//!
//! CI's wasm job proves the crate and the showcase *compile* for the target.
//! Compiling is exactly what these APIs do, so that job cannot catch them;
//! this scan is the guard for the runtime half: it forbids the APIs by name
//! in source, with an explicit per-file allowlist for the uses that are
//! legitimate.
//!
//! Two kinds of use are allowed, and each allowlist entry says which it is:
//!
//! - **Native-only APIs by design.** `fs::File`, keymap-loading, and the
//!   editor's theme-from-file all exist to read the local disk; their docs say
//!   they are native-only. A wasm consumer simply doesn't call them.
//! - **Test-only code.** Guard modules and `#[cfg(test)]` blocks never reach
//!   a wasm binary.
//!
//! Every entry is also asserted to still be true — a file that stops using
//! the API fails the scan until its entry is deleted, so the allowlist cannot
//! go stale, and a *new* use anywhere fails until someone consciously adds an
//! entry (or, better, reaches for a wasm-safe alternative: `web-time` for
//! clocks, gpui's asset system for bundled files).
//!
//! Like the other guards, this lives in the lib rather than `tests/` because
//! `cargo test --lib` is the command that works everywhere. Each scan asserts
//! it actually looked at something before its "found nothing" is trusted.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// This module names every forbidden string in its own literals, so it would
/// flag itself. It is the one file the scan skips outright, and the scan
/// asserts the count so the exemption cannot silently widen.
const SELF: &str = "wasm_compat_guard.rs";

/// A std API that misbehaves on `wasm32-unknown-unknown`, the files allowed
/// to use it, and why each is allowed.
struct ForbiddenApi {
    /// Substring that marks a use. Matched against comment-stripped source,
    /// so prose mentions don't count; string literals do, which is why
    /// [`SELF`] is skipped by name.
    needle: &'static str,
    /// Only flag lines that also contain one of these (empty = flag any
    /// occurrence). This is how `std::time::Duration` — fine on wasm — stays
    /// allowed while `Instant` and `SystemTime` from the same import line are
    /// not.
    line_must_also_contain: &'static [&'static str],
    /// What happens on wasm, shown in the failure message.
    why: &'static str,
    /// (path relative to `src/`, justification) — every entry is asserted to
    /// still match, so deleting the last use of an API also means deleting
    /// its entry here.
    allowed: &'static [(&'static str, &'static str)],
}

const FORBIDDEN: &[ForbiddenApi] = &[
    ForbiddenApi {
        needle: "std::time",
        line_must_also_contain: &["Instant", "SystemTime"],
        why: "`std::time::Instant::now()` and `SystemTime::now()` panic on \
              wasm32-unknown-unknown. Use the `web-time` crate (already a dependency), \
              which re-exports the std types on native targets and browser-backed ones \
              on wasm — see `input/state.rs` for the pattern.",
        allowed: &[],
    },
    ForbiddenApi {
        needle: "std::fs",
        line_must_also_contain: &[],
        why: "there is no filesystem on wasm32-unknown-unknown; `std::fs` compiles and then \
              errors at runtime. Bundle files with the asset system, or document the API as \
              native-only and add it to this allowlist.",
        allowed: &[
            ("fs.rs", "the file-editing module exists to read/write the local disk; native-only by design and documented as such"),
            ("keymap/mod.rs", "loads keymap JSON from user-supplied paths; native-only by design and documented as such"),
            ("editor/syntax_highlighter.rs", "`load_theme_from_file` reads a .tmTheme from disk; native-only by design and documented as such"),
            ("a11y.rs", "test-only: golden-file assertions inside `#[cfg(test)] mod tests`"),
            ("element_id.rs", "test-only: source scan inside `#[cfg(test)] mod tests`"),
            ("build_profile_guard.rs", "test-only guard module"),
            ("release_input_validation.rs", "test-only guard module"),
            ("release_version_guard.rs", "test-only guard module"),
            ("undying_thread_guard.rs", "test-only guard module"),
        ],
    },
    ForbiddenApi {
        needle: "std::thread::spawn",
        line_must_also_contain: &[],
        why: "spawning OS threads traps on wasm32-unknown-unknown (gpui's web platform runs \
              background work on web workers instead). Use gpui's BackgroundExecutor.",
        allowed: &[(
            "utils/element_manager.rs",
            "test-only: a thread-safety test inside `#[cfg(test)] mod tests`",
        )],
    },
];

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

/// True if `code` uses the API: contains `needle` on a line that also
/// mentions one of `line_must_also_contain` (or on any line when that list is
/// empty).
fn uses_api(code: &str, api: &ForbiddenApi) -> bool {
    code.lines().any(|line| {
        line.contains(api.needle)
            && (api.line_must_also_contain.is_empty()
                || api
                    .line_must_also_contain
                    .iter()
                    .any(|extra| line.contains(extra)))
    })
}

#[test]
fn wasm_hostile_apis_appear_only_where_allowed() {
    let src = repo_root().join("src");
    let files = rust_files(&src);
    assert!(
        files.len() > 20,
        "walked src/ and found {} Rust files — the walker is broken",
        files.len(),
    );

    let mut scanned = 0usize;
    let mut matched: Vec<(usize, String)> = Vec::new();

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
        let code = code_only(&std::fs::read_to_string(file).expect("source file is readable"));

        for (index, api) in FORBIDDEN.iter().enumerate() {
            if !uses_api(&code, api) {
                continue;
            }
            let allowed = api.allowed.iter().any(|(path, _)| *path == relative);
            assert!(
                allowed,
                "{relative} uses `{}`{}, which is not on the allowlist: {} \
                 If this use is genuinely native-only or test-only, add an entry to \
                 `FORBIDDEN` in {SELF} saying which; otherwise use the wasm-safe \
                 alternative the message names.",
                api.needle,
                if api.line_must_also_contain.is_empty() {
                    String::new()
                } else {
                    format!(" (with {:?})", api.line_must_also_contain)
                },
                api.why,
            );
            matched.push((index, relative.clone()));
        }
    }

    assert_eq!(
        scanned,
        files.len() - 1,
        "exactly one file — `{SELF}`, which names the forbidden strings in its own \
         literals — is exempt from this scan. {} were skipped.",
        files.len() - scanned,
    );

    // Every allowlist entry must still be earning its place. A stale entry is
    // how "being ported on a parallel branch" quietly becomes "allowed
    // forever".
    let matched: BTreeSet<(usize, String)> = matched.into_iter().collect();
    for (index, api) in FORBIDDEN.iter().enumerate() {
        for (path, justification) in api.allowed {
            assert!(
                matched.contains(&(index, (*path).to_string())),
                "src/{path} no longer uses `{}` — delete its allowlist entry from {SELF} \
                 (it was allowed because: {justification}).",
                api.needle,
            );
        }
    }
}
