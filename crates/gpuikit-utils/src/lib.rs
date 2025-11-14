//! Common utilities and helpers for GPUI applications
//!
//! This crate provides a collection of utility functions and helpers
//! commonly needed when building GPUI applications.

use anyhow::Result;
use gpui::{point, px, Bounds, Pixels, Point, Size};

pub mod geometry;
pub mod string;
pub mod task;

// Re-export commonly used utilities
pub use geometry::{center_rect, expand_bounds, point_in_bounds};
pub use string::{truncate_string, wrap_text};
pub use task::{debounce, throttle};

/// Common utility functions
pub mod common {
    use super::*;

    /// Clamp a value between min and max
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Linear interpolation between two values
    pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
        start + (end - start) * t.clamp(0.0, 1.0)
    }

    /// Map a value from one range to another
    pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        let normalized = (value - in_min) / (in_max - in_min);
        out_min + normalized * (out_max - out_min)
    }
}

/// Geometry utilities module
pub mod geometry {
    use super::*;

    /// Calculate the center point of a rectangle
    pub fn center_rect(bounds: Bounds<Pixels>) -> Point<Pixels> {
        point(
            bounds.origin.x + bounds.size.width / 2.0,
            bounds.origin.y + bounds.size.height / 2.0,
        )
    }

    /// Check if a point is within bounds
    pub fn point_in_bounds(point: Point<Pixels>, bounds: Bounds<Pixels>) -> bool {
        point.x >= bounds.origin.x
            && point.x <= bounds.origin.x + bounds.size.width
            && point.y >= bounds.origin.y
            && point.y <= bounds.origin.y + bounds.size.height
    }

    /// Expand bounds by a given amount
    pub fn expand_bounds(bounds: Bounds<Pixels>, amount: Pixels) -> Bounds<Pixels> {
        Bounds {
            origin: point(bounds.origin.x - amount, bounds.origin.y - amount),
            size: Size {
                width: bounds.size.width + amount * 2.0,
                height: bounds.size.height + amount * 2.0,
            },
        }
    }

    /// Contract bounds by a given amount
    pub fn contract_bounds(bounds: Bounds<Pixels>, amount: Pixels) -> Bounds<Pixels> {
        expand_bounds(bounds, -amount)
    }

    /// Calculate the distance between two points
    pub fn distance(p1: Point<Pixels>, p2: Point<Pixels>) -> Pixels {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        px((dx.0 * dx.0 + dy.0 * dy.0).sqrt())
    }

    /// Calculate the intersection of two rectangles
    pub fn intersect_bounds(a: Bounds<Pixels>, b: Bounds<Pixels>) -> Option<Bounds<Pixels>> {
        let x1 = a.origin.x.max(b.origin.x);
        let y1 = a.origin.y.max(b.origin.y);
        let x2 = (a.origin.x + a.size.width).min(b.origin.x + b.size.width);
        let y2 = (a.origin.y + a.size.height).min(b.origin.y + b.size.height);

        if x2 > x1 && y2 > y1 {
            Some(Bounds {
                origin: point(x1, y1),
                size: Size {
                    width: x2 - x1,
                    height: y2 - y1,
                },
            })
        } else {
            None
        }
    }
}

/// String manipulation utilities
pub mod string {
    use super::*;

    /// Truncate a string to a maximum length with ellipsis
    pub fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len < 3 {
            s.chars().take(max_len).collect()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    /// Wrap text to a specified width (character count)
    pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.len() + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Convert a string to title case
    pub fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Convert snake_case to camelCase
    pub fn snake_to_camel(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;

        for ch in s.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push_str(&ch.to_uppercase().to_string());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Convert camelCase to snake_case
    pub fn camel_to_snake(s: &str) -> String {
        let mut result = String::new();

        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push_str(&ch.to_lowercase().to_string());
        }

        result
    }
}

/// Task and async utilities
pub mod task {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// Debounce function calls
    pub struct Debouncer {
        last_call: Arc<Mutex<Option<Instant>>>,
        delay: Duration,
    }

