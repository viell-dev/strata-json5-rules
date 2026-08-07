//! The JSON5 token kind enum.

use strata_json_rules::JsonToken;

/// Token kinds produced by the JSON5 token rules.
///
/// Kinds reused from `strata-json-rules` are wrapped verbatim: with the strict rule set, the
/// reachable [`Json5Token::Json`] values are the six structural characters and the
/// `true`/`false`/`null` literals (the JSON whitespace, string, number, BOM, and EOF kinds are
/// replaced by JSON5-owned kinds and never produced).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Json5Token {
    /// A token produced by a reused `strata-json-rules` rule: `{` `}` `[` `]` `:` `,` and the
    /// `true`/`false`/`null` literals.
    Json(JsonToken),
    /// An ECMAScript 5.1 `IdentifierName` (other than the classified words below), usable as an
    /// unquoted object member name.
    Identifier,
    /// The word `Infinity`, spelled exactly and without escapes: a number value, and — being an
    /// `IdentifierName` — also a valid unquoted member name.
    Infinity,
    /// The word `NaN`, spelled exactly and without escapes: a number value, and — being an
    /// `IdentifierName` — also a valid unquoted member name.
    NaN,
    /// A complete single- or double-quoted string, span covering both quotes.
    String,
    /// A complete number: decimal (leading/trailing decimal point allowed), hexadecimal, or a
    /// signed `Infinity`/`NaN`, with an optional leading `+` or `-`.
    Number,
    /// A run of JSON5 whitespace, including line terminators and the byte-order mark.
    Whitespace,
    /// A `//` comment, excluding the terminating line terminator.
    LineComment,
    /// A `/* ... */` comment, including both delimiters.
    BlockComment,
    /// End of input.
    Eof,
}

impl Json5Token {
    /// Returns whether this kind is trivia — grammatically insignificant source content that a
    /// fidelity-preserving CST retains instead of discarding.
    pub const fn is_trivia(self) -> bool {
        match self {
            Self::Whitespace | Self::LineComment | Self::BlockComment => true,
            Self::Json(token) => token.is_trivia(),
            _ => false,
        }
    }

    /// A short human-readable description for diagnostics.
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Json(token) => token.describe(),
            Self::Identifier => "an identifier",
            Self::Infinity => "'Infinity'",
            Self::NaN => "'NaN'",
            Self::String => "a string",
            Self::Number => "a number",
            Self::Whitespace => "whitespace",
            Self::LineComment => "a line comment",
            Self::BlockComment => "a block comment",
            Self::Eof => "end of input",
        }
    }
}
