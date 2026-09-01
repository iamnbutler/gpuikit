# Examples

Two kinds of example live here, and the difference decides where new work goes.

**`showcase.rs` covers components.** One binary, one nav, one page per element:
what it looks like, what its variants are, what it does when you click it. If
you want to know whether `gpuikit` has a slider and what it looks like, this is
the only place you should have to look.

**Every other example covers an interaction or an integration** — something
that needs its own window, its own keymap, its own running loop, or a whole
small app:

| Example | What it is for |
| --- | --- |
| `markdown_selection.rs` | Dragging a selection across blocks, and a `cmd-c` binding of its own |
| `markdown_streaming.rs` | A reply arriving at frame rate through `Markdown::append` |
| `input/sandbox.rs` | The text input's editing, IME and key handling |
| `dialog_example.rs` | A dialog driving a window |
| `popover_demo.rs` | Popover placement against window edges |
| `context_menu.rs` | Right-click menus |
| `tasks.rs` | A small app built out of the kit |

## The convention

- **A new component lands its showcase page in the same PR.** Two tests in
  `src/elements.rs` enforce this: every `pub mod` in `src/elements.rs` needs a
  row in the showcase's `ELEMENT_COVERAGE` table, and every page named in that
  table has to be an arm of the showcase's render match. A component that
  genuinely should not have a page is spelled `("name", "none: <reason>")` —
  the reason is then recorded where a reviewer is already looking. The
  Coverage page in the showcase renders the same table, so the list is in
  front of anyone who opens it, not only in a test.
- **A new capability on an existing component is at minimum *named* on that
  component's page in the same PR**, pointing at the example that goes
  further, by path. Markdown selection worked in the showcase for months and
  nobody could tell, because nothing said so.
- **Prefer a page over a new example binary.** Each binary is another full
  link of gpui — see the build note below. Reach for a binary only when the
  thing needs a window, a keymap or a loop the showcase cannot give it.
- **Sample content is a regression surface, not just a demo.**
  `SAMPLE_MARKDOWN` carries the shapes that have broken recently — nested
  lists, nested ordered lists, loose lists — so a renderer regression is
  visible to anyone who opens the showcase.
- **Pages exist in the default build.** A page for a feature-gated component
  renders a placeholder saying how to get the real thing (see the Editor page)
  rather than disappearing. Requiring `--features editor` for the showcase
  would make every other page pay for syntect; dropping the page is what let
  the editor go undemonstrated in the first place.
- **If `showcase.rs` grows much further, split it into `examples/showcase/` —
  one binary, many files** — rather than adding a second binary.
- **A new example binary is declared in `Cargo.toml` with
  `required-features = ["examples"]`.** Dropping a file into `examples/` is not
  enough and is worse than nothing: cargo autodiscovers any top-level
  `examples/*.rs` or `examples/*/main.rs` as a target, an autodiscovered target
  cannot carry `required-features`, and so that one file gets linked by every
  `cargo test` and `--all-targets` build in the repository. A test in
  `src/build_profile_guard.rs` reads this directory and fails on exactly that.

## Building

Every example is behind the `examples` feature, which enables nothing else:

```sh
cargo run --example showcase --features examples                     # every page, plain code fences
cargo run --example showcase --features examples,editor              # + highlighted fences, live editor buffer
cargo run --example markdown_streaming --features examples,stitch
```

Checks, in the order they are quickest to run:

```sh
cargo fmt --check
cargo test --lib                                          # coverage guards, and the build-config guards
cargo check --all-targets --features examples             # type-checks the examples without linking them
cargo check --all-targets --features examples,editor      # about 3 minutes cold on 4 cores
```

`cargo check` fingerprints on file content rather than mtime, so `touch`ing a
file does not force a re-check — a 0.2s "Finished" means nothing was
recompiled.

`cargo test --lib` above is also what `.tasks/verify` runs: it is this
repository's declared verification suite, so an automated build and a human
checking their work run the same command. Keep the two in step — and read that
file's header before widening it, because `--all-features` and `--all-targets`
each undo the gate described below. `scripts/run-tests.sh` wraps the same
command when you want the summary lines judged rather than the exit status.

## Why the gate, and what a killed link looks like

Each example is a full link of gpui. `cargo build --all-targets` and a bare
`cargo test` link all eight, and cargo sizes `-j` from the CPU count with no
knowledge of how much memory the machine has, so several `ld` processes run at
once and the kernel kills one:

```
= note: collect2: fatal error: ld terminated with signal 9 [Killed]
```

That message names no crate, no file and no symbol, so it reads as a compile
error that does not exist. Three separate runs in one week were spent hunting a
type error that was never there. `required-features = ["examples"]` is what
keeps those eight links out of a build that did not ask for a demo;
`[profile.dev] debug = "line-tables-only"` and, on Linux,
`-Csplit-debuginfo=unpacked` from `.cargo/config.toml` are what shrink the links
you do ask for. Backtraces keep file and line; a debugging session that wants
the full detail back asks for it on that build alone with
`RUSTFLAGS="-Cdebuginfo=2"` (which, being an environment variable, also replaces
the config file's flags rather than adding to them).

**`--all-features` re-enables the gate.** Cargo offers no way to hold a feature
back from that flag, so `cargo test --all-features` still links all eight and
rests entirely on the debug-info reductions. `cargo test --lib` is the command
to reach for in a constrained environment.

`src/build_profile_guard.rs` holds all of this in place, under `cargo test
--lib`.

## A piped build reports the pipe's exit status

This is not a build setting and no test can hold it: a shell pipeline reports
the exit status of its *last* command, so an OOM-killed link inside

```sh
cargo build --all-targets 2>&1 | tail -50
```

exits 0 and reads as success. The failure then surfaces later, somewhere
unrelated, as a missing binary. Either turn it on:

```sh
set -o pipefail
cargo build --all-targets 2>&1 | tail -50
```

or read the real status back afterwards:

```sh
cargo build --all-targets 2>&1 | tail -50; exit ${PIPESTATUS[0]}
```
