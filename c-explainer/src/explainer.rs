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

//! Convert ASTs to a human-readable explanations

use core::fmt::Write;

use alloc::{format, string::String};

use crate::ast::{Declaration, Declarator, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plurality {
    Singular,
    Plural,
}

/// Returns a new string with the noun articulated as "a" or "an" depending on its first letter.
fn articulate(dst: &mut String, noun: &str) {
    match noun.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => write!(dst, "an {noun}").unwrap(),
        Some(_) => write!(dst, "a {noun}").unwrap(),
        None => (),
    }
}

/// Naively returns the plural form of a noun.
fn make_plural(dst: &mut String, noun: &str) {
    match noun.chars().last() {
        Some('s' | 'x' | 'z') => write!(dst, "{noun}es").unwrap(),
        Some(_) => write!(dst, "{noun}s").unwrap(),
        None => (),
    }
}

#[must_use]
pub fn explain_declaration(decl: &Declaration) -> String {
    let (mut explanation, plurality) = explain_declarator(&decl.declarator);
    match (decl.base_type, plurality) {
        (Type::Primitive(ty), Plurality::Singular) => articulate(&mut explanation, ty.as_ref()),
        (Type::Primitive(ty), Plurality::Plural) => make_plural(&mut explanation, ty.as_ref()),
        (Type::Record(kind, name), _) => write!(&mut explanation, "{kind} {name}").unwrap(),
    }
    explanation
}

#[must_use]
fn explain_declarator(declarator: &Declarator) -> (String, Plurality) {
    match declarator {
        Declarator::Anonymous => (String::new(), Plurality::Singular),
        Declarator::Ident(name) => (format!("\"{name}\", "), Plurality::Singular),
        Declarator::Ptr(inner) => {
            let (mut explanation, plurality) = explain_declarator(inner);
            explanation.push_str(match plurality {
                Plurality::Singular => "a pointer to ",
                Plurality::Plural => "pointers to ",
            });
            (explanation, plurality)
        }
        Declarator::Array(inner, len) => {
            let (mut explanation, plurality) = explain_declarator(inner);
            explanation.push_str(match plurality {
                Plurality::Singular => "an array of ",
                Plurality::Plural => "arrays of ",
            });
            if let Some(len) = len {
                write!(&mut explanation, "{len} ").unwrap();
            }
            (explanation, Plurality::Plural)
        }
        Declarator::Function { func, params } => {
            let (mut explanation, plurality) = explain_declarator(func);
            explanation.push_str("a function that takes ");
            if params.is_empty() {
                explanation.push_str("no parameters");
            } else {
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        if i == params.len() - 1 {
                            explanation.push_str(", and ");
                        } else {
                            explanation.push_str(", ");
                        }
                    }
                    let param_explanation = explain_declaration(param);
                    explanation.push_str(&param_explanation);
                }
            }
            explanation.push_str(" and returns ");
            (explanation, plurality)
        }
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;
    use pretty_assertions::assert_eq;

    use super::*;

    /// Parse the first argument and assert that its explanation matches the second argument.
    fn run(expression: &str, expected: &str) {
        let decl = crate::parser::parser().parse(expression).unwrap();
        let result = explain_declaration(&decl);
        assert_eq!(result, expected);
    }

    #[test]
    fn explain_primitive_var() {
        run("int x", "\"x\", an int");
    }

    /// Ensures "a" and "an" are used appropriately.
    #[test]
    fn test_articles() {
        run("int x", "\"x\", an int");
        run("signed int x", "\"x\", a signed int");
    }

    #[test]
    fn explain_ptr_to_primitive() {
        run("int *p", "\"p\", a pointer to an int");
    }

    #[test]
    fn explain_array_of_primitive() {
        run("int arr[]", "\"arr\", an array of ints");
    }

    #[test]
    fn explain_array_of_primitive_with_size() {
        run("int arr[10]", "\"arr\", an array of 10 ints");
    }

    #[test]
    fn explain_2d_array_of_primitive() {
        run(
            "int arr[10][20]",
            "\"arr\", an array of 10 arrays of 20 ints",
        );
    }

    #[test]
    fn explain_nested_ptrs() {
        run(
            "char ***p",
            "\"p\", a pointer to a pointer to a pointer to a char",
        );
    }

    #[test]
    fn explain_array_of_ptrs() {
        run("int *arr[10]", "\"arr\", an array of 10 pointers to ints");
    }

    #[test]
    fn explain_ptr_to_array() {
        run("int (*p)[10]", "\"p\", a pointer to an array of 10 ints");
    }

    #[test]
    fn explain_function_with_no_params() {
        run(
            "void func()",
            "\"func\", a function that takes no parameters and returns a void",
        );
    }
}
