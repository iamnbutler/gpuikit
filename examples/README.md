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

Checks, in the order they are quickest to run:

```sh
cargo fmt --check
cargo test --lib                        # includes the two coverage guards
cargo check --all-targets
cargo check --all-targets --features editor   # about 3 minutes cold on 4 cores
```

Note that an unqualified `cargo test` also *links* every example, and linking
the showcase has been OOM-killed in a constrained environment. `cargo test
--lib` plus `cargo check --all-targets` covers the same ground without the
link. `cargo check` fingerprints on file content rather than mtime, so
`touch`ing a file does not force a re-check — a 0.2s "Finished" means nothing
was recompiled.
