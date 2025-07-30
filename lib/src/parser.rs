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

//! Parser for C declarations.

use core::str::FromStr;

use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec::Vec};
use chumsky::{
    extra::Full,
    inspector::Inspector,
    prelude::*,
    text::{ident, int, keyword},
};
use error::RichWrapper;

use crate::ast::{
    Declaration, Declarator, PrimitiveType, QualifiedType, RecordKind, Type, TypeQualifier,
    TypeQualifiers,
};

mod error;

pub type Extra<'src> = Full<RichWrapper<'src>, State, ()>;

/// Parser state
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    custom_types: Vec<String>,
}

impl<'src, I: Input<'src>> Inspector<'src, I> for State {
    type Checkpoint = ();

    fn on_token(&mut self, _token: &I::Token) {}

    fn on_save<'parse>(
        &self,
        _cursor: &chumsky::input::Cursor<'src, 'parse, I>,
    ) -> Self::Checkpoint {
    }

    fn on_rewind<'parse>(
        &mut self,
        _marker: &chumsky::input::Checkpoint<'src, 'parse, I, Self::Checkpoint>,
    ) {
    }
}

/// From <https://www.open-std.org/jtc1/sc22/WG14/www/docs/n1256.pdf> section 6.7.2.
#[must_use]
fn primitive_type_parser<'src>() -> impl Parser<'src, &'src str, PrimitiveType, Extra<'src>> + Clone
{
    /// Macro to generate choices from a nicer syntax.
    /// Turns something like `unsigned long int` into
    /// `keyword("unsigned").padded().then(keyword("long").padded()).then(keyword("int").padded)`.
    macro_rules! gen_choices {
        ( $( $first:ident $($more:ident)* , )* ) => {
            choice(( $(
                keyword(stringify!($first)).padded()
                $(.then(keyword(stringify!($more)).padded()))*
                .to(PrimitiveType(stringify!($first $($more)*))),
            )* ))
        };
    }

    // We're limited to 26 choices per `choice()` so we split into two
    choice((
        gen_choices![
            unsigned long long int,
            unsigned long long,
            unsigned long int,
            unsigned short int,
            unsigned short,
            unsigned long,
            unsigned int,
            unsigned char,
            unsigned,
            signed long long int,
            signed long long,
            signed long int,
            signed long,
            signed short int,
            signed short,
            signed char,
            signed int,
            signed,
            long long int,
            long double _Complex,
            long double,
            long long,
            long int,
            long,
            short int,
            short,
        ],
        gen_choices![
            float _Complex,
            float,
            double _Complex,
            double,
            void,
            char,
            int,
            _Bool,
        ],
    ))
    .padded()
    .labelled("primitive type")
}

/// Helper enum to represent the possible suffixes of a declarator. This is needed so we have one
/// concrete type which can be used for all suffixes, allowing us to mix suffixes inside
/// a `choice().repeated()`, which requires the same type for all branches.
#[derive(Debug, Clone)]
enum SuffixInfo<'src> {
    Array(Option<usize>),
    Function(Vec<Declaration<'src>>),
}

