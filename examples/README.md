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

## Building

```sh
cargo run --example showcase                     # every page, plain code fences
cargo run --example showcase --features editor   # + highlighted fences, live editor buffer
cargo run --example markdown_streaming --features stitch
```

Checks, in the order they are quickest to run — `scripts/check.sh` runs all of
them, and `scripts/check.sh --link` additionally links every example:

```sh
cargo fmt --check
cargo test --lib                        # includes the coverage and build guards
cargo check --all-targets
cargo check --all-targets --features editor   # about 3 minutes cold on 4 cores
```

### Linking an example is a whole-program link of gpui

Every example binary links the entire gpui stack, and that link costs the same
no matter which example it is, because the bytes are gpui's and not the
example's. Peak `ld` RSS, measured on this repository:

| Example | Source | Peak `ld` before | Peak `ld` now |
| --- | --- | --- | --- |
| `popover_demo.rs` | 5 KB | 2687 MB | **818 MB** |
| `showcase.rs` | 152 KB | 2730 MB | **960 MB** |

A 30x spread in source size, a few percent spread in link memory. Across all
eight examples under `--features editor` the peak ranges only from 892 MB to
991 MB. There is no "heavy example" to fix; picking examples to trim is picking
at random.

What made the difference is `[profile.dev.package."*"] debug = 0` in
`Cargo.toml`: debug info for dependencies was nearly all of what the linker
held, and nothing in this repository is ever debugged inside gpui. Backtraces
through gpui keep their function names; they lose file and line, which at the
`opt-level = 2` those dependencies already build at were approximate anyway.
gpuikit's own code keeps `debug = 2`: cutting that too was measured, and it
bought a further 158 MB on `showcase` (960 -> 802 MB) in exchange for the
locals in the frames a gpuikit contributor is actually standing in. Not worth
it once the dependency tier is gone, which is why `dev-debuginfo` should be
rare rather than routine.

**When you need the full thing back, do not edit `[profile.dev]`** — forgetting
to put it back is the bug. Use the profile that already exists:

```sh
cargo run --profile dev-debuginfo --example showcase
```

**These settings apply when gpuikit is the crate being built, and not when it
is a dependency.** Cargo takes profiles from the workspace root, so an
application depending on gpuikit gets its own profile, and the same link is the
same size there. The equivalent in *that* manifest is:

```toml
[profile.dev.package."*"]
debug = 0
```

### `ld terminated with signal 9 [Killed]`

That is the kernel's OOM killer, not a compile error. It names no crate and no
symbol, which is exactly why it reads as one. cargo runs one linker per job and
picks `-j` from the CPU count with no knowledge of memory, so N cores want
roughly N x 1 GB free at the link step — and `cargo build --all-targets`
on a 4-core / 5.9 GiB / no-swap box used to have three linkers killed and exit
101. Nothing in your code is wrong. The way past it is fewer jobs:

```sh
CARGO_BUILD_JOBS=1 cargo build --all-targets
CARGO_BUILD_JOBS=1 scripts/check.sh
```

Nothing in the repository caps `-j`, because a `.cargo/config.toml` `[build]
jobs` would serialise every build for every contributor to protect a case that
is now well inside budget.

### A piped build can report success when it failed

```sh
cargo build --all-targets | tee build.log   # exits 0 even when the build died
```

A pipeline's status is its *last* command's, so `tee` reports for cargo. This
is how a killed link was once read as a green run. `scripts/check.sh` sets
`set -euo pipefail` and takes each cargo status from `${PIPESTATUS[0]}`; if you
pipe a build by hand, do the same.

### Two costs that look smaller than they are

- **`--all-targets` compiles gpui a second time.** It pulls the
  dev-dependencies, and gpui is one of them with `test-support` on — a
  different feature set, so the whole gpui stack is rebuilt. That is most of
  the difference between `cargo check --all-targets` and `cargo check`, and
  it is worth knowing before blaming the link for the wall clock.
- **An unqualified `cargo test` links every example.** `cargo test --lib` plus
  `cargo check --all-targets` covers the same ground without the links, which
  is why the guard tests live in the lib rather than in `tests/`. `cargo check`
  fingerprints on file content rather than mtime, so `touch`ing a file does not
  force a re-check — a 0.2s "Finished" means nothing was recompiled.

### Adding an example

`[package] autoexamples = false`, so the `[[example]]` blocks in `Cargo.toml`
are the whole list: a new file under `examples/` is built by nothing until it
has a block. That is deliberate — each binary is another full link — and
`src/build_memory_guard.rs` fails the test suite if a discoverable example
(`examples/<name>.rs` or `examples/<name>/main.rs`) has no block, so the
silence never lasts past `cargo test --lib`.
