use oxur_ast::error::{LexError, ParseError, Position};

#[test]
fn test_position_new() {
    let pos = Position::new(42, 10, 5);
    assert_eq!(pos.offset, 42);
    assert_eq!(pos.line, 10);
    assert_eq!(pos.column, 5);
}

#[test]
fn test_position_display() {
    let pos = Position::new(0, 5, 10);
    let output = format!("{}", pos);
    assert_eq!(output, "line 5, column 10");
}

#[test]
fn test_lex_error_unexpected_char() {
    let pos = Position::new(0, 1, 1);
    let error = LexError::UnexpectedChar { ch: '@', pos };
    let output = format!("{}", error);
    assert!(output.contains('@'));
    assert!(output.contains("line 1"));
}

#[test]
fn test_lex_error_unterminated_string() {
    let pos = Position::new(5, 2, 3);
    let error = LexError::UnterminatedString { pos };
    let output = format!("{}", error);
    assert!(output.contains("Unterminated"));
    assert!(output.contains("line 2"));
}

#[test]
fn test_lex_error_invalid_escape() {
    let pos = Position::new(10, 3, 5);
    let error = LexError::InvalidEscape { ch: 'x', pos };
    let output = format!("{}", error);
    assert!(output.contains("escape"));
    assert!(output.contains('x'));
}

#[test]
fn test_lex_error_unexpected_eof() {
    let error = LexError::UnexpectedEof;
    let output = format!("{}", error);
    assert!(output.contains("end of input") || output.contains("EOF"));
}

#[test]
fn test_parse_error_unexpected_token() {
    let pos = Position::new(0, 1, 1);
    let error = ParseError::UnexpectedToken {
        token: "foo".to_string(),
        pos,
    };
    let output = format!("{}", error);
    assert!(output.contains("foo"));
}

#[test]
fn test_parse_error_expected() {
    let pos = Position::new(0, 1, 1);
    let error = ParseError::Expected {
        expected: "symbol".to_string(),
        found: "number".to_string(),
        pos,
    };
    let output = format!("{}", error);
    assert!(output.contains("symbol"));
    assert!(output.contains("number"));
}

#[test]
fn test_parse_error_unterminated_list() {
    let pos = Position::new(0, 1, 1);
    let error = ParseError::UnterminatedList { pos };
    let output = format!("{}", error);
    assert!(output.contains("Unterminated"));
    assert!(output.contains("list"));
}

#[test]
fn test_parse_error_unexpected_close_paren() {
    let pos = Position::new(0, 1, 1);
    let error = ParseError::UnexpectedCloseParen { pos };
    let output = format!("{}", error);
    assert!(output.contains("closing") || output.contains("parenthesis"));
}

#[test]
fn test_parse_error_empty_input() {
    let error = ParseError::EmptyInput;
    let output = format!("{}", error);
    assert!(output.contains("Empty"));
}

#[test]
fn test_parse_error_from_lex_error() {
    let pos = Position::new(0, 1, 1);
    let lex_error = LexError::UnexpectedChar { ch: '@', pos };
    let parse_error: ParseError = lex_error.into();
    let output = format!("{}", parse_error);
    assert!(output.contains("Lexer"));
}
