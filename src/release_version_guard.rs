//! Tests for the guard that stops a release workflow publishing a version
//! `CHANGELOG.md` does not name — in both workflows that can reach
//! `cargo publish`.
//!
//! `release.yml` was guarded first, by #171. It is not the file that publishes:
//! `release-deploy.yml` runs `cargo publish`, and reaches it by three doors —
//! a `workflow_call` from `release.yml`, a `workflow_dispatch` with free-form
//! `version`/`tag`, and a `push:` of any tag matching
//! `v[0-9]+.[0-9]+.[0-9]+*`. Only the first passed under a guard living in the
//! other file. The second half of this module covers the step that closes the
//! other two, in `release-deploy.yml`'s `publish` job.
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
const RELEASE_DEPLOY_YML: &str = include_str!("../.github/workflows/release-deploy.yml");

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

// ---------------------------------------------------------------------------
// release-deploy.yml — the workflow that actually publishes.
//
// Everything above is about `release.yml`, which computes a version and pushes
// a tag. It does not run `cargo publish`; this file does, and it can be reached
// without `release.yml` having run at all — by a `workflow_dispatch`, or by a
// `push:` of any tag matching `v[0-9]+.[0-9]+.[0-9]+*`. The step covered below
// is the guard on those routes.
//
// The complication is that the script does not exist at any tag cut before it
// landed, and the `publish` job checks out `ref: <tag>`. So the step decides
// what an *absent* script means against a version floor, and these tests pin
// both halves: where the floor sits, and what the step's own shell does on each
// side of it.
// ---------------------------------------------------------------------------

/// The step this adds to `release-deploy.yml`'s `publish` job. Named once so a
/// rename fails in one place rather than passing by finding nothing.
const PUBLISH_GUARD_STEP: &str = "Verify CHANGELOG.md names the version being published";

/// Every version this repository has already tagged — `v0.3.0 v0.4.0 v0.5.0
/// v0.5.1 v0.7.0`, without the `v`. None of those trees carries
/// `verify-release-version.sh`, which is why the step needs an absent branch at
/// all, and three of them (0.3.0, 0.5.0, 0.5.1) would *fail* the guard if it
/// were somehow made available to them: their topmost changelog headings are
/// 0.2.0, 0.4.0 and 0.4.0. That is the reason the branch is a warning rather
/// than an attempt to check the old trees anyway.
const TAGS_THAT_PREDATE_THE_GUARD: [&str; 5] = ["0.3.0", "0.4.0", "0.5.0", "0.5.1", "0.7.0"];

/// One step, from its `- name:` line to the next step's, or the end of the
/// file. Steps in both workflows sit at six spaces.
fn step<'a>(yaml: &'a str, name: &str) -> &'a str {
    let needle = format!("      - name: {name}\n");
    let at = yaml.find(&needle).unwrap_or_else(|| {
        panic!(
            "no step named `{name}`. If it was renamed, rename it here too — \
             otherwise this test passes by finding nothing"
        )
    });
    let rest = &yaml[at..];
    let end = rest[1..]
        .find("\n      - name: ")
        .map_or(rest.len(), |i| i + 1);
    &rest[..end]
}

/// A step's `run: |` body, dedented, ready to hand to `bash -c`.
///
/// Every non-blank line must carry the block's ten-space indent. A body that
/// was silently truncated at the first line indented less than that would make
/// every behavioural test below pass against nothing, which is the failure mode
/// `release_input_validation.rs` guards with its own `run:`-body floor.
fn run_body(step: &str) -> String {
    const KEY: &str = "        run: |\n";
    const INDENT: &str = "          ";

    let at = step
        .find(KEY)
        .unwrap_or_else(|| panic!("the step has no `run: |` block:\n{step}"));
    let mut body = String::new();

    for line in step[at + KEY.len()..].lines() {
        if line.trim().is_empty() {
            body.push('\n');
            continue;
        }
        let dedented = line.strip_prefix(INDENT).unwrap_or_else(|| {
            panic!(
                "a line of the `run:` body is indented less than the block's ten \
                 spaces, so the body is being read as ending before it: {line:?}"
            )
        });
        body.push_str(dedented);
        body.push('\n');
    }

    assert!(
        body.lines().filter(|l| !l.trim().is_empty()).count() >= 20,
        "the guard step's body came out as {} non-blank lines, which is shorter than \
         the step has ever been. A truncated body passes every behavioural test below \
         by doing nothing:\n{body}",
        body.lines().filter(|l| !l.trim().is_empty()).count()
    );

    body
}

