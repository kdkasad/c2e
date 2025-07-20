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

#![no_std]

// Enable use of types which require heap memory.
extern crate alloc;

use alloc::boxed::Box;
use chumsky::{
    prelude::*,
    text::{ident, keyword},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<'src> {
    pub base_type: PrimitiveType,
    pub declarator: Declarator<'src>,
}

// Convert from a tuple `(PrimitiveType, Declarator)` to a `Declaration`
impl<'src> From<(PrimitiveType, Declarator<'src>)> for Declaration<'src> {
    fn from((base_type, declarator): (PrimitiveType, Declarator<'src>)) -> Self {
        Declaration {
            base_type,
            declarator,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    Void,
    Char,
    Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declarator<'src> {
    Ident(&'src str),
    Ptr(Box<Declarator<'src>>),
}

pub fn parser<'src>()
-> impl Parser<'src, &'src str, Declaration<'src>, chumsky::extra::Err<Rich<'src, char>>> {
    let primitive_type = choice((
        keyword("void").to(PrimitiveType::Void),
        keyword("char").to(PrimitiveType::Char),
        keyword("int").to(PrimitiveType::Int),
    ))
    .padded();

    let declarator = recursive(|declarator| {
        choice((
            // Identifier
            ident().map(Declarator::Ident),
            // Pointer declarator
            just('*')
                .ignore_then(declarator)
                .map(Box::new)
                .map(Declarator::Ptr),
        ))
        .padded()
    });

    primitive_type.then(declarator).map(Declaration::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn ptr(val: Declarator) -> Declarator {
        Declarator::Ptr(Box::new(val))
    }

    fn ident(val: &str) -> Declarator {
        Declarator::Ident(val)
    }

    #[test]
    fn test_basic_int_var() {
        let expected = Declaration {
            base_type: PrimitiveType::Int,
            declarator: ident("myvar123"),
        };
        assert_eq!(expected, parser().parse("int myvar123").unwrap());
    }

    #[test]
    fn test_basic_int_ptr_var() {
        let expected = Declaration {
            base_type: PrimitiveType::Int,
            declarator: ptr(ident("p")),
        };
        let cases = ["int *p", "int*p", "int* p", "int *\np"];
        for case in cases {
            assert_eq!(expected, parser().parse(case).unwrap());
        }
    }
}
