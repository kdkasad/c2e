//! Parser error wrapper

use core::{fmt::Display, ops::Deref};

use chumsky::{
    error::{Error as ChumskyError, Rich, RichPattern},
    input::Input,
    label::LabelError,
    util::MaybeRef,
};

/// Wrapper newtype around [`Rich`] to provide a custom [`Display`] implementation.
#[derive(Debug, Clone)]
pub struct RichWrapper<'src>(Rich<'src, char>);

impl<'src> From<Rich<'src, char>> for RichWrapper<'src> {
    fn from(value: Rich<'src, char>) -> Self {
        Self(value)
    }
}

impl<'src> Deref for RichWrapper<'src> {
    type Target = Rich<'src, char>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for RichWrapper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "at {}: ", self.0.span())?;
        match self.0.reason() {
            chumsky::error::RichReason::ExpectedFound { expected, found } => {
                write!(f, "expected ")?;
                match expected.as_slice() {
                    [] => write!(f, "[unknown]")?,
                    [thing] => write!(f, "{}", thing.wrap())?,
                    [a, b] => write!(f, "{} or {}", a.wrap(), b.wrap())?,
                    [rest @ .., last] => {
                        for thing in rest {
                            write!(f, "{}, ", thing.wrap())?;
                        }
                        write!(f, "or {}", last.wrap())?;
                    }
                }
                write!(f, ", but found ")?;
                match found {
                    Some(token) => write!(f, "'{}'", **token)?,
                    None => write!(f, "end of input")?,
                }
            }
            chumsky::error::RichReason::Custom(msg) => {
                msg.fmt(f)?;
            }
        }
        Ok(())
    }
}

/// Type alias for the token type of a `&str` input.
type StrToken<'src> = <&'src str as Input<'src>>::Token;

/// Delegate [`LabelError`] to [`Rich`].
impl<'src, L> LabelError<'src, &'src str, L> for RichWrapper<'src>
where
    L: Into<RichPattern<'src, StrToken<'src>>>,
{
    #[inline]
    fn expected_found<E: IntoIterator<Item = L>>(
        expected: E,
        found: Option<MaybeRef<'src, StrToken>>,
        span: <&'src str as Input<'src>>::Span,
    ) -> Self {
        let inner = <Rich<'src, char> as LabelError<'src, &'src str, L>>::expected_found(
            expected, found, span,
        );
        Self(inner)
    }

    #[inline]
    fn merge_expected_found<E: IntoIterator<Item = L>>(
        self,
        expected: E,
        found: Option<MaybeRef<'src, StrToken<'src>>>,
        span: <&'src str as Input<'src>>::Span,
    ) -> Self
    where
        Self: ChumskyError<'src, &'src str>,
    {
        let inner = <Rich<'src, char> as LabelError<'src, &'src str, L>>::merge_expected_found(
            self.0, expected, found, span,
        );
        Self(inner)
    }

    #[inline]
    fn replace_expected_found<E: IntoIterator<Item = L>>(
        self,
        expected: E,
        found: Option<MaybeRef<'src, <&'src str as Input<'src>>::Token>>,
        span: <&'src str as Input<'src>>::Span,
    ) -> Self {
        let inner = <Rich<'src, char> as LabelError<'src, &'src str, L>>::replace_expected_found(
            self.0, expected, found, span,
        );
        Self(inner)
    }

    #[inline]
    fn label_with(&mut self, label: L) {
        <Rich<'src, char> as LabelError<'src, &'src str, L>>::label_with(&mut self.0, label);
    }

    #[inline]
    fn in_context(&mut self, _label: L, _span: <&'src str as Input<'src>>::Span) {
        todo!("we don't use this function, so we don't implement it yet");
        // <Rich<'src, char> as LabelError<'src, &'src str, L>>::in_context(&mut self.0, label, span);
    }
}

/// Delegate [`Error`][ChumskyError] to [`Rich`].
impl<'src> ChumskyError<'src, &'src str> for RichWrapper<'src> {
    fn merge(self, other: Self) -> Self {
        let inner = <Rich<'src, char> as ChumskyError<'src, &'src str>>::merge(self.0, other.0);
        Self(inner)
    }
}

/// Wrapper for [`RichPattern`] to provide a custom [`Display`] implementation.
struct RichPatternWrapper<'src>(&'src RichPattern<'src, char>);

impl Display for RichPatternWrapper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            RichPattern::Token(tok) => write!(f, "'{}'", **tok),
            RichPattern::Label(l) => write!(f, "{l}"),
            RichPattern::Identifier(i) => write!(f, "'{i}'"),
            RichPattern::Any => write!(f, "anything"),
            RichPattern::SomethingElse => write!(f, "something else"),
            RichPattern::EndOfInput => write!(f, "end of input"),
        }
    }
}

/// Extension trait to provide a convenient `.wrap()` method on [`RichPattern`]s to wrap it with
/// a [`RichPatternWrapper`].
trait RichPatternExt {
    fn wrap(&self) -> RichPatternWrapper<'_>;
}

impl RichPatternExt for RichPattern<'_, char> {
    fn wrap(&self) -> RichPatternWrapper<'_> {
        RichPatternWrapper(self)
    }
}

/// Tests for the `RichWrapper` and `RichPatternWrapper` implementations.
///
/// These tests ensure that the custom `Display` implementations work as expected. Instead of
/// constructing the errors directly, we just parse invalid inputs. This is less robust but much
/// easier to maintain.
#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use chumsky::{Parser, label::LabelError};

    use crate::parser::parser;

    #[test]
    fn expected_label() {
        let errs = parser().parse(" ").into_errors();
        assert_eq!(errs.len(), 1);
        let err = errs.first().unwrap();
        assert_eq!(
            err.to_string(),
            "at 1..1: expected anything, type qualifier, or type, but found end of input"
        );
    }

    #[test]
    fn expected_one_option() {
        let errs = parser().parse("int foo[0").into_errors();
        assert_eq!(errs.len(), 1);
        let err = errs.first().unwrap();
        assert_eq!(
            err.to_string(),
            "at 9..9: expected ']', but found end of input"
        );
    }

    #[test]
    #[should_panic(
        expected = "not yet implemented: we don't use this function, so we don't implement it yet"
    )]
    fn in_context() {
        let mut errs = parser().parse("in").into_errors();
        assert_eq!(errs.len(), 1);
        let mut err = errs.swap_remove(0);
        err.in_context("lkasjdf", (1..2).into());
    }

    #[test]
    fn expected_anything() {
        let errs = parser().parse("int f(").into_errors();
        assert_eq!(errs.len(), 1);
        let err = errs.first().unwrap();
        assert_eq!(
            err.to_string(),
            "at 6..6: expected anything, function parameter, or ')', but found end of input"
        );
    }
}
