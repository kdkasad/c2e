/*
 * This program is free software: you can redistribute it and/or modify it under the terms of
 * the GNU General Public License as published by the Free Software Foundation, either version
 * 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
 * without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program. If
 * not, see <https://www.gnu.org/licenses/>.
 */

//! JS bindings for [`c2e`].

use std::fmt::Write;

use c2e::{ast::Declaration, chumsky::Parser};
use fmt::HtmlFormatter;
use wasm_bindgen::prelude::*;

mod fmt;

/// Explain the given C source code declaration.
#[wasm_bindgen]
pub fn explain(formatter: &HtmlFormatter, src: &str) -> Result<String, Vec<String>> {
    c2e::parser::parser()
        .parse(src)
        .into_result()
        .map(|decls| explain_declarations(formatter, &decls))
        .map_err(|errs| errs.into_iter().map(|err| err.to_string()).collect())
}

fn explain_declarations(formatter: &HtmlFormatter, decls: &[Declaration<'_>]) -> String {
    match decls {
        [] => String::new(),
        [decl] => explain_to_html(formatter, decl),
        [decls @ .., last] => {
            let mut s = String::new();
            for decl in decls {
                write!(&mut s, "{};\n\n", explain_to_html(formatter, decl)).unwrap();
            }
            write!(&mut s, "{};", explain_to_html(formatter, last)).unwrap();
            s
        }
    }
}

fn explain_to_html(formatter: &HtmlFormatter, declaration: &Declaration<'_>) -> String {
    c2e::explainer::explain_declaration(declaration).format_to_string(formatter)
}

#[cfg(test)]
mod tests {
    use crate::fmt::ClassMapping;

    use super::*;

    use pretty_assertions::assert_eq;

    fn get_formatter() -> HtmlFormatter {
        let mapping = ClassMapping {
            qualifier: Some("q".to_string()),
            primitive_type: Some("p".to_string()),
            user_defined_type: Some("u".to_string()),
            identifier: Some("i".to_string()),
            number: Some("n".to_string()),
        };
        HtmlFormatter::new(mapping)
    }

    #[test]
    fn explain_empty() {
        let output = explain(&get_formatter(), "").unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn explain_success_single() {
        let output = explain(&get_formatter(), "int main()").unwrap();
        assert_eq!(
            output,
            r#"a function named <span class="i">main</span> that takes no parameters and returns an <span class="p">int</span>"#
        );
    }

    #[test]
    fn explain_success_multiple() {
        let output = explain(&get_formatter(), "int main(); int foo(int a);").unwrap();
        assert_eq!(
            output,
            r#"a function named <span class="i">main</span> that takes no parameters and returns an <span class="p">int</span>;

a function named <span class="i">foo</span> that takes (an <span class="p">int</span> named <span class="i">a</span>) and returns an <span class="p">int</span>;"#
        );
    }

    #[test]
    fn explain_error() {
        let output = explain(&get_formatter(), "int main(");
        let errors = output.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("expected"));
    }
}
