# Streaming markdown: `append`, coalesced background parse, partial-syntax preprocessing

`Markdown` gains `append(&str, cx)` and now parses off the UI thread. A delta extends the source
and schedules a full re-parse on the background executor; the previously parsed events keep
rendering until the new parse lands, so the document never blanks mid-stream. Requests arriving
while a parse is running coalesce — `ParseState::{Idle, Parsing { dirty }}` marks the document
dirty instead of spawning a second parse, and the running loop re-runs once against the newest
source, so ten deltas during one parse cost one extra parse rather than ten. The parse loop lives
inside a single `cx.spawn` whose handle is owned by the entity (a task re-spawning itself would
drop its own handle, and owning it here means dropping the document cancels a parse in flight).
`set_source` moves to the same path and is now a no-op when the source is unchanged;
`Markdown::new` still parses synchronously, so a document is never empty on its first frame.
`append` keeps the selection where `set_source` still drops it — selection positions are
`(run, byte offset within the run)`, so text arriving at the end of a document cannot disturb a
selection made earlier in it. New accessors: `parsed_source()`, `is_parsing()`,
`preprocess_partial()`, `set_preprocess_partial()`.

Separately, and behind the new non-default `stitch` feature, each parse first runs the source
through [mdstitch](https://docs.rs/mdstitch), which closes the syntax a partial document leaves
open (`**bold` with no closer, `[label](htt`) — that is the half that actually stops the flicker;
`append` alone only makes the parse cheaper. `LinkMode::TextOnly` is used deliberately: mdstitch's
default rewrites an incomplete link to a placeholder URL, which this renderer would draw as a live
clickable link. The feature is off by default because mdstitch declares `rust-version = 1.95`,
above this crate's declared floor — so a downstream app gets the threading fix by default and opts
into the flicker fix with `features = ["stitch"]`; whether to make it default (and raise the
declared `rust-version`) is left as a maintainer's call. One behaviour change worth flagging for
downstream apps: `set_source` is no longer synchronous, so `events()` read in the same turn now
reports the previous parse. Nothing in this repo did that. Also included: 14 tests in
`src/markdown/mod.rs` (sync first parse, append extends, empty append inert, old parse keeps
rendering mid-parse, five deltas coalescing into exactly two parses, a 200-delta stream landing on
the final source, identical `set_source` not re-parsing, selection kept on append and dropped on
`set_source`, dropping the document mid-parse, plus four `stitch`-gated ones), a
`examples/markdown_streaming.rs` demo that drips a reply in 3 characters at a time, and
CHANGELOG/README notes.

Verification: PASSED — `cargo test --all-targets` (218 tests), `cargo test --features stitch --all-targets` (222 tests), `cargo test --doc --features stitch` (2 passed), `cargo check --all-features`, `cargo clippy --all-targets --features stitch` (no warnings in the touched files), `cargo fmt --all --check`. The example compiles and links, but this machine has no display, so the visual claim (no flicker, no blanking) rests on the tests and not on having watched it run.