    impl Debouncer {
        /// Create a new debouncer with the specified delay
        pub fn new(delay_ms: u64) -> Self {
            Self {
                last_call: Arc::new(Mutex::new(None)),
                delay: Duration::from_millis(delay_ms),
            }
        }

        /// Check if the function should be called
        pub fn should_run(&self) -> bool {
            let mut last_call = self.last_call.lock().unwrap();
            let now = Instant::now();

            match *last_call {
                Some(last) if now.duration_since(last) < self.delay => false,
                _ => {
                    *last_call = Some(now);
                    true
                }
            }
        }
    }

    /// Create a debounced function
    pub fn debounce(delay_ms: u64) -> Debouncer {
        Debouncer::new(delay_ms)
    }

    /// Throttle function calls
    pub struct Throttler {
        last_call: Arc<Mutex<Option<Instant>>>,
        interval: Duration,
    }

    impl Throttler {
        /// Create a new throttler with the specified interval
        pub fn new(interval_ms: u64) -> Self {
            Self {
                last_call: Arc::new(Mutex::new(None)),
                interval: Duration::from_millis(interval_ms),
            }
        }

        /// Check if the function should be called
        pub fn should_run(&self) -> bool {
            let mut last_call = self.last_call.lock().unwrap();
            let now = Instant::now();

            match *last_call {
                Some(last) if now.duration_since(last) >= self.interval => {
                    *last_call = Some(now);
                    true
                }
                None => {
                    *last_call = Some(now);
                    true
                }
                _ => false,
            }
        }
    }

    /// Create a throttled function
    pub fn throttle(interval_ms: u64) -> Throttler {
        Throttler::new(interval_ms)
    }
}

/// Time utilities (requires "time" feature)
#[cfg(feature = "time")]
pub mod time {
    use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};

    /// Format a timestamp for display
    pub fn format_timestamp(timestamp: i64) -> String {
        let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get a human-readable relative time string
    pub fn relative_time(datetime: DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(datetime);

        if duration.num_seconds() < 60 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            let mins = duration.num_minutes();
            format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if duration.num_hours() < 24 {
            let hours = duration.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if duration.num_days() < 30 {
            let days = duration.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else if duration.num_days() < 365 {
            let months = duration.num_days() / 30;
            format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
        } else {
            let years = duration.num_days() / 365;
            format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
        }
    }
}

/// UUID utilities (requires "uuid" feature)
#[cfg(feature = "uuid")]
pub mod uuid {
    use uuid::Uuid;

    /// Generate a new UUID v4
    pub fn generate() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate a short UUID (first 8 characters)
    pub fn generate_short() -> String {
        Uuid::new_v4().to_string()[..8].to_string()
    }

    /// Validate a UUID string
    pub fn is_valid(uuid_str: &str) -> bool {
        Uuid::parse_str(uuid_str).is_ok()
    }
}

/// Serialization utilities (requires "serde" feature)
#[cfg(feature = "serde")]
pub mod serde_utils {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json;

    /// Serialize a value to pretty JSON
    pub fn to_pretty_json<T: Serialize>(value: &T) -> Result<String> {
        serde_json::to_string_pretty(value).map_err(Into::into)
    }

    /// Serialize a value to compact JSON
    pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
        serde_json::to_string(value).map_err(Into::into)
    }

    /// Deserialize from JSON string
    pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T> {
        serde_json::from_str(json).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(common::clamp(5, 0, 10), 5);
        assert_eq!(common::clamp(-5, 0, 10), 0);
        assert_eq!(common::clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(string::truncate_string("Hello World", 5), "He...");
        assert_eq!(string::truncate_string("Hi", 5), "Hi");
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a long text that needs wrapping";
        let wrapped = string::wrap_text(text, 10);
        assert!(wrapped.len() > 1);
        assert!(wrapped.iter().all(|line| line.len() <= 10));
    }

    #[test]
    fn test_snake_camel_conversion() {
        assert_eq!(string::snake_to_camel("hello_world"), "helloWorld");
        assert_eq!(string::camel_to_snake("helloWorld"), "hello_world");
    }
}
