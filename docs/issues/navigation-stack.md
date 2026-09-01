# NavigationStack: push/pop is state the caller owns; focus handoff is the element

## What it is

Drill-in navigation: a stack of pages, a back affordance, and a path the caller can manipulate — SwiftUI's `NavigationStack`/`NavigationLink`, libadwaita's `AdwNavigationView`. The crate has `Breadcrumb` (`src/elements/breadcrumb.rs`) rendering a trail and nothing owning a stack.

The design question this issue exists to answer: in gpui, where apps hold state in Entities and render from it, is a navigation stack an element, an Entity-holding pattern, or a small state struct plus a back-button element? **The last one, plus one element the pattern cannot supply: the page container that moves focus.** What ships:

- **`NavPath<T>`** — a plain value (a thin `Vec<T>` newtype: `push`, `pop`, `pop_to_root`, `top`, `is_at_root`) the caller holds in their own Entity, exactly as `SidebarState` and Splitter's ratio are the caller's. The root is not in the path and cannot be popped — SwiftUI's rule ("the stack doesn't allow the root view to be removed") adopted verbatim.
- **`nav_page`** — a `RenderOnce` container carrying `Role::Group` named by the page title, whose job is the focus handoff on push and pop. This is the part that must be an element: a recipe cannot move focus.
- **`back_button`** — the control, shipped for the same reason `SidebarTrigger` ships: the affordance has accessibility obligations (a name naming the destination, a keyboard binding, absence at root) that belong on the control.

The mapping from path top to page is a `match` in the caller's render — the Rust spelling of `navigationDestination(for:)` is a match arm, and the toolkit must not grow a destination registry to replace what the language already does better.

## Why it survives triage

The stack itself is state, and this crate's answer to state is unbending — "State is the caller's" (`src/elements/sidebar.rs`), "The ratio is the caller's" (`src/elements/splitter.rs`). A `NavigationStack` element that owned its pages across frames would be the first element in the crate to do so, and everything a caller wants from a manipulable path — deep-linking, restoring on launch, pop-to-root from a command — falls out of the path being a value in their Entity and dies if it is not. SwiftUI landed in the same place after years of `NavigationView`: the path is a binding the app owns, "the stack adds items to the collection as it adds views... your code can also modify the array to change the views on the stack".

But the pattern alone is not enough, and the reason is focus. When the visible page is replaced, a keyboard user's focus is standing on an element that no longer exists, and a screen-reader user has been told nothing. Neither problem can be solved in module docs; both need an element that participates in rendering. That is the whole of what `nav_page` is, and the whole of why this issue produces code rather than only a document.

