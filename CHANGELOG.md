# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking Changes

- `InputGroup` is gone, replaced by `TextField`
  (`gpuikit::elements::text_field`). The group drew an addon cell, a stripped
  input and another addon cell as three sibling boxes and spent most of its
  code disguising them as one; the field is a single bordered box that owns the
  border, background, radius, hover/focus/disabled states and padding, with
  optional adornments laid inside it. Migration:
  `input_group(&state, cx).left_addon(InputAddon::icon(icon))` becomes
  `text_field(&state, cx).prefix(Adornment::icon(icon))`, and `right_addon` /
  `InputAddon::text` become `suffix` / `Adornment::text`.
  `InputAddon::button` has no replacement on purpose — a button that is its own
  box beside a field is composition,
  `h_stack().child(text_field(…)).child(button(…))`; an action *inside* the
  field is `Adornment::element`
- `KbdSize` is gone; `Kbd` takes the shared `ControlSize` like every other
  control. `kbd("S").size(KbdSize::Small)` becomes `kbd("S").small()`, and
  `KbdSize::Default` is now `ControlSize::Medium`
- `IconButton`'s pixel API is rems. `.size(px(24.))` becomes `.box_size(…)` —
  renamed because `.size()` read as though it set the *control* size, which is
  now `.small()` / `.medium()` / `.large()` — and `.width`, `.height` and
  `.icon_size` take `impl Into<Rems>` instead of `impl Into<Pixels>`
- `DropdownMenu::build` takes a `ControlSize` as its third argument, so a
  popup's rows are the size of the trigger they dropped out of
- `Input` applies the rung's font size and line height in the same base text
  style that already forced the theme foreground, so a wrapper's `.text_lg()`
  no longer reaches an input. This is deliberate — a declared height and
  inherited text disagree, and the height is what a row is aligned on —
  but `.text_size()` on the input itself still wins, as before
- `Theme` gains a non-`Option` `controls: ControlScale` field. Themes built
  through `Theme::new` (which is all of the bundled ones) are unaffected;
  a struct-literal `Theme { … }` has to name it
- `SelectableText::new` takes two more arguments: the run's plain text, and a
  `RunRole` saying how the run is announced. A run can no longer be built
  without deciding what it is. It exists to serve the markdown renderer, so
  callers outside this crate are unlikely
- pulldown-cmark 0.12 → 0.13.4. gpuikit's own rendering is unchanged, but
  pulldown-cmark types are part of gpuikit's public surface — `MarkdownEvent`
  hands out a `pulldown_cmark::Event<'static>`, and `CodeBlockKind`,
  `LinkType`, `Options` and `Parser` are re-exported — so a downstream crate
  with its own `pulldown-cmark = "0.12"` dependency will end up with two
  non-unifying copies of `Event` until it bumps too

### Added

- `Table` (`gpuikit::elements::table`), with sorting and row selection folded
  in as opt-ins rather than shipped as a second "data table" element. A
  `Column<R>` carries a header, a width, an alignment and a per-cell render
  closure returning any element; a header stays put over a body that `max_h`
  caps and scrolls; cells wrap. The data-view state stays with the caller: the
  table is handed rows that are already filtered and already sorted plus the
  `SortDescriptor` describing how, and reports `SortRequest` / `SelectRequest`
  / `SelectAllRequest` back, so nothing moves until the caller moves it. The
  header checkbox selects all — with `Checkbox`'s indeterminate middle state —
  only for a caller that asked for it with `on_select_all`, because "all" is
  only meaningful where the caller's table has all the rows. Filtering is a
  `TextField` above the table, not a table feature. `ColumnWidth` has `Flex`
  and `Fixed` arms and no content-sized one; see its doc comment for why that
  needs a hand-written element. No accessibility roles yet — the convention
  they need has not landed, and the element's module docs record what it will
  need and two findings that decision has to cover
- `CheckState` and `checkbox_box()` (`gpuikit::elements::checkbox`): the box a
  checkbox draws, without the row, the label or the click handling, plus the
  three-state value and its `from_count` / `toggled` rules. `Checkbox` is an
  entity, so an element drawing one box per row cannot mint one per frame;
  `Checkbox::render` goes through the same box, so there is only one of them in
  the crate. `Checkbox`'s own API and rendering are unchanged
