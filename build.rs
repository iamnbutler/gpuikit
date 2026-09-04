//! Exposes the gpui version this crate actually resolved to as the
//! `GPUI_VERSION` compile-time env var, so the showcase can render it instead
//! of a hand-written string that drifts the moment the dependency is bumped.
//!
//! The version is read from `Cargo.lock`, which is the resolved truth rather
//! than the requirement in `Cargo.toml` — and it is baked in at compile time,
//! so it survives into the wasm build, which has no lockfile at runtime. When
//! there is no lockfile (a downstream crate building gpuikit as a dependency),
//! it falls back to `unknown`; only the `showcase` example reads the var, and
//! that example is only ever built inside this repository.

use std::path::Path;

fn main() {
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock.display());

    let version = std::fs::read_to_string(&lock)
        .ok()
        .and_then(|contents| gpui_version(&contents))
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GPUI_VERSION={version}");
}

/// The `version` of the `gpui-unofficial` package in a `Cargo.lock`. Every
/// `[[package]]` block lists `name` then `version` on consecutive lines, so
/// the version wanted is the first `version = "…"` after the matching `name`.
fn gpui_version(lock: &str) -> Option<String> {
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == r#"name = "gpui-unofficial""# {
            for next in lines.by_ref() {
                if let Some(rest) = next.trim().strip_prefix("version = ") {
                    return Some(rest.trim_matches('"').to_string());
                }
            }
        }
    }
    None
}
