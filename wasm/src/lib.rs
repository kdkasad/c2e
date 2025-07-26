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

use c2e::{chumsky::Parser, explainer::explain_declaration};
use wasm_bindgen::prelude::*;

/// Explain the given C source code declaration.
#[wasm_bindgen]
pub fn explain(src: &str) -> Result<String, Vec<String>> {
    c2e::parser::parser()
        .parse(src)
        .into_result()
        .map(|decl| explain_declaration(&decl))
        .map_err(|errs| errs.into_iter().map(|err| err.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn explain_success() {
        let output = explain("int main()").unwrap();
        assert_eq!(
            output,
            "a function named main that takes no parameters and returns an int"
        );
    }

    #[test]
    fn explain_error() {
        let output = explain("int main(");
        let errors = output.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("expected"));
    }
}