/// `[major, minor, patch]`, with any pre-release suffix dropped — the same
/// reading the step's own `${VERSION%%-*}` plus `sort -V` gives.
fn version_key(version: &str) -> Vec<u64> {
    let base = version.split('-').next().unwrap_or(version);
    let parts: Vec<u64> = base
        .split('.')
        .map(|part| {
            part.parse().unwrap_or_else(|_| {
                panic!("`{version}` is not a numeric x.y.z version, so it cannot be ordered")
            })
        })
        .collect();
    assert_eq!(
        parts.len(),
        3,
        "`{version}` does not have three numeric components"
    );
    parts
}

/// The `FLOOR=` the step sets, read back out of the workflow rather than
/// restated here — a second copy of it would agree with the first exactly until
/// somebody changed one.
fn publish_guard_floor() -> String {
    run_body(step(RELEASE_DEPLOY_YML, PUBLISH_GUARD_STEP))
        .lines()
        .find_map(|line| line.trim().strip_prefix("FLOOR=").map(str::to_string))
        .expect("the guard step no longer sets `FLOOR=`, which is what decides what an absent script means")
}

/// A stand-in for the tree the `publish` job checks out: a `CHANGELOG.md` whose
/// topmost versioned heading is `top_heading`, and a `.github/scripts/` that
/// either carries the real guard script or does not.
fn published_tree(top_heading: &str, carries_the_guard: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");

    std::fs::write(
        dir.path().join("CHANGELOG.md"),
        format!("# Changelog\n\n## [Unreleased]\n\n## [{top_heading}] - 2026-01-01\n\n- a note\n"),
    )
    .expect("the fixture changelog is writable");

    if carries_the_guard {
        std::fs::create_dir_all(dir.path().join(".github/scripts"))
            .expect("the fixture script directory is creatable");
        std::fs::copy(repo_root().join(SCRIPT), dir.path().join(SCRIPT))
            .expect("the real guard script is readable");
    }

    dir
}

/// The step's actual shell, run against a tree, the way the runner runs it:
/// `bash -e`, the version bound in the environment rather than pasted in.
fn run_publish_guard(tree: &std::path::Path, version: &str) -> std::process::Output {
    Command::new("bash")
        .arg("-e")
        .arg("-c")
        .arg(run_body(step(RELEASE_DEPLOY_YML, PUBLISH_GUARD_STEP)))
        .current_dir(tree)
        .env("VERSION", version)
        .output()
        .expect("bash is available to run the guard step")
}

/// The guard has to come before the install, the check and the publish — both
/// because it is worthless after `cargo publish` and because a wrong version
/// should cost a second rather than an apt install and a toolchain.
///
/// Step headers are compared rather than the position of the string `cargo
/// publish`, because the guard's own comment says those words twice and a
/// `find(guard) < find("cargo publish")` therefore reads the file's order
/// backwards. The last assertion keeps the header anchor honest.
#[test]
fn the_publish_job_runs_the_guard_before_it_installs_or_publishes_anything() {
    let guard_step = step(RELEASE_DEPLOY_YML, PUBLISH_GUARD_STEP);
    assert!(
        guard_step.contains(SCRIPT),
        "the publish guard step no longer names {SCRIPT}, so nothing checks the \
         version being published against the tag's CHANGELOG.md:\n{guard_step}"
    );
    assert!(
        repo_root().join(SCRIPT).exists(),
        "{SCRIPT} is gone, and release-deploy.yml calls it"
    );

    let header = |name: &str| {
        let needle = format!("      - name: {name}\n");
        RELEASE_DEPLOY_YML.find(&needle).unwrap_or_else(|| {
            panic!(
                "release-deploy.yml has no step named `{name}`. If it was renamed, \
                 rename it here too — otherwise this test passes by finding nothing"
            )
        })
    };

    let guard = header(PUBLISH_GUARD_STEP);
    for later in [
        "Install system dependencies",
        "Install Rust toolchain",
        "Run cargo check",
        "Dry run publish",
        "Publish to crates.io",
    ] {
        assert!(
            guard < header(later),
            "`{later}` runs before the changelog guard, so the guard fires after the \
             time or the damage it exists to save"
        );
    }

    assert!(
        step(RELEASE_DEPLOY_YML, "Publish to crates.io").contains("cargo publish"),
        "the step this guard is ordered against no longer runs `cargo publish`, so \
         the ordering above is against the wrong anchor"
    );
}