- A shared control size scale, in `gpuikit::theme::control`. `ControlSize`
  names a rung — `Small` / `Medium` / `Large`, 16 / 20 / 24px at a 16px root,
  `Medium` the default — and `Themeable::control` resolves it into a
  `ControlMetrics` carrying height, horizontal padding, gap, radius, text size,
  line box and *ink*, how much of its box a control's graphic fills. Every
  control that can share a row takes one through the new
  `traits::control_sized::ControlSized` trait, with free `.small()` /
  `.medium()` / `.large()`: `Button`, `IconButton`, `Checkbox`, `Switch`,
  `Toggle`, `Select`, `Dropdown`, `Badge`, `Kbd`, `Input`, `TextField`,
  `Textarea` and `Field`. All dimensions are rems, and a theme rescales the
  whole set at once through `Theme::controls`
- `TextField` (`gpuikit::elements::text_field`) — the single-line counterpart
  to `Textarea`, and the replacement for `InputGroup`. One bordered box, with
  optional `prefix`/`suffix` `Adornment`s (an icon, a short label, or any
  element) laid inside it. Two behaviours improve out of the shape: a click
  anywhere in the box focuses the text, and a disabled field is actually inert
  — it renders its value as static text — rather than a dimmed live input that
  still took keystrokes
- A "Control Sizes" showcase page under a new Foundations nav section: every
  control on one row, one row per rung, each row on a tinted stripe exactly the
  rung's height so a control off its rung is visible at a glance. Backed by
  cross-element tests in `src/elements/control_size_tests.rs` that draw the
  same row in a test window and measure each box
- `docs/component-triage.md` — a decision per component for all 29 entries of
  the deferred list, with 13 ready-to-file issue bodies under `docs/issues/`,
  and tests in `src/elements.rs` that fail the build if the verdict table stops
  describing the crate
- `LoadingIndicator::playing(bool)`. A paused indicator renders its first frame
  and subscribes to nothing, so it costs its window no redraws at all;
  `App::reduce_motion` has the same effect regardless of the setting. The
  showcase's Loading page has a Pause/Play button for it, which is the quickest
  way to tell the cost of the indicators apart from the cost of the page
- Showcase pages for the four elements that had none: Slider, Typography and
  Empty get their own nav entries, and Toggle — which is a pressed/unpressed
  button, distinct from Switch — joins Checkbox and Switch on the Toggle page.
  An Editor page renders a live buffer with `--features editor` and a
  placeholder saying how to get one without it
- A Coverage page listing every module in `src/elements/` against the page that
  shows it. Two tests in `src/elements.rs` cross-check that table against the
  crate: an element module with no row fails the build, and a row naming a page
  the nav cannot reach fails too. An element that should not have a page is
  spelled `("name", "none: <reason>")`
- `examples/README.md`, recording what belongs in the showcase (components)
  against what belongs in its own example binary (interactions and
  integrations), and the build commands

### Changed

- The showcase's Markdown page demonstrates what the renderer can do rather
  than only that it renders: it says which build you are running (highlighted
  code fences or not, partial-syntax closing or not), notes the accessibility
  roles every block reports, has a Selection section with a live readout and a
  "Copy selection" button, and streams a reply through `Markdown::append` — each
  section naming the standalone example that goes further. `SAMPLE_MARKDOWN`
  now carries the nested, nested-ordered and loose list shapes that broke this
  week, so a renderer regression is visible to anyone who opens the showcase
