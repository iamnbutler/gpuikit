# Prerequisite: decide once how an element reports an accessibility role

## The problem

**Almost no element in `src/elements/` reports an accessibility role.**
`grep -rn '\.role(' src/elements/` returns exactly one module,
`src/elements/sidebar.rs`. Everything else with roles is in `src/markdown/`,
which reports them for blocks and text runs and has the tests to go with it.

Component issues came out of the #59 re-triage
(`docs/component-triage.md`), and #146 makes the accessibility answer a
precondition for each of them. Without a convention, those issues will each
invent a different mechanism, and the first two will be incompatible.

**`sidebar.rs` went ahead of this issue**, and did so knowingly: its own issue
required a `Complementary` landmark, and honouring this blocker would have
meant shipping a panel with no role at all. It is therefore the accidental
worked example question 5 asks for — and if this convention chooses
differently, it is the first thing to migrate. Everything about the mechanism
is confined to two `div`s in one file, and both of its ids come from the
caller, so nothing new is minted that `no_element_mints_a_constant_id` would
not catch. The data points it produced are recorded below as data points, not
as decisions.

## Why it is not just "add roles"

`src/element_id.rs` documents the trap in detail, and it is worth reading in
full before touching this. The short version: gpui builds an accesskit node for
an element that has **both** an id and an `Element::a11y_role`, and it hashes
the element's whole id path into the node id. A duplicate id is a
`debug_assert!` in debug builds and a silently missing node in release ones.

That means **adding roles turns latent id collisions into crashes**. The
crate's ids were audited and scoped in an earlier change, and
`element_id::tests::no_element_mints_a_constant_id` guards the shape — but that
scan explicitly cannot check whether an `(name, index)` sibling id has a
properly scoped parent. Scoping the ids first and adding roles second is the
order.

## What has to be decided

1. **Where a role is declared.** *Answered by shipping `sidebar.rs`*: gpui's
   `.role()` lives on `StatefulInteractiveElement` and is reachable on any
   `div().id(…)` (`gpui/src/elements/div.rs:1249`), so a `RenderOnce` struct
   does **not** have to become a real `Element`. This was the open question
   that could have constrained everything else; it does not. What remains is
   whether the crate wants a trait wrapping it or bare `.role()` calls.
2. **How an element is *named*.** A role without an accessible name is close to
   useless. A `Button` has its label; an `IconButton` has only an icon and
   needs an explicit name. Decide whether that is a `.label()` on the element,
   reuse of the tooltip text, or something new — and whether it is optional.
3. **How state is reported.** Checked, expanded, selected, disabled, and the
   value of a slider or progress bar. Several elements already track all of
   these internally. *Data point from `sidebar.rs`*: gpui has
   `.aria_expanded(bool)`, `.aria_selected(bool)`, `.aria_toggled(Toggled)` and
   the numeric-value family alongside `.role()`, so the mechanism exists; the
   open part is **which element carries the state**. `Sidebar` puts
   `aria-expanded` on `SidebarTrigger` rather than on the region, because that
   is the control that changes it, which is why the trigger exists as a
   sub-component at all. A convention that says "state goes on the region"
   would invert that.
4. **How it is tested.** A convention nothing checks is a convention that
   decays. `src/markdown/` has the local precedent for testing roles.
   *Data point from `sidebar.rs`*: an element with **both** a role and a mouse
   listener cannot be drawn with `VisualTestContext::draw`. Registering a mouse
   listener reads `Window::current_view`, which is only set while a *view*
   renders, so a bare draw panics inside gpui at `window.rs` with an opaque
   `Option::unwrap()` on `None`. `sidebar.rs`'s test module has a one-field
   `Harness` view (opened with `cx.open_window` plus `run_until_parked`, with a
   draw counter so a harness that never drew cannot pass silently) that the
   next element reporting a role can reuse.
5. **The rollout order.** Almost certainly: the simplest element first, as the
   worked example the other issues can point at.

## Scope

The decision plus one worked example. Not a crate-wide sweep — that follows,
and follows more safely once there is something to copy.

## Why now

Several issues are waiting on it, `src/traits/visual_focus.rs` already shows
the crate is willing to have a cross-cutting convention live in a trait, and
there is now one element in `src/elements/` doing this by hand — which is
exactly the situation this issue exists to stop happening ten times.

---

*This is a prerequisite rather than a component, so it has no rung and no
showcase page of its own. It is reachable from
[`docs/component-triage.md`](../component-triage.md).*
