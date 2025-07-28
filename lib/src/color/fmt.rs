//! Utilities for formatting highlighted text.

use super::HighlightedText;

pub trait Formatter {
    /// Formats the given [`HighlightedText`] into a destination writer.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the destination fails.
    fn format(&self, dst: &mut impl core::fmt::Write, text: &HighlightedText) -> core::fmt::Result;
}

/// Formatter which discards all formatting and returns plain text.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainFormatter;

impl PlainFormatter {
    /// Creates a new `PlainFormatter`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl super::Formatter for PlainFormatter {
    /// Formats the given [`HighlightedText`] into a destination writer.
    ///
    /// This implementation discards all formatting and writes only the plain text.
    fn format(
        &self,
        dst: &mut impl core::fmt::Write,
        text: &super::HighlightedText,
    ) -> core::fmt::Result {
        text.iter()
            .try_for_each(|segment| dst.write_str(&segment.text))
    }
}
