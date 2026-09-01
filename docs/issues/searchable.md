# Searchable: a recipe in `docs/`, not an element

## What it is

SwiftUI's `.searchable` — type into a field, a visible collection narrows. In this crate that capability already shipped three times as three different things: **Combobox** (`src/elements/combobox.rs`) filters to *choose a value*, **Command** (`src/elements/command.rs`) filters to *run an action*, and **Table**'s answer is a sentence in its module docs — "Filtering is a `TextField` above the table. It is not a table feature and there is no filter input inside this element" — demonstrated live on the showcase's Table page. What remains of `.searchable` after those three is not a fourth element. It is the filter-a-visible-collection recipe, currently stated once for Table and nowhere for List, with four questions (debounce, empty state, highlighting, focus, announcement) each caller re-answers alone. The deliverable is `docs/searchable.md`.

## Why it survives triage

**The verdict is: a document, not an element.** The house has explicit precedent — `docs/component-triage.md` says "a settled question becomes a document in `docs/`", and both `docs/overlays.md` and `docs/menus-and-listboxes.md` are exactly that: a question every component would otherwise answer differently, decided once in prose that tests can check. Filtering is the same shape. The element-sized versions already exist or were already refused:

- A `Searchable` element would re-fight `docs/issues/command.md`'s settled argument that "Matching is not this crate's business" — a shipped matcher is a shipped ranking opinion no caller can replace without replacing the component.
- A filter prop on `Table` was refused in the module's own docs; a filter prop on `List` would reopen the identical question to reach the opposite answer.
- The one shared function that exists, `matches_query` (`src/elements/listbox.rs`), is `pub(crate)` on purpose and is two lines: lowercase, `contains`. Exporting it would be an import that saves nothing; the showcase already writes those two lines inline and that *is* the API.

What the document decides, flatly, so no caller decides it again:

- **Debounce: no.** For an in-memory collection the recipe is `cx.observe` the filter's `InputState` and re-derive rows on notify — the showcase's exact wiring. The crate contains zero debounce machinery and needs zero; debouncing belongs to async data sources, which are the application's, with `cx.spawn` if it wants one.
- **Empty state: always, and from stock parts.** `Table::empty()` exists for precisely "the state a filter produces every time it matches nothing"; for `List` and ad-hoc collections the answer is `empty()` from `src/elements/empty.rs` — icon, title, description, and an action, which should be "clear the filter".
- **Match highlighting: gpui's primitive, no new util.** `StyledText::with_highlights(Range<usize>, HighlightStyle)` exists upstream; the recipe shows computing the range with `to_lowercase().find()` alongside the same two-line match. The crate ships no fuzzy matcher and no range util, for the `command.md` reason.
- **The filter is state the caller owns.** Rows arrive at the collection already filtered — Table's division of labour, generalized. The recipe never puts the query inside the collection element.

## Prior art

- **SwiftUI `.searchable(text:placement:prompt:)`** — what the modifier *owns* is the split to copy: the binding (here: an `Entity<InputState>`), the placement (here: composition — the field sits where the caller puts it), the prompt (here: `.placeholder`). What it also owns — tokens, suggestions, scopes, `isSearching` — is deliberately not copied; see Non-goals.
- **This crate's own three**: `combobox.rs`, `command.rs`, `table.rs` module docs. The document's first section is a router: filtering to choose a value is Combobox, to run an action is Command, to narrow what you are looking at is this recipe.
- **`docs/overlays.md` / `docs/menus-and-listboxes.md`** — the genre being written in.
- Re-open all of these before writing.

## What it has to close in this crate

- **`docs/searchable.md`** — the router paragraph, the wiring (a `text_field` above the collection, `cx.observe`, re-derive), the four decisions above, and the keyboard and announcement sections below. Table's module doc gets one line pointing at it; the doc restates Table's sentence rather than moving it, since coverage tests key off module docs.
- **A List demonstration.** The recipe is only *stated* for Table today. The showcase List (or Table) page demonstrating filter + `empty()` + highlight closes the gap between "said" and "shown"; Table's page already has the filter and the empty message, so highlighting is the only new showcase work.
- **Keyboard focus flow, decided.** Nothing in `src/input/bindings.rs` or `src/keymap/` binds a focus-the-filter key today. The recipe's answer: `cmd-f` as an app-level `KeyBinding` with a `None` context, focusing the field's `FocusHandle`. Slash-to-focus is named and declined: `/` is text in any focused input, so it only works with a key-context dance (`combobox.rs`'s "The keyboard, which is the hard part" documents the machinery) that a filter field does not earn. Escape-to-clear rides the `InputState` propagation shape combobox already documents: clear if non-empty, else `cx.propagate()` so an enclosing Dialog still closes.

## Accessibility

The result count is the accessibility of this pattern — a filter that silently empties a list is invisible to a screen reader — and the honest answer is that **the crate cannot announce it yet**. A results live-region needs the live-region decision `src/a11y.rs` has explicitly not taken: `ELEMENTS_WITHOUT_A_ROLE` records both `alert` ("needs the live-region decision this convention has not taken") and `toast` ("the live-region decision `alert` is also waiting on — both should be taken once"). This document adds the third client of that one decision and says so, the same way `table.rs` names gpui's missing `aria_sort` as an upstream ask rather than working around it. Until then: what already works is per-row `position_in_set`/`size_of_set` — Command's rows announce "3 of 40", which is the count, delivered on arrow-down — and the recipe says a filtered List should announce the same pair once List has roles. The `empty()` element renders the zero state visibly; when the live-region decision lands, the count announcement and `alert`/`toast` adopt it together.

## Sizing

n/a for a document. The field in the recipe is a `TextField`, which is already `ControlSized`; the recipe adds no dimensions of its own.

## Showcase

No new page — `showcase_coverage` keys off `pub mod`, and this ships none. The Table page is the existing demonstration; this issue's showcase work is adding match highlighting to it (or to the List page) so every claim in the doc is visible in one place.

## Non-goals

- **A `Searchable` element, trait, or modifier.** The argument above; if this issue grows one, it has become the thing it is against.
- **Tokens and multi-select chips** — that is a multi-select Combobox, already named "a second issue" in `combobox.rs`'s "Not built".
- **Suggestions** — a field that proposes completions *is* Combobox; the router paragraph says so.
- **Search scopes and placement** — application shell concerns, like the rejected Menubar.
- **A shipped matcher, fuzzy or otherwise** — settled by `docs/issues/command.md`; making `matches_query` `pub` is declined above.
- **Debounce machinery, async sources, pagination** — the application's; Pagination's rejection already names the missing paginated data source.