/// The repository rule, at this step: the version arrives bound under `env:`
/// and the body contains no `${{ }}`. `release_input_validation.rs` enforces
/// that over the whole file; this says it about the step specifically, since it
/// is also what makes the body executable bash for the tests below.
#[test]
fn the_publish_guard_step_takes_the_version_as_data() {
    let guard_step = step(RELEASE_DEPLOY_YML, PUBLISH_GUARD_STEP);

    assert!(
        guard_step.contains("VERSION: ${{ needs.prepare.outputs.version }}"),
        "the guard step no longer binds the prepared version under `env:`, so it is \
         judging something else:\n{guard_step}"
    );

    let body = run_body(guard_step);
    assert!(
        !body.contains("${{"),
        "the guard step interpolates a workflow expression into its `run:` body. Bind \
         it under `env:` and read it as \"$NAME\" — see the rule at the top of the \
         file:\n{body}"
    );
    assert!(
        body.contains("\"$VERSION\""),
        "the guard step's body no longer reads the bound version, so it is checking \
         something other than the version being published:\n{body}"
    );
    assert!(
        !guard_step.contains("if:"),
        "the guard step grew an `if:`, so some route to `cargo publish` no longer runs \
         it:\n{guard_step}"
    );

    // Vacuity: the check above is a `!contains`, which a step reader returning
    // the wrong slice would pass for free.
    assert!(
        step(RELEASE_YML, "Update Cargo.toml version").contains("if:"),
        "the step reader cannot see an `if:` it is looking straight at, so the \
         assertion above proves nothing"
    );
}

/// The floor is a boundary between two eras, and both edges of it matter. Too
/// low and a version that should carry the guard publishes without one; too
/// high and the first real release goes out unchecked.
///
/// The upper bound holds for good because `Cargo.toml`'s version only moves up.
#[test]
fn the_floor_sits_above_every_existing_tag_and_at_or_below_the_prepared_version() {
    let floor = publish_guard_floor();
    let key = version_key(&floor);

    for tag in TAGS_THAT_PREDATE_THE_GUARD {
        assert!(
            version_key(tag) < key,
            "the guard floor {floor} is at or below {tag}, a tag whose tree does not \
             carry {SCRIPT}. Publishing that tag would be refused rather than warned \
             about, which breaks the recovery path the workflow_dispatch trigger \
             exists for"
        );
    }

    let prepared = cargo_toml_version();
    assert!(
        key <= version_key(&prepared),
        "the guard floor {floor} is above {prepared}, the version Cargo.toml is \
         preparing, so the next release publishes with nothing checking it. The floor \
         is a boundary between two eras, not a number to keep current"
    );
}

/// The ordinary case: the tag carries the script, so the step runs it and the
/// step's exit code is the script's.
#[test]
fn the_guard_runs_the_scripts_the_tag_carries() {
    let tree = published_tree("1.4.0", true);
    let out = run_publish_guard(tree.path(), "1.4.0");

    assert!(
        out.status.success(),
        "the guard rejected 1.4.0 against a changelog whose topmost heading is \
         1.4.0:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("::warning::"),
        "the guard warned about a tree that carries the script and agrees with it"
    );
}

