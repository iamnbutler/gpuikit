# Scope markdown run ids per document, and give runs real a11y roles

Markdown text runs were minting *global* element ids (`md-run-1`, `md-run-2`, …)
from a counter that restarts at zero for every document, so any two markdown
documents in one frame produced the same ids — the normal case in an app that
renders one `Markdown` entity per chat message. gpui hashes an element's whole
`GlobalElementId` path into an accesskit node id and rejects duplicates: a
`debug_assert!` panic in debug builds, a silently dropped accessibility node in
release. The panic was only latent because no `SelectableText` carried a role, so
none of these nodes ever reached the tree — and any role would have restored it.
Each document now renders its runs under a per-document element
(`md-doc-<entity id>`, overridable via the new `MarkdownElement::id`), so run ids
are unique by construction. The renderer's element and run counters, which had to
tick in lockstep at all five call sites, are now the single counter behind
`next_run`: a run's index is its element id, its selection identity and its
per-frame registry slot at once.

With the ids safe, the accessibility work follows. The document root is
`Role::Document`, and every run now carries a required `RunRole` — paragraph,
heading (with its level), block quote, list item or code — mapped to the matching
accesskit role and labelled with the run's text. The label goes in the node's
`label`, not its `value`: accesskit only names a node from `value` under
`Role::Label`, so copying gpui's own text-element pattern would have produced
nameless nodes. `SelectableText::new` therefore takes two more arguments (the
plain text and the role) — the one breaking change, and the element needs the
plain text stored because `StyledText` does not hand it back and the a11y node is
built during prepaint, before any layout exists. Accessibility cannot be switched
on in a test (the active flag is only ever set by a platform adapter's activation
callback, which the test platform has none of), so the new `#[gpui::test]` cases
reconstruct exactly what gpui reads, at the same point in the frame: two documents
in one frame yield 10 disjoint id paths of the shape `…md-doc-<n>.md-run-<m>`, ids
stay put across frames, an explicit `.id()` separates two elements over one entity,
and every run kind reports its role, label and level. Table cells, images, and a
`Role::List` parent for list items are deliberately left out of scope; they need
structural changes to the renderer rather than a role.

Verification: PASSED — `cargo test --all-features -j 1` (335 lib tests + 2
doctests, 0 failures), plus `cargo fmt --all` and `cargo clippy --all-targets`
clean of new warnings. The two id-collision tests were also confirmed to fail
against the pre-fix scoping.
