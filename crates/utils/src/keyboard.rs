use gpui::{Keystroke, Modifiers};

/// Parses a string representation of a keyboard shortcut into a GPUI Keystroke
///
/// # Examples
///
/// ```
/// use common::util::keystroke_builder;
///
/// // Creates Cmd+S keystroke
/// let save = keystroke_builder("cmd-s");
///
/// // Creates Ctrl+Shift+P keystroke
/// let palette = keystroke_builder("ctrl-shift-p");
///
/// // Creates simple 'a' keystroke (no modifiers)
/// let type_a = keystroke_builder("a");
/// ```
pub fn keystroke_builder(str: &str) -> Keystroke {
    let parts: Vec<&str> = str.split('-').collect();

    let mut modifiers = Modifiers {
        control: false,
        alt: false,
        shift: false,
        platform: false,
        function: false,
    };

    let mut key_char = None;

    // The last part is the key, everything before it is a modifier
    let key = if parts.is_empty() {
        ""
    } else {
        parts[parts.len() - 1]
    };

    for i in 0..parts.len() - 1 {
        match parts[i].to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.control = true,
            "alt" | "option" => modifiers.alt = true,
            "shift" => modifiers.shift = true,
            "cmd" | "meta" | "command" | "platform" => modifiers.platform = true,
            "fn" | "function" => modifiers.function = true,
            _ => (),
        }
    }

    if !modifiers.control
        && !modifiers.alt
        && !modifiers.shift
        && !modifiers.platform
        && !modifiers.function
    {
        key_char = Some(key.to_string());
    }

    Keystroke {
        modifiers,
        key: key.into(),
        key_char,
    }
}
