# gpuikit

<img width="2400" height="1424" alt="CleanShot 2025-12-05 at 14 31 13@2x" src="https://github.com/user-attachments/assets/4d3bddc5-83c2-4afc-b767-01047bdf46fa" />

A UI toolkit for GPUI applications.

🚧 Note: Expect every release to have many, undocumented breaking changes for now. Use at your own risk and pin your versions 🚧

## Usage

```toml
[dependencies]
gpuikit = "0.8"

# OR to enable the text editor component:
# gpuikit = { version = "0.8", features = ["editor"] }

# OR, for streaming markdown, to close the syntax a half-written document
# leaves open before parsing it (needs Rust 1.95+):
# gpuikit = { version = "0.8", features = ["stitch"] }
```

## Features

All features are off by default.

| Feature | Needs Rust | What it adds |
| --- | --- | --- |
| `editor` | 1.85 | The `Editor` component, and the syntect-backed syntax highlighting that markdown code fences use once an app calls `markdown::init_code_highlighting`. Pulls in `syntect` |
| `stitch` | **1.95** | Closes the syntax a partially streamed markdown document leaves open (`**bold`, `[label](htt`) before parsing, so streaming text does not flicker between literal markers and styled text. Pulls in [mdstitch](https://docs.rs/mdstitch), which declares `rust-version = "1.95.0"`. `markdown::preprocessing_available()` reports which build you got |
| `runtime_shaders` | 1.85 | Compiles Metal shaders at runtime instead of at build time, so a macOS build needs no Xcode Metal toolchain |
| `schema` | 1.85 | Adds the `schemars` dependency. Nothing in the crate derives `JsonSchema` yet, so today this only affects your dependency graph |

### Minimum Rust version

gpuikit declares `rust-version = "1.85"`. That is a statement about **this
crate's own source** — it uses async closures, and gpui is edition 2024, which
needs the same 1.85 — and not a guarantee about a whole build. The crate is
edition 2021, so cargo's v2 feature resolver does not hold dependencies back to
gpuikit's floor, and several of them already declare more (cosmic-text and
smol_str 1.89, image and time 1.88, oo7 1.92 on the Linux secret-service path).
On a toolchain near 1.85 you will most likely meet a dependency's floor before
you meet gpuikit's. A recent stable is the practical answer.

`stitch` is the one feature that raises the floor of gpuikit itself, to **1.95**.
Leaving it off costs you only the partial-syntax closing: `Markdown::append` and
the background parser are unconditional, so streaming still works — a
half-written `**bold` just flashes as literal asterisks until its closer
arrives.

## Control sizes

Controls that can share a row share one size scale. `ControlSize` names a rung
— `Small` / `Medium` / `Large`, 16 / 20 / 24px at a 16px root, `Medium` the
default — and the theme resolves it into every dimension a control needs:
height, padding, gap, radius, text size, line box, and how much of its box the
control's graphic fills.

```rust,ignore
use gpuikit::traits::control_sized::ControlSized;

h_stack()
    .child(button("save", "Save").large())
    .child(badge("2").large())
    .child(text_field(&state, cx).large())
```

Every control in that list is the same height, because none of them names one:
the rung does. A theme can rescale the whole set at once through
`Theme::controls`, and `Themeable::control_scale` is the method to override for
a custom theme type.

## Streaming markdown

Markdown parses off the UI thread, and content that arrives a piece at a time
goes in through `Markdown::append`:

```rust,ignore
markdown.update(cx, |markdown, cx| markdown.append(&delta, cx));
```

The previous parse keeps rendering until the new one lands, so the document
never blanks, and deltas arriving during a parse coalesce into a single
follow-up parse. The `stitch` feature additionally closes unterminated syntax
(`**bold`, `[label](htt`) before parsing, which is what stops a streaming
document flickering between literal markers and styled text. See
`examples/markdown_streaming.rs`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Components

Nearly 40 components including: Accordion, Alert, Avatar, Badge, Breadcrumb, Button, Card, Checkbox, Collapsible, Context Menu, Dialog, Field, Input, Popover, Progress, Radio Group, Scroll Area, Select, Slider, Switch, Tabs, Textarea, Toast, Toggle, Tooltip, and more.

See [todo.md](todo.md) for the full list, and
[docs/component-triage.md](docs/component-triage.md) for a decision — shipped,
issue, or rejected with a reason — on every component that was once on the
deferred roster.
