# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking Changes

- Context Menu was rewritten. It is now an element you attach to a trigger you
  have already built — `context_menu(id, my_element).menu(|menu, window, cx| …)`
  — rather than an `Entity<ContextMenuState>` the view has to own and render.
  `ContextMenuState`, `ContextMenu::trigger` and `menu_separator` are gone;
  `menu_item` takes only a label, and entries are assembled with
  `menu.item(…).separator().header(…)` instead of a `Vec<MenuEntry>`

### Added

- Context Menu: gpui action support (`menu_item("Rename").action(Box::new(Rename))`),
  which dispatches to whatever was focused before the menu opened and reads the
  item's keyboard shortcut from the keymap instead of hardcoding it
- Context Menu: section headers, checkmark items (`toggled`), keyboard
  navigation that skips separators and disabled items, hover and keyboard focus
  kept in sync, scroll-into-view in long menus, focus restored on dismiss, and
  edge-aware positioning
- `examples/context_menu.rs`, demonstrating both closure- and action-driven items

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
