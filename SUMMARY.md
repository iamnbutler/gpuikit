# Parse each line once in the editor's syntax highlighter (#135)

`SyntaxHighlighter::highlight_line_with_context` advanced the same `ParseState` over the
same line twice — once to get the ops it renders, and once again at the end of the function
"to update state" — and it was that second, doubled state it cached under
`(language, line_number)`. Every line therefore handed the next line a scope stack as if
it had occurred twice, so any multi-line construct broke the line after it. The parse now
happens exactly once: the `Ok` arm of that single `parse_line` call both yields the ops the
runs are built from and inserts the mutated `ParseState` into the cache, so the rendered
line and the state the next line starts from provably come from the same parse. An `Err`
still renders nothing and caches nothing. The parse-and-cache block moved above the theme
lookup for a borrow reason rather than a stylistic one — `Highlighter::new(theme)` holds an
immutable borrow of `inner` across the rest of the function, so inserting below it would
need a `ParseState::clone` per line. The two adjacent problems the issue mentioned (cache
keys carry no document identity; neither map is bounded) are now documented on the fields
and deliberately left alone, since fixing the first means threading a document id through
the public API.

The symptom is not quite the one the issue predicted, which matters for how this is tested:
the text after `*/` is not left comment-coloured — the corrupt scope stack flattens the
whole following line to a single plain colour — so an assertion of the form "line 3 is not
comment-coloured" passes with the bug present. The dependable oracle is a per-byte
comparison against the stateless `highlight_block`, which already parses each line once.
Five tests were added: the named JavaScript block-comment regression, a line-by-line vs.
block-pass comparison over six fixtures, a single-line guard, the unknown-language early
return, and one at the `Editor` level that goes through `highlight_line` /
`ensure_parse_states` the way the paint loop does. The fixtures were picked by measurement,
not intuition — a plain Rust `/* … */` does *not* diverge, because Rust's block comments
nest and the doubled parse merely doubles the depth, so the Rust fixture is deliberately
unbalanced (two opens, one close on a line). Each regression test was confirmed to fail with
the second parse temporarily put back, and to pass with it removed. `cargo fmt --all --check`
is clean and `cargo clippy --features editor --lib --tests` emits 52 warnings, identical to
the pre-change baseline measured on a stash of these edits.

Verification: PASSED — `cargo test --all-features --lib` (395 passed, 0 failed); also `cargo fmt --all --check` clean and `cargo test --features editor --doc` (2 passed)
