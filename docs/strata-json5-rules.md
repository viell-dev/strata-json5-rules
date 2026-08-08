# strata-json5-rules

`strata-json5-rules` is the JSON5 ([json5.org](https://json5.org), specified at
<https://spec.json5.org/>) token-rules crate for the Strata parser ecosystem, composed on
`strata-json-rules` per its documented superset pattern: the JSON rules whose shapes JSON5 keeps
unchanged are reused — lifted into a wider kind enum — the ones JSON5 widens are replaced by one
JSON5-owned rule each, and the shapes JSON5 adds (identifiers, comments) get new rules. It is
standalone on purpose — a parser for JSON5 (or a superset of it) depends on just these rules and
adds its own, without pulling in a parser.

The crate sits at the rules layer of the per-language stack:

```text
strata-json-rules -> strata-json5-rules (this crate) -> strata-json5-parser -> strata-json5
```

- **Edition / MSRV:** Rust 2024, MSRV `1.87.0`.
- **License:** MPL-2.0.
- **Dependencies:** `strata-json-rules` and `strata-lexer` (which bring `strata-reader` and
  `strata-lease`).

## 1. Module and public-API map

| Module           | Public items |
|------------------|--------------|
| `kind`           | `Json5Token` (with `is_trivia()`, `describe()`) |
| `rules`          | `WhitespaceRule`, `StrictRuleSet`, `strict_rule_set()`, `strict_scanner()`, `is_json5_whitespace()` |
| `rules::comment` | `LineCommentRule`, `BlockCommentRule` |
| `rules::ident`   | `IdentifierRule`, `is_identifier_start()`, `is_identifier_continue()` (the Unicode approximation lives here) |
| `rules::number`  | `NumberRule` |
| `rules::string`  | `StringRule` |

Everything is re-exported at the crate root except the `rules::ident` classifier functions
(reachable at their module path), alongside `strata_json_rules::{JsonToken, PunctRule,
LiteralRule, SharedRuleSet}` — so downstream code can name the kinds carried inside `Json5Token`
and compose on this crate without depending on `strata-json-rules` directly (`SharedRuleSet` is
kind-generic upstream, so it wraps JSON5 rule sets too).

## 2. Token kinds

`Json5Token` wraps `strata-json-rules`' kind where rules are reused: with the strict set, the
reachable `Json(JsonToken)` values are exactly the six structural characters and the
`true`/`false`/`null` literals — the JSON whitespace, string, number, BOM, and EOF kinds are
replaced by JSON5-owned kinds and never produced. The JSON5-owned kinds are `Identifier`,
`Infinity`, `NaN`, `String`, `Number`, `Whitespace`, `LineComment`, `BlockComment`, and `Eof`.
`Whitespace` and both comment kinds are trivia (`is_trivia()`). A further superset wraps
`Json5Token` in its own kind the same way this crate wraps `JsonToken`.

## 3. The rules

The composition over `strata-json-rules`, shape by shape:

- **Reused, lifted with `MatcherExt::map_kind`:** `PunctRule` (the six structural characters are
  identical in JSON5) and `LiteralRule` (`true`/`false`/`null`, spelled exactly as in JSON).
  Reused rules keep their upstream validation.
- **Replaced by one JSON5 rule each:** whitespace, strings, and numbers. Each JSON shape is a
  *strict subset* of its JSON5 counterpart (every RFC 8259 string is a valid JSON5 string, and
  so on), so per the ecosystem's composition guidance one JSON5 rule is preferred over layering
  the JSON rule under an extension rule.
- **Added:** `IdentifierRule`, `LineCommentRule`, `BlockCommentRule`. The comment rules are
  separate zero-sized matchers so a superset or sibling dialect can pick either independently.
- **Dropped:** `BomRule` — U+FEFF is ordinary JSON5 whitespace (spec Table 3), anywhere in the
  input, so `WhitespaceRule` covers it and there is no BOM rule, no BOM policy, and no
  `allow_bom` option anywhere in the JSON5 stack.

Two lexing choices worth knowing:

- `true`/`false`/`null` are also valid ES 5.1 `IdentifierName`s, so the lifted `LiteralRule` and
  `IdentifierRule` tie on those exact words. `IdentifierRule` declares `priority() == -1`, so
  the literal wins the maximal-munch tie deterministically; longer words (`truex`) lex as
  identifiers by ordinary maximal munch.
- Bare `Infinity`/`NaN` lex through `IdentifierRule`, which classifies the exact words into the
  dedicated kinds `Json5Token::Infinity`/`NaN`; `NumberRule` matches them only *after an
  explicit sign* (`+Infinity`, `-NaN`). Reason: they are simultaneously number values and valid
  member names; lexing the bare words as `Number` would force the parser to accept arbitrary
  numbers in name position. The classification compares *raw source bytes*: an escaped spelling
  such as `Infinit\u0079` is an ordinary identifier (valid as a member name, but not a number
  value — the grammar requires the literal characters).

The JSON5-owned shapes:

- `WhitespaceRule` — a run of the `is_json5_whitespace` set (spec Table 3): tab, LF, VT, FF, CR,
  space, NBSP, U+2028, U+2029, U+FEFF, and the Unicode `Zs` category (hardcoded from Unicode
  16.0: U+1680, U+2000–U+200A, U+202F, U+205F, U+3000). Line terminators are inter-token
  whitespace in JSON5.
- `IdentifierRule` — ES 5.1 `IdentifierName`, with `\uXXXX` escapes allowed anywhere in the name
  and validated per position exactly as ES 5.1 requires (`1` cannot *begin* an identifier).
  An invalid escape or character simply ends the match; a backslash no rule covers then surfaces
  as an unrecognized byte. Non-ASCII classification is approximate (§5).
- `StringRule` — `'...'` or `"..."`; the other quote kind is ordinary content. Escapes: the
  single-escape characters `' " \ b f n r t v`, `\0` (not followed by a decimal digit), `\xHH`,
  `\uHHHH`, line continuations (`\` before LF, CR, CRLF, U+2028, or U+2029), and any other
  non-digit character after the backslash (ES 5.1 `NonEscapeCharacter`, covering JSON's `\/`).
  Unescaped LF/CR are lexical errors; unescaped U+2028/U+2029 are *legal* content (the parser
  records the spec-mandated warning), and so are raw control characters. `\` + digit 1–9 and
  malformed `\xHH`/`\uHHHH` are errors at the backslash; an unclosed string is
  `UnterminatedDelimiter` at the opening quote. Lone surrogate escapes are lexically valid, as
  upstream; decoding stays at the resolver boundary.
- `NumberRule` — optional `+`/`-`, then decimal (leading/trailing decimal point allowed, no
  leading zeros), `0x`/`0X` hex integer, or (signed only) `Infinity`/`NaN`. `08`, `1e`, and `0x`
  without digits are lexical errors at the offending byte; a bare sign or `.` matches nothing.
  Hex floats are not merged: the rule measures `0x1` and the parser rejects the adjacent `.5`.
- `LineCommentRule` — `//` up to (excluding) a line terminator (LF, CR, U+2028, U+2029) or end
  of input; the terminator lexes as ordinary whitespace, keeping blank-line counting uniform.
  `BlockCommentRule` — `/*` to the first `*/` (no nesting), both delimiters in the span; an
  unclosed comment is `UnterminatedDelimiter` at the opening `/*`. Comment and string bodies are
  validated as UTF-8, the same policy as upstream.

Lexical errors carry exact positions. Inside tokens that can span lines (strings with
continuations, block comments) the position is computed by walking the bytes with the lexer
core's own rules — CR, LF, and CRLF each count one line break; U+2028/U+2029 do *not* advance
the diagnostic line, mirroring the byte-oriented core exactly.

`StrictRuleSet` is a named struct with a manual `MatcherSet` impl — fully monomorphic, `Clone` +
`Copy` — rather than a matcher tuple: the lexer's tuple impls stop at six elements and this set
has eight rules. `strict_rule_set()` constructs it and `strict_scanner()` wraps it in a
`SpecScanner` with `Json5Token::Eof`.

## 4. Composition points for further supersets

The lifting chain continues one level up: a superset of JSON5 defines its own kind enum wrapping
`Json5Token`, lifts each kept rule, and adds or replaces only what differs. The JSON5-owned
rules lift once; the reused JSON rules lift twice:

```rust
enum DialectToken { Json5(Json5Token), HashComment, Eof }

let wrap: fn(Json5Token) -> DialectToken = DialectToken::Json5;

let rules: MatcherVec<DialectToken> = vec![
    HashCommentRule.boxed(),                              // the shape the dialect adds
    WhitespaceRule.map_kind(wrap).boxed(),                // JSON5's rules, lifted once
    IdentifierRule.map_kind(wrap).boxed(),
    StringRule.map_kind(wrap).boxed(),
    NumberRule.map_kind(wrap).boxed(),
    PunctRule.map_kind(Json5Token::Json).map_kind(wrap).boxed(), // JSON's, lifted twice
    LiteralRule.map_kind(Json5Token::Json).map_kind(wrap).boxed(),
    // strata-json5-rules' comment rules deliberately omitted: this dialect uses `#` only.
];
```

Rules to change are simply not included; reused rules keep their validation and priorities at
any lifting depth — `08` stays a lexical error and the literal/identifier tie-break survives two
lifts. `tests/composition.rs` is the worked example.

## 5. Unicode identifier approximation

ES 5.1 defines identifier characters by Unicode general category (`Lu Ll Lt Lm Lo Nl` to start;
plus `Mn Mc Nd Pc`, ZWNJ, ZWJ to continue). `std` does not expose general categories and the
crate adds no dependencies, so non-ASCII classification approximates; ASCII and `\uXXXX` escape
handling (including per-position validation of the escaped character) are exact. Deviations:

- **Start** uses `char::is_alphabetic` (`Alphabetic` = `Lu Ll Lt Lm Lo Nl` +
  `Other_Alphabetic`): accepts everything ES 5.1 accepts; over-accepts `Other_Alphabetic`
  (mostly combining marks, e.g. U+0903) that ES 5.1 allows only in continuation position.
- **Continue** uses `is_alphabetic || is_numeric`, plus a hardcoded `Pc` table and ZWNJ/ZWJ:
  over-accepts `No` (e.g. `²`); under-accepts `Mn`/`Mc` marks that are not `Other_Alphabetic`
  (e.g. combining U+0301 — precomposed `é` is fine).

The classifier is two public functions (`rules::ident::is_identifier_start` /
`is_identifier_continue`) with the non-ASCII fallback isolated in two private functions, so an
exact category table can drop in without touching the rule.

## 6. Design choices

- **Reuse by lifting where identical, replace where wider.** Shapes JSON5 keeps unchanged are
  the upstream rule structs behind a kind-mapping wrapper, so JSON-level fixes propagate; shapes
  JSON5 widens are one JSON5 rule each, because layering a strict-subset rule under an extension
  rule buys nothing and splits one token shape across two matchers.
- **Validation at the lexical boundary.** String/number/comment shape violations are caught
  here with exact positions, as upstream; a parsed CST is known-well-formed without a decoding
  pass. Escape decoding and numeric conversion stay downstream, served by
  `strata-json5-parser`'s `decode` module.
- **Word-kind classification over grammar guessing.** `Infinity`/`NaN` (and the
  literal-vs-identifier tie) are resolved in the rules with dedicated kinds and a priority,
  so the parser never inspects lexeme bytes to decide a token's role.
- **Rules standalone, policy downstream.** As upstream: the set describes which shapes exist.
  JSON5 leaves less policy than JSON — there is no BOM option at all — but depth limits and
  the U+2028/U+2029 warnings are still parser concerns.

## 7. Quality bar

Same as the ecosystem: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D
warnings` under the strict lint set in `Cargo.toml`, and `cargo test` all clean. Lint exceptions
use `#[expect(lint, reason = "…")]` on the smallest existing scope.

## 8. Limitations and follow-up

- Unicode identifier classification is approximate outside ASCII (§5) pending an exact table.
- The public API is not settled; this is an initial-development crate on an experimental branch.
- Package verification (`cargo package`) is blocked until `strata-json-rules` and `strata-lexer`
  are published and the path dependencies gain versions.