Deliberately **not** built: transitions and swipe-back gestures (the triage already rejected Drawer as "a phone gesture on a desktop toolkit", and `AdwNavigationView`'s one-finger swipe is the same gesture); a forward stack (build it when a consumer has a browser); a destination registry; route serialisation; and any notion of the stack rendering more than its top page.

## Prior art

- **SwiftUI `NavigationStack`** ([developer.apple.com/documentation/swiftui/navigationstack](https://developer.apple.com/documentation/swiftui/navigationstack)). Take: the path as a caller-owned homogeneous collection (`init(path:root:)` with `[Park]`) — `NavPath<T>` is exactly this; programmatic navigation as plain mutation (append to push, `removeLast` to pop, replace wholesale to deep-link); back disabled at root. Leave: `NavigationPath`'s type-erased heterogeneous path — Rust callers have enums, and an enum path is better in every way; and `navigationDestination(for:)`'s type-keyed registry — that exists because SwiftUI cannot write a match over view state, and gpui callers can.
- **libadwaita `AdwNavigationView` / `AdwNavigationPage`** ([class.NavigationView.html](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.NavigationView.html)). Take: the page as the unit carrying a `title` (which names the back button and the group), `pop_to_tag` as the argument for addressable ancestors (`NavPath::pop_to` where `T: PartialEq`), `can-pop` as the argument for a guarded pop the caller vetoes; the keyboard vocabulary — Alt+Left to go back, Escape "configurable via `pop-on-escape`"; and its accessibility answer: a page is `GTK_ACCESSIBLE_ROLE_GROUP`. Leave: gestures and transition animation, per Non-goals; and the widget owning the stack — adwaita's stack lives in the widget because GTK holds widget trees; gpui holds Entities.
- **macOS System Settings** (and SwiftUI-on-Mac practice: detail-pane drill-in gets "its own `NavigationStack`... because Mac and visionOS sidebars stay visible during pushes" — [developer.apple.com forum thread 721192](https://developer.apple.com/forums/thread/721192)). Take: the desktop shape — the stack lives *inside one pane*, the sidebar stays put, the back button sits at the top of the pane's content, small and chevron-led. This is the composition with SplitView's issue (`docs/issues/split-view.md`): a `NavPath` in the Entity that owns the detail column, nothing more.
- **In-house: `SelectState`** (`src/elements/select.rs`) is the precedent for the crate shipping a state struct the caller holds in an `Entity<...>` — proof the pattern is already house style; `Breadcrumb` is the trail renderer a `NavPath` feeds (a recipe in the module docs: each ancestor becomes a `breadcrumb_item` whose `on_click` is `pop_to`).

Re-open every cited source before implementing — names above, not lines.

## What it has to close in this crate

- **`NavPath<T>` semantics, pinned by tests with no window**: pop at root is a no-op, `pop_to` with an absent value is a no-op not a clear, `top()` is `None` at root. The pure-function-first shape of `SidebarLayout::resolve` and `SplitterGeometry`, applied to a `Vec`.
- **The focus handoff, on push.** `nav_page` must, on the first frame after its identity changes, move focus to the page — the container itself, as a focusable-but-not-tabbable target (a11y section 4's machinery distinguishes exactly this), so a screen reader announces the newly-focused group *by its page title* and a keyboard user's Tab starts from the top of the new page. The identity change is observable through the element id (`scoped(&id, page_key)`), and the grab-focus-on-appear mechanism is the `canvas` paint hook `src/elements/splitter.rs` already uses. This is the heart of the element and the section below.
- **The focus handoff, on pop.** The element that pushed is usually gone or stale; the honest target is the back button when the pop came from keyboard/back, and the page container otherwise. `back_button` therefore takes an optional caller `FocusHandle` exactly as `SidebarTrigger::focus_handle` does.
- **The back button's name names the destination**: "Back to Settings", built from the previous page's title — a bare "Back" is the name rule (`a11y.rs` section 2) satisfied in letter and violated in spirit. At root the button is absent, not disabled: a control that can never do anything is clutter to a screen reader, and SwiftUI hides it ("disables backward navigation controls when the stack is empty" — hidden, on every Apple platform).
- **The keyboard binding.** Alt/Cmd+Left (platform-conditional) dispatching a `Pop` action, installed the way `bind_focus_keys` installs Tab (`src/a11y.rs`), bound only while a `nav_page` subtree has focus — the key-context discussion in a11y section 4's Tab notes is the map for getting this wrong correctly. `pop-on-escape` is deliberately not adopted: Escape belongs to overlays here (`docs/overlays.md`).
- **The Breadcrumb bridge.** `src/elements/breadcrumb.rs` is pre-convention: no `Accessible`, and `ELEMENTS_WITHOUT_A_ROLE` records it waiting on "Role::Navigation around a Role::List of Role::Link". The recipe wiring `NavPath` to `Breadcrumb` makes this issue the consumer that adoption was waiting for; adopting it is its own change, named here so nobody builds a second trail.

## Accessibility

This is the section the component exists for.

- **The page is `Role::Group`, named by its title** — adwaita's answer, adopted. The title is a constructor argument, not a builder: a page with no name is a group a screen reader calls nothing, the same reasoning that made `splitter`'s name positional. `Role::Group` joins neither `role_requires_a_name` nor `role_requires_keyboard_focus` lists blindly — the name requirement should be enforced by the constructor instead, since a bare `Role::Group` is legal elsewhere.
- **Focus movement is the announcement.** This crate has taken no live-region decision (`ELEMENTS_WITHOUT_A_ROLE`'s `alert` entry says so), and this component must not force one: moving focus to a named group is the assistive-technology notification, needs no live region, and is what WAI-ARIA SPA-navigation practice converged on (focus the new content's container or heading). If the live-region decision ever lands, a page-change announcement is additive.
- **The back button is a named `Role::Button`**, focusable, through `Accessible` + `.announce` like every control since the convention; its name carries the destination as above. It is a plain control — no `aria-expanded`, no state, because the state (the path) is not its to report.
- **Nothing announces the stack itself.** A stack of which only one page exists per frame has no container semantics to report; the trail, when shown, is `Breadcrumb`'s `Role::Navigation` (once adopted). Two nested landmarks for one navigation would be the sidebar-drawer's shadow-panel mistake (`src/elements/sidebar.rs`) all over again.
- **Tests**: the declaration half through `Accessible` (`a11y::test_support::announced`), and the focus handoff by drawing a real view and asserting where focus is after a push and a pop — the `tab_reaches_the_band` harness in `src/elements/splitter.rs` is the copyable worked example, including its warnings about `AnyElement` and `VisualTestContext::draw`.

## Sizing

`ControlSized` on `back_button`, every dimension off the rung — it is an `IconButton`-shaped control and shares its metrics. `nav_page` adds no dimension at all: a page is its content's size, and per `src/theme/control.rs`'s "What belongs here" note there is nothing shape-specific to key off the rung.

## Showcase

A build requirement per `showcase_coverage` in `src/elements.rs`: a page holding a small drill-in demo — a `NavPath<DemoPage>` in the showcase Entity, three levels deep, the back button, the Alt+Left binding live, a `pop_to_root` button proving the path is manipulable, and the Breadcrumb bridge rendered from the same path so one value visibly drives both affordances.

## Non-goals

- Push/pop transitions, swipe gestures, rubber-banding — the Drawer rejection's reasoning, standing.
- A forward stack, history, or forward navigation.
- A destination registry or any type-keyed routing; the caller's `match` is the router.
- Owning the path. If this issue grows an element that stores pages across frames, it has become the thing this argument is against.

## Blocked on

- Nothing hard. The Breadcrumb `Role::Navigation` adoption is named in "What it has to close" but does not block: the stack, the page and the back button are complete without the trail.
