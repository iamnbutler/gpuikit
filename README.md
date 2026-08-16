# gpuikit

<img width="2400" height="1424" alt="CleanShot 2025-12-05 at 14 31 13@2x" src="https://github.com/user-attachments/assets/4d3bddc5-83c2-4afc-b767-01047bdf46fa" />

A UI toolkit for GPUI applications.

🚧 Note: Expect every release to have many, undocumented breaking changes for now. Use at your own risk and pin your versions 🚧

## Usage

```toml
[dependencies]
gpuikit = "0.7"

# OR to enable the text editor component:
# gpuikit = { version = "0.7", features = ["editor"] }

# OR, for streaming markdown, to close the syntax a half-written document
# leaves open before parsing it (needs Rust 1.95+):
# gpuikit = { version = "0.7", features = ["stitch"] }
```

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

40+ components including: Accordion, Alert, Avatar, Badge, Breadcrumb, Button, Card, Checkbox, Collapsible, Dialog, Dropdown, Field, Input, Popover, Progress, Radio Group, Scroll Area, Select, Skeleton, Slider, Switch, Tabs, Textarea, Toast, Toggle, Tooltip, and more.

See [todo.md](todo.md) for the full list and roadmap.
