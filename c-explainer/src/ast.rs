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

//! Abstract syntax tree (AST) types

use core::fmt::Display;

use alloc::boxed::Box;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<'src> {
    pub base_type: Type<'src>,
    pub declarator: Declarator<'src>,
}

// Convert from a tuple `(Type, Declarator)` to a `Declaration`
impl<'src> From<(Type<'src>, Declarator<'src>)> for Declaration<'src> {
    fn from((base_type, declarator): (Type<'src>, Declarator<'src>)) -> Self {
        Declaration {
            base_type,
            declarator,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Type<'src> {
    Primitive(PrimitiveType),
    Record(RecordKind, &'src str),
    // TODO: user-defined (typedef) types
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, parse_display::Display, parse_display::FromStr)]
#[display(style = "title case")]
pub enum RecordKind {
    Union,
    Struct,
    Enum,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PrimitiveType(pub(crate) &'static str);

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declarator<'src> {
    Ident(&'src str),
    Ptr(Box<Declarator<'src>>),
    Array(Box<Declarator<'src>>, Option<usize>),
}
