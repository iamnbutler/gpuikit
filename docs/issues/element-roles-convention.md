# Prerequisite: decide once how an element reports an accessibility role

## The problem

**No element in `src/elements/` reports an accessibility role.**
`grep -rn '\.role(' src/elements/` returns nothing. The crate's only
accessibility work is in `src/markdown/`, which reports roles for blocks and
text runs and has the tests to go with it.

Ten component issues came out of the #59 re-triage
(`docs/component-triage.md`), and #146 makes the accessibility answer a
precondition for each of them. Without a convention, those ten issues will each
invent a different mechanism, and the first two will be incompatible.

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

1. **Where a role is declared.** Most elements here are `RenderOnce` structs
   returning a `div`, not hand-written `Element` impls, so the role has to
   reach the `div`. Is there a `.role()` on gpui's `div`, or does an element
   that reports a role have to become a real `Element`? This is the first thing
   to find out and it may constrain everything else.
2. **How an element is *named*.** A role without an accessible name is close to
   useless. A `Button` has its label; an `IconButton` has only an icon and
   needs an explicit name. Decide whether that is a `.label()` on the element,
   reuse of the tooltip text, or something new — and whether it is optional.
3. **How state is reported.** Checked, expanded, selected, disabled, and the
   value of a slider or progress bar. Several elements already track all of
   these internally.
4. **How it is tested.** A convention nothing checks is a convention that
   decays. `src/markdown/` has the local precedent for testing roles.
5. **The rollout order.** Almost certainly: the simplest element first, as the
   worked example the other issues can point at.

## Scope

The decision plus one worked example. Not a crate-wide sweep — that follows,
and follows more safely once there is something to copy.

## Why now

Ten issues are waiting on it, and `src/traits/visual_focus.rs` already shows
the crate is willing to have a cross-cutting convention live in a trait.

---

*This is a prerequisite rather than a component, so it has no rung and no
showcase page of its own. It is reachable from
[`docs/component-triage.md`](../component-triage.md).*
