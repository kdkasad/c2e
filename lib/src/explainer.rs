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

use alloc::string::{String, ToString};

use crate::ast::{Declaration, Declarator, QualifiedType, Type};

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
    explain_declaration_impl(decl).msg
}

#[derive(Debug)]
struct Explanation {
    /// Name of the root identifier being explained
    identifier_name: Option<String>,
    /// String containing English explanation
    msg: String,
    plurality: Plurality,
}

impl Explanation {
    fn new() -> Self {
        Self {
            identifier_name: None,
            msg: String::new(),
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

fn explain_declaration_impl(decl: &Declaration) -> Explanation {
    let mut explanation = explain_declarator(&decl.declarator);
    match (decl.base_type, explanation.plurality) {
        (QualifiedType(_, Type::Primitive(_)), Plurality::Singular) => {
            articulate(&mut explanation.msg, &decl.base_type.to_string());
        }
        (QualifiedType(_, Type::Primitive(_)), Plurality::Plural) => {
            make_plural(&mut explanation.msg, &decl.base_type.to_string());
        }
        (qt, Plurality::Singular) => articulate(&mut explanation.msg, &qt.to_string()),
        (qt, _) => write!(&mut explanation.msg, "{qt}").unwrap(),
    }
    if let Plurality::Singular = explanation.plurality
        && let Some(name) = &explanation.identifier_name
    {
        write!(&mut explanation.msg, " named {name}").unwrap();
    }
    explanation
}

#[must_use]
fn explain_declarator(declarator: &Declarator) -> Explanation {
    match declarator {
        Declarator::Anonymous => Explanation::new(),
        Declarator::Ident(name) => Explanation::new().with_identifier_name((*name).to_string()),
        Declarator::Ptr(inner, qualifiers) => {
            let mut sub = explain_declarator(inner);
            match sub.plurality {
                Plurality::Singular => write!(&mut sub.msg, "a {qualifiers}pointer "),
                Plurality::Plural => write!(&mut sub.msg, "{qualifiers}pointers "),
            }
            .unwrap();
            if let Some(name) = &sub.identifier_name {
                write!(&mut sub.msg, "named {name} ").unwrap();
            }
            sub.msg.push_str("to ");
            sub.unnamed()
        }
        Declarator::Array(inner, len) => {
            let mut sub = explain_declarator(inner);
            sub.msg.push_str(match sub.plurality {
                Plurality::Singular => "an array ",
                Plurality::Plural => "arrays ",
            });
            if let Some(name) = &sub.identifier_name {
                write!(&mut sub.msg, "named {name} ").unwrap();
            }
            sub.msg.push_str("of ");
            if let Some(len) = len {
                write!(&mut sub.msg, "{len} ").unwrap();
            }
            sub.unnamed().plural()
        }
        Declarator::Function { func, params } => {
            let mut sub = explain_declarator(func);
            match (&sub.identifier_name, sub.plurality) {
                (None, Plurality::Singular) => write!(&mut sub.msg, "a function that takes "),
                (None, Plurality::Plural) => write!(&mut sub.msg, "functions that take "),
                (Some(name), Plurality::Singular) => {
                    write!(&mut sub.msg, "a function named {name} that takes ")
                }
                (Some(_), Plurality::Plural) => unreachable!("an identifier cannot be plural"),
            }
            .unwrap();
            match &params[..] {
                [] => sub.msg.push_str("no parameters"),
                [param] => write!(&mut sub.msg, "({})", explain_declaration(param)).unwrap(),
                [a, b] => write!(
                    &mut sub.msg,
                    "({} and {})",
                    explain_declaration(a),
                    explain_declaration(b)
                )
                .unwrap(),
                [rest @ .., last] => {
                    sub.msg.push('(');
                    for param in rest {
                        write!(&mut sub.msg, "{}, ", explain_declaration(param)).unwrap();
                    }
                    write!(&mut sub.msg, "and {})", explain_declaration(last)).unwrap();
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
    fn run(expression: &str, expected: &str) {
        let decls = crate::parser::parser().parse(expression).unwrap();
        assert_eq!(decls.len(), 1, "Expected exactly one declaration");
        let result = explain_declaration(&decls[0]);
        assert_eq!(result, expected);
    }

    #[test]
    fn explain_primitive_var() {
        run("int x", "an int named x");
    }

    /// Ensures "a" and "an" are used appropriately.
    #[test]
    fn test_articles() {
        run("int x", "an int named x");
        run("signed int x", "a signed int named x");
    }

    #[test]
    fn test_articulate() {
        let mut s = String::new();
        articulate(&mut s, "int");
        assert_eq!(s, "an int");
        s.clear();
        articulate(&mut s, "cow");
        assert_eq!(s, "a cow");
        s.clear();
        articulate(&mut s, "");
        assert_eq!(s, "");
    }

    #[test]
    fn test_make_plural() {
        let test = |word, expected| {
            let mut s = String::new();
            make_plural(&mut s, word);
            assert_eq!(s, expected);
        };
        test("cat", "cats");
        test("box", "boxes");
        test("int", "ints");
        test("", "");
    }

    #[test]
    fn explain_ptr_to_primitive() {
        run("int *p", "a pointer named p to an int");
    }

    #[test]
    fn explain_array_of_primitive() {
        run("int arr[]", "an array named arr of ints");
    }

    #[test]
    fn explain_array_of_primitive_with_size() {
        run("int arr[10]", "an array named arr of 10 ints");
    }

    #[test]
    fn explain_2d_array_of_primitive() {
        run(
            "int arr[10][20]",
            "an array named arr of 10 arrays of 20 ints",
        );
    }

    #[test]
    fn explain_nested_ptrs() {
        run(
            "char ***p",
            "a pointer named p to a pointer to a pointer to a char",
        );
    }

    #[test]
    fn explain_array_of_ptrs() {
        run("int *arr[10]", "an array named arr of 10 pointers to ints");
    }

    #[test]
    fn explain_ptr_to_array() {
        run("int (*p)[10]", "a pointer named p to an array of 10 ints");
    }

    #[test]
    fn explain_function_with_no_params() {
        run(
            "void func()",
            "a function named func that takes no parameters and returns a void",
        );
    }

    #[test]
    fn explain_array_of_functions() {
        run(
            "char *(*(*bar)[5])(int)",
            "a pointer named bar to an array of 5 pointers to functions that take (an int) and return a pointer to a char",
        );
    }

    #[test]
    fn explain_qualifiers() {
        run("const int x", "a const int named x");
        run("volatile int x", "a volatile int named x");
        run(
            "int *const restrict x",
            "a const restrict pointer named x to an int",
        );
        run(
            "const char *const str",
            "a const pointer named str to a const char",
        );
    }

    #[test]
    fn explain_struct_var() {
        run("struct point p", "a struct point named p");
    }

    #[test]
    fn explain_function_one_unnamed_param() {
        run(
            "int foo(const char *)",
            "a function named foo that takes (a pointer to a const char) and returns an int",
        );
    }

    #[test]
    fn explain_function_one_named_param() {
        run(
            "int foo(const char *bar)",
            "a function named foo that takes (a pointer named bar to a const char) and returns an int",
        );
    }

    #[test]
    fn explain_anonymous_function() {
        run(
            "int (*)(const char *)",
            "a pointer to a function that takes (a pointer to a const char) and returns an int",
        );
    }

    #[test]
    fn explain_function_two_params() {
        run(
            "int add(int a, int b)",
            "a function named add that takes (an int named a and an int named b) and returns an int",
        );
    }

    #[test]
    fn explain_function_three_params() {
        run(
            "void print(int a, char *b, float c)",
            "a function named print that takes (an int named a, a pointer named b to a char, and a float named c) and returns a void",
        );
    }

    #[test]
    fn explain_array_of_struct() {
        run("struct point p[]", "an array named p of struct point");
    }
}
