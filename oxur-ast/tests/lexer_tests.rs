use oxur_ast::sexp::lexer::{Lexer, TokenType};

#[test]
fn test_empty_input() {
    let mut lexer = Lexer::new("");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].typ, TokenType::Eof);
}

#[test]
fn test_single_lparen() {
    let mut lexer = Lexer::new("(");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].typ, TokenType::LParen);
    assert_eq!(tokens[0].lexeme, "(");
    assert_eq!(tokens[1].typ, TokenType::Eof);
}

#[test]
fn test_single_rparen() {
    let mut lexer = Lexer::new(")");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].typ, TokenType::RParen);
    assert_eq!(tokens[0].lexeme, ")");
}

#[test]
fn test_matching_parens() {
    let mut lexer = Lexer::new("()");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].typ, TokenType::LParen);
    assert_eq!(tokens[1].typ, TokenType::RParen);
    assert_eq!(tokens[2].typ, TokenType::Eof);
}

#[test]
fn test_nested_parens() {
    let mut lexer = Lexer::new("(())");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].typ, TokenType::LParen);
    assert_eq!(tokens[1].typ, TokenType::LParen);
    assert_eq!(tokens[2].typ, TokenType::RParen);
    assert_eq!(tokens[3].typ, TokenType::RParen);
}

#[test]
fn test_simple_symbol() {
    let mut lexer = Lexer::new("foo");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].typ, TokenType::Symbol);
    assert_eq!(tokens[0].lexeme, "foo");
}

#[test]
fn test_multiple_symbols() {
    let mut lexer = Lexer::new("foo bar baz");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].typ, TokenType::Symbol);
    assert_eq!(tokens[0].lexeme, "foo");
    assert_eq!(tokens[1].typ, TokenType::Symbol);
    assert_eq!(tokens[1].lexeme, "bar");
    assert_eq!(tokens[2].typ, TokenType::Symbol);
    assert_eq!(tokens[2].lexeme, "baz");
}

#[test]
fn test_symbol_with_special_chars() {
    let mut lexer = Lexer::new("foo-bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Symbol);
    assert_eq!(tokens[0].lexeme, "foo-bar");

    let mut lexer = Lexer::new("foo_bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo_bar");

    let mut lexer = Lexer::new("foo+bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo+bar");

    let mut lexer = Lexer::new("foo*bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo*bar");

    let mut lexer = Lexer::new("foo/bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo/bar");

    let mut lexer = Lexer::new("foo=bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo=bar");

    let mut lexer = Lexer::new("foo<bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo<bar");

    let mut lexer = Lexer::new("foo>bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo>bar");

    let mut lexer = Lexer::new("foo!bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo!bar");

    let mut lexer = Lexer::new("foo?bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo?bar");

    let mut lexer = Lexer::new("foo&bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo&bar");

    let mut lexer = Lexer::new("foo'bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "foo'bar");
}

#[test]
fn test_keyword() {
    let mut lexer = Lexer::new(":foo");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].typ, TokenType::Keyword);
    assert_eq!(tokens[0].lexeme, "foo");
}

#[test]
fn test_multiple_keywords() {
    let mut lexer = Lexer::new(":name :kind");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].typ, TokenType::Keyword);
    assert_eq!(tokens[0].lexeme, "name");
    assert_eq!(tokens[1].typ, TokenType::Keyword);
    assert_eq!(tokens[1].lexeme, "kind");
}

#[test]
fn test_string_simple() {
    let mut lexer = Lexer::new(r#""hello""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].typ, TokenType::String);
    assert_eq!(tokens[0].lexeme, "hello");
}

#[test]
fn test_string_empty() {
    let mut lexer = Lexer::new(r#""""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::String);
    assert_eq!(tokens[0].lexeme, "");
}

#[test]
fn test_string_with_spaces() {
    let mut lexer = Lexer::new(r#""hello world""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "hello world");
}

#[test]
fn test_string_escape_sequences() {
    let mut lexer = Lexer::new(r#""hello\nworld""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "hello\nworld");

    let mut lexer = Lexer::new(r#""tab\there""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "tab\there");

    let mut lexer = Lexer::new(r#""return\rhere""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "return\rhere");

    let mut lexer = Lexer::new(r#""back\\slash""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "back\\slash");

    let mut lexer = Lexer::new(r#""quote\"here""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "quote\"here");
}

#[test]
fn test_string_unterminated() {
    let mut lexer = Lexer::new(r#""hello"#);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_string_invalid_escape() {
    let mut lexer = Lexer::new(r#""hello\x""#);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_number_positive() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Number);
    assert_eq!(tokens[0].lexeme, "42");
}

#[test]
fn test_number_zero() {
    let mut lexer = Lexer::new("0");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Number);
    assert_eq!(tokens[0].lexeme, "0");
}

#[test]
fn test_number_negative() {
    let mut lexer = Lexer::new("-42");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Number);
    assert_eq!(tokens[0].lexeme, "-42");
}

#[test]
fn test_number_large() {
    let mut lexer = Lexer::new("123456789");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "123456789");
}

#[test]
fn test_nil() {
    let mut lexer = Lexer::new("nil");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Nil);
    assert_eq!(tokens[0].lexeme, "nil");
}

#[test]
fn test_nil_in_list() {
    let mut lexer = Lexer::new("(nil)");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1].typ, TokenType::Nil);
}

#[test]
fn test_whitespace_handling() {
    let mut lexer = Lexer::new("  foo  bar  ");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].lexeme, "foo");
    assert_eq!(tokens[1].lexeme, "bar");
}

#[test]
fn test_newline_handling() {
    let mut lexer = Lexer::new("foo\nbar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].lexeme, "foo");
    assert_eq!(tokens[1].lexeme, "bar");
}

#[test]
fn test_tab_handling() {
    let mut lexer = Lexer::new("foo\tbar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].lexeme, "foo");
    assert_eq!(tokens[1].lexeme, "bar");
}

#[test]
fn test_comment_single_line() {
    let mut lexer = Lexer::new("; this is a comment\nfoo");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].lexeme, "foo");
}

#[test]
fn test_comment_end_of_line() {
    let mut lexer = Lexer::new("foo ; comment");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].lexeme, "foo");
}

#[test]
fn test_comment_only() {
    let mut lexer = Lexer::new("; just a comment");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].typ, TokenType::Eof);
}

#[test]
fn test_complex_expression() {
    let input = r#"(defn add ((a i32) (b i32)) i32
      (+ a b))"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // Should have: (, defn, add, (, (, a, i32, ), (, b, i32, ), ), i32, (, +, a, b, ), ), EOF
    assert!(tokens.len() > 10);
    assert_eq!(tokens[0].typ, TokenType::LParen);
    assert_eq!(tokens[1].lexeme, "defn");
}

#[test]
fn test_position_tracking() {
    let mut lexer = Lexer::new("foo\nbar");
    let tokens = lexer.tokenize().unwrap();

    // First token should be on line 1
    assert_eq!(tokens[0].pos.line, 1);
    assert_eq!(tokens[0].pos.column, 1);

    // Second token should be on line 2
    assert_eq!(tokens[1].pos.line, 2);
    assert_eq!(tokens[1].pos.column, 1);
}

#[test]
fn test_unexpected_character() {
    let mut lexer = Lexer::new("@");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_minus_without_number() {
    let mut lexer = Lexer::new("- ");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].typ, TokenType::Symbol);
    assert_eq!(tokens[0].lexeme, "-");
}
