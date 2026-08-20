# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking Changes

- **`gpuikit::elements::dropdown` is gone in full.** `Dropdown`,
  `DropdownState`, `DropdownChanged`, `DropdownMenu`, `DropdownOption` and
  `dropdown()` are deleted, and `src/elements/dropdown.rs` with them.
  `Dropdown` and `Select` were one component under two names — the same
  bordered trigger with a chevron, the same popup one gap below it, the same
  `ControlSize`, and `select.rs` importing `DropdownMenu`, `DropdownOption` and
  `MENU_GAP` *from* `dropdown.rs` to get there. The only behavioural difference
  was that a `Dropdown`'s selection could not be absent, which is a constructor
  argument rather than a component. `Select` takes the union of the two APIs:

  | Was | Is |
  | --- | --- |
  | `dropdown(id, options, value)` | `select(id, options).selected(value)` |
  | `DropdownState::new(…)` | `SelectState::new(…)` |
  | `DropdownChanged` | `SelectChanged` |
  | `state.selected` (a `T`) | `state.selected` (an `Option<T>`) |
  | `state.set_selected(value, cx)` | `state.set_selected(Some(value), cx)` |
  | `DropdownMenu`, `DropdownOption` | no replacement — the popup is private |

  `on_change`, `full_width`, `disabled`, `control_size`, `is_open`,
  `is_disabled` and `set_disabled` carry over unchanged, and `Select` adds
  `placeholder` and `clear()`, which a `Dropdown` could not express. 0.8.0's
  note that `DropdownMenu::build` gained a `ControlSize` argument is now moot:
  the type it describes no longer exists
- The popup is **private**. `DropdownMenu` becomes `Listbox`, a private type in
  `src/elements/select.rs`, and `DropdownOption` — a newtype over
  `SharedString` — is deleted rather than renamed; the popup takes plain
  labels. This is the part of the decision that enforces itself: a public popup
  type is what let one element get built on another's internals in the first
  place, so the next chooser in this neighbourhood has to either grow
  `select.rs` or write its own, and both are visible in review.
  `MENU_GAP` becomes a private `LISTBOX_GAP` for the same reason
- Element ids and debug selectors move with the names: `dropdown-menu` /
  `dropdown-option` become `select-listbox` / `select-option`, and the test-only
  selectors become `gpuikit-select-trigger` / `gpuikit-select-popup`. These are
  internal, but a consumer that had pinned a test to one would notice
- **`select()` and `Select::new()` take the accessible name as their second
  argument**: `select(id, name, options)`. `Role::ComboBox` is in
  `a11y::role_requires_a_name`, so an honest role for the trigger forces a
  name, and every naming source that convention allows was unavailable here — a
  select's visible text is its *value* (naming the control after it would
  rename the control every time the user changed it), the placeholder
  disappears the moment a choice is made and defaults to "Select…", and gpui has
  no `labelled_by` builder, so a `Field` or `Label` beside the control cannot
  name it. A required constructor argument is what `src/a11y.rs`'s section 2
  prescribes for this case; it was written for `IconButton` and `Select` got
  there first. `select("country", vec![…])` becomes
  `select("country", "Country", vec![…])`
- **`gpuikit::traits::visual_focus` is deleted**, with the `VisualFocus` trait
  and the `FocusStyle` enum in it. It had no implementors anywhere in this
  crate and could not usefully gain one: it requires `gpui::Focusable`, which a
  `RenderOnce` control cannot implement. `theme::focus_ring` is now this
  crate's answer to the same question, and carrying both would be a second fork
  of one decision

### Added

- **The select popup answers the keyboard.** Up and Down move a highlight
  through the options and wrap at each end, Home and End jump to the ends, Enter
  or Space chooses the highlighted option and closes, Escape closes without
  choosing, Tab and Shift-Tab close and move focus on, and typing a printable
  character jumps to the next option whose label starts with it — press the same
  letter again to walk the options that share it. Every close the keyboard asked
  for hands the trigger back its focus. 0.8.0 shipped a select that announced
  `ComboBox` / `ListBox` / `ListBoxOption` to a screen reader and then moved
  keyboard focus into a popup with nothing to do in it; this is the other half
  of that pair
