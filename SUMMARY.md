# One convention for how an element reports an accessibility role

`src/a11y.rs` is the decision `docs/issues/element-roles-convention.md` asked
for, and its module docs are the decision record — one numbered section per
question the issue posed, kept next to the code that implements it. An element
declares what it announces by implementing the new
`traits::accessible::Accessible`, returning an `A11y` value that carries a
role, an accessible name and whatever state goes with the role (`toggled`,
`selected`, `expanded`, a text or bounded-number `value`, `orientation`,
`level`, `position_in_set`), and applies it to the root element it was already
building with one method, `.announce(a11y)`. No element has to become a
hand-written `Element`: `Announce` is blanket-implemented for gpui's
`StatefulInteractiveElement`, which a `Div` becomes only once `.id()` has been
called, so **"no id, no role" is enforced by the type system** and roles stay
out of the duplicate-node trap `src/element_id.rs` documents. The name is
*required* for the roles listed in `role_requires_a_name` — a nameless
`Role::Button` trips a `debug_assert!`, so `button("save", "")` now panics in
debug builds — and it is derived from the element's own visible text where it
has any, taken as a constructor argument where it has none, and never from the
tooltip. `disabled` and `sort_direction` are not modelled: gpui has no
`aria_disabled` or `aria_sort`, so both are recorded as upstream asks at the
point the crate would use them rather than as fields that would silently do
nothing (a disabled control is distinguishable only by the `Click` action its
node does not offer).

`Button` is the worked example — it announces `Role::Button` named by its
label, so there is no second string to keep in step. Two things went beyond the
spec because the guard demanded it: `Sidebar`/`SidebarTrigger`, which had
shipped roles ahead of this decision, and markdown's `Role::Document` are
migrated onto `announce` with no behaviour change, which lets
`a11y::tests::no_element_calls_gpuis_a11y_builders_directly` — a source scan
modelled on `element_id`'s constant-id scan — cover all of `src/` rather than
part of it. Roles are tested by rendering a component and calling the two
`Element` methods gpui's own accessibility walk calls, `a11y_role` and
`write_a11y_info`, which is the only way to read a node back given that
accessibility cannot be switched on in a test; the helper's blind spots
(`Component<C>`, and `AnyElement`, neither of which forwards those methods) are
documented where a future test will meet them. Docs follow the decision:
`docs/issues/element-roles-convention.md` gains a settled banner and its five
answers, `docs/component-triage.md`'s blocker row flips along with the three
other places that referred to it, `src/elements/table.rs`'s two recorded
findings are answered, and `todo.md` / `CHANGELOG.md` follow. No crate-wide
sweep and no showcase page: the state fields are proven against a bare `div`,
and this is a prerequisite rather than a component.

Verification: PASSED — `cargo test --all-targets` (375 lib tests, plus
`cargo test --lib --all-features`, `cargo test --doc`, `cargo check
--all-features`, `cargo fmt --check`, and `cargo clippy --lib --all-targets`,
which reports no new warnings). The guard scan was also checked against a
deliberate violation, which it caught with a file:line.
