# Four issues: syntect newlines, a shared control scale, TextField, and the #59 re-triage

**One newline convention for syntect (#142).** The crate parses against
`SyntaxSet::load_defaults_newlines()`, whose grammars anchor rules to end of
line, but the editor stripped the `\n` before feeding a line to syntect while
`highlight_block` kept it. That was not theoretical: a JavaScript or C `//`
comment never closed and painted the following line as comment, and a Python
string left unterminated at end of line ran on into the next. `GapBuffer` gains
`to_lines_with_endings()` — same line count as `to_lines`, and concatenating it
reproduces the buffer byte for byte — and the four `ensure_parse_states` call
sites use it. `Editor::highlight_line` keeps its external contract (runs summing
to exactly the display line `shape_line` is given) by parsing the line with the
newline the buffer says follows it and trimming the runs back. The editor-level
test from the double-parse fix is strengthened to a per-byte comparison against
`highlight_block`, and two new tests cover the end-of-line cases; both were
confirmed to fail with the old behaviour temporarily restored. Note that a Rust
`//` fixture does *not* diverge in syntect 5.3's default newline grammars, which
is why the existing fixture never caught this.

**A shared control scale (#141), and TextField in place of InputGroup (#144).**
`ControlSize` (`Small` / `Medium` / `Large` — 16 / 20 / 24px at a 16px root,
`Medium` the default) is resolved by the theme into a `ControlMetrics` carrying
height, horizontal padding, gap, radius, text size, line box and *ink*, and
every control that can share a row takes one through a new `ControlSized` trait.
Controls whose height was previously emergent — `Select`, `Dropdown`, `Badge`,
`Kbd`, `Input` — now declare one, which also fixes the zero-height single-line
input (iamnbutler/tasks#919). `Switch` and `Toggle` draw one derived track
instead of two drifted ones, and its thumb now fits inside the track's border
rather than overflowing it by 2px. `InputGroup` is deleted and replaced by
`TextField`: one bordered box that owns the border, background, radius,
hover/focus/disabled states and padding, with optional `prefix`/`suffix`
adornments laid inside it, rather than three sibling boxes and the code to
disguise them as one. A click anywhere in the box focuses the text, and a
disabled field is actually inert — it renders its value as static text — rather
than a dimmed live input that still took keystrokes. `TextField` takes a rung
like everything else, so #144's `FIELD_HEIGHT` constant never lands. Breaking
changes (`KbdSize` removed, `IconButton`'s pixel API now rems with `.size()`
renamed to `.box_size()`, `DropdownMenu::build` taking a `ControlSize`, `Input`
no longer inheriting a wrapper's text size, `Theme` gaining a `controls` field)
are spelled out with migrations in the CHANGELOG. New cross-element tests draw
one of every control on an `items_start` row in a test window and measure each
box; reverting `Badge` to a 16px height fails them with a measured-row dump. A
"Control Sizes" showcase page under a new Foundations section does the same
thing visually, one row per rung on a tinted stripe exactly the rung's height.

**Re-triage of #59's deferred components (#146).** The deliverable here is a
decision, not an implementation. `docs/component-triage.md` gives all 29 entries
of #59's list exactly one of three verdicts — 8 Shipped, 11 Rejected with a
reason and a named revisit trigger, 10 with an issue of their own — and
`docs/issues/` holds 13 ready-to-file issue bodies: the 10 surviving components
plus three prerequisites the triage surfaced by checking #59 against the source
rather than against itself. One row of #146's own blocker table does not hold:
`src/traits/portal.rs` is 486 lines of overlay positioning math with zero
callers, zero implementors and zero tests, while six elements each hand-roll
`anchored()`/`deferred()`; and no element in `src/elements/` reports an
accessibility role, so ten issues would each have invented a different
mechanism. Both are now prerequisite issues. Five tests in `src/elements.rs`
parse the verdict table and fail the build when it stops describing the crate.
`todo.md`'s two deferred lists are replaced by a pointer.

Two caveats worth a human's eye. First, the rungs are landed and measured but
nobody has *looked* at them — this environment is headless, so the showcase
compiles and links but cannot be run. The knobs are all in
`ControlScale::default()` and `ControlMetrics::track()`; `ink`, the track aspect
ratio and the Small rung's 11px text are the likely first adjustments, and
`Kbd` lost its heavier bottom border because a 14px line box plus 3px of border
does not fit a 16px box. Second, the triage's external citations (Headless UI,
Primer, shadcn, Zed) are by project and component name rather than by file and
line, and the document says so: anyone implementing one of those issues should
re-open the source rather than trust a name in a table. The claims about *this*
repository — the unused portal, the absent roles, the six hand-rolled overlays,
`select.rs` importing from `dropdown.rs` — were each verified by grep.

Verification: PASSED — `cargo fmt --all --check`, `cargo test --lib --all-features` (449 passed, 0 failed), `cargo test --doc --all-features` (2 passed), `cargo check --all-targets --all-features`, `cargo clippy --lib --tests --all-features` (45/48 warnings against a stashed-clean baseline of 46/49, so no new ones), and `cargo build --example showcase -j 1` links. `cargo test --all-features` is not usable here: `ld` gets OOM-killed linking the example binaries.