- The **highlight** is a new state, distinct from the **choice**. The choice is
  the control's value and persists; the highlight is where the keyboard has got
  to and dies with the popup. The popup opens with the highlight on the chosen
  row — or on the first row when nothing is chosen — and hovering a row moves the
  same highlight rather than drawing a second one
- `elements::select::bind_select_keys(cx)`, `elements::select::LISTBOX_CONTEXT`
  and the `select::{HighlightNext, HighlightPrevious, HighlightFirst,
  HighlightLast, ChooseHighlighted, DismissListbox}` actions. `gpuikit::init`
  calls `bind_select_keys`, so an app that calls `init` needs nothing; an app
  that assembles its own keymap has to include it, which is why it is public.
  The keys are actions rather than a raw `on_key_down` because gpui dispatches
  bound actions first — a raw Escape handler would lose the key to `Dialog`'s
  own binding whenever a select sat inside a dialog. Tab is the exception and is
  deliberately *not* a binding: it is an action listener on the popup, because
  `a11y`'s context-less `tab` binding outranks every scoped one
- `A11y::active_descendant(bool)`, `A11y::is_active_descendant()` and the
  `aria-activedescendant` apply in `Announce::announce` — the highlighted row
  claims it, which is how a screen reader is told which row the keyboard is on
  while focus sits on the popup. It is the odd field on `A11y`: a plain `bool`
  rather than an `Option<bool>` (gpui's builder takes no argument, so `Some(false)`
  would be a state the crate could hold and never report), set on the descendant
  rather than the container, and the one field no test in this crate can read
  back off a node. All three, and exactly what its guard does and does not
  catch, are documented at the field
