use c2e::color::{Highlight, HighlightedText, fmt::Formatter};
use wasm_bindgen::prelude::wasm_bindgen;

/// Data structure which maps [`Highlight`]s to class names.
#[derive(Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct ClassMapping {
    pub qualifier: Option<String>,
    pub primitive_type: Option<String>,
    pub user_defined_type: Option<String>,
    pub identifier: Option<String>,
    pub number: Option<String>,
    pub quasi_keyword: Option<String>,
}

#[wasm_bindgen]
impl ClassMapping {
    #[wasm_bindgen(constructor)]
    pub fn new(
        qualifier: Option<String>,
        primitive_type: Option<String>,
        user_defined_type: Option<String>,
        identifier: Option<String>,
        number: Option<String>,
        quasi_keyword: Option<String>,
    ) -> Self {
        Self {
            qualifier,
            primitive_type,
            user_defined_type,
            identifier,
            number,
            quasi_keyword,
        }
    }
}

/// Formatter which formats [`HighlightedText`] into HTML, using `<span>` elements with classes for
/// styling.
///
/// Text with [`Highlight::None`] will not be wrapped in a `<span>` element. Text with other
/// highlights will be wrapped in a `<span>` element with a class corresponding to the highlight
/// type according to this formatter's `class_mapping`. If the class mapping contains `None`, the
/// text will not be wrapped in a `<span>` element.
#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct HtmlFormatter {
    colors: ClassMapping,
}

#[wasm_bindgen]
impl HtmlFormatter {
    /// Creates a new boxed formatter with the given class mapping.
    #[wasm_bindgen(constructor)]
    pub fn new(colors: ClassMapping) -> Self {
        Self { colors }
    }
}

impl Formatter for HtmlFormatter {
    fn format(&self, dst: &mut impl core::fmt::Write, text: &HighlightedText) -> core::fmt::Result {
        text.0
            .iter()
            .filter(|segment| !segment.text.is_empty())
            .try_for_each(|segment| {
                let class = match segment.highlight {
                    Highlight::Qualifier => self.colors.qualifier.as_deref(),
                    Highlight::PrimitiveType => self.colors.primitive_type.as_deref(),
                    Highlight::UserDefinedType => self.colors.user_defined_type.as_deref(),
                    Highlight::Ident => self.colors.identifier.as_deref(),
                    Highlight::Number => self.colors.number.as_deref(),
                    Highlight::QuasiKeyword => self.colors.quasi_keyword.as_deref(),
                    _ => None,
                };

                if let Some(class_name) = class {
                    write!(
                        dst,
                        r#"<span class="{}">{}</span>"#,
                        html_escape::encode_quoted_attribute(class_name),
                        html_escape::encode_text(&segment.text)
                    )
                } else {
                    write!(dst, "{}", html_escape::encode_text(&segment.text))
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use c2e::color::HighlightedTextSegment;

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_html_formatter() {
        let formatter = HtmlFormatter::new(ClassMapping::new(
            Some("qualifier".to_string()),
            Some("primitive-type".to_string()),
            Some("user-defined-type".to_string()),
            None,
            Some("number".to_string()),
            Some("quasi".to_string()),
        ));

        let text = HighlightedText(vec![
            HighlightedTextSegment::new("pt", Highlight::PrimitiveType),
            HighlightedTextSegment::new("\n", Highlight::None),
            HighlightedTextSegment::new("id", Highlight::Ident),
            HighlightedTextSegment::new("\n", Highlight::None),
            HighlightedTextSegment::new("tq", Highlight::Qualifier),
            HighlightedTextSegment::new("\n", Highlight::None),
            HighlightedTextSegment::new("10", Highlight::Number),
            HighlightedTextSegment::new("\n", Highlight::None),
            HighlightedTextSegment::new("udt", Highlight::UserDefinedType),
            HighlightedTextSegment::new("", Highlight::None),
            HighlightedTextSegment::new("\n", Highlight::None),
            HighlightedTextSegment::new("lksjdf", Highlight::QuasiKeyword),
            HighlightedTextSegment::new("\n", Highlight::None),
        ]);

        let mut output = String::new();
        formatter.format(&mut output, &text).unwrap();

        assert_eq!(
            output,
            r#"<span class="primitive-type">pt</span>
id
<span class="qualifier">tq</span>
<span class="number">10</span>
<span class="user-defined-type">udt</span>
<span class="quasi">lksjdf</span>
"#
        );
    }
}