- The showcase calls `markdown::init_code_highlighting` when built with
  `--features editor`, so its ` ```rust ` fence is actually highlighted. It was
  silently inactive
- Markdown parses off the UI thread. `set_source` and `append` schedule a
  parse on the background executor and the previously parsed events keep
  rendering until it lands, so the view never blanks; deltas arriving during a
  parse coalesce into one follow-up parse rather than one each.
  `Markdown::new` still parses synchronously, so a document is never empty on
  its first frame. **This is a behaviour change**: `events()` read in the same
  turn as `set_source` now reports the previous parse. `parsed_source()` says
  which source the current events came from and `is_parsing()` whether one is
  in flight
- `set_source` with the source the document already has does nothing

### Fixed

- A focused input no longer swallows `Copy` when it has nothing selected. gpui
  clears `propagate_event` before every bubble-phase listener, so an
  empty-selection `copy` that simply returned was indistinguishable from one
  that handled the action, and ⌘C never reached anything further out on the
  focus path — a markdown selection elsewhere in the window, say. `InputState`
  now calls `cx.propagate()` in that branch; an input with a selection consumes
  the action exactly as before, and `Cut` is untouched because an empty
  selection means something there (it cuts the current line)
- A single-line `input()` is no longer zero pixels tall. It paints text and has
  no children, so an `Auto` height resolved to zero and the field was invisible
  until whatever contained it happened to set a height — which is why
  `InputGroup` hardcoded `h(px(36.))` and `examples/input/sandbox.rs` wraps one
  in a `div().h(px(40.))`. An `Auto`-height single-line input now falls back to
  its rung's height (iamnbutler/tasks#919)
- Controls that can share a row are the same height. `Button` was 16px,
  `Toggle` 20px, `Switch` and `IconButton` 24px, `InputGroup` 36px, and
  `Select`, `Dropdown`, `Badge`, `Kbd` and `Input` declared no height at all —
  whatever padding plus a line box came to. All of them now declare one from
  the shared scale
- `Switch` and `Toggle` draw the same track. They had drifted to 2.75×1.5rem
  and 2.25×1.25rem with nothing holding them together, and both thumbs
  overflowed their track by 2px: an absolute inset is relative to the padding
  box, so the 1px border was not subtracted anywhere. The shape is now derived
  once, in `ControlMetrics::track`
- `Field`'s beside-label no longer guesses at the input's height. The
  `pt(rems(0.5)) // Align with input` is gone; the label's box is exactly the
  input's box, so the two lines of text centre against each other
- The editor and the markdown code-fence highlighter are on one newline
  convention. The crate parses against `SyntaxSet::load_defaults_newlines()`,
  whose grammars anchor rules to end of line, but the editor stripped the `\n`
  before feeding a line to syntect while `highlight_block` kept it. A
  JavaScript or C `//` comment therefore never closed and painted the following
  line as comment, and a Python string left unterminated at end of line ran on
  into the next. `GapBuffer::to_lines_with_endings()` is the accessor the
  highlighting path now uses; `Editor::highlight_line` parses the line with the
  separator the buffer says follows it and trims the runs back to the painted
  bytes, so its external contract — runs summing to exactly the display line
  `shape_line` is given — is unchanged. The editor-level test from the
  double-parse fix now asserts against `highlight_block` per byte
- A `LoadingIndicator` no longer pins its window at the display refresh rate.
  It animated through `Animation::new(..).repeat()`, and a gpui
  `AnimationElement` asks for another frame for as long as its animation is
  unfinished — `.repeat()` never is. `Window::request_animation_frame` is
  `on_next_frame(|_, cx| cx.notify(current_view))`, so one spinner re-armed a
  notify of the *enclosing view* forever and everything else on that window —
  sidebar, scroll area and all — re-laid-out and repainted 60–120 times a
  second whether or not the spinner's glyph had changed. Indicators now share
  one clock that wakes at the union of the frame boundaries its subscribers
  asked for, 2–10 times a second, and notifies exactly the views showing an
  indicator; when the last one goes away it stops entirely and costs nothing
  until one is rendered again. The showcase's Loading page kept all seven
  variants, and now redraws about 39 times a second instead of 120 — a
  realistic app with one spinner goes from 120 to 8. **Behaviour change**:
  indicators share an epoch, so one mounted mid-cycle starts at the shared
  timeline's current frame rather than at frame 0 — two braille spinners on a
  page now spin in step
- The showcase's dev profile compiles dependencies with `opt-level = 2`. gpui
  is compiled once and then only linked, so this costs nothing on an
  incremental build of this crate; `[profile.dev]` itself stays at
  `opt-level = 0`, so iterating here compiles exactly as fast as before
