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
    ast::{Declaration, Declarator, QualifiedType, Type, TypeQualifier},
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
    if decl.base_type.0.contains(TypeQualifier::Typedef) {
        explain_typedef(decl)
    } else {
        explain_declaration_impl(decl)
    }
    .msg
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
        Type::Record(_, _) | Type::Custom(_) => Highlight::UserDefinedType,
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
    let mut explanation = explain_declarator(&decl.declarator, false);
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

/// Explains a declaration whose `base_type` contains a [`typedef` qualifier][TypeQualifier::Typedef].
///
/// # Panics
///
/// Panics if the declaration's `base_type` does not contain a
/// [`typedef` qualifier][TypeQualifier::Typedef].
fn explain_typedef(decl: &Declaration) -> Explanation {
    assert!(decl.base_type.0.contains(TypeQualifier::Typedef));

    let mut new_type = decl.base_type;
    new_type.0.remove(TypeQualifier::Typedef);
    let type_str = format_qualified_type(&new_type);

    let mut explanation = Explanation::new();
    explanation.msg.push_str("a type");

    let declarator_explanation = explain_declarator(&decl.declarator, true);

    if let Some(name) = declarator_explanation.identifier_name {
        explanation.msg.push_str(" named ");
        explanation.msg.push(HighlightedTextSegment::new(
            name,
            Highlight::UserDefinedType,
        ));
    }

    explanation.msg.push_str(" defined as ");
    explanation.msg.extend(declarator_explanation.msg.0);

    match declarator_explanation.plurality {
        Plurality::Singular => {
            let article = article_for(&type_str[0]);
            explanation.msg.push_str(article);
            explanation.msg.extend(type_str.0);
        }
        Plurality::Plural => {
            let suffix = plural_suffix_for(type_str.last().unwrap());
            explanation.msg.extend(type_str.0);
            explanation.msg.push_str(suffix);
        }
    }

    explanation
}

