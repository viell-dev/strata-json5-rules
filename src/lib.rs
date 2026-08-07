//! JSON5 (json5.org) token rules for the Strata parser ecosystem.
//!
//! Each rule is a zero-sized [`Matcher`](strata_lexer::Matcher) recognizing one JSON5 token shape
//! from read-only lookahead, following the same measure-then-advance model as
//! `strata-json-rules`. The set is composed on top of `strata-json-rules` per its documented
//! superset pattern:
//!
//! - **Reused from JSON, lifted with [`MapMatcher`]:** [`PunctRule`] (the six structural
//!   characters are identical in JSON5) and [`LiteralRule`] (`true`/`false`/`null` are spelled
//!   exactly as in JSON). Their [`JsonToken`] kinds are wrapped as [`Json5Token::Json`].
//! - **Replaced with JSON5 shapes:** whitespace (JSON5's set is a strict superset of JSON's four
//!   bytes), strings (single or double quotes, more escapes, line continuations), and numbers
//!   (signs, hex, `Infinity`/`NaN`, leading/trailing decimal points). Where the JSON rule's shape
//!   is a strict subset of the JSON5 shape, one JSON5 rule is preferred over layering the two.
//! - **Added:** identifiers ([`IdentifierRule`]) and comments ([`LineCommentRule`],
//!   [`BlockCommentRule`]).
//! - **Dropped:** `BomRule` — U+FEFF is ordinary JSON5 whitespace ([spec] Table 3), anywhere in
//!   the input, so [`WhitespaceRule`] covers it and no BOM policy option exists.
//!
//! `true`, `false`, and `null` are also valid ECMAScript 5.1 `IdentifierName`s, so on those exact
//! words the lifted [`LiteralRule`] and [`IdentifierRule`] both match with equal length.
//! [`IdentifierRule`] declares `priority() == -1` so the literal deterministically wins the
//! maximal-munch tie; any longer word (`truex`) still lexes as an identifier.
//!
//! Rules are exported individually so supersets of JSON5 can compose further, exactly as this
//! crate composes on `strata-json-rules`. The reused JSON rule structs and [`SharedRuleSet`] are
//! re-exported, so composing on this crate does not require a direct `strata-json-rules`
//! dependency. The crate is standalone on purpose: a parser for JSON5 (or a superset language)
//! depends on these rules without pulling in a parser.
//!
//! [spec]: https://spec.json5.org/
//! [`MapMatcher`]: strata_lexer::MapMatcher

mod kind;
pub mod rules;

pub use kind::Json5Token;
pub use rules::{
    BlockCommentRule, IdentifierRule, JsonToken, LineCommentRule, LiteralRule, NumberRule,
    PunctRule, SharedRuleSet, StrictRuleSet, StringRule, WhitespaceRule, is_json5_whitespace,
    strict_rule_set, strict_scanner,
};