- `docs/menus-and-listboxes.md` — the decision record for the above, plus the
  convention that keeps it: the sentence that separates the two families ("a
  listbox offers *values* to choose between, and the choice persists; a menu
  offers *actions* to invoke, and nothing stays selected"), a family table, why
  `Select` is the name that survived, why the popup is private, what the two
  families share (placement, via `docs/overlays.md`, and nothing else), the
  migration table, and what would reopen it. It also reserves the freed name:
  a future menu-of-actions-from-a-trigger is `DropdownMenu`, built on
  `context_menu.rs`'s items rather than on `Select`
- **Keyboard focus is part of the role announcement.** `A11y` gains
  `focusable()`, `focus_handle(handle)` and `not_focusable(why)`, and
  `Announce::announce` *applies* the answer — `focusable().tab_stop(true)`, or
  `track_focus(&handle.tab_stop(true))` for a caller's handle — in the same call
  that reports the role, so the two cannot drift apart. `Button` announced
  `Role::Button` and could not take keyboard focus, which promised a screen
  reader a control a keyboard could not reach; that is what this closes.
  For every role in the new `a11y::role_requires_keyboard_focus`, saying
  nothing is a `debug_assert!` — the counterpart of the missing-name one.
  `not_focusable` takes a reason on purpose: it is what distinguishes a
  decision from a call someone made to silence the assertion
- `a11y::role_requires_keyboard_focus` — the standalone control roles
  (`Button`, `DefaultButton`, `CheckBox`, `Switch`, `RadioButton`, `Link`,
  `Slider`, `SpinButton`, the combo boxes and the text inputs). The
  composite-item roles (`MenuItem*`, `Tab`, `TreeItem`, `ListBoxOption`) are
  deliberately excluded — they are arrow-key targets inside a composite that
  owns the one tab stop, so no per-item rule can be right — as are the
  landmarks and containers, which are read rather than operated. Both declines
  are written into the function's docs
- `a11y::bind_focus_keys`, `a11y::FocusNext` / `FocusPrevious`, and
  `a11y::FocusNavigation::moves_focus_on_tab`. gpui ships `Window::focus_next`
  / `focus_prev` and binds neither, so Tab did nothing at all; `gpuikit::init`
  now installs the bindings, **before** `input::bind_input_keys`, which is what
  keeps Tab inside a focused text input. `announce` puts the listener on every
  control it makes focusable; an app puts it on the element it focuses at
  startup so that the *first* Tab works, which `examples/showcase.rs` now
  demonstrates
- `theme::focus_ring(color)` and `theme::FOCUS_RING_WIDTH` — one definition of
  the ring a keyboard-focused control draws. A spread `BoxShadow` rather than a
  border, so arriving focus does not resize the control and reflow its
  neighbours, applied through gpui's `focus_visible` so that clicking a control
  does not leave a ring behind it. `Button`, `SidebarTrigger` and `Select`'s
  trigger all draw it
- `Select` declares its roles, the first element adopted after the convention:
  the trigger announces `Role::ComboBox` with its name, `expanded`, and — as
  its *value* — the label of the chosen option; the popup announces
  `Role::ListBox` named after the control it dropped out of; and each row
  announces `Role::ListBoxOption` with `selected`, its position and the size of
  the set. The trigger is a tab stop, and Enter or Space opens the popup. The
  popup still has **no keyboard model** — no arrow keys, no Escape, no roving
  focus between options — and reports no `aria-activedescendant`, because that
  property names the row keyboard focus is virtually on and there is no such
  row yet. Both arrive together, as a follow-up
- `A11y::size_of_set`, which gpui's `aria_size_of_set` had not been modelled
  for. It is what turns `position_in_set`'s "3" into "3 of 8"
- `a11y::ELEMENTS_WITHOUT_A_ROLE` and
  `a11y::tests::every_element_module_declares_a_role` — the rollout order, as
  something a test counts rather than prose. Every `pub mod` in
  `src/elements.rs` must either implement `Accessible` or name itself in that
  list with a reason saying what it would announce or what has to exist first,
  checked in both directions so the list can only shrink. It starts at 36
  entries, which is the finding rather than the bloat: only `Button`,
  `Sidebar`, `Splitter` and now `Select` are adopted, and the gap was
  previously invisible. Modelled on `overlay_coverage::every_overlay_is_written_down`
- `Button::focus_handle` and `SidebarTrigger::focus_handle`, for a caller that
  has to move focus *to* one of them. Optional, and most callers want nothing:
  gpui mints a handle for a focusable element and keeps it in that element's own
  element state, so a `RenderOnce` control is the same focus target across
  frames without anything above it holding state. `focus_survives_a_redraw`
  pins it, and it is why no existing `button(…)` call site changed
- `mod family_coverage` in `src/elements.rs` holds that document to the crate,
  in the shape `triage_coverage` and `overlay_coverage` already use: every row
  of the family table names a real `pub mod` and one of the two families, **no
  module in one family names a module in the other in a Rust path** — the
  mechanical form of the mistake this change undid — and
  `src/elements/dropdown.rs` has not come back

- `gpuikit::a11y` and `gpuikit::traits::accessible`: one convention for how an
  element reports an accessibility role, which
  `docs/issues/element-roles-convention.md` asked for and ten component issues
  were waiting on. An element implements `Accessible`, returning an `A11y`
  value — a role, an accessible name, and whatever state goes with the role
  (`toggled`, `selected`, `expanded`, a text or bounded-number `value`,
  `orientation`, `level`, `position_in_set`) — and applies it to the root
  element it was already building with one method, `.announce(a11y)`. Nothing
  has to become a hand-written `Element`: `Announce` is blanket-implemented for
  gpui's `StatefulInteractiveElement`, so **"no id, no role" is enforced by the
  type system** and roles stay out of `src/element_id.rs`'s duplicate-node
  trap. The accessible name is *required* for the roles in
  `a11y::role_requires_a_name` — a nameless `Role::Button` is a
  `debug_assert!`, so `button("save", "")` now panics in debug builds — and it
  comes from the element's own visible text where it has any, a constructor
  argument where it has none, and never from the tooltip. That module's docs
  are the decision record, one section per question the issue asked
- `Button` is the worked example: it announces `Role::Button` named by its
  label, so there is no second string to keep in step with it. Its tests read
  the real `accesskit::Node` back through `a11y::test_support`, which calls the
  two `Element` methods gpui's own accessibility walk calls — the only way to
  see a node, since accessibility cannot be switched on in a test
- `a11y::tests::no_element_calls_gpuis_a11y_builders_directly`: a source scan,
  modelled on `element_id`'s constant-id scan, that fails the build if anything
  under `src/` calls gpui's `.role()` / `.aria_*()` builders outside
  `src/a11y.rs`. If `A11y` lacks a property, the intended move is to add the
  field and apply it in `Announce::announce`
- **`gpuikit::elements::splitter`** — a draggable divider between two panes,
  which is `docs/issues/resizable.md`'s answer to #59's "Resizable" and
  deliberately not a pane tree. `splitter(id, name, ratio)` plus `.start()`,
  `.end()`, `.min_start()`, `.min_end()`, `.on_resize()`, `Orientable` and
  `ControlSized`. The split ratio stays the caller's: the element takes the
  current one, emits the new one, and stores nothing about where the boundary
  sits, so a layout can still be persisted, restored or reset from outside.
  The drawn line is `Separator`'s existing 1px hairline; the *interactive*
  band around it is 6/8/12px off the rung, which is the difference between a
  divider that feels good and one nobody can hit. The band reports
  `Role::Splitter` with its position, both floors and one arrow key's worth as
  percentages, and follows the WAI-ARIA window splitter keyboard contract
  (axis arrows step, `home`/`end` go to the floors). Not built, on purpose: a
  pane tree, nested groups, persisted layout, a collapse gesture,
  `Disableable`, and enter-collapses-the-pane. Three panes is two splitters,
  nested by the caller. `Sidebar` is **not** migrated onto it — that changes
  `Sidebar`'s API and is its own decision
- `Role::Splitter` joins `a11y::role_requires_a_name`. A divider has no visible
  text to borrow a name from, so `splitter`'s name is a constructor argument,
  exactly as `icon_button`'s is
- `Role::Splitter` joins `a11y::role_requires_keyboard_focus` too, and the
  band's raw `tab_index(0)` — a mechanism nothing else in the crate used — is
  gone with it. `Splitter`'s announcement declares `.focus_handle(handle)` and
  `Announce::announce` applies it, so the role and the focus answer are decided
  in one place like every other keyboard-operable control. The band's own
  `track_focus` had to go along with the `tab_index`: `announce` tracks
  `handle.tab_stop(true)`, and a second plain `track_focus` after it would put
  the non-stop handle back and take the splitter out of the tab order. One
  behavioural difference beyond the route into the tab order: `announce` also
  applies `moves_focus_on_tab()`, so Tab *out* of a focused splitter is now
  answered by the band itself rather than by an ancestor listener
- `.github/scripts/verify-release-version.sh`, and one step in
  `release.yml` that calls it: a release now refuses to run unless the version
  it computed is the one `CHANGELOG.md`'s topmost `## [x.y.z]` heading names.
  The two ways `release.yml` computes a version stopped agreeing when #170
  landed 0.8.0 *into* `Cargo.toml` as a prepared release — `version_type:
  minor`, the shape every previous release used, now computes 0.9.0 and
  publishes it, skipping a version. This has already happened here once: the
  tags are `v0.3.0 v0.4.0 v0.5.0 v0.5.1 v0.7.0`, and there is no `v0.6.0`
  though the changelog has a heading for it. `cargo publish` cannot be undone
  and a published version cannot be reused even after `cargo yank`, so the
  previous guard — a sentence in an input's description — was not one. No new
  inputs. The step carries no `if:`, so a dry run reports a wrong version
  instead of staying silent, and it also refuses a tag that already exists.
  The supported flow is #170's: prepare the release in a pull request, then
  dispatch with `custom_version: x.y.z`
- `.github/scripts/validate-custom-version.sh`, and a `Validate custom_version`
  step in `release.yml` that runs it immediately after the checkout — before
  `cargo install cargo-edit`, so a typo is answered in seconds rather than
  after a toolchain install. `Calculate new version` used to interpolate
  `${{ inputs.custom_version }}` straight into its own `run:` body and then
  validate the result two lines below, which cannot work: `${{ }}` is
  substituted while the step's script is being *generated*, so the value was
  already part of the script by the time the check meant to judge it ran, and a
  value carrying a quote and a newline put its second line outside the `if`
  entirely. The grammar accepted is unchanged
  (`1.2.3`, or `1.2.3-beta.1`) — `verify-release-version.sh` and `CHANGELOG.md`'s
  `## [x.y.z]` headings are matched against the result, so widening it would
  desynchronise them — but the check is now bash `[[ =~ ]]` rather than
  `grep -qE`, because `grep` judges one line at a time and so passed anything
  whose first line was a version. The empty value is accepted, since that is
  what every `version_type` dispatch sends. No new inputs, and the step carries
  no `if:`
- `src/release_input_validation.rs`, nine tests holding `release.yml` to the
  rule on every `cargo test --lib`: no `${{ }}` in any `run:` body, the
  custom_version input only ever an `env:` binding, the validator ordered ahead
  of every step that uses or writes the version, and the validator's own
  answers — including that it reports a usage error (2) separately from a
  rejected version (1). The `run:`-body parser is itself tested against a
  fixture, so a parser that silently stopped matching cannot report success.
  **The module covers `release.yml` only**: `release-deploy.yml`, which runs
  `cargo publish` with `CARGO_REGISTRY_TOKEN` set for every job, has the same
  defect in five `run:` blocks and is tracked separately

### Changed

- **The chosen option in an open select now shows a check mark instead of a
  filled row.** Before, the row you had chosen was the one painted in the accent
  colour. Now every row reserves a small slot on its left, the chosen row shows
  a check in it, and the accent fill marks the row the keyboard or the pointer
  is currently on. Two states arrived where there had been one — what you chose,
  and where you are — and one filled row could not say both. This is the only
  part of the keyboard work a consumer sees whether or not they use a keyboard,
  and it follows the fixed-width check slot `context_menu.rs` already draws its
  toggled items with. If the visual is reworked later, the property that has to
  survive is that **the highlighted row and the chosen row stay distinguishable
  when they are different rows** — that state is the whole reason the keyboard
  model exists
- Every `run:` block in `release.yml` takes its outside values from `env:`
  bindings (`CURRENT`, `CUSTOM_VERSION`, `VERSION_TYPE`, `NEW_VERSION`, `TAG`,
  `BRANCH`, `REPOSITORY`, `PREVIOUS_VERSION`, `DRY_RUN`) rather than `${{ }}`
  interpolation, and a comment at the top of the file states the rule for the
  next person adding an input. Behaviour is unchanged; the one incidental
  repair is that `cargo set-version ${{ ... }}` was unquoted and is now
  `cargo set-version "$NEW_VERSION"`. The rule is a repository rule, but
  `release-deploy.yml` does not satisfy it yet
- `Button`, `SidebarTrigger` and `Select`'s trigger take keyboard focus when
  they are enabled, and decline it — **with a stated reason** — when they are
  disabled. Tab now moves focus between gpuikit controls, and a disabled control
  is not a tab stop; those are the two behavioural changes a consumer could
  notice. A disabled control leaving the tab order is the weaker of the two
  ARIA-sanctioned answers, and is forced by gpui still having no `aria_disabled`
  to announce the other with
- `src/a11y.rs`'s module docs grow a section 4, "How the keyboard reaches it",
  and the old sections 4 and 5 renumber to 5 and 6. Section 6 no longer states
  the rollout order in prose — `ELEMENTS_WITHOUT_A_ROLE` is the checked form of
  it
- `Sidebar` and `SidebarTrigger`, which shipped roles ahead of the convention,
  are migrated onto it — as their issue said they would be if the convention
  chose differently. Behaviour is unchanged: the panel still reports
  `Role::Complementary` with its optional name, and `aria-expanded` still sits
  on the trigger, which the convention now states as a rule ("state goes on the
  element that changes it"). The markdown document's `Role::Document` moves the
  same way
- Two properties this convention deliberately cannot express, because gpui
  cannot: `disabled` (gpui has no `aria_disabled`; a disabled control is
  distinguishable only by the `Click` action its node does not offer) and
  `sort_direction` (no `aria_sort`, the finding `src/elements/table.rs` had
  recorded). Both are documented as upstream asks at the point the crate would
  use them, rather than modelled as fields that would silently do nothing
- `docs/component-triage.md` says **who took its verdicts**. A new attribution
  section, anchored on `<!-- ratification -->`, separates the three kinds of
  claim the verdict table flattens into one column: the 12 Shipped rows are
  facts a test checks, the 6 Issue rows are cheap proposals, and the 11
  Rejected rows are one agent's reading of this crate — **proposed and not
  ratified**. No rejection reason is softened, because the reasons were never
  the problem; a rejection that hedges is the "deferred / maybe / someday" the
  document exists to replace. What was missing was attribution
- The `Closing note for #59` section is **gone**, replaced by a past-tense
  "What became of #59". #59 was closed `COMPLETED` on 2026-03-25, five months
  before that document existed, so the ready-to-paste comment asserted a
  pending action nobody could take — and its counts ("10 have shipped", "8
  have a ready-to-file issue body") were the one part of a machine-checked
  document no test reached. Counts now live only where `triage_coverage` can
  see them
- `every_control_on_a_row_is_the_same_height` is renamed
  `every_sized_control_on_a_row_is_the_same_height`, and carries the two
  exclusion lists it was missing: the six elements that are not on the shared
  size scale at all (`ToggleGroup`, `Tabs` and `Alert`, whose height is their
  padding plus a line box; `Slider`, `Progress` and `RadioGroup`, which
  hard-code a track or glyph size), and the seven that *are* `ControlSized` and
  still are not measured (`Field`, `Input`, `Textarea`, `Table`, `Sidebar`,
  `SidebarTrigger`, `CheckboxBox`), each with its one-line reason. Sixteen
  implementors, nine in the row: someone adding a tenth can now tell unfinished
  coverage from a broken scale. No behaviour changed
- Prose counts in `docs/component-triage.md` that had drifted are corrected
  with the `Resizable` row's move: the verdict split is **12 Shipped / 6 Issue
  / 11 Rejected**, and `EXPECTED` in `src/elements.rs` moves with it. "Ten rows
  below are Shipped" (stale since `Sidebar` shipped in #169) and "eight
  surviving components" go with them, and the dependency graph's edge onto the
  deleted `docs/issues/sidebar.md` is discharged
- Two new tests in `src/elements.rs`'s `triage_coverage` hold the above to the
  document: the attribution section exists, precedes the table it is about,
  still says the rejections are proposed rather than ratified, and states the
  same three counts `EXPECTED` enforces; and the #59 section neither promises a
  paste nor loses the date. An unchecked claim in prose is how the missing
  attribution went unnoticed in the first place

### Fixed

- **`ld` is no longer OOM-killed while linking the examples.** Eight example
  binaries live in `examples/`, and each is a full link of gpui.
  `cargo build --all-targets` and a bare `cargo test` link all eight; cargo
  sizes `-j` from the CPU count with no knowledge of a memory limit, so several
  `ld` processes run at once — each holding that binary's debug info, which
  `[profile.dev] debug = 2` made maximal — and the kernel kills one. The
  message, `ld terminated with signal 9 [Killed]`, names no crate and no
  symbol, so it reads as a compile error that does not exist. Three changes,
  all build configuration and no library code: `[profile.dev]` now sets
  `debug = "line-tables-only"`; a new `.cargo/config.toml` passes
  `-Csplit-debuginfo=unpacked` on Linux targets, where the dev default of `off`
  copies every byte of DWARF through the linker into the image; and every
  `[[example]]` now carries `required-features = ["examples"]` against a
  feature that enables nothing, so a build that did not ask for a demo does not
  link one. `examples/context_menu.rs` is declared in `Cargo.toml` for the
  first time, because an autodiscovered target cannot carry
  `required-features`.

  **The cost.** Debug builds keep file and line in backtraces but lose the type
  and variable detail a debugger wants; a session that needs it asks on that
  build alone with `RUSTFLAGS="-Cdebuginfo=2"`. Running an example now needs
  `--features examples`, and `cargo check --all-targets` no longer type-checks
  the examples without it — `examples/README.md` and the commands in it are
  updated. `src/build_profile_guard.rs` holds all three settings, plus the
  absence of a new autodiscovered example, under `cargo test --lib`.

  **What this does not cover.** `cargo test --all-features` still links all
  eight: cargo has no way to hold a feature back from `--all-features`, so the
  gate does not apply to the command that produced the original kill on
  `markdown_streaming`. That case rests entirely on the debug-info reductions,
  whose magnitude here is unmeasured. If it still dies, the next levers, in
  order: `[profile.dev.package."*"] debug = 0` — dependency debug info is the
  bulk, and the guard test rejects only a *raised* override, so this stays
  open — and then moving the examples into a package of their own, which is the
  only arrangement in which no invocation of this library's own cargo commands
  builds them at all, `--all-features` included.

- **`Slider`'s value maths reads the thumb radius at the rem size the thumb is
  drawn at.** `value_from_position` inset the usable track by a hardcoded
  `px(6.)` while `render` drew the thumb at `rems(0.75)` — the same length only
  at gpui's default 16px rem. At any other rem size the value-to-position
  mapping skewed, worst towards the ends of the track: at a 32px rem, a press
  three quarters along the track set 73.4 instead of 75. Both now read one
  private `THUMB_SIZE` constant, resolved against `Window::rem_size()` when the
  event is handled, so the drawn thumb and the inset cannot disagree. The
  public API is unchanged
- **`Slider` keeps the drag after the pointer leaves the track.** The movement
  and the release are registered on the *window* now, from the `canvas` paint
  closure that already measures the track — the pattern `Splitter` and `Input`
  established. Only the press stays on the track div, because a press always
  starts there. Before this, all three were plain `div` listeners, and a `div`
  listener only fires while its hitbox is hovered: a drag that left the track
  froze the thumb at the edge instead of pinning it to the end of the range,
  and a button released outside the track was never delivered at all, leaving
  `is_dragging` stuck `true` and the thumb wearing its dragging border. A move
  with no button held now ends the drag too, which covers the release the
  window never saw — the pointer left the *window* with the button down, or
  another handler swallowed the mouse-up. The public API is unchanged

## [0.8.0] - 2026-08-17

The release that makes streaming markdown depend on a published version rather
than a git rev. `Markdown::append`, the off-thread parser and the optional
`stitch` feature have all been on `main` since 0.7.0 with nothing to name them
by. Also corrects `rust-version`, which claimed 1.75 and has not been true for
some time.

### Breaking Changes

- `gpuikit::traits::portal` is gone — `Portal`, `PortalPosition`,
  `AnchorCorner`, `AnchorEdge` and `FitMode` with it. 486 lines of positioning
  math with zero callers, zero implementors and zero tests, read against all
  six of this crate's overlay call sites and adopted at none of them:
  `gpui::anchored()` offers every corner, fit mode and offset `PortalPosition`
  did — plus edge-centre anchors and `.position()` — and computes them in
  `prepaint`, where the overlay's measured size and the viewport size exist.
  Those are exactly the two arguments `calculate_position` demanded from its
  callers, and no `render()` body has either. Migration: `.offset(point)`
  becomes `anchored().offset(point)`, and `FitMode::SnapToViewport` becomes
  `anchored().snap_to_window_with_margin(margin)` — the two calls `Portal`
  stood in for. The convention that replaces the trait is `docs/overlays.md`,
  checked by `overlay_coverage` in `src/elements.rs`
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

- `Sidebar` (`gpuikit::elements::sidebar`): a panel docked to the left or right
  edge of the window, with a caller-owned width and expanded/collapsed state.
  Collapsed is a **rail** of icon controls rather than a `when(open, …)`, which
  is what makes it more than a conditional; and once the window is narrower
  than a breakpoint (640px by default, `overlay_below`/`never_overlay` to move
  or disable it) an expanded panel becomes a dismissible drawer with a scrim,
  leaving a rail-width gutter behind so the content does not reflow. It ships
  **no** menu/group/header/footer sub-components — `List` (with
  `ListEntry::header`), `Separator` and `Button` are the contents, and both the
  new showcase page and the showcase's own navigation are composed exactly that
  way, replacing the hand-rolled `div`-with-a-border the showcase used to draw.
  The one sub-component, `SidebarTrigger`, exists for an accessibility reason:
  the panel reports `Role::Complementary` with an accessible name, and
  `aria-expanded` belongs on the control that changes the state. These are the
  first elements in `src/elements/` to report a role at all — ahead of
  `docs/issues/element-roles-convention.md`, which could not be honoured
  without shipping a landmark with no role; that issue now records what this
  found, and this file is the first thing to migrate if the convention chooses
  differently. `SidebarLayout::resolve` is a pure function and is where to
  argue about the push-versus-overlay behaviour. No resizable edge — the width
  is the caller's and never changes itself
- `InputState` learns read-only: `read_only(bool)` (builder),
  `set_read_only(bool, cx)` and `is_read_only()`. It closes every *user* path
  into the content — typing, IME composition, paste, cut's removal, the delete
  family, tab, newlines, undo and redo — while leaving focus, cursor movement,
  selection, copy, scrolling and the programmatic setters
  (`set_content`, `insert_text`, `delete_backward`, `undo_action`,
  `redo_action`) alone, the same split a browser's `readonly` attribute draws.
  The delete family is guarded individually rather than left to the funnel,
  because each *moves the selection* before deleting through it. `set_read_only`
  no-ops on an unchanged value, so a wrapper may impose it every frame.
  `TextField::read_only` is new; `Textarea::read_only` is unchanged in
  signature and enforced rather than cosmetic — see Fixed
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


- `Markdown::append`, for content that arrives a piece at a time (an LLM reply,
  a log tail). It extends the source instead of making the caller rebuild and
  re-set the whole document, and unlike `set_source` it keeps the selection —
  selection positions are `(run, byte offset)`, so text arriving at the end
  cannot disturb a selection made earlier
- Optional `stitch` feature: closes the syntax a partially streamed document
  leaves open (`**bold` with no closer, `[label](htt`) before parsing, so
  streaming text does not flicker between literal markers and styled text.
  Off by default — [mdstitch](https://docs.rs/mdstitch) declares
  `rust-version = "1.95.0"`, above this crate's 1.85, so turning it on raises
  the toolchain your build needs. `markdown::preprocessing_available()`
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
- `rust-version` is `1.85`, correcting a declared `1.75` that was never kept —
  the crate uses async closures and gpui is edition 2024, both of which need
  1.85. Nothing about what compiles changes; the manifest now says what was
  already true. It remains a statement about this crate's own source: cargo does
  not hold dependencies back to it, and several already declare more
  (cosmic-text and smol_str 1.89, oo7 1.92 on Linux)
- The `stitch` feature's toolchain requirement is written down rather than met
  in a build log: it needs **Rust 1.95**, against this crate's 1.85, because
  mdstitch declares `rust-version = "1.95.0"`. The README gains a feature table
  and a minimum-Rust-version section, and the crate docs docs.rs renders list
  all four features instead of only `editor`

### Fixed

- `Textarea::disabled(true)` produces a control that is actually inert. It used
  to set `opacity(0.65)` over a fully live `text_area()`, so the textarea
  looked disabled while still taking focus, keystrokes and IME input. A
  disabled `Textarea` now paints its value as static text with **no live
  element at all**, which is the only thing that also stops it taking focus —
  a painted `Input` registers its actions and its IME handler and is in the tab
  order, so read-only alone cannot implement `disabled`. The consequence to
  agree with: a disabled textarea *clips* a value longer than its rows instead
  of scrolling it; `read_only` is the option that keeps scrolling, and the
  showcase demonstrates both. `TextField` already worked this way and now
  shares the helper rather than the crate growing a second copy
- `Textarea::read_only(true)` means something. It was the same lie with
  different colours — and said so in its own doc comment — and now imposes
  `InputState`'s new read-only flag. **Behaviour change to note:**
  `Textarea::read_only` and `TextField::read_only` *write to the `InputState`
  they are given*, at the top of `render`. That is the only way a
  wrapper-level property can be enforced, and it is scoped as tightly as it
  can be: the wrapper field is an `Option<bool>`, so a wrapper that never calls
  it says nothing and a state its owner made read-only is never quietly handed
  back. A read-only `Input` also paints no caret — a caret promises that what
  you type lands there
- `Dropdown`, `Select` and `Popover` popups no longer hang out of the window by
  their own gap. Each put its distance from the trigger on the *child* of
  `anchored()` as a margin, and `Anchored::prepaint` fits the union of its
  children's **layout** bounds to the window — a margin is outside that union,
  so gpui clamped each popup into the window correctly and the margin then
  pushed it straight back out. The gap moves to `anchored().offset(…)`, which
  is inside what gpui measures. Measured, not theorised: the new test in
  `dropdown.rs` reported a popup spanning 0px to 244px in a 240px-tall window
  before the fix
- **Visible behaviour change.** A markdown code block whose fence has not
  closed yet is drawn as plain monospace, and gains its syntax colors the
  moment the closing fence arrives. The highlight cache is keyed on the whole
  block's text, so a fence streaming in through `Markdown::append` missed it on
  every delta — a full syntect pass over the block-so-far per rendered frame,
  quadratic in the block's final length — and deposited one cache entry per
  prefix, which evicted every settled block in the document when the cache hit
  its cap. An unclosed block now reaches the code block element with no
  language, which is the path a bare fence already took: no syntect pass and no
  cache entry. The consequence to agree with: a *static* document ending in an
  unclosed fence renders plain forever. New `markdown::has_open_code_fence`, a
  byte scan of the raw source that is deliberately more eager to close a fence
  than pulldown-cmark, so every disagreement costs a streaming block an
  optimization rather than taking a settled block's colors away
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
