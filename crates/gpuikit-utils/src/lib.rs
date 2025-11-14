//! Common utilities and helpers for GPUI applications
//!
//! This crate provides a collection of utility functions and helpers
//! commonly needed when building GPUI applications.

use gpui::{point, Bounds, Pixels, Point, Size};

/// Geometry utilities
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
}

/// String manipulation utilities
pub mod string {
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
}

/// Task and async utilities
pub mod task {
    use std::sync::{Arc, Mutex};
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

// Re-export commonly used utilities
pub use geometry::{center_rect, expand_bounds, point_in_bounds};
pub use string::{truncate_string, wrap_text};
pub use task::{debounce, throttle};

#[cfg(test)]
mod tests {
    use super::*;

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
}
