//! Closing the syntax a partially streamed document leaves open.
//!
//! A document arriving a few characters at a time is, most of the time, not
//! valid markdown: `**bold` has no closer yet, `[label](htt` is half a link.
//! Parsed as-is it renders as literal asterisks that turn into bold text one
//! delta later — the flicker that makes streaming markdown look broken.
//!
//! With the `stitch` feature this hands the source to
//! [mdstitch](https://docs.rs/mdstitch) before parsing, which closes those
//! markers. Without the feature it is the identity function, so the rest of
//! the module does not need to know which build it got — only
//! [`preprocessing_available`] does.

use std::borrow::Cow;

/// Close any syntax the source leaves open, so a partial document parses as
/// the document it is becoming.
///
/// Returns the input unchanged (as [`Cow::Borrowed`]) when there is nothing to
/// close, and always when built without the `stitch` feature.
#[cfg(feature = "stitch")]
pub fn close_open_syntax(source: &str) -> Cow<'_, str> {
    use mdstitch::{LinkMode, StitchOptions};

    // `LinkMode::Protocol`, the default, rewrites `[label](htt` to a
    // placeholder URL — which this renderer would draw as a live clickable
    // link to nowhere. `TextOnly` leaves the label as plain text until the
    // real URL arrives.
    let options = StitchOptions::default().link_mode(LinkMode::TextOnly);
    mdstitch::stitch(source, &options)
}

/// Close any syntax the source leaves open, so a partial document parses as
/// the document it is becoming.
///
/// Returns the input unchanged (as [`Cow::Borrowed`]) when there is nothing to
/// close, and always when built without the `stitch` feature.
#[cfg(not(feature = "stitch"))]
pub fn close_open_syntax(source: &str) -> Cow<'_, str> {
    Cow::Borrowed(source)
}

/// Whether this build can close open syntax — true with the `stitch` feature,
/// false without it, where `close_open_syntax` is the identity.
///
/// Worth surfacing in an app that streams: the difference is visible.
pub fn preprocessing_available() -> bool {
    cfg!(feature = "stitch")
}
