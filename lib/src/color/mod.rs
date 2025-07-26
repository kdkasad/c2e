use core::ops::{Deref, DerefMut};

use alloc::{string::String, vec::Vec};
use fmt::Formatter;

pub mod fmt;

/// Defines types of highlights that can be applied to parts of the explanation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Highlight {
    /// No highlight
    None,
    /// Highlight a type/storage class qualifier
    Qualifier,
    /// Highlight a primitive type
    PrimitiveType,
    /// Highlight a user-defined type, like a struct or typedef
    UserDefinedType,
    /// Highlight a user-defined type
    Ident,
    /// Highlight a number literal
    Number,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedTextSegment {
    pub text: String,
    pub highlight: Highlight,
}

/// Represents a piece of text with a single highlight type.
impl HighlightedTextSegment {
    /// Creates a new `HighlightedText` instance.
    #[must_use]
    pub fn new(text: impl Into<String>, highlight: Highlight) -> Self {
        Self {
            text: text.into(),
            highlight,
        }
    }
}

impl<T: Into<String>> From<T> for HighlightedTextSegment {
    /// Converts a `String` into a `HighlightedText` with no highlight.
    fn from(text: T) -> Self {
        Self::new(text.into(), Highlight::None)
    }
}

/// Represents a piece of text made up of multiple segments, each with its own highlight type.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HighlightedText(pub Vec<HighlightedTextSegment>);

impl Deref for HighlightedText {
    type Target = Vec<HighlightedTextSegment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HighlightedText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<HighlightedTextSegment>> for HighlightedText {
    /// Converts a `Vec<HighlightedTextSegment>` into a `HighlightedText`.
    fn from(segments: Vec<HighlightedTextSegment>) -> Self {
        Self(segments)
    }
}

impl From<String> for HighlightedText {
    /// Converts a `String` into a `HighlightedText` with no highlight.
    fn from(text: String) -> Self {
        Self(alloc::vec![HighlightedTextSegment::from(text)])
    }
}

impl HighlightedText {
    /// Creates a new empty [`HighlightedText`] instance.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Pushes the given string as a new segment with [`Highlight::None`].
    /// If the last existing segment has the same highlight, it appends to that segment instead of
    /// creating a new one.
    pub fn push_str(&mut self, text: &str) {
        if let Some(last) = self.0.last_mut()
            && last.highlight == Highlight::None
        {
            last.text.push_str(text);
        } else {
            self.push(HighlightedTextSegment::new(text, Highlight::None));
        }
    }

    /// Formats the highlighted text using the provided formatter, returning a string.
    ///
    /// # Panics
    ///
    /// Panics if the formatter fails to write to the string.
    #[must_use]
    pub fn format_to_string(&self, formatter: &impl Formatter) -> String {
        let mut output = String::new();
        formatter.format(&mut output, self).unwrap();
        output
    }

    // Returns a new [`HighlightedText`] where consecutive segments with the same highlight type
    // are coalesced into a single segment.
    #[cfg(test)]
    pub(crate) fn coalesced(self) -> Self {
        let mut coalesced: Vec<HighlightedTextSegment> = Vec::new();
        for segment in self.0 {
            if let Some(last) = coalesced.last_mut() {
                if last.highlight == segment.highlight {
                    last.text.push_str(&segment.text);
                } else {
                    coalesced.push(segment);
                }
            } else {
                coalesced.push(segment);
            }
        }
        Self(coalesced)
    }
}

#[cfg(test)]
mod tests {
    use super::fmt::PlainFormatter;
    use super::*;

    use alloc::vec;
    use pretty_assertions::assert_eq;

    #[test]
    fn segment_new() {
        let segment = HighlightedTextSegment::new("Hello, world!", Highlight::Ident);
        assert_eq!(segment.text, "Hello, world!");
        assert_eq!(segment.highlight, Highlight::Ident);
    }

    #[test]
    fn segment_from_string() {
        let segment: HighlightedTextSegment = String::from("hello").into();
        assert_eq!(segment.text, "hello");
        assert_eq!(segment.highlight, Highlight::None);
    }

    #[test]
    fn text_new() {
        let text = HighlightedText::new();
        assert_eq!(text.0, Vec::new());
    }

    #[test]
    fn text_getters() {
        let text = HighlightedText::from(vec![
            HighlightedTextSegment::new("Hello, ", Highlight::Ident),
            HighlightedTextSegment::new("world!", Highlight::PrimitiveType),
        ]);
        assert_eq!(text.len(), 2);
        assert_eq!(text[0].text, "Hello, ");
        assert_eq!(text[1].text, "world!");
    }

    #[test]
    fn format_to_string() {
        let text = HighlightedText::from(vec![
            HighlightedTextSegment::new("Hello, ", Highlight::Ident),
            HighlightedTextSegment::new("world!", Highlight::PrimitiveType),
        ]);
        let formatter = PlainFormatter::new();
        let s = text.format_to_string(&formatter);
        assert_eq!(s, "Hello, world!");
    }

    #[test]
    fn text_from_string() {
        let mut text: HighlightedText = String::from("this is an ").into();
        // Create a string so we have a non-static lifetime.
        let ty = String::from("int");
        text.push(HighlightedTextSegment::new(&ty, Highlight::PrimitiveType));
        text.push(HighlightedTextSegment::new(" named ", Highlight::None));
        text.push(HighlightedTextSegment::new("foo", Highlight::Ident));
        assert_eq!(
            text.format_to_string(&PlainFormatter::new()),
            "this is an int named foo"
        );
    }
}
