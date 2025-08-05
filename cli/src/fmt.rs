//! Formatter for printing highlighted text to a terminal.

use c2e::color::{Highlight, HighlightedText};
use termcolor::Color;

#[derive(Debug, Clone, Copy)]
pub struct ColorMap {
    pub qualifier: Color,
    pub primitive_type: Color,
    pub user_defined_type: Color,
    pub identifier: Color,
    pub number: Color,
    pub quasi_keyword: Color,
}

impl ColorMap {
    /// Returns the [`Color`] for the given [`Highlight`] according to this color map.
    pub fn color_for_highlight(&self, highlight: Highlight) -> Option<Color> {
        match highlight {
            Highlight::Qualifier => Some(self.qualifier),
            Highlight::PrimitiveType => Some(self.primitive_type),
            Highlight::UserDefinedType => Some(self.user_defined_type),
            Highlight::Ident => Some(self.identifier),
            Highlight::Number => Some(self.number),
            Highlight::QuasiKeyword => Some(self.quasi_keyword),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CliFormatter {
    colors: ColorMap,
}

impl CliFormatter {
    /// Creates a new [`CliFormatter`] with the given color mapping.
    #[must_use]
    pub const fn new(colors: ColorMap) -> Self {
        Self { colors }
    }

    /// Writes the given highlighted text to the destination writer, applying colors based on the
    /// highlight type according to this formatter's color map.
    pub fn format(
        &self,
        dst: &mut impl termcolor::WriteColor,
        text: HighlightedText,
    ) -> std::io::Result<()> {
        for segment in text
            .0
            .into_iter()
            .filter(|segment| !segment.text.is_empty())
        {
            if let Some(color) = self.colors.color_for_highlight(segment.highlight) {
                dst.set_color(termcolor::ColorSpec::new().set_fg(Some(color)))?;
            }
            write!(dst, "{}", segment.text)?;
            dst.reset()?;
        }
        Ok(())
    }
}