/// Returns a parser which parses a C declaration.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn parser<'src>() -> impl Parser<'src, &'src str, Vec<Declaration<'src>>, Extra<'src>> {
    // Parses a declaration. Returns `Declaration`.
    let declaration = recursive(|declaration| {
        // Parses zero or more type qualifiers. Returns `TypeQualifiers`.
        let qualifiers = choice((
            keyword("const").to(TypeQualifier::Const),
            keyword("volatile").to(TypeQualifier::Volatile),
            keyword("restrict").to(TypeQualifier::Restrict),
        ))
        .labelled("type qualifier")
        .padded()
        .repeated()
        .collect::<TypeQualifiers>();

        let primitive_type = primitive_type_parser();
        let r#type = choice((
            // Primitive type
            primitive_type.map(Type::Primitive),
            // Record (struct/union/enum) type
            choice([keyword("struct"), keyword("union"), keyword("enum")])
                .map(|k| RecordKind::from_str(k).unwrap())
                .then(ident().padded())
                .map(|(kind, id)| Type::Record(kind, id)),
            // Custom (typedef) type
            ident()
                .padded()
                .try_map_with(|ident: &str, info| {
                    let state: &mut State = info.state();
                    if state.custom_types.iter().any(|ty| ty == ident) {
                        Ok(Type::Custom(ident))
                    } else {
                        Err(Rich::custom(
                            info.span(),
                            format!("\"{ident}\" is used as a type but has not been defined"),
                        )
                        .into())
                    }
                })
                .labelled("custom type"),
        ))
        .labelled("type");
        let qualified_type = qualifiers.clone().then(r#type).map(QualifiedType::from);

        let declarator = recursive(|declarator| {
            // Parses a declarator atom: either an identifier or parenthesized declarator.
            // Returns `Declarator`.
            let atom = choice((
                ident().map(Declarator::Ident),
                declarator
                    .clone()
                    .delimited_by(just('(').padded(), just(')').padded()),
            ));

            // Parses array declarator suffix. Returns `SuffixInfo`.
            let array_suffix = int(10)
                .try_map(|s, span| usize::from_str(s).map_err(|err| Rich::custom(span, err).into()))
                .or_not()
                .delimited_by(just('[').padded(), just(']').padded())
                .labelled("array brackets");

            // Parses function parameter list. Returns `Vec<Declaration>`.
            let func_param_list = declaration
                .labelled("function parameter")
                .separated_by(just(',').padded())
                .allow_trailing()
                .collect::<Vec<Declaration>>();

            // Parses function declarator suffix. Returns `SuffixInfo`.
            let func_suffix = choice((
                // Special case: func(void) means no parameters
                keyword("void")
                    .delimited_by(just('(').padded(), just(')').padded())
                    .to(Vec::new()),
                func_param_list.delimited_by(just('(').padded(), just(')').padded()),
            ))
            .labelled("function parentheses");

            // Parses atom with zero or more suffixes.
            // Returns `Declarator`.
            let with_suffixes = atom
                .or_not()
                .map(|atom| atom.unwrap_or(Declarator::Anonymous))
                .foldl(
                    choice((
                        array_suffix.map(SuffixInfo::Array),
                        func_suffix.map(SuffixInfo::Function),
                    ))
                    .repeated(),
                    |inner, suffix| match suffix {
                        SuffixInfo::Array(size) => Declarator::Array(Box::new(inner), size),
                        SuffixInfo::Function(params) => Declarator::Function {
                            func: Box::new(inner),
                            params,
                        },
                    },
                );

            // Parses a suffixed atom with zero or more pointer prefixes.
            // Returns `Declarator`.
            just('*')
                .padded()
                .ignore_then(qualifiers)
                .repeated()
                .foldr(with_suffixes, |qualifiers, inner| {
                    Declarator::Ptr(Box::new(inner), qualifiers)
                })
        });

        qualified_type
            .then(declarator)
            .map(Declaration::from)
            .padded()
    });

    choice((
        // Parses a typedef declaration. Returns `Declaration`.
        keyword("typedef")
            .padded()
            .ignore_then(declaration.clone())
            .map_with(|mut decl, info| {
                // If the typedef has a name, add it to the custom types in the state.
                if let Some(name) = decl.declarator.name() {
                    let state: &mut State = info.state();
                    state.custom_types.push(name.to_owned());
                }
                // Add the typedef qualifier and return the declaration.
                decl.base_type.0.insert(TypeQualifier::Typedef);
                decl
            }),
        // Parses a regular declaration. Returns `Declaration`.
        declaration,
    ))
    .separated_by(just(';').padded().repeated().at_least(1))
    .allow_trailing()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{format, string::ToString, vec, vec::Vec};
    use pretty_assertions::assert_eq;

    /// Qualified version of [`primitive()`].
    fn qprimitive<'src, I>(
        qualifiers: I,
        r#type: &'static str,
        declarator: Declarator<'src>,
    ) -> Declaration<'src>
    where
        I: IntoIterator<Item = TypeQualifier>,
    {
        Declaration {
            base_type: QualifiedType(
                TypeQualifiers(qualifiers.into_iter().collect()),
                Type::Primitive(PrimitiveType(r#type)),
            ),
            declarator,
        }
    }

    fn primitive<'src>(r#type: &'static str, declarator: Declarator<'src>) -> Declaration<'src> {
        qprimitive([], r#type, declarator)
    }

    fn qrecord<'src, I>(
        qualifiers: I,
        kind: &'static str,
        name: &'static str,
        declarator: Declarator<'src>,
    ) -> Declaration<'src>
    where
        I: IntoIterator<Item = TypeQualifier>,
    {
        Declaration {
            base_type: QualifiedType(
                TypeQualifiers(qualifiers.into_iter().collect()),
                Type::Record(kind.parse().unwrap(), name),
            ),
            declarator,
        }
    }

    fn anon() -> Declarator<'static> {
        Declarator::Anonymous
    }

    fn qptr<I>(qualifiers: I, val: Declarator) -> Declarator
    where
        I: IntoIterator<Item = TypeQualifier>,
    {
        Declarator::Ptr(
            Box::new(val),
            TypeQualifiers(qualifiers.into_iter().collect()),
        )
    }

    fn ptr(val: Declarator) -> Declarator {
        qptr([], val)
    }

    fn ident(val: &str) -> Declarator {
        Declarator::Ident(val)
    }

    fn array(d: Declarator, size: impl Into<Option<usize>>) -> Declarator {
        Declarator::Array(Box::new(d), size.into())
    }

    fn func<'src>(
        func: Declarator<'src>,
        args: impl Into<Vec<Declaration<'src>>>,
    ) -> Declarator<'src> {
        Declarator::Function {
            func: Box::new(func),
            params: args.into(),
        }
    }

    #[test]
    fn test_basic_int_var() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")).into(),
            declarator: ident("myvar123"),
        };
        assert_eq!(vec![expected], parser().parse("int myvar123").unwrap());
    }

    #[test]
    fn test_basic_int_ptr_vars() {
        let expected = vec![Declaration {
            base_type: Type::Primitive(PrimitiveType("int")).into(),
            declarator: ptr(ident("p")),
        }];
        let cases = ["int *p", "int*p", "int* p", "int *\np"];
        for case in cases {
            assert_eq!(expected, parser().parse(case).unwrap());
        }
    }

    #[test]
    fn test_nested_ptrs() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("char")).into(),
            declarator: ptr(ptr(ptr(ident("p")))),
        };
        assert_eq!(vec![expected], parser().parse("char ***p").unwrap());
    }

    #[test]
    fn test_record_vars() {
        let cases = [
            ("struct foo bar", RecordKind::Struct),
            ("enum foo bar", RecordKind::Enum),
            ("union foo bar", RecordKind::Union),
        ];
        for (input, record_kind) in cases {
            let expected = Declaration {
                base_type: Type::Record(record_kind, "foo").into(),
                declarator: ident("bar"),
            };
            assert_eq!(vec![expected], parser().parse(input).unwrap());
        }
    }

    #[test]
    fn test_all_primitive_types() {
        let cases = [
            "unsigned long long int",
            "unsigned long long",
            "unsigned long int",
            "unsigned short int",
            "unsigned short",
            "unsigned long",
            "unsigned int",
            "unsigned char",
            "unsigned",
            "signed long long int",
            "signed long long",
            "signed long int",
            "signed long",
            "signed short int",
            "signed short",
            "signed char",
            "signed int",
            "signed",
            "long long int",
            "long double _Complex",
            "long double",
            "long long",
            "long int",
            "long",
            "short int",
            "short",
            "float _Complex",
            "float",
            "double _Complex",
            "double",
            "void",
            "char",
            "int",
            "_Bool",
        ];
        for r#type in cases {
            let expected = Declaration {
                base_type: Type::Primitive(PrimitiveType(r#type)).into(),
                declarator: ident("foo"),
            };
            let src = format!("{type} foo");
            assert_eq!(vec![expected], parser().parse(&src).unwrap());
        }
    }

    #[test]
    fn test_array_declarator_no_size() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")).into(),
            declarator: array(ptr(ident("foo")), None),
        };
        assert_eq!(vec![expected], parser().parse("int (*foo)[]").unwrap());
    }

    #[test]
    fn test_array_declarator_with_size() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")).into(),
            declarator: array(ptr(ident("foo")), Some(10)),
        };
        assert_eq!(vec![expected], parser().parse("int (*foo)[10]").unwrap());
    }

    #[test]
    fn test_multi_dimen_array_and_ptr() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("char")).into(),
            declarator: ptr(array(array(ident("foo"), 3), 2)),
        };
        assert_eq!(vec![expected], parser().parse("char *foo[3][2]").unwrap());
    }

    #[test]
    fn test_function_no_args() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")).into(),
            declarator: func(ident("foo"), []),
        };
        assert_eq!(vec![expected], parser().parse("int foo()").unwrap());
    }

    #[test]
    fn test_function_single_unnamed_arg() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")).into(),
            declarator: func(ident("foo"), [primitive("int", anon())]),
        };
        assert_eq!(vec![expected], parser().parse("int foo(int)").unwrap());
    }

    #[test]
    fn test_function_single_named_arg() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")).into(),
            declarator: func(ident("foo"), [primitive("int", ident("bar"))]),
        };
        assert_eq!(vec![expected], parser().parse("int foo(int bar)").unwrap());
    }

    #[test]
    fn test_function_multiple_named_args() {
        let expected = primitive(
            "int",
            ptr(func(
                ident("foo"),
                [
                    primitive("int", ident("bar")),
                    primitive("char", ident("baz")),
                ],
            )),
        );
        assert_eq!(
            vec![expected],
            parser().parse("int *foo(int bar, char baz)").unwrap()
        );
    }

    #[test]
    fn parse_qualified_primitive() {
        assert_eq!(
            vec![qprimitive([TypeQualifier::Const], "int", ident("x"))],
            parser().parse("const int x").unwrap()
        );
    }

    #[test]
    fn parse_const_char_ptr() {
        assert_eq!(
            vec![qprimitive(
                [TypeQualifier::Const],
                "char",
                ptr(ident("str"))
            )],
            parser().parse("const char *str").unwrap()
        );
    }

    #[test]
    fn parse_qualified_ptr() {
        assert_eq!(
            vec![qprimitive(
                [TypeQualifier::Const],
                "int",
                qptr([TypeQualifier::Volatile], ident("x"))
            )],
            parser().parse("const int *volatile x").unwrap()
        );
    }

    #[test]
    fn parse_struct_var() {
        assert_eq!(
            vec![qrecord(
                [TypeQualifier::Const],
                "struct",
                "foo",
                ident("bar")
            )],
            parser().parse("const struct foo bar").unwrap()
        );
    }

    #[test]
    fn parse_complex_1() {
        assert_eq!(
            vec![primitive(
                "void",
                func(
                    ptr(ident("cb")),
                    vec![qrecord([], "struct", "foo", ptr(anon()))]
                )
            )],
            parser().parse("void (*cb)(struct foo *)").unwrap()
        );
    }

    #[test]
    fn parse_complex_2() {
        let expected = qprimitive(
            [TypeQualifier::Const],
            "char",
            ptr(func(
                ptr(ident("func")),
                vec![
                    qprimitive(
                        [],
                        "void",
                        func(
                            ptr(ident("cb")),
                            vec![qrecord([], "struct", "foo", ptr(anon()))],
                        ),
                    ),
                    primitive("int", anon()),
                    qprimitive(
                        [TypeQualifier::Const],
                        "char",
                        qptr([TypeQualifier::Restrict], ident("my_str")),
                    ),
                ],
            )),
        );
        assert_eq!(
            vec![expected],
            parser()
                .parse(
                    "const char *(*func)(void (*cb)(struct foo *), int, const char *restrict my_str)"
                )
                .unwrap()
        );
    }

    #[test]
    fn parse_invalid_array_length() {
        let result = parser().parse("int arr[x]");
        let errors = result.into_errors();
        assert_eq!(errors.len(), 1, "expected one error");
        assert_eq!(
            errors[0].span().into_range(),
            8..9,
            "error position mismatch"
        );
    }

    #[test]
    fn parse_out_of_bounds_array_length() {
        let src = format!("int arr[{}0]", usize::MAX);
        let result = parser().parse(&src);
        let errors = result.into_errors();
        assert_eq!(errors.len(), 1, "expected one error");
        assert_eq!(
            errors[0].span().into_range(),
            8..29,
            "error position mismatch"
        );
        assert_eq!(
            errors[0].to_string(),
            "at 8..29: number too large to fit in target type"
        );
    }

    #[test]
    fn parse_multiple_declarations() {
        let expected = vec![
            primitive("int", ident("a")),
            qprimitive([TypeQualifier::Const], "char", ident("b")),
            qrecord([], "struct", "foo", ident("c")),
        ];

        let src = "int a; const char b; struct foo c;";
        assert_eq!(expected, parser().parse(src).unwrap());

        // Same test with extra semicolons
        let src = "int a; const char b;;; struct foo c;";
        assert_eq!(expected, parser().parse(src).unwrap());
    }

    #[test]
    fn parse_empty() {
        assert_eq!(parser().parse("").unwrap(), vec![]);
    }

    #[test]
    fn parse_typedef_declaration() {
        let expected = qprimitive([TypeQualifier::Typedef], "int", ident("foo"));
        let parser = parser();
        assert_eq!(vec![expected], parser.parse("typedef int foo").unwrap());
    }

    #[test]
    fn parse_typedef_reference() {
        let expected = Declaration {
            base_type: QualifiedType(
                TypeQualifiers([TypeQualifier::Const].into_iter().collect()),
                Type::Custom("foo"),
            ),
            declarator: ptr(ident("bar")),
        };
        let mut state = State {
            custom_types: vec!["foo".to_owned()],
        };
        assert_eq!(
            vec![expected],
            parser()
                .parse_with_state("const foo *bar", &mut state)
                .unwrap()
        );
    }
}
