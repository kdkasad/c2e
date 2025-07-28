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

use core::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use alloc::{boxed::Box, vec::Vec};
use enumflags2::BitFlags;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<'src> {
    pub base_type: QualifiedType<'src>,
    pub declarator: Declarator<'src>,
}

// Convert from a tuple `(Type, Declarator)` to a `Declaration`
impl<'src> From<(QualifiedType<'src>, Declarator<'src>)> for Declaration<'src> {
    fn from((base_type, declarator): (QualifiedType<'src>, Declarator<'src>)) -> Self {
        Declaration {
            base_type,
            declarator,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, parse_display::Display)]
pub enum Type<'src> {
    #[display("{0}")]
    Primitive(PrimitiveType),
    #[display("{0} {1}")]
    Record(RecordKind, &'src str),
    // TODO: user-defined (typedef) types
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, parse_display::Display)]
#[display("{0}{1}")]
pub struct QualifiedType<'src>(pub TypeQualifiers, pub Type<'src>);

impl<'src> From<(TypeQualifiers, Type<'src>)> for QualifiedType<'src> {
    fn from((qualifiers, ty): (TypeQualifiers, Type<'src>)) -> Self {
        QualifiedType(qualifiers, ty)
    }
}

impl<'src> From<Type<'src>> for QualifiedType<'src> {
    fn from(ty: Type<'src>) -> Self {
        QualifiedType(TypeQualifiers::default(), ty)
    }
}

/// Qualifier for a type
#[derive(Debug, Copy, Clone, PartialEq, Eq, parse_display::Display)]
#[display(style = "title case")]
#[enumflags2::bitflags]
#[repr(u8)]
pub enum TypeQualifier {
    /// `const`
    Const,
    /// `volatile`
    Volatile,
    /// `restrict`
    Restrict,
}

/// Bit set of [type qualifiers][TypeQualifier]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TypeQualifiers(pub BitFlags<TypeQualifier>);

impl Deref for TypeQualifiers {
    type Target = BitFlags<TypeQualifier>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TypeQualifiers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Format the type qualifiers as a space-separated list.
///
/// # Examples
///
/// ```
/// # use c2e::ast::TypeQualifiers;
/// let empty = TypeQualifiers::default();
/// assert_eq!(&empty.to_string(), "");
/// ```
///
/// ```
/// # use c2e::ast::{TypeQualifiers, TypeQualifier};
/// let mut qualifiers = TypeQualifiers([
///     TypeQualifier::Const,
///     TypeQualifier::Volatile
/// ].into_iter().collect());
/// assert_eq!(&qualifiers.to_string(), "const volatile");
/// ```
impl Display for TypeQualifiers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.iter().enumerate().try_for_each(|(i, qualifier)| {
            if i == 0 {
                write!(f, "{qualifier}")
            } else {
                write!(f, " {qualifier}")
            }
        })
    }
}

impl chumsky::container::Container<TypeQualifier> for TypeQualifiers {
    fn push(&mut self, item: TypeQualifier) {
        self.insert(item);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, parse_display::Display, parse_display::FromStr)]
#[display(style = "title case")]
pub enum RecordKind {
    Union,
    Struct,
    Enum,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, parse_display::Display)]
pub struct PrimitiveType(pub(crate) &'static str);

impl AsRef<str> for PrimitiveType {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declarator<'src> {
    /// Represents the base of an anonymous (unnamed) declaration, such as a function parameter.
    /// I.e., this is where [`Declarator::Ident`] would be used if the declaration had a name.
    Anonymous,
    Ident(&'src str),
    Ptr(Box<Declarator<'src>>, TypeQualifiers),
    Array(Box<Declarator<'src>>, Option<usize>),
    Function {
        func: Box<Declarator<'src>>,
        params: Vec<Declaration<'src>>,
    },
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_primitive_type_as_ref() {
        let int_type = PrimitiveType("int");
        assert_eq!(AsRef::<str>::as_ref(&int_type), "int");
    }

    #[test]
    fn type_qualifiers_display() {
        let mut qualifiers = TypeQualifiers::default();
        assert_eq!(qualifiers.to_string(), "");

        qualifiers.insert(TypeQualifier::Const);
        assert_eq!(qualifiers.to_string(), "const");

        qualifiers.insert(TypeQualifier::Volatile);
        assert_eq!(qualifiers.to_string(), "const volatile");
    }
}
