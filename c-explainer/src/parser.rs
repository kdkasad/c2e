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

use alloc::boxed::Box;
use chumsky::{
    prelude::*,
    text::{ident, int, keyword},
};

use crate::ast::{Declaration, Declarator, PrimitiveType, RecordKind, Type};

/// From <https://www.open-std.org/jtc1/sc22/WG14/www/docs/n1256.pdf> section 6.7.2.
#[must_use]
fn primitive_type_parser<'src>()
-> impl Parser<'src, &'src str, PrimitiveType, chumsky::extra::Err<Rich<'src, char>>> {
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
}

/// Returns a parser which parses a C declaration.
#[must_use]
pub fn parser<'src>()
-> impl Parser<'src, &'src str, Declaration<'src>, chumsky::extra::Err<Rich<'src, char>>> {
    let primitive_type = primitive_type_parser();
    let r#type = choice((
        // Primitive type
        primitive_type.map(Type::Primitive),
        // Record (struct/union/enum) type
        choice([keyword("struct"), keyword("union"), keyword("enum")])
            .map(|k| RecordKind::from_str(k).unwrap())
            .then(ident().padded())
            .map(|(kind, id)| Type::Record(kind, id)),
    ));

    let declarator = recursive(|declarator| {
        // Parses a declarator atom: either an identifier or parenthesized declarator
        let atom = choice((
            ident().map(Declarator::Ident),
            declarator
                .clone()
                .delimited_by(just('(').padded(), just(')').padded()),
        ));

        // Parses array declarator suffix
        let array_suffix = int(10)
            .try_map(|s, span| usize::from_str(s).map_err(|err| Rich::custom(span, err)))
            .or_not()
            .delimited_by(just('[').padded(), just(']').padded());

        // Parses atom with zero or more suffixes
        let with_suffixes = atom.foldl(array_suffix.repeated(), |inner, maybe_size| {
            Declarator::Array(Box::new(inner), maybe_size)
        });

        // Parses a suffixed atom with zero or more pointer prefixes
        just('*')
            .padded()
            .repeated()
            .foldr(with_suffixes, |_op, inner| Declarator::Ptr(Box::new(inner)))
    });

    r#type.then(declarator).map(Declaration::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::format;
    use pretty_assertions::assert_eq;

    fn ptr(val: Declarator) -> Declarator {
        Declarator::Ptr(Box::new(val))
    }

    fn ident(val: &str) -> Declarator {
        Declarator::Ident(val)
    }

    fn array(d: Declarator, size: impl Into<Option<usize>>) -> Declarator {
        Declarator::Array(Box::new(d), size.into())
    }

    #[test]
    fn test_basic_int_var() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")),
            declarator: ident("myvar123"),
        };
        assert_eq!(expected, parser().parse("int myvar123").unwrap());
    }

    #[test]
    fn test_basic_int_ptr_vars() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")),
            declarator: ptr(ident("p")),
        };
        let cases = ["int *p", "int*p", "int* p", "int *\np"];
        for case in cases {
            assert_eq!(expected, parser().parse(case).unwrap());
        }
    }

    #[test]
    fn test_nested_ptrs() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("char")),
            declarator: ptr(ptr(ptr(ident("p")))),
        };
        assert_eq!(expected, parser().parse("char ***p").unwrap());
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
                base_type: Type::Record(record_kind, "foo"),
                declarator: ident("bar"),
            };
            assert_eq!(expected, parser().parse(input).unwrap());
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
                base_type: Type::Primitive(PrimitiveType(r#type)),
                declarator: ident("foo"),
            };
            let src = format!("{type} foo");
            assert_eq!(expected, parser().parse(&src).unwrap());
        }
    }

    #[test]
    fn test_array_declarator_no_size() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")),
            declarator: array(ptr(ident("foo")), None),
        };
        assert_eq!(expected, parser().parse("int (*foo)[]").unwrap());
    }

    #[test]
    fn test_array_declarator_with_size() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("int")),
            declarator: array(ptr(ident("foo")), Some(10)),
        };
        assert_eq!(expected, parser().parse("int (*foo)[10]").unwrap());
    }

    #[test]
    fn test_multi_dimen_array_and_ptr() {
        let expected = Declaration {
            base_type: Type::Primitive(PrimitiveType("char")),
            declarator: ptr(array(array(ident("foo"), 3), 2)),
        };
        assert_eq!(expected, parser().parse("char *foo[3][2]").unwrap());
    }
}
