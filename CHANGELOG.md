# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking Changes

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

### Changed

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

- The editor's syntax highlighter parses each line once instead of twice, so a
  multi-line construct no longer corrupts the line after it. `highlight_line`
  advanced the same `ParseState` over the line a second time "to update state",
  and cached that — the state as if the line had occurred twice — for the next
  line. The line after a JavaScript block comment, a Rust raw string or a Python
  `'''` string lost its highlighting and flattened to one plain colour
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
