# gpui-editor

🚧 This crate is a work in progress – it is currently in a prototype state 🚧

A open editor component for gpui – aspires to a Monaco-equivalent for gpui

## Game plan

Building a standalone editor component is probably like trying to boil the ocean... so I'll be trying to break it down into some smaller steps.

1. Just make it work for the simplest possible usecase, with the simplest possible approach (GapBuffer-based.)
2. Split up the pieces that should be generic/shared with other components - likely split up into smaller crates to power this and [gpui-kit](https://github.com/iamnbutler/gpui-kit). Think `theme`, `utils`, `highlight`, etc.
3. Start using in anger, leveling up testing, using in other projects, etc.
4. Syntax highligting???? LSP???, ???
5. Level up the editing approach - Likely some mix of the current approach and [ropes, sumtree](https://zed.dev/blog/zed-decoded-rope-sumtree).

There are probably 800 missing steps in there, we'll just have to figure it out on the way.

Note: It's likely this crate becomes `gpui-kit-editor` in the future.

### Try it:

- **Navigation**: Arrow keys for cursor movement
- **Editing**: Type to insert, Backspace/Delete to remove text
- **Themes**: `Cmd+T` / `Cmd+Shift+T` to cycle through themes
- **Languages**: `Cmd+L` / `Cmd+Shift+L` to cycle through language samples

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

By submitting a contribution to this repository, you agree to license it under both Apache-2.0 and MIT as described above.

### Third-party licenses

- gpui: [LICENSE-APACHE](https://github.com/zed-industries/zed/blob/main/LICENSE-APACHE)
