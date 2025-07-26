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

use alloc::{
    string::{String, ToString},
    vec,
};

use crate::{
    ast::{Declaration, Declarator, QualifiedType, Type},
    color::{Highlight, HighlightedText, HighlightedTextSegment},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plurality {
    Singular,
    Plural,
}

/// Returns the appropriate article ("a" or "an") for the given noun, followed by a space.
fn article_for(noun: &HighlightedTextSegment) -> &'static str {
    match noun.text.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an ",
        Some(_) => "a ",
        None => "",
    }
}

/// Naively returns the plural suffix for a noun.
fn plural_suffix_for(noun: &HighlightedTextSegment) -> &'static str {
    match noun.text.chars().last() {
        Some('s' | 'x' | 'z') => "es",
        Some(_) => "s",
        None => "",
    }
}

#[must_use]
pub fn explain_declaration(decl: &Declaration) -> HighlightedText {
    explain_declaration_impl(decl).msg
}

#[derive(Debug)]
struct Explanation {
    /// Name of the root identifier being explained
    identifier_name: Option<String>,
    /// String containing English explanation
    msg: HighlightedText,
    plurality: Plurality,
}

impl Explanation {
    fn new() -> Self {
        Self {
            identifier_name: None,
            msg: HighlightedText::new(),
            plurality: Plurality::Singular,
        }
    }

    /// Sets `identifier_name` to the given name.
    fn with_identifier_name(mut self, name: String) -> Self {
        self.identifier_name = Some(name);
        self
    }

    /// Clears `identifier_name`.
    fn unnamed(mut self) -> Self {
        self.identifier_name = None;
        self
    }

    /// Sets `plurality` to [`Plurality::Singular`].
    fn singular(mut self) -> Self {
        self.plurality = Plurality::Singular;
        self
    }

    /// Sets `plurality` to [`Plurality::Plural`].
    fn plural(mut self) -> Self {
        self.plurality = Plurality::Plural;
        self
    }
}

fn format_qualified_type(qt: &QualifiedType) -> HighlightedText {
    let highlight = match qt.1 {
        Type::Primitive(_) => Highlight::PrimitiveType,
        Type::Record(_, _) => Highlight::UserDefinedType,
    };
    let highlighted_unqualified_type = HighlightedTextSegment::new(qt.1.to_string(), highlight);

    if qt.0.is_empty() {
        vec![highlighted_unqualified_type]
    } else {
        let qualifiers = qt.0.to_string();
        vec![
            HighlightedTextSegment::new(qualifiers, Highlight::Qualifier),
            HighlightedTextSegment::new(" ", Highlight::None),
            highlighted_unqualified_type,
        ]
    }
    .into()
}

fn explain_declaration_impl(decl: &Declaration) -> Explanation {
    let mut explanation = explain_declarator(&decl.declarator);
    let highlighted_type = format_qualified_type(&decl.base_type);
    match explanation.plurality {
        Plurality::Singular => {
            let article = article_for(&highlighted_type[0]);
            explanation.msg.push_str(article);
            explanation.msg.extend(highlighted_type.0);
        }
        Plurality::Plural => {
            let suffix = plural_suffix_for(highlighted_type.last().unwrap());
            explanation.msg.extend(highlighted_type.0);
            explanation.msg.push_str(suffix);
        }
    }
    if let Some(name) = &explanation.identifier_name {
        explanation.msg.push_str(" named ");
        explanation
            .msg
            .push(HighlightedTextSegment::new(name, Highlight::Ident));
    }
    explanation
}