- The showcase rebuilt its whole sidebar on every frame — 24 `format!`s, some
  seventy `SharedString`s and 48 boxed closures — to change which one row was
  highlighted. The rows are built once in `Showcase::new` from a new
  `NAV_SECTIONS` constant, and a frame clones them (`ListEntry` is `Rc`-backed)
  and stamps `selected`
- The editor's syntax highlighter parses each line once instead of twice, so a
  multi-line construct no longer corrupts the line after it. `highlight_line`
  advanced the same `ParseState` over the line a second time "to update state",
  and cached that — the state as if the line had occurred twice — for the next
  line. The line after a JavaScript block comment, a Rust raw string or a Python
  `'''` string lost its highlighting and flattened to one plain colour
- A loose markdown list — one whose items are separated by a blank line —
  renders as a list again. CommonMark wraps a loose item's content in a
  paragraph, and the renderer flushed every paragraph as body text, so every
  marker, indent and number disappeared and the list was announced to assistive
  technology as a sequence of paragraphs. A paragraph ending inside an open
  item is now flushed as that item's row. An item holding several blocks draws
  its marker once: later blocks reserve the marker's width so their text stays
  in the item's column, take no ordinal, and are announced as paragraphs rather
  than inflating the list's item count
- A markdown list nested under an item no longer swallows that item's text.
  `- x` with an indented `- y` under it rendered as a single row: the nested
  list opened while the parent's text was still buffered, so the child's first
  item picked it up and the parent emitted nothing. The parent now gets its own
  row, at its own indent, with its own marker — and, in an ordered list, its own
  number, so a nested list no longer renumbers its parent's siblings
- Markdown list items and table cells now wrap. The text beside a list marker,
  and the text in a table cell, is a flex item, and a flex item's automatic
  minimum size is one unbroken line — so a long item ran off the edge of the
  document instead of wrapping the way the same text in a paragraph does
- Markdown text runs no longer collide across documents. Runs were minting
  global ids (`md-run-1`, `md-run-2`, …) from a counter that restarted at zero
  for every document, so two markdown documents in one frame — one `Markdown`
  entity per chat message, say — produced the same ids. gpui hashes an
  element's whole id path into an accessibility node id and refuses duplicates:
  a panic in debug builds, a silently dropped node in release. Each document
  now renders its runs under its own element, so run ids are unique by
  construction
- Ten elements no longer mint an element id that is the same for every instance
  of them. `Alert`'s dismiss button, `Textarea` and the context menu popup were
  genuine collisions — two of them in one frame shared an id — and the dialog
  panel and close button, the dropdown menu, the popover panel, the toast
  container, action and dismiss button, and the slider track were unique only
  by accident of an ancestor they do not control. gpui keys element state on an
  element's whole id path and hashes that path into an accessibility node id,
  where a duplicate is a `debug_assert!` in debug builds and a silently dropped
  node in release, so each of these was one `a11y_role` away from a crash. Ids
  are now derived from the entity backing the element, or from the id its
  caller gave it

### Added

- `Markdown::append`, for content that arrives a piece at a time (an LLM reply,
  a log tail). It extends the source instead of making the caller rebuild and
  re-set the whole document, and unlike `set_source` it keeps the selection —
  selection positions are `(run, byte offset)`, so text arriving at the end
  cannot disturb a selection made earlier