/// The case the whole thing is for: a tag pushed by hand from a tree whose
/// changelog was never updated.
#[test]
fn the_guard_refuses_a_version_the_tags_changelog_does_not_name() {
    let tree = published_tree("1.4.0", true);
    let out = run_publish_guard(tree.path(), "1.5.0");

    assert_eq!(
        out.status.code(),
        Some(1),
        "the guard answered {:?} for a version the changelog does not name; 1 is \
         `mismatch`, and must stay distinct from 2, `wired up wrong`",
        out.status.code()
    );

    let message = String::from_utf8_lossy(&out.stderr);
    assert!(
        message.contains("1.5.0") && message.contains("1.4.0"),
        "the refusal names neither the version being published nor the one the \
         changelog carries, which is the whole point of printing one:\n{message}"
    );
}

/// The recovery path, tag by tag. A `workflow_dispatch` runs the workflow from
/// the ref you pick while `publish` checks out `ref: <tag>`, so new workflow
/// code meets an old tree — which is exactly what that trigger exists for. This
/// branch has to warn rather than fail, or the recovery route is closed.
#[test]
fn every_tag_that_predates_the_guard_publishes_with_a_warning() {
    let floor = publish_guard_floor();

    for tag in TAGS_THAT_PREDATE_THE_GUARD {
        // The heading is deliberately wrong for three of these five in reality;
        // it is irrelevant here, because the script is not there to read it.
        let tree = published_tree("0.1.0", false);
        let out = run_publish_guard(tree.path(), tag);

        assert!(
            out.status.success(),
            "publishing {tag} — a tag cut before {SCRIPT} existed — was refused. That \
             closes the recovery path:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );

        let log = String::from_utf8_lossy(&out.stdout);
        assert!(
            log.contains("::warning::"),
            "publishing {tag} without the guard produced no `::warning::`, so the run \
             is green and silent about having checked nothing:\n{log}"
        );
        assert!(
            log.contains(tag) && log.contains(&floor),
            "the warning for {tag} names neither the version nor the floor {floor}, so \
             a reader cannot tell what was skipped or why:\n{log}"
        );
        assert!(
            log.contains("Not checked:"),
            "the warning for {tag} does not say in plain text that nothing was \
             checked. `::warning::` is an annotation; the log line is what somebody \
             reading the job output sees:\n{log}"
        );
    }
}

/// At or above the floor, an absent script means it was removed from a tree
/// that should carry it. That is a refusal, not a warning.
///
/// The next-minor-plus-ten case is the one a string comparison gets wrong:
/// `0.18.0` sorts below `0.8.0` as text, so a `[ "$BASE" \< "$FLOOR" ]` would
/// wave it through. The pre-release case is the one that decides which era
/// `0.8.0-beta.1` belongs to.
#[test]
fn the_floor_and_everything_above_it_is_refused_when_the_script_is_absent() {
    let floor = publish_guard_floor();
    let [major, minor, _] = version_key(&floor)[..] else {
        unreachable!("version_key returns three components")
    };

    let at_or_above = [
        floor.clone(),
        format!("{floor}-beta.1"),
        format!("{major}.{}.0", minor + 1),
        format!("{major}.{}.0", minor + 10),
        format!("{}.0.0", major + 1),
    ];

    for version in at_or_above {
        let tree = published_tree(&version, false);
        let out = run_publish_guard(tree.path(), &version);

        assert_eq!(
            out.status.code(),
            Some(1),
            "publishing {version} with {SCRIPT} missing answered {:?}. It is at or \
             above the floor {floor}, so the script's absence means it was removed \
             from a tree that should carry it:\nstdout: {}\nstderr: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );

        let message = String::from_utf8_lossy(&out.stderr);
        assert!(
            message.contains(&version) && message.contains(&floor),
            "the refusal for {version} names neither it nor the floor {floor}:\n{message}"
        );
    }
}

/// End to end against the real repository: the step, the script, `CHANGELOG.md`
/// and `Cargo.toml` are checked to agree as they stand right now, so a release
/// cut from this tree would pass its own guard.
#[test]
fn the_guard_passes_against_this_repository_as_it_stands() {
    let out = run_publish_guard(&repo_root(), &cargo_toml_version());

    assert!(
        out.status.success(),
        "the publish guard, run against this repository with the version Cargo.toml \
         is preparing, refuses it. A release cut from this tree would abort:\nstdout: \
         {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("::warning::"),
        "the guard warned rather than checked against this repository, which does \
         carry {SCRIPT} — so the presence test is not seeing the file that is there"
    );
}