#[allow(clippy::too_many_lines)]
#[must_use]
fn explain_declarator(declarator: &Declarator) -> Explanation {
    match declarator {
        Declarator::Anonymous => Explanation::new(),
        Declarator::Ident(name) => Explanation::new().with_identifier_name((*name).to_string()),
        Declarator::Ptr(inner, qualifiers) => {
            let mut sub = explain_declarator(inner);
            let qualifiers_text = if qualifiers.is_empty() {
                None
            } else {
                Some(HighlightedTextSegment::new(
                    qualifiers.to_string(),
                    Highlight::Qualifier,
                ))
            };
            match sub.plurality {
                Plurality::Singular => {
                    sub.msg.push_str("a ");
                    if let Some(qualifiers_text) = qualifiers_text {
                        sub.msg.push(qualifiers_text);
                        sub.msg.push_str(" ");
                    }
                    sub.msg.push_str("pointer ");
                }
                Plurality::Plural => {
                    if let Some(qualifiers_text) = qualifiers_text {
                        sub.msg.push(qualifiers_text);
                        sub.msg.push_str(" ");
                    }
                    sub.msg.push_str("pointers ");
                }
            }
            if let Some(name) = &sub.identifier_name {
                sub.msg.push_str("named ");
                sub.msg
                    .push(HighlightedTextSegment::new(name, Highlight::Ident));
                sub.msg.push_str(" ");
            }
            sub.msg.push_str("to ");
            sub.unnamed()
        }
        Declarator::Array(inner, len) => {
            let mut sub = explain_declarator(inner);
            sub.msg.push_str(match sub.plurality {
                Plurality::Singular => "an array",
                Plurality::Plural => "arrays",
            });
            if let Some(name) = &sub.identifier_name {
                sub.msg.push_str(" named ");
                sub.msg
                    .push(HighlightedTextSegment::new(name, Highlight::Ident));
            }
            sub.msg.push_str(" of ");
            if let Some(len) = len {
                sub.msg.push(HighlightedTextSegment::new(
                    len.to_string(),
                    Highlight::Number,
                ));
                sub.msg.push_str(" ");
            }
            sub.unnamed().plural()
        }
        Declarator::Function { func, params } => {
            let mut sub = explain_declarator(func);
            match (&sub.identifier_name, sub.plurality) {
                (None, Plurality::Singular) => sub.msg.push_str("a function that takes "),
                (None, Plurality::Plural) => sub.msg.push_str("functions that take "),
                (Some(name), Plurality::Singular) => {
                    sub.msg.push_str("a function named ");
                    sub.msg
                        .push(HighlightedTextSegment::new(name, Highlight::Ident));
                    sub.msg.push_str(" that takes ");
                }
                (Some(_), Plurality::Plural) => unreachable!("an identifier cannot be plural"),
            }
            match &params[..] {
                [] => sub.msg.push_str("no parameters"),
                [param] => {
                    sub.msg.push_str("(");
                    sub.msg.extend(explain_declaration(param).0);
                    sub.msg.push_str(")");
                }
                [a, b] => {
                    sub.msg.push_str("(");
                    sub.msg.extend(explain_declaration(a).0);
                    sub.msg.push_str(" and ");
                    sub.msg.extend(explain_declaration(b).0);
                    sub.msg.push_str(")");
                }
                [rest @ .., last] => {
                    sub.msg.push_str("(");
                    for param in rest {
                        sub.msg.extend(explain_declaration(param).0);
                        sub.msg.push_str(", ");
                    }
                    sub.msg.push_str("and ");
                    sub.msg.extend(explain_declaration(last).0);
                    sub.msg.push_str(")");
                }
            }
            sub.msg.push_str(match sub.plurality {
                Plurality::Singular => " and returns ",
                Plurality::Plural => " and return ",
            });
            sub.unnamed().singular()
        }
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;
    use pretty_assertions::assert_eq;

    use super::*;

    /// Parse the first argument and assert that its explanation matches the second argument.
    fn run(expression: &str, expected: &[HighlightedTextSegment]) {
        let decls = crate::parser::parser().parse(expression).unwrap();
        assert_eq!(
            decls.len(),
            1,
            "Expected exactly one declaration for input {expression}"
        );
        let result = explain_declaration(&decls[0]);
        assert_eq!(
            &result.coalesced().0,
            expected,
            "Wrong output for input {expression}"
        );
    }

    #[test]
    fn explain_primitive_var() {
        // run("int x", "an int named x");
        run(
            "int x",
            &[
                HighlightedTextSegment::new("an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("x", Highlight::Ident),
            ],
        );
    }

    /// Ensures "a" and "an" are used appropriately.
    #[test]
    fn test_articles() {
        run(
            "int x",
            &[
                HighlightedTextSegment::new("an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("x", Highlight::Ident),
            ],
        );
        run(
            "signed int x",
            &[
                HighlightedTextSegment::new("a ", Highlight::None),
                HighlightedTextSegment::new("signed int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("x", Highlight::Ident),
            ],
        );
    }

    #[test]
    fn test_article_for() {
        assert_eq!(article_for(&"int".into()), "an ");
        assert_eq!(article_for(&"cow".into()), "a ");
        assert_eq!(article_for(&"".into()), "");
    }

    #[test]
    fn test_make_plural() {
        assert_eq!(plural_suffix_for(&"cat".into()), "s");
        assert_eq!(plural_suffix_for(&"box".into()), "es");
        assert_eq!(plural_suffix_for(&"int".into()), "s");
        assert_eq!(plural_suffix_for(&"".into()), "");
    }

    #[test]
    fn explain_ptr_to_primitive() {
        run(
            "int *p",
            &[
                HighlightedTextSegment::new("a pointer named ", Highlight::None),
                HighlightedTextSegment::new("p", Highlight::Ident),
                HighlightedTextSegment::new(" to an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_array_of_primitive() {
        run(
            "int arr[]",
            &[
                HighlightedTextSegment::new("an array named ", Highlight::None),
                HighlightedTextSegment::new("arr", Highlight::Ident),
                HighlightedTextSegment::new(" of ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new("s", Highlight::None),
            ],
        );
    }

    #[test]
    fn explain_array_of_primitive_with_size() {
        // run("int arr[10]", "an array named arr of 10 ints");
        run(
            "int arr[10]",
            &[
                HighlightedTextSegment::new("an array named ", Highlight::None),
                HighlightedTextSegment::new("arr", Highlight::Ident),
                HighlightedTextSegment::new(" of ", Highlight::None),
                HighlightedTextSegment::new("10", Highlight::Number),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new("s", Highlight::None),
            ],
        );
    }

    #[test]
    fn explain_2d_array_of_primitive() {
        run(
            "int arr[10][20]",
            &[
                HighlightedTextSegment::new("an array named ", Highlight::None),
                HighlightedTextSegment::new("arr", Highlight::Ident),
                HighlightedTextSegment::new(" of ", Highlight::None),
                HighlightedTextSegment::new("10", Highlight::Number),
                HighlightedTextSegment::new(" arrays of ", Highlight::None),
                HighlightedTextSegment::new("20", Highlight::Number),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new("s", Highlight::None),
            ],
        );
    }

    #[test]
    fn explain_nested_ptrs() {
        run(
            "char ***p",
            &[
                HighlightedTextSegment::new("a pointer named ", Highlight::None),
                HighlightedTextSegment::new("p", Highlight::Ident),
                HighlightedTextSegment::new(" to a pointer to a pointer to a ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_array_of_ptrs() {
        run(
            "int *arr[10]",
            &[
                HighlightedTextSegment::new("an array named ", Highlight::None),
                HighlightedTextSegment::new("arr", Highlight::Ident),
                HighlightedTextSegment::new(" of ", Highlight::None),
                HighlightedTextSegment::new("10", Highlight::Number),
                HighlightedTextSegment::new(" pointers to ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new("s", Highlight::None),
            ],
        );
    }

    #[test]
    fn explain_ptr_to_array() {
        run(
            "int (*p)[10]",
            &[
                HighlightedTextSegment::new("a pointer named ", Highlight::None),
                HighlightedTextSegment::new("p", Highlight::Ident),
                HighlightedTextSegment::new(" to an array of ", Highlight::None),
                HighlightedTextSegment::new("10", Highlight::Number),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new("s", Highlight::None),
            ],
        );
    }

    #[test]
    fn explain_function_with_no_params() {
        // run(
        //     "void func()",
        //     "a function named func that takes no parameters and returns a void",
        // );
        run(
            "void func()",
            &[
                HighlightedTextSegment::new("a function named ", Highlight::None),
                HighlightedTextSegment::new("func", Highlight::Ident),
                HighlightedTextSegment::new(
                    " that takes no parameters and returns a ",
                    Highlight::None,
                ),
                HighlightedTextSegment::new("void", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_array_of_functions() {
        run(
            "char *(*(*bar)[5])(int)",
            &[
                HighlightedTextSegment::new("a pointer named ", Highlight::None),
                HighlightedTextSegment::new("bar", Highlight::Ident),
                HighlightedTextSegment::new(" to an array of ", Highlight::None),
                HighlightedTextSegment::new("5", Highlight::Number),
                HighlightedTextSegment::new(
                    " pointers to functions that take (an ",
                    Highlight::None,
                ),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(") and return a pointer to a ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_qualifiers() {
        run(
            "const int x",
            &[
                HighlightedTextSegment::new("a ", Highlight::None),
                HighlightedTextSegment::new("const", Highlight::Qualifier),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("x", Highlight::Ident),
            ],
        );
        run(
            "volatile int x",
            &[
                HighlightedTextSegment::new("a ", Highlight::None),
                HighlightedTextSegment::new("volatile", Highlight::Qualifier),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("x", Highlight::Ident),
            ],
        );
        run(
            "int *const restrict x",
            &[
                HighlightedTextSegment::new("a ", Highlight::None),
                HighlightedTextSegment::new("const restrict", Highlight::Qualifier),
                HighlightedTextSegment::new(" pointer named ", Highlight::None),
                HighlightedTextSegment::new("x", Highlight::Ident),
                HighlightedTextSegment::new(" to an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
            ],
        );
        run(
            "const char *const str",
            &[
                HighlightedTextSegment::new("a ", Highlight::None),
                HighlightedTextSegment::new("const", Highlight::Qualifier),
                HighlightedTextSegment::new(" pointer named ", Highlight::None),
                HighlightedTextSegment::new("str", Highlight::Ident),
                HighlightedTextSegment::new(" to a ", Highlight::None),
                HighlightedTextSegment::new("const", Highlight::Qualifier),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_struct_var() {
        run(
            "struct point p",
            &[
                HighlightedTextSegment::new("a ", Highlight::None),
                HighlightedTextSegment::new("struct point", Highlight::UserDefinedType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("p", Highlight::Ident),
            ],
        );
    }

    #[test]
    fn explain_function_one_unnamed_param() {
        run(
            "int foo(const char *)",
            &[
                HighlightedTextSegment::new("a function named ", Highlight::None),
                HighlightedTextSegment::new("foo", Highlight::Ident),
                HighlightedTextSegment::new(" that takes (a pointer to a ", Highlight::None),
                HighlightedTextSegment::new("const", Highlight::Qualifier),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
                HighlightedTextSegment::new(") and returns an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_function_one_named_param() {
        run(
            "int foo(const char *bar)",
            &[
                HighlightedTextSegment::new("a function named ", Highlight::None),
                HighlightedTextSegment::new("foo", Highlight::Ident),
                HighlightedTextSegment::new(" that takes (a pointer named ", Highlight::None),
                HighlightedTextSegment::new("bar", Highlight::Ident),
                HighlightedTextSegment::new(" to a ", Highlight::None),
                HighlightedTextSegment::new("const", Highlight::Qualifier),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
                HighlightedTextSegment::new(") and returns an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_anonymous_function() {
        run(
            "int (*)(const char *)",
            &[
                HighlightedTextSegment::new(
                    "a pointer to a function that takes (a pointer to a ",
                    Highlight::None,
                ),
                HighlightedTextSegment::new("const", Highlight::Qualifier),
                HighlightedTextSegment::new(" ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
                HighlightedTextSegment::new(") and returns an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_function_two_params() {
        run(
            "int add(int a, int b)",
            &[
                HighlightedTextSegment::new("a function named ", Highlight::None),
                HighlightedTextSegment::new("add", Highlight::Ident),
                HighlightedTextSegment::new(" that takes (an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("a", Highlight::Ident),
                HighlightedTextSegment::new(" and an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("b", Highlight::Ident),
                HighlightedTextSegment::new(") and returns an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_function_three_params() {
        run(
            "void print(int a, char *b, float c)",
            &[
                HighlightedTextSegment::new("a function named ", Highlight::None),
                HighlightedTextSegment::new("print", Highlight::Ident),
                HighlightedTextSegment::new(" that takes (an ", Highlight::None),
                HighlightedTextSegment::new("int", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("a", Highlight::Ident),
                HighlightedTextSegment::new(", a pointer named ", Highlight::None),
                HighlightedTextSegment::new("b", Highlight::Ident),
                HighlightedTextSegment::new(" to a ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
                HighlightedTextSegment::new(", and a ", Highlight::None),
                HighlightedTextSegment::new("float", Highlight::PrimitiveType),
                HighlightedTextSegment::new(" named ", Highlight::None),
                HighlightedTextSegment::new("c", Highlight::Ident),
                HighlightedTextSegment::new(") and returns a ", Highlight::None),
                HighlightedTextSegment::new("void", Highlight::PrimitiveType),
            ],
        );
    }

    #[test]
    fn explain_array_of_struct() {
        run(
            "struct point p[]",
            &[
                HighlightedTextSegment::new("an array named ", Highlight::None),
                HighlightedTextSegment::new("p", Highlight::Ident),
                HighlightedTextSegment::new(" of ", Highlight::None),
                HighlightedTextSegment::new("struct point", Highlight::UserDefinedType),
                HighlightedTextSegment::new("s", Highlight::None),
            ],
        );
    }

    #[test]
    fn explain_plural_qualifiers() {
        run(
            "char *const p[]",
            &[
                HighlightedTextSegment::new("an array named ", Highlight::None),
                HighlightedTextSegment::new("p", Highlight::Ident),
                HighlightedTextSegment::new(" of ", Highlight::None),
                HighlightedTextSegment::new("const", Highlight::Qualifier),
                HighlightedTextSegment::new(" pointers to ", Highlight::None),
                HighlightedTextSegment::new("char", Highlight::PrimitiveType),
                HighlightedTextSegment::new("s", Highlight::None),
            ],
        );
    }
}
