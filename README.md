# gpuikit

A work-in-progress modular UI toolkit for GPUI applications.

🚧 Note: Expect every release to have many, undocumented breaking changes for now. Use at your own risk and pin your verisons 🚧

**Crates**

- [`gpuikit` - ![crates.io](https://img.shields.io/crates/v/gpuikit.svg)](https://crates.io/crates/gpuikit) - Main crate with core components and re-exports
- [`gpuikit-theme` - ![crates.io](https://img.shields.io/crates/v/gpuikit-theme.svg)](https://crates.io/crates/gpuikit-theme) - Theming module
- [`gpuikit-keymap` - ![crates.io](https://img.shields.io/crates/v/gpuikit-keymap.svg)](https://crates.io/crates/gpuikit-keymap) - Keyboard shortcut and keymap management
- [`gpuikit-editor` - ![crates.io](https://img.shields.io/crates/v/gpuikit-editor.svg)](https://crates.io/crates/gpuikit-editor) - Text editor component

## Usage

Add the main crate to use all components:

```toml
[dependencies]
gpui = "SOME_VERSION"
gpuikit = "0.0.1"
# not supported yet, but in the future:
# gpuikit = { version = "0.0.1", features = ["theme", "keymap"] }
```

Or add individual crates as needed:

```toml
[dependencies]
gpuikit-theme = "0.0.1"
gpuikit-keymap = "0.0.1"
gpuikit-editor = "0.0.1"
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
