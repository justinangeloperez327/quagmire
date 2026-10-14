use quagmire_compiler::lexer::{Keyword as K, LexMode as M, Lexer, Symbol as S, TokenKind as T};
use quagmire_compiler::{SourceMap, Token};

fn kinds(text: &str, mode: M) -> Vec<T> {
    let mut sources = SourceMap::new();
    let id = sources.add("lexer.qg", text);
    let mut lexer = Lexer::new(&sources, id).unwrap();
    let mut result = Vec::new();
    loop {
        let token = lexer.next_token(mode).unwrap();
        if token.kind == T::Eof {
            break;
        }
        result.push(token.kind);
    }
    result
}

fn error_code(text: &str, mode: M) -> String {
    let mut sources = SourceMap::new();
    let id = sources.add("invalid.qg", text);
    let mut lexer = Lexer::new(&sources, id).unwrap();
    loop {
        match lexer.next_token(mode) {
            Err(error) => {
                sources.slice(error.span).unwrap();
                assert!(!error.render(&sources).unwrap().is_empty());
                return error.code;
            }
            Ok(token) => assert_ne!(token.kind, T::Eof, "expected failure for {text:?}"),
        }
    }
}

#[test]
fn keyword_boundaries_comments_and_longest_operators() {
    assert_eq!(
        kinds(
            "state stateful = 2; // comment\r\n/* é🙂 */ computed yes = stateful >= 1 && true !== false;",
            M::Code
        ),
        vec![
            T::Keyword(K::State),
            T::Identifier("stateful".into()),
            T::Symbol(S::Assign),
            T::Number(2.0),
            T::Symbol(S::Semicolon),
            T::Keyword(K::Computed),
            T::Identifier("yes".into()),
            T::Symbol(S::Assign),
            T::Identifier("stateful".into()),
            T::Symbol(S::GreaterEqual),
            T::Number(1.0),
            T::Symbol(S::And),
            T::Keyword(K::True),
            T::Symbol(S::NotEqual),
            T::Keyword(K::False),
            T::Symbol(S::Semicolon),
        ]
    );
    assert_eq!(
        kinds("=== !== <= >= && || < > + - * / % ! =", M::Code),
        vec![
            T::Symbol(S::Equal),
            T::Symbol(S::NotEqual),
            T::Symbol(S::LessEqual),
            T::Symbol(S::GreaterEqual),
            T::Symbol(S::And),
            T::Symbol(S::Or),
            T::Symbol(S::Less),
            T::Symbol(S::Greater),
            T::Symbol(S::Plus),
            T::Symbol(S::Minus),
            T::Symbol(S::Star),
            T::Symbol(S::Slash),
            T::Symbol(S::Percent),
            T::Symbol(S::Not),
            T::Symbol(S::Assign),
        ]
    );
}

#[test]
fn punctuation_and_all_reserved_words() {
    assert_eq!(
        kinds("(){}[];,:.", M::Code),
        vec![
            T::Symbol(S::LeftParen),
            T::Symbol(S::RightParen),
            T::Symbol(S::LeftBrace),
            T::Symbol(S::RightBrace),
            T::Symbol(S::LeftBracket),
            T::Symbol(S::RightBracket),
            T::Symbol(S::Semicolon),
            T::Symbol(S::Comma),
            T::Symbol(S::Colon),
            T::Symbol(S::Dot)
        ]
    );
    let words = kinds(
        "component state computed function view if else for in return import export from let const true false null",
        M::Code,
    );
    assert_eq!(words.len(), 18);
    assert!(words.iter().all(|kind| matches!(kind, T::Keyword(_))));
}

#[test]
fn numbers_have_explicit_signs_and_no_silent_coercion() {
    assert_eq!(
        kinds("0 0.5 0e2 12.25e-2 -2 +3 4.name", M::Code),
        vec![
            T::Number(0.0),
            T::Number(0.5),
            T::Number(0.0),
            T::Number(0.1225),
            T::Symbol(S::Minus),
            T::Number(2.0),
            T::Symbol(S::Plus),
            T::Number(3.0),
            T::Number(4.0),
            T::Symbol(S::Dot),
            T::Identifier("name".into()),
        ]
    );
    for value in [
        "01", "00.1", "1e", "1e+", "1e999", "0x10", "1_000", "1thing",
    ] {
        assert_eq!(error_code(value, M::Code), "QG1004", "{value}");
    }
}

