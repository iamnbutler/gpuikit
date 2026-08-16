# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking Changes

- `SelectableText::new` takes two more arguments: the run's plain text, and a
  `RunRole` saying how the run is announced. A run can no longer be built
  without deciding what it is. It exists to serve the markdown renderer, so
  callers outside this crate are unlikely

### Fixed

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

### Added

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