- Optional `stitch` feature: closes the syntax a partially streamed document
  leaves open (`**bold` with no closer, `[label](htt`) before parsing, so
  streaming text does not flicker between literal markers and styled text.
  Off by default — [mdstitch](https://docs.rs/mdstitch) requires a newer
  compiler than this crate declares. `markdown::preprocessing_available()`
  reports which build you got, and `Markdown::set_preprocess_partial` turns it
  off per document
- `examples/markdown_streaming.rs`, a reply dripping in through `append`
- Markdown documents and their text runs are now in the accessibility tree.
  The document is a `Role::Document`; each run is a heading, paragraph, list
  item, block quote or code node, labelled with its text, and headings report
  their level
- `MarkdownElement::id`, to override the element id a document — and therefore
  all of its runs — is scoped under. The default is derived from the `Markdown`
  entity, so it is unique and stable across frames already; set it when the
  same entity is rendered more than once in one frame.
  `MarkdownElement::element_id` reads back whichever applies
- `RunRole`, and `HeadingLevel::level()`
- `element_id`, the rule for minting element ids written down once, with the
  two helpers that implement it — `element_id::for_entity(name, entity_id)` for
  an element backed by an entity and `element_id::scoped(&parent_id, part)` for
  a named part of one — and a note on what does and does not scope an id in
  gpui (an `Entity<V: Render>` child does, a `RenderOnce` struct does not,
  `deferred()` neither scopes nor unscopes). A test scans this crate's own
  source and fails on any element that mints a constant id
- `Textarea::id`, to override the element id a textarea renders under, and
  `Textarea::element_id` to read back whichever applies. The default is derived
  from the `InputState` entity; set it when one state is rendered by more than
  one textarea in a frame
- Fenced code blocks are syntax highlighted from their info string. The
  language was parsed and then thrown away; it now reaches the element and is
  highlighted by the `editor` feature's syntect-backed `SyntaxHighlighter`.
  **Opt in per app** with `markdown::init_code_highlighting(cx)` after
  `gpuikit::init` — loading syntect's syntax and theme sets costs tens of
  milliseconds and a few megabytes, which a document containing no code should
  not pay. Requires the `editor` feature; without it, without the init call,
  without an info string, or for a language syntect has no grammar for, blocks
  render as the plain monospace they did before. Highlights are cached per
  block on (text, language, theme). The syntect theme follows the block's
  background — `base16-ocean.dark` on a dark surface, `InspiredGitHub` on a
  light one — or can be pinned with
  `markdown::set_code_highlight_theme(cx, CodeHighlightTheme::Pinned(..))`;
  `markdown::code_highlight_themes(cx)` lists the names it accepts
- `markdown::normalize_language`, the fence-info-string-to-language-token rule
  (leading word only, so ` ```rust,ignore ` is `rust`; a small alias table;
  `text`/`plaintext`/`plain`/`none` mean no language)
- `SyntaxHighlighter::highlight_block`, which highlights a whole block in one
  stateless pass and returns `HighlightStyle`s, plus
  `SyntaxHighlighter::resolve_language` and `SyntaxHighlighter::current_theme`.
  Unlike `highlight_line`, `highlight_block` keeps its parse state local to the
  call, so two blocks of one language cannot contaminate each other by both
  starting at line 0

## [0.7.0] - 2026-08-15

### Breaking Changes

- Context Menu was rewritten. It is now an element you attach to a trigger you
  have already built — `context_menu(id, my_element).menu(|menu, window, cx| …)`
  — rather than an `Entity<ContextMenuState>` the view has to own and render.
  `ContextMenuState`, `ContextMenu::trigger` and `menu_separator` are gone;
  `menu_item` takes only a label, and entries are assembled with
  `menu.item(…).separator().header(…)` instead of a `Vec<MenuEntry>`

### Added

- Markdown text selection: drag to select across a whole document, double-click
  for a word, triple-click for a block. `Markdown::selected_text()` returns the
  selection for the embedding app to put on the clipboard, and
  `MarkdownStyle::selection_background` styles it. Selecting inside one document
  clears the selection in its siblings, so a page of separate documents behaves
  like one. Needs the retained `Entity<Markdown>` form — see
  `examples/markdown_selection.rs`
- `MarkdownStyle::soft_break_as_hard_break`, for source where a single newline
  is meant as a line break, as in LLM and GitHub-flavored output
- Context Menu: gpui action support (`menu_item("Rename").action(Box::new(Rename))`),
  which dispatches to whatever was focused before the menu opened and reads the
  item's keyboard shortcut from the keymap instead of hardcoding it
- Context Menu: section headers, checkmark items (`toggled`), keyboard
  navigation that skips separators and disabled items, hover and keyboard focus
  kept in sync, scroll-into-view in long menus, focus restored on dismiss, and
  edge-aware positioning
- `examples/context_menu.rs` and `examples/markdown_selection.rs`

### Fixed

- Markdown inline links and inline code no longer end the paragraph they appear
  in. A link mid-sentence used to flush the run and push the rest of the
  sentence onto its own line

## [0.6.0] - 2026-08-14

### Breaking Changes

- `InputBindings` gained `submit` and `insert_newline` fields; `InputStateEvent` gained a `Submit` variant

### Added

- Input submit events: configure `InputState::submit_on(SubmitOn::Enter)` (enter sends, shift-enter for newlines) or `SubmitOn::CmdEnter` (cmd-enter / ctrl-enter sends). The configured keystroke emits `InputStateEvent::Submit`, leaving content in place for the subscriber to read and clear. Default is `None` — existing inputs are unchanged
- New input actions `Submit` (default `cmd-enter` / `ctrl-enter`) and `InsertNewline` (default `shift-enter`, always a newline regardless of submit mode)

### Fixed

- Input content text now defaults to the theme foreground color; it previously inherited the window text style, which bottoms out at gpui's default black — invisible on dark themes. An explicit `.text_color()` on the element still wins

## [0.5.0] - 2026-08-11

Recorded retroactively. Both removals below shipped in 0.5.0 but were never
written down — they rode along in an otherwise unrelated showcase PR
([#121](https://github.com/iamnbutler/gpuikit/pull/121)), so anyone upgrading
from 0.4.x met an `unresolved import` with no explanation. See
[#120](https://github.com/iamnbutler/gpuikit/issues/120).

### Breaking Changes

- **Removed the Skeleton component** (`gpuikit::elements::skeleton`), pending a
  rewrite that does not lag. Its pulse used `Animation::new(1500ms).repeat()`,
  and a gpui `AnimationElement` requests another frame for as long as its
  animation is live — which for a repeating animation is forever. One skeleton
  therefore pinned its whole window at the display refresh rate, re-laying-out
  and repainting every other element on it. `Skeleton::animated(false)` was not
  an escape hatch: the animation was attached unconditionally and the callback
  simply returned the element unchanged. For a static placeholder in the
  meantime, a plain `div().bg(cx.theme().surface_secondary())` sized to the
  content is the direct replacement
- **Removed the Grain component** (`gpuikit::elements::grain`). It paints one
  quad per 4px cell inside a `canvas` — on the order of 60k quads for a
  1200×800 overlay — which is affordable only on a window that never repaints.
  It comes back as a shader or a tiled texture, not as quads

## [0.4.0] - 2026-04-05

### Breaking Changes

- Switched from gpui git dependency to [gpui-unofficial](https://github.com/iamnbutler/gpui-unofficial) on crates.io
- gpuikit is now published on crates.io

### Changed

- GPUI dependencies now come from crates.io (`gpui-unofficial` v0.230.2) instead of the Zed git repo
- Updated install instructions — use `gpuikit = "0.4"` instead of a git dependency

## [0.2.0] - 2026-04-01

Initial public release with 40+ components.

### Components

**Layout & Structure**
- Accordion, AspectRatio, Card, Collapsible, List, ScrollArea, Separator, Tabs

**Forms & Inputs**
- Button, ButtonGroup, Checkbox, Dropdown, Field, Input, InputGroup, Label, RadioGroup, Select, Slider, Switch, Textarea, Toggle, ToggleGroup

**Feedback & Status**
- Alert, Badge, Loading Indicator, Progress, Skeleton, Toast, Tooltip

**Overlays**
- Context Menu, Dialog, Popover

**Data Display**
- Avatar, Breadcrumb, Empty, Kbd, Typography

**Effects**
- Grain (noise texture overlay)

### Theme System

- `Themeable` trait for consistent styling across components
- `ActiveTheme` extension trait for easy theme access
- Semantic color methods: `fg()`, `bg()`, `surface()`, `border()`, `accent()`, `overlay()`, etc.
- Component-specific theme methods for buttons, inputs, and more

### Features

- `editor` - Syntax-highlighted code editor component
- `schema` - JSON schema generation via schemars