#[test]
fn strings_decode_escapes_and_unicode_without_executing() {
    assert_eq!(
        kinds(
            r#""é\n\t\u0041\uD83D\uDE42" 'it\'s' "<script>" "\\\"""#,
            M::Code
        ),
        vec![
            T::String("é\n\tA🙂".into()),
            T::String("it's".into()),
            T::String("<script>".into()),
            T::String("\\\"".into()),
        ]
    );
    for value in [
        r#""\x41""#,
        r#""\uZZZZ""#,
        r#""\uD800""#,
        r#""\uDC00""#,
        r#""\uD800\u0041""#,
    ] {
        assert_eq!(error_code(value, M::Code), "QG1003");
    }
    for value in ["\"unfinished", "'line\nbreak'", "\"\\"] {
        assert_eq!(error_code(value, M::Code), "QG1002");
    }
}

#[test]
fn unsupported_characters_and_operators_are_diagnostics() {
    for value in [
        "==", "!=", "++", "--", "+=", "=>", "??", "?.", "...", "<<", "&", "`text`", "é",
    ] {
        assert_eq!(error_code(value, M::Code), "QG1001");
    }
    assert_eq!(error_code("/* unfinished", M::Code), "QG1005");
    assert_eq!(
        kinds("/* not /* nested */ name", M::Code),
        vec![T::Identifier("name".into())]
    );
}

