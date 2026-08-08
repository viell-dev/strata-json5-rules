# strata-json5-rules

JSON5 ([json5.org](https://json5.org)) token rules for the Strata parser ecosystem, extending
the JSON rules from `strata-json-rules`.

## Status

This crate is in initial development. The public API is not settled yet.

## Usage

```rust
use std::io::Cursor;
use strata_json5_rules::{Json5Token, JsonToken, strict_scanner};
use strata_lexer::{LexError, Lexer};

fn main() -> Result<(), LexError> {
    let input = "// note\n{key: 0x1F}";
    let mut lexer = Lexer::new(Cursor::new(input), strict_scanner());
    let mut kinds = Vec::new();

    loop {
        let token = lexer.next_token()?;
        let done = token.kind == Json5Token::Eof;
        kinds.push((token.kind, token.span().start(), token.span().end()));

        if done {
            break;
        }
    }

    // The shapes JSON5 adds: comments, unquoted names, hex numbers...
    assert_eq!(kinds.first(), Some(&(Json5Token::LineComment, 0, 7)));
    assert_eq!(kinds.get(3), Some(&(Json5Token::Identifier, 9, 12)));
    assert_eq!(kinds.get(6), Some(&(Json5Token::Number, 14, 18)));

    // ...and the JSON shapes lifted from strata-json-rules, unchanged.
    assert_eq!(
        kinds.get(2),
        Some(&(Json5Token::Json(JsonToken::LeftBrace), 8, 9))
    );
    Ok(())
}
```

A superset of JSON5 lifts these rules into its own kind enum with
`strata_lexer::MatcherExt::map_kind` — the same way this crate lifts the JSON rules — and adds
only the rules that differ; see [docs/strata-json5-rules.md](docs/strata-json5-rules.md) for the
design reference and the composition pattern.

## Minimum Supported Rust Version

The MSRV is **1.87.0**.

## License

Licensed under the Mozilla Public License, Version 2.0 (see [LICENSE](LICENSE)
or <https://www.mozilla.org/en-US/MPL/2.0/>).

MPL-2.0 is a file-level weak copyleft license: its obligations attach to this crate's files (in
source and compiled form) and to modifications of them, not to your own source files that merely
use this crate. Distributing a binary that includes this crate obliges you to tell recipients
that the crate is included and how to obtain its source; for an unmodified crate, pointing them
at the exact crates.io release is a reasonable way to do that. Example material (documentation
code examples, `examples/`, integration tests, and benchmarks) is additionally licensed under
[MIT-0](LICENSE-MIT-0), so it can be copied freely. See
[LICENSE-CLARIFICATION.md](LICENSE-CLARIFICATION.md) for the maintainers' full statement of
intent regarding normal Rust usage (linking, monomorphization, macro expansion), binary
distribution, and generated code, and [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution
policy.