#[allow(clippy::too_many_lines)]
#[must_use]
fn explain_declarator(declarator: &Declarator, skip_name: bool) -> Explanation {
    match declarator {
        Declarator::Anonymous => Explanation::new(),
        Declarator::Ident(name) => Explanation::new().with_identifier_name((*name).to_string()),
        Declarator::Ptr(inner, qualifiers) => {
            let mut sub = explain_declarator(inner, skip_name);
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
                    sub.msg.push(HighlightedTextSegment::new(
                        "pointer",
                        Highlight::QuasiKeyword,
                    ));
                }
                Plurality::Plural => {
                    if let Some(qualifiers_text) = qualifiers_text {
                        sub.msg.push(qualifiers_text);
                        sub.msg.push_str(" ");
                    }
                    sub.msg.push(HighlightedTextSegment::new(
                        "pointers",
                        Highlight::QuasiKeyword,
                    ));
                }
            }
            sub.msg.push_str(" ");
            if let Some(name) = &sub.identifier_name
                && !skip_name
            {
                sub.msg.push_str("named ");
                sub.msg
                    .push(HighlightedTextSegment::new(name, Highlight::Ident));
                sub.msg.push_str(" ");
                sub.identifier_name = None;
            }
            sub.msg.push_str("to ");
            sub
        }
        Declarator::Array(inner, len) => {
            let mut sub = explain_declarator(inner, skip_name);
            match sub.plurality {
                Plurality::Singular => {
                    sub.msg.push_str("an ");
                    sub.msg.push(HighlightedTextSegment::new(
                        "array",
                        Highlight::QuasiKeyword,
                    ));
                }
                Plurality::Plural => {
                    sub.msg.push(HighlightedTextSegment::new(
                        "arrays",
                        Highlight::QuasiKeyword,
                    ));
                }
            }
            // sub.msg.push_str(match sub.plurality {
            //     Plurality::Singular => "an array",
            //     Plurality::Plural => "arrays",
            // });
            if let Some(name) = &sub.identifier_name
                && !skip_name
            {
                sub.msg.push_str(" named ");
                sub.msg
                    .push(HighlightedTextSegment::new(name, Highlight::Ident));
                sub.identifier_name = None;
            }
            sub.msg.push_str(" of ");
            if let Some(len) = len {
                sub.msg.push(HighlightedTextSegment::new(
                    len.to_string(),
                    Highlight::Number,
                ));
                sub.msg.push_str(" ");
            }
            sub.plural()
        }
        Declarator::Function { func, params } => {
            let mut sub = explain_declarator(func, skip_name);
            let name = if skip_name {
                &None
            } else {
                &sub.identifier_name
            };
            match (name, sub.plurality) {
                (None, Plurality::Singular) => {
                    sub.msg.push_str("a ");
                    sub.msg.push(HighlightedTextSegment::new(
                        "function",
                        Highlight::QuasiKeyword,
                    ));
                    sub.msg.push_str(" that takes ");
                }
                (None, Plurality::Plural) => {
                    sub.msg.push(HighlightedTextSegment::new(
                        "functions",
                        Highlight::QuasiKeyword,
                    ));
                    sub.msg.push_str(" that take ");
                }
                (Some(name), Plurality::Singular) => {
                    sub.msg.push_str("a ");
                    sub.msg.push(HighlightedTextSegment::new(
                        "function",
                        Highlight::QuasiKeyword,
                    ));
                    sub.msg.push_str(" named ");
                    sub.msg
                        .push(HighlightedTextSegment::new(name, Highlight::Ident));
                    sub.msg.push_str(" that takes ");
                    sub.identifier_name = None;
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
            sub.singular()
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

    macro_rules! hltext {
        ( $($text:literal $hl:ident)+ ) => {
            &[
                $(hltext!(line $text $hl)),+
            ]
        };
        ( line $text:literal n ) => {
            HighlightedTextSegment::new($text, Highlight::None)
        };
        ( line $text:literal pt ) => {
            HighlightedTextSegment::new($text, Highlight::PrimitiveType)
        };
        ( line $text:literal i ) => {
            HighlightedTextSegment::new($text, Highlight::Ident)
        };
        ( line $text:literal q ) => {
            HighlightedTextSegment::new($text, Highlight::Qualifier)
        };
        ( line $text:literal qk ) => {
            HighlightedTextSegment::new($text, Highlight::QuasiKeyword)
        };
        ( line $text:literal num ) => {
            HighlightedTextSegment::new($text, Highlight::Number)
        };
        ( line $text:literal udt ) => {
            HighlightedTextSegment::new($text, Highlight::UserDefinedType)
        };
    }

    #[test]
    fn explain_primitive_var() {
        // run("int x", "an int named x");
        run(
            "int x",
            hltext![
                "an " n
                "int" pt
                " named " n
                "x" i
            ],
        );
    }

    /// Ensures "a" and "an" are used appropriately.
    #[test]
    fn test_articles() {
        run(
            "int x",
            hltext![
                "an " n
                "int" pt
                " named " n
                "x" i
            ],
        );
        run(
            "signed int x",
            hltext![
                "a " n
                "signed int" pt
                " named " n
                "x" i
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
            hltext![
                "a " n
                "pointer" qk
                " named " n
                "p" i
                " to an " n
                "int" pt
            ],
        );
    }

    #[test]
    fn explain_array_of_primitive() {
        run(
            "int arr[]",
            hltext![
                "an " n
                "array" qk
                " named " n
                "arr" i
                " of " n
                "int" pt
                "s" n
            ],
        );
    }

    #[test]
    fn explain_array_of_primitive_with_size() {
        // run("int arr[10]", "an array named arr of 10 ints");
        run(
            "int arr[10]",
            hltext![
                "an " n
                "array" qk
                " named " n
                "arr" i
                " of " n
                "10" num
                " " n
                "int" pt
                "s" n
            ],
        );
    }

    #[test]
    fn explain_2d_array_of_primitive() {
        run(
            "int arr[10][20]",
            hltext![
                "an " n
                "array" qk
                " named " n
                "arr" i
                " of " n
                "10" num
                " " n
                "arrays" qk
                " of " n
                "20" num
                " " n
                "int" pt
                "s" n
            ],
        );
    }

    #[test]
    fn explain_nested_ptrs() {
        run(
            "char ***p",
            hltext![
                "a " n
                "pointer" qk
                " named " n
                "p" i
                " to a " n
                "pointer" qk
                " to a " n
                "pointer" qk
                " to a " n
                "char" pt
            ],
        );
    }

    #[test]
    fn explain_array_of_ptrs() {
        run(
            "int *arr[10]",
            hltext![
                "an " n
                "array" qk
                " named " n
                "arr" i
                " of " n
                "10" num
                " " n
                "pointers" qk
                " to " n
                "int" pt
                "s" n
            ],
        );
    }

    #[test]
    fn explain_ptr_to_array() {
        run(
            "int (*p)[10]",
            hltext![
                "a " n
                "pointer" qk
                " named " n
                "p" i
                " to an " n
                "array" qk
                " of " n
                "10" num
                " " n
                "int" pt
                "s" n
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
            hltext![
                "a " n
                "function" qk
                " named " n
                "func" i
                " that takes no parameters and returns a " n
                "void" pt
            ],
        );
    }

    #[test]
    fn explain_array_of_functions() {
        run(
            "char *(*(*bar)[5])(int)",
            hltext![
                "a " n
                "pointer" qk
                " named " n
                "bar" i
                " to an " n
                "array" qk
                " of " n
                "5" num
                " " n
                "pointers" qk
                " to " n
                "functions" qk
                " that take (an " n
                "int" pt
                ") and return a " n
                "pointer" qk
                " to a " n
                "char" pt
            ],
        );
    }

    #[test]
    fn explain_qualifiers() {
        run(
            "const int x",
            hltext![
                "a " n
                "const" q
                " " n
                "int" pt
                " named " n
                "x" i
            ],
        );
        run(
            "volatile int x",
            hltext![
                "a " n
                "volatile" q
                " " n
                "int" pt
                " named " n
                "x" i
            ],
        );
        run(
            "int *const restrict x",
            hltext![
                "a " n
                "const restrict" q
                " " n
                "pointer" qk
                " named " n
                "x" i
                " to an " n
                "int" pt
            ],
        );
        run(
            "const char *const str",
            hltext![
                "a " n
                "const" q
                " " n
                "pointer" qk
                " named " n
                "str" i
                " to a " n
                "const" q
                " " n
                "char" pt
            ],
        );
    }

    #[test]
    fn explain_struct_var() {
        run(
            "struct point p",
            hltext![
                "a " n
                "struct point" udt
                " named " n
                "p" i
            ],
        );
    }

    #[test]
    fn explain_function_one_unnamed_param() {
        run(
            "int foo(const char *)",
            hltext![
                "a " n
                "function" qk
                " named " n
                "foo" i
                " that takes (a " n
                "pointer" qk
                " to a " n
                "const" q
                " " n
                "char" pt
                ") and returns an " n
                "int" pt
            ],
        );
    }

    #[test]
    fn explain_function_one_named_param() {
        run(
            "int foo(const char *bar)",
            hltext![
                "a " n
                "function" qk
                " named " n
                "foo" i
                " that takes (a " n
                "pointer" qk
                " named " n
                "bar" i
                " to a " n
                "const" q
                " " n
                "char" pt
                ") and returns an " n
                "int" pt
            ],
        );
    }

    #[test]
    fn explain_anonymous_function() {
        run(
            "int (*)(const char *)",
            hltext![
                "a " n
                "pointer" qk
                " to a " n
                "function" qk
                " that takes (a " n
                "pointer" qk
                " to a " n
                "const" q
                " " n
                "char" pt
                ") and returns an " n
                "int" pt
            ],
        );
    }

    #[test]
    fn explain_function_two_params() {
        run(
            "int add(int a, int b)",
            hltext![
                "a " n
                "function" qk
                " named " n
                "add" i
                " that takes (an " n
                "int" pt
                " named " n
                "a" i
                " and an " n
                "int" pt
                " named " n
                "b" i
                ") and returns an " n
                "int" pt
            ],
        );
    }

    #[test]
    fn explain_function_three_params() {
        run(
            "void print(int a, char *b, float c)",
            hltext![
                "a " n
                "function" qk
                " named " n
                "print" i
                " that takes (an " n
                "int" pt
                " named " n
                "a" i
                ", a " n
                "pointer" qk
                " named " n
                "b" i
                " to a " n
                "char" pt
                ", and a " n
                "float" pt
                " named " n
                "c" i
                ") and returns a " n
                "void" pt
            ],
        );
    }

    #[test]
    fn explain_array_of_struct() {
        run(
            "struct point p[]",
            hltext![
                "an " n
                "array" qk
                " named " n
                "p" i
                " of " n
                "struct point" udt
                "s" n
            ],
        );
    }

    #[test]
    fn explain_plural_qualifiers() {
        run(
            "char *const p[]",
            hltext![
                "an " n
                "array" qk
                " named " n
                "p" i
                " of " n
                "const" q
                " " n
                "pointers" qk
                " to " n
                "char" pt
                "s" n
            ],
        );
    }

    /// Anonymous typedefs are technically not valid C, but we handle them gracefully.
    #[test]
    fn explain_anon_typedef() {
        run(
            "typedef char *",
            hltext![
                "a type defined as a " n
                "pointer" qk
                " to a " n
                "char" pt
            ],
        );
    }

    #[test]
    fn explain_typedef() {
        run(
            "typedef struct point point_t",
            hltext![
                "a type named " n
                "point_t" udt
                " defined as a " n
                "struct point" udt
            ],
        );
    }

    #[test]
    fn explain_pointer_typedef() {
        run(
            "typedef const char *string",
            hltext![
                "a type named " n
                "string" udt
                " defined as a " n
                "pointer" qk
                " to a " n
                "const" q
                " " n
                "char" pt
            ],
        );
    }

    #[test]
    fn explain_typedef_plural_end() {
        run(
            "typedef int nums[]",
            hltext![
                "a type named " n
                "nums" udt
                " defined as an " n
                "array" qk
                " of " n
                "int" pt
                "s" n
            ],
        );
    }

    #[test]
    fn explain_function_typedef() {
        run(
            "typedef int (*compare_t)(const void *, const void *)",
            hltext![
                "a type named " n
                "compare_t" udt
                " defined as a " n
                "pointer" qk
                " to a " n
                "function" qk
                " that takes (a " n
                "pointer" qk
                " to a " n
                "const" q
                " " n
                "void" pt
                " and a " n
                "pointer" qk
                " to a " n
                "const" q
                " " n
                "void" pt
                ") and returns an " n
                "int" pt
            ],
        );
    }
}