#[test]
fn tag_names_are_not_code_keywords_and_values_do_not_decode_entities() {
    assert_eq!(
        kinds(r#"button on:click={"#, M::Tag),
        vec![
            T::Name("button".into()),
            T::Name("on:click".into()),
            T::Symbol(S::Assign),
            T::Symbol(S::LeftBrace),
        ]
    );
    assert_eq!(
        kinds(r#"for aria-hidden="false" data-id="&amp;" />"#, M::Tag),
        vec![
            T::Name("for".into()),
            T::Name("aria-hidden".into()),
            T::Symbol(S::Assign),
            T::String("false".into()),
            T::Name("data-id".into()),
            T::Symbol(S::Assign),
            T::String("&amp;".into()),
            T::SelfCloseTagEnd,
        ]
    );
    for value in ["on:", "on::click", "// comment", "@name"] {
        assert_eq!(error_code(value, M::Tag), "QG1006");
    }
}

#[test]
fn text_entities_remain_one_text_token_not_markup() {
    assert_eq!(
        kinds(
            "&lt;script&gt; &amp; &quot; &apos; &#123; &#125; &#x1F642;",
            M::Text
        ),
        vec![T::Text("<script> & \" ' { } 🙂".into()),]
    );
    for value in [
        "&unknown;",
        "&amp",
        "&#;",
        "&#x;",
        "&#xD800;",
        "&#1114112;",
        "&#999999999999;",
    ] {
        assert_eq!(error_code(value, M::Text), "QG1007");
    }
    assert_eq!(
        kinds("// literal /* literal */", M::Text),
        vec![T::Text("// literal /* literal */".into())]
    );
}

#[test]
fn view_whitespace_and_directives_follow_node_boundaries() {
    assert_eq!(kinds("\n   ", M::Text), vec![]);
    assert_eq!(kinds(" ", M::Text), vec![T::Text(" ".into())]);
    assert_eq!(
        kinds("\n  Hello\n  ", M::Text),
        vec![T::Text("\n  Hello\n  ".into())]
    );
    assert_eq!(
        kinds("Hello if (ready)", M::Text),
        vec![T::Text("Hello if (ready)".into())]
    );
    assert_eq!(
        kinds("ifonly (ready)", M::Text),
        vec![T::Text("ifonly (ready)".into())]
    );
    let mut sources = SourceMap::new();
    let id = sources.add("view.qg", "\n  if (ready)");
    let mut lexer = Lexer::new(&sources, id).unwrap();
    assert_eq!(lexer.next_token(M::Text).unwrap().kind, T::Keyword(K::If));
    assert_eq!(
        lexer.next_token(M::Code).unwrap().kind,
        T::Symbol(S::LeftParen)
    );
    assert_eq!(
        lexer.next_token(M::Code).unwrap().kind,
        T::Identifier("ready".into())
    );
    assert_eq!(
        lexer.next_token(M::Code).unwrap().kind,
        T::Symbol(S::RightParen)
    );
}

#[test]
fn modes_keep_nested_objects_strings_and_comparisons_out_of_view_scanning() {
    let mut sources = SourceMap::new();
    let text =
        r#"<button title={{ name: "}" }.name} on:click={increment}>Count: {count < 2}</button>"#;
    let id = sources.add("view.qg", text);
    let mut lexer = Lexer::new(&sources, id).unwrap();
    let expected = [
        (M::Text, T::OpenTag),
        (M::Tag, T::Name("button".into())),
        (M::Tag, T::Name("title".into())),
        (M::Tag, T::Symbol(S::Assign)),
        (M::Tag, T::Symbol(S::LeftBrace)),
        (M::Code, T::Symbol(S::LeftBrace)),
        (M::Code, T::Identifier("name".into())),
        (M::Code, T::Symbol(S::Colon)),
        (M::Code, T::String("}".into())),
        (M::Code, T::Symbol(S::RightBrace)),
        (M::Code, T::Symbol(S::Dot)),
        (M::Code, T::Identifier("name".into())),
        (M::Code, T::Symbol(S::RightBrace)),
        (M::Tag, T::Name("on:click".into())),
        (M::Tag, T::Symbol(S::Assign)),
        (M::Tag, T::Symbol(S::LeftBrace)),
        (M::Code, T::Identifier("increment".into())),
        (M::Code, T::Symbol(S::RightBrace)),
        (M::Tag, T::TagEnd),
        (M::Text, T::Text("Count: ".into())),
        (M::Text, T::Symbol(S::LeftBrace)),
        (M::Code, T::Identifier("count".into())),
        (M::Code, T::Symbol(S::Less)),
        (M::Code, T::Number(2.0)),
        (M::Code, T::Symbol(S::RightBrace)),
        (M::Text, T::CloseTag),
        (M::Tag, T::Name("button".into())),
        (M::Tag, T::TagEnd),
        (M::Code, T::Eof),
    ];
    let mut last_end = 0;
    for (mode, kind) in expected {
        let token: Token = lexer.next_token(mode).unwrap();
        assert_eq!(token.kind, kind);
        assert!(token.span.range().start >= last_end);
        last_end = token.span.range().end;
        sources.slice(token.span).unwrap();
    }
    assert_eq!(last_end, text.len());
}

#[test]
fn lookahead_is_non_consuming_even_on_errors_and_eof_is_stable() {
    let mut sources = SourceMap::new();
    let id = sources.add("peek.qg", "  \"é🙂\"  ");
    let mut lexer = Lexer::new(&sources, id).unwrap();
    let peeked = lexer.peek(M::Code).unwrap();
    assert_eq!(lexer.offset(), 0);
    let token = lexer.next_token(M::Code).unwrap();
    assert_eq!(token, peeked);
    assert_eq!(sources.slice(token.span).unwrap(), "\"é🙂\"");
    let eof = lexer.next_token(M::Code).unwrap();
    assert_eq!(eof.kind, T::Eof);
    assert_eq!(eof, lexer.next_token(M::Code).unwrap());
    assert_eq!(eof.span.range(), 12..12);
    let bad = sources.add("bad.qg", "@");
    let lexer = Lexer::new(&sources, bad).unwrap();
    assert!(lexer.peek(M::Code).is_err());
    assert_eq!(lexer.offset(), 0);
}
