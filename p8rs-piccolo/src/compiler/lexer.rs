use alloc::vec::Vec;
use core::fmt;
use gc_arena::Collect;
use thiserror::Error;
use p8rs_types::p8num::P8Num;
use p8rs_types::p8scii::{self, LossyIteratorEx};
use crate::compiler::string_utils::{is_bin_digit, is_hex_digit};
use crate::peek_nth::{IteratorExt, PeekableNth};
use super::{
    string_utils::{is_alpha, is_digit, is_newline},
    StringInterner,
};

#[derive(Clone)]
pub enum Token<S> {
    Break,
    Do,
    Else,
    ElseIf,
    End,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    For,
    While,
    Repeat,
    Until,
    Return,
    Then,
    True,
    False,
    Not,
    And,
    Or,
    Minus,
    Add,
    Mul,
    Div,
    IDiv,
    Pow,
    Len,
    Peek,
    Peek2Mod,
    Peek4,
    Print,
    BitNotXor,
    BitAnd,
    BitOr,
    BitXor,
    ShiftRightArithmetic,
    ShiftRightLogical,
    ShiftLeft,
    RotateRight,
    RotateLeft,
    Concat,
    Dots,
    Assign,
    AssignAdd,
    AssignSub,
    AssignMul,
    AssignDiv,
    AssignIDiv,
    AssignMod,
    AssignPow,
    AssignBitAnd,
    AssignBitOr,
    AssignBitXor,
    AssignShiftRightArithmetic,
    AssignShiftRightLogical,
    AssignShiftLeft,
    AssignRotateRight,
    AssignRotateLeft,
    AssignConcat,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equal,
    NotEqual,
    Dot,
    SemiColon,
    Colon,
    DoubleColon,
    Comma,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Number(P8Num),
    Name(S),
    String(S),
}

impl<S: AsRef<[u8]>> PartialEq for Token<S> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Break, Token::Break) => true,
            (Token::Do, Token::Do) => true,
            (Token::Else, Token::Else) => true,
            (Token::ElseIf, Token::ElseIf) => true,
            (Token::End, Token::End) => true,
            (Token::Function, Token::Function) => true,
            (Token::Goto, Token::Goto) => true,
            (Token::If, Token::If) => true,
            (Token::In, Token::In) => true,
            (Token::Local, Token::Local) => true,
            (Token::Nil, Token::Nil) => true,
            (Token::For, Token::For) => true,
            (Token::While, Token::While) => true,
            (Token::Repeat, Token::Repeat) => true,
            (Token::Until, Token::Until) => true,
            (Token::Return, Token::Return) => true,
            (Token::Then, Token::Then) => true,
            (Token::True, Token::True) => true,
            (Token::False, Token::False) => true,
            (Token::Not, Token::Not) => true,
            (Token::And, Token::And) => true,
            (Token::Or, Token::Or) => true,
            (Token::Minus, Token::Minus) => true,
            (Token::Add, Token::Add) => true,
            (Token::Mul, Token::Mul) => true,
            (Token::Div, Token::Div) => true,
            (Token::IDiv, Token::IDiv) => true,
            (Token::Pow, Token::Pow) => true,
            (Token::Len, Token::Len) => true,
            (Token::Peek, Token::Peek) => true,
            (Token::Peek2Mod, Token::Peek2Mod) => true,
            (Token::Peek4, Token::Peek4) => true,
            (Token::Print, Token::Print) => true,
            (Token::BitNotXor, Token::BitNotXor) => true,
            (Token::BitAnd, Token::BitAnd) => true,
            (Token::BitOr, Token::BitOr) => true,
            (Token::BitXor, Token::BitXor) => true,
            (Token::ShiftRightArithmetic, Token::ShiftRightArithmetic) => true,
            (Token::ShiftRightLogical, Token::ShiftRightLogical) => true,
            (Token::ShiftLeft, Token::ShiftLeft) => true,
            (Token::RotateRight, Token::RotateRight) => true,
            (Token::RotateLeft, Token::RotateLeft) => true,
            (Token::Concat, Token::Concat) => true,
            (Token::Dots, Token::Dots) => true,
            (Token::Assign, Token::Assign) => true,
            (Token::AssignAdd, Token::AssignAdd) => true,
            (Token::AssignSub, Token::AssignSub) => true,
            (Token::AssignMul, Token::AssignMul) => true,
            (Token::AssignDiv, Token::AssignDiv) => true,
            (Token::AssignIDiv, Token::AssignIDiv) => true,
            (Token::AssignMod, Token::AssignMod) => true,
            (Token::AssignPow, Token::AssignPow) => true,
            (Token::AssignBitAnd, Token::AssignBitAnd) => true,
            (Token::AssignBitOr, Token::AssignBitOr) => true,
            (Token::AssignBitXor, Token::AssignBitXor) => true,
            (Token::AssignShiftRightArithmetic, Token::AssignShiftRightArithmetic) => true,
            (Token::AssignShiftRightLogical, Token::AssignShiftRightLogical) => true,
            (Token::AssignShiftLeft, Token::AssignShiftLeft) => true,
            (Token::AssignRotateRight, Token::AssignRotateRight) => true,
            (Token::AssignRotateLeft, Token::AssignRotateLeft) => true,
            (Token::AssignConcat, Token::AssignConcat) => true,
            (Token::LessThan, Token::LessThan) => true,
            (Token::LessEqual, Token::LessEqual) => true,
            (Token::GreaterThan, Token::GreaterThan) => true,
            (Token::GreaterEqual, Token::GreaterEqual) => true,
            (Token::Equal, Token::Equal) => true,
            (Token::NotEqual, Token::NotEqual) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::SemiColon, Token::SemiColon) => true,
            (Token::Colon, Token::Colon) => true,
            (Token::DoubleColon, Token::DoubleColon) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::LeftParen, Token::LeftParen) => true,
            (Token::RightParen, Token::RightParen) => true,
            (Token::LeftBracket, Token::LeftBracket) => true,
            (Token::RightBracket, Token::RightBracket) => true,
            (Token::LeftBrace, Token::LeftBrace) => true,
            (Token::RightBrace, Token::RightBrace) => true,
            (Token::Number(a), Token::Number(b)) => a == b,
            (Token::Name(a), Token::Name(b)) => a.as_ref() == b.as_ref(),
            (Token::String(a), Token::String(b)) => a.as_ref() == b.as_ref(),
            _ => false,
        }
    }
}

impl<S: AsRef<[u8]>> fmt::Debug for Token<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Break => write!(f, "Break"),
            Token::Do => write!(f, "Do"),
            Token::Else => write!(f, "Else"),
            Token::ElseIf => write!(f, "ElseIf"),
            Token::End => write!(f, "End"),
            Token::Function => write!(f, "Function"),
            Token::Goto => write!(f, "Goto"),
            Token::If => write!(f, "If"),
            Token::In => write!(f, "In"),
            Token::Local => write!(f, "Local"),
            Token::Nil => write!(f, "Nil"),
            Token::For => write!(f, "For"),
            Token::While => write!(f, "While"),
            Token::Repeat => write!(f, "Repeat"),
            Token::Until => write!(f, "Until"),
            Token::Return => write!(f, "Return"),
            Token::Then => write!(f, "Then"),
            Token::True => write!(f, "True"),
            Token::False => write!(f, "False"),
            Token::Not => write!(f, "Not"),
            Token::And => write!(f, "And"),
            Token::Or => write!(f, "Or"),
            Token::Minus => write!(f, "Minus"),
            Token::Add => write!(f, "Add"),
            Token::Mul => write!(f, "Mul"),
            Token::Div => write!(f, "Div"),
            Token::IDiv => write!(f, "IDiv"),
            Token::Pow => write!(f, "Pow"),
            Token::Len => write!(f, "Len"),
            Token::Peek => write!(f, "Peek"),
            Token::Peek2Mod => write!(f, "Peek2Mod"),
            Token::Peek4 => write!(f, "Peek4"),
            Token::Print => write!(f, "Print"),
            Token::BitNotXor => write!(f, "BitNotXor"),
            Token::BitAnd => write!(f, "BitAnd"),
            Token::BitOr => write!(f, "BitOr"),
            Token::BitXor => write!(f, "BitXor"),
            Token::ShiftRightArithmetic => write!(f, "ShiftRightArithmetic"),
            Token::ShiftRightLogical => write!(f, "ShiftRightLogical"),
            Token::ShiftLeft => write!(f, "ShiftLeft"),
            Token::RotateRight => write!(f, "RotateRight"),
            Token::RotateLeft => write!(f, "RotateLeft"),
            Token::Concat => write!(f, "Concat"),
            Token::Dots => write!(f, "Dots"),
            Token::Assign => write!(f, "Assign"),
            Token::AssignAdd => write!(f, "AssignAdd"),
            Token::AssignSub => write!(f, "AssignSub"),
            Token::AssignMul => write!(f, "AssignMul"),
            Token::AssignDiv => write!(f, "AssignDiv"),
            Token::AssignIDiv => write!(f, "AssignIDiv"),
            Token::AssignMod => write!(f, "AssignMod"),
            Token::AssignPow => write!(f, "AssignPow"),
            Token::AssignBitAnd => write!(f, "AssignBitAnd"),
            Token::AssignBitOr => write!(f, "AssignBitOr"),
            Token::AssignBitXor => write!(f, "AssignBitXor"),
            Token::AssignShiftRightArithmetic => write!(f, "AssignShiftRight"),
            Token::AssignShiftRightLogical => write!(f, "AssignShiftRightLogical"),
            Token::AssignShiftLeft => write!(f, "AssignShiftLeft"),
            Token::AssignRotateRight => write!(f, "AssignRotateRight"),
            Token::AssignRotateLeft => write!(f, "AssignRotateLeft"),
            Token::AssignConcat => write!(f, "AssignConcat"),
            Token::LessThan => write!(f, "LessThan"),
            Token::LessEqual => write!(f, "LessEqual"),
            Token::GreaterThan => write!(f, "GreaterThan"),
            Token::GreaterEqual => write!(f, "GreaterEqual"),
            Token::Equal => write!(f, "Equal"),
            Token::NotEqual => write!(f, "NotEqual"),
            Token::Dot => write!(f, "Dot"),
            Token::SemiColon => write!(f, "SemiColon"),
            Token::Colon => write!(f, "Colon"),
            Token::DoubleColon => write!(f, "T_PAAMAYIM_NEKUDOTAYIM"),
            Token::Comma => write!(f, "Comma"),
            Token::LeftParen => write!(f, "LeftParen"),
            Token::RightParen => write!(f, "RightParen"),
            Token::LeftBracket => write!(f, "LeftBracket"),
            Token::RightBracket => write!(f, "RightBracket"),
            Token::LeftBrace => write!(f, "LeftBrace"),
            Token::RightBrace => write!(f, "RightBrace"),
            Token::Number(d) => write!(f, "{:?}", d),
            Token::Name(n) => write!(f, "Name({})", p8scii::Printable(n.as_ref())),
            Token::String(s) => write!(f, "String(\"{:?}\")", p8scii::Printable(s.as_ref())),
        }
    }
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LexError {
    #[error("short string not finished, expected matching {}", p8scii::to_char(*.0))]
    UnfinishedShortString(u8),
    #[error("unexpected character: {}", p8scii::to_char(*.0))]
    UnexpectedCharacter(u8),
    #[error("hexadecimal digit expected")]
    HexDigitExpected,
    #[error("missing '{{' in \\u{{xxxx}} escape")]
    EscapeUnicodeStart,
    #[error("missing '}}' in \\u{{xxxx}} escape")]
    EscapeUnicodeEnd,
    #[error("invalid unicode value in \\u{{xxxx}} escape")]
    EscapeUnicodeInvalid,
    #[error("\\ddd escape out of 0-255 range")]
    EscapeDecimalTooLarge,
    #[error("invalid escape sequence")]
    InvalidEscape,
    #[error("invalid long string delimiter")]
    InvalidLongStringDelimiter,
    #[error("unfinished long string")]
    UnfinishedLongString,
    #[error("malformed number")]
    BadNumber,
}

/// A 0-indexed line number of the current source input.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Collect)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[collect(require_static)]
pub struct LineNumber(pub u64);

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", u128::from(self.0) + 1)
    }
}

type LexerSource<'a> = PeekableNth<impl Iterator<Item = u8> + 'a, 4>;

pub struct Lexer<'a, S> {
    source: LexerSource<'a>,
    interner: S,
    string_buffer: Vec<u8>,
    line_number: u64,
}

impl<'a, S> Lexer<'a, S>
where
    S: StringInterner,
{
    #[define_opaque(LexerSource)]
    pub fn new(source: &[u8], interner: S) -> Lexer<'_, S> {
        Lexer {
            source: p8scii::from_utf8(source).lossy().peekable_nth(),
            interner,
            string_buffer: Vec::new(),
            line_number: 0,
        }
    }
    
    /// Current line number of the source file.
    pub fn line_number(&self) -> LineNumber {
        LineNumber(self.line_number)
    }
    
    pub fn interner_mut(&mut self) -> &mut S {
        &mut self.interner
    }

    pub fn skip_whitespace(&mut self) -> Result<(), LexError> {
        let mut do_skip_whitespace = || {
            while let Some(c) = self.peek(0) {
                match c {
                    b' ' | b'\t' => {
                        self.advance(1);
                    }

                    b'\n' | b'\r' => {
                        self.read_line_end(false)?;
                    }

                    b'-' => {
                        if self.peek(1) != Some(b'-') {
                            break;
                        } else {
                            self.advance(2);

                            match (self.peek(0), self.peek(1)) {
                                (Some(b'['), Some(b'=')) | (Some(b'['), Some(b'[')) => {
                                    // long comment
                                    self.read_long_string(false)?;
                                }
                                _ => {
                                    // Short comment, read until end of line
                                    while let Some(c) = self.peek(0) {
                                        if is_newline(c) {
                                            break;
                                        } else {
                                            self.advance(1);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    b'/' => {
                        if self.peek(1) != Some(b'/') {
                            break;
                        } else {
                            self.advance(2);
                            
                            while let Some(c) = self.peek(0) {
                                if is_newline(c) {
                                    break;
                                } else {
                                    self.advance(1);
                                }
                            }
                        }
                    }

                    _ => break,
                }
            }

            Ok(())
        };

        match do_skip_whitespace() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.reset();
                Err(err)
            }
        }
    }

    /// Reads the next token, or None if the end of the source has been reached.
    pub fn read_token(&mut self) -> Result<Option<Token<S::String>>, LexError> {
        self.skip_whitespace()?;

        let mut do_read_token = || {
            if let Some(c) = self.peek(0) {
                Ok(Some(match c {
                    b' ' | b'\t' | b'\n' | b'\r' => {
                        unreachable!("whitespace should have been skipped");
                    }
                    
                    b'+' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::AssignAdd
                        } else {
                            Token::Add
                        }
                    }

                    b'-' => {
                        let next = self.peek(1);
                        if next == Some(b'=') {
                            self.advance(2);
                            Token::AssignSub
                        } else if next != Some(b'-') {
                            self.advance(1);
                            Token::Minus
                        } else {
                            unreachable!("whitespace should have been skipped");
                        }
                    }
                    
                    b'*' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::AssignMul
                        } else {
                            Token::Mul
                        }
                    }
                    
                    b'/' => {
                        let next = self.peek(1);
                        if next == Some(b'=') {
                            self.advance(2);
                            Token::AssignDiv
                        } else if next != Some(b'/') {
                            self.advance(1);
                            Token::Div
                        } else {
                            unreachable!("whitespace should have been skipped");
                        }
                    }
                    
                    b'\\' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::AssignIDiv
                        } else {
                            Token::IDiv
                        }
                    }
                    
                    b'%' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::AssignMod
                        } else {
                            Token::Peek2Mod
                        }
                    }
                    
                    b'^' => {
                        self.advance(1);
                        let next = self.peek(0);
                        if next == Some(b'^') {
                            self.advance(1);
                            if self.peek(0) == Some(b'=') {
                                self.advance(1);
                                Token::AssignBitXor
                            } else {
                                Token::BitXor
                            }
                        } else if next == Some(b'=') {
                            self.advance(1);
                            Token::AssignPow
                        } else {
                            Token::Pow
                        }
                    }
                    
                    b'&' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::AssignBitAnd
                        } else {
                            Token::BitAnd
                        }
                    }
                    
                    b'|' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::AssignBitOr
                        } else {
                            Token::BitOr
                        }
                    }

                    b'[' => {
                        let next = self.peek(1);
                        if next == Some(b'=') || next == Some(b'[') {
                            self.read_long_string(true)?;
                            Token::String(self.take_string())
                        } else {
                            self.advance(1);
                            Token::LeftBracket
                        }
                    }

                    b'=' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::Equal
                        } else {
                            Token::Assign
                        }
                    }

                    b'<' => {
                        self.advance(1);
                        let next = self.peek(0);
                        if next == Some(b'<') {
                            self.advance(1);
                            let next = self.peek(0);
                            if next == Some(b'>') {
                                self.advance(1);
                                if self.peek(0) == Some(b'=') {
                                    self.advance(1);
                                    Token::AssignRotateLeft
                                } else {
                                    Token::RotateLeft
                                }
                            } else if next == Some(b'=') {
                                self.advance(1);
                                Token::AssignShiftLeft
                            } else {
                                Token::ShiftLeft
                            }
                        } else if next == Some(b'=') {
                            self.advance(1);
                            Token::LessEqual
                        } else {
                            Token::LessThan
                        }
                    }

                    b'>' => {
                        self.advance(1);
                        let next = self.peek(0);
                        if next == Some(b'>') {
                            self.advance(1);
                            let next = self.peek(0);
                            if next == Some(b'>') {
                                self.advance(1);
                                if self.peek(0) == Some(b'=') {
                                    self.advance(1);
                                    Token::AssignShiftRightLogical
                                } else {
                                    Token::ShiftRightLogical
                                }
                            } else if next == Some(b'<') {
                                self.advance(1);
                                if self.peek(0) == Some(b'=') {
                                    self.advance(1);
                                    Token::AssignRotateRight
                                } else {
                                    Token::RotateRight
                                }
                            } else if next == Some(b'=') {
                                self.advance(1);
                                Token::AssignShiftRightArithmetic
                            } else {
                                Token::ShiftRightArithmetic
                            }
                        } else if next == Some(b'=') {
                            self.advance(1);
                            Token::GreaterEqual
                        } else {
                            Token::GreaterThan
                        }
                    }

                    b'~' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::NotEqual
                        } else {
                            Token::BitNotXor
                        }
                    }
                    
                    b'!' => {
                        self.advance(1);
                        if self.peek(0) == Some(b'=') {
                            self.advance(1);
                            Token::NotEqual
                        } else {
                            return Err(LexError::UnexpectedCharacter(c));
                        }
                    }

                    b':' => {
                        self.advance(1);
                        if self.peek(0) == Some(b':') {
                            self.advance(1);
                            Token::DoubleColon
                        } else {
                            Token::Colon
                        }
                    }

                    b'"' | b'\'' => {
                        self.read_short_string()?;
                        Token::String(self.take_string())
                    }

                    b'.' => {
                        if self.peek(1) == Some(b'.') {
                            let next = self.peek(2);
                            if next == Some(b'.') {
                                self.advance(3);
                                Token::Dots
                            } else if next == Some(b'=') {
                                self.advance(3);
                                Token::AssignConcat
                            } else {
                                self.advance(2);
                                Token::Concat
                            }
                        } else if self.peek(1).map(is_digit).unwrap_or(false) {
                            self.read_numeral()?
                        } else {
                            self.advance(1);
                            Token::Dot
                        }
                    }

                    c => {
                        if is_digit(c) {
                            self.read_numeral()?
                        } else if let Some(t) = get_char_token(c) {
                            self.advance(1);
                            t
                        } else if is_alpha(c) {
                            self.string_buffer.clear();
                            self.string_buffer.push(c);
                            self.advance(1);

                            while let Some(c) = self.peek(0) {
                                if is_alpha(c) || is_digit(c) {
                                    self.string_buffer.push(c);
                                    self.advance(1);
                                } else {
                                    break;
                                }
                            }

                            if let Some(t) = get_reserved_word_token(self.string_buffer.as_slice()) {
                                t
                            } else {
                                Token::Name(self.take_string())
                            }
                        } else {
                            return Err(LexError::UnexpectedCharacter(c));
                        }
                    }
                }))
            } else {
                Ok(None)
            }
        };

        match do_read_token() {
            Ok(Some(token)) => Ok(Some(token)),
            res => {
                self.reset();
                res
            }
        }
    }

    // End of stream encountered, clear any input handles and temp buffers
    fn reset(&mut self) {
        // self.source = &[]; // Is this needed?
        self.string_buffer.clear();
    }

    // Read any of "\n", "\r", "\n\r", or "\r\n" as a single newline, and increment the current line
    // number. If `append_buffer` is true, then appends the read newline to the string buffer.
    fn read_line_end(&mut self, append_string: bool) -> Result<(), LexError> {
        let newline = self.peek(0).unwrap();
        assert!(is_newline(newline));
        self.advance(1);
        // We always append a single plain `\n` character for any newline characters, matching the
        // behavior of PUC-Rio Lua.
        if append_string {
            self.string_buffer.push(b'\n');
        }

        if let Some(next_newline) = self.peek(0) {
            if is_newline(next_newline) && next_newline != newline {
                self.advance(1);
            }
        }

        self.line_number += 1;
        Ok(())
    }

    // Read a string on a single line delimited by ' or " that allows for \ escaping of certain
    // characters. Always reads the contained string into the string buffer.
    fn read_short_string(&mut self) -> Result<(), LexError> {
        let start_quote = self.peek(0).unwrap();
        assert!(start_quote == b'\'' || start_quote == b'"');
        self.advance(1);
        
        self.string_buffer.clear();
        
        loop {
            let c = self.peek(0).ok_or(LexError::UnfinishedShortString(start_quote))?;

            if is_newline(c) {
                return Err(LexError::UnfinishedShortString(start_quote));
            }

            if c == start_quote {
                self.advance(1);
                break;
            } else {
                self.string_buffer.push(c);
                self.advance(1);
                
                if c == b'\\' {
                    let next = self.peek(0).ok_or(LexError::UnfinishedShortString(start_quote))?;
                    if is_newline(next) {
                        self.read_line_end(true)?;
                    } else {
                        self.string_buffer.push(next);
                        self.advance(1);
                    }
                }
            }
        }
        
        match p8scii::unescape_in_place(&mut self.string_buffer) {
            Ok(len) => self.string_buffer.truncate(len),
            Err(p8scii::UnescapeError::InvalidEscapeSeq(..)) => return Err(LexError::InvalidEscape),
            Err(p8scii::UnescapeError::DecimalTooLarge(..)) => return Err(LexError::EscapeDecimalTooLarge),
        }

        Ok(())
    }

    // Read a [=*[...]=*] sequence with matching numbers of '='. If `into_string` is true, writes
    // the contained string into the string buffer.
    fn read_long_string(&mut self, into_string: bool) -> Result<(), LexError> {
        assert_eq!(self.peek(0).unwrap(), b'[');
        self.advance(1);

        if into_string {
            self.string_buffer.clear();
        }

        let mut open_sep_length = 0;
        while self.peek(0) == Some(b'=') {
            self.advance(1);
            open_sep_length += 1;
        }

        if self.peek(0) != Some(b'[') {
            return Err(LexError::InvalidLongStringDelimiter);
        }
        self.advance(1);

        if matches!(self.peek(0), Some(b'\n' | b'\r')) {
            // If the long string starts imediately with a newline, we read it and do *not* put it
            // into the string buffer, matching the behavior of PUC-Rio Lua. (and PICO-8 too!)
            self.read_line_end(false)?;
        }
        
        if into_string {
            self.string_buffer.clear();
        }
        
        loop {
            let c = self.peek(0).ok_or(LexError::UnfinishedLongString)?;
            
            match c {
                b'\n' | b'\r' => {
                    self.read_line_end(into_string)?;
                }
                
                b']' => {
                    let mut close_sep_length = 0;
                    self.advance(1);
                    while self.peek(0) == Some(b'=') {
                        self.advance(1);
                        close_sep_length += 1;
                    }
                    
                    if open_sep_length == close_sep_length && self.peek(0) == Some(b']') {
                        self.advance(1);
                        break;
                    } else {
                        // If it turns out this is not a valid long string close delimiter, we need
                        // to add the invalid close delimiter to the string.
                        if into_string {
                            self.string_buffer.push(b']');
                            for _ in 0..close_sep_length {
                                self.string_buffer.push(b'=');
                            }
                        }
                    }
                }
                
                c => {
                    if into_string {
                        self.string_buffer.push(c);
                    }
                    self.advance(1);
                }
            }
        }

        Ok(())
    }

    // Reads a binary, hex or decimal integer or floating point identifier. Allows decimal numbers (123.456),
    // hex numbers (0xdead.beef) and binary numbers (0b11110.10110)
    fn read_numeral(&mut self) -> Result<Token<S::String>, LexError> {
        let p1 = self.peek(0).unwrap();
        assert!(p1 == b'.' || is_digit(p1));
        
        self.string_buffer.clear();
        
        let p2 = self.peek(1);
        let is_hex = p1 == b'0' && (p2 == Some(b'x') || p2 == Some(b'X'));
        let is_bin = p1 == b'0' && (p2 == Some(b'b') || p2 == Some(b'B'));
        if is_hex || is_bin {
            self.advance(2);
        }
        
        let mut has_radix = false;
        while let Some(c) = self.peek(0) {
            if c == b'.' && !has_radix {
                self.string_buffer.push(b'.');
                has_radix = true;
                self.advance(1);
            } else if (is_hex && is_hex_digit(c)) || (is_bin && is_bin_digit(c)) || (!is_hex && !is_bin && is_digit(c)) {
                self.string_buffer.push(c);
                self.advance(1);
            } else {
                break;
            }
        }
        
        Ok(Token::Number(
            if is_hex {
                P8Num::from_ascii_radix(&self.string_buffer, 16)
            } else if is_bin {
                P8Num::from_ascii_radix(&self.string_buffer, 2)
            } else {
                P8Num::from_ascii_radix(&self.string_buffer, 10)
            }
            .map_err(|_| LexError::BadNumber)?,
        ))
    }

    fn peek(&mut self, n: usize) -> Option<u8> {
        self.source.peek_nth(n).copied()
    }

    fn advance(&mut self, n: usize) {
        assert!(
            n <= self.source.peek_len(),
            "cannot advance over un-peeked characters"
        );
        for _ in 0..n {
            self.source.next();
        }
    }

    fn take_string(&mut self) -> S::String {
        let s = self.interner.intern(&self.string_buffer);
        self.string_buffer.clear();
        s
    }
}

fn get_char_token<S>(c: u8) -> Option<Token<S>> {
    match c {
        b',' => Some(Token::Comma),
        b';' => Some(Token::SemiColon),
        b'#' => Some(Token::Len),
        b'@' => Some(Token::Peek),
        b'$' => Some(Token::Peek4),
        b'?' => Some(Token::Print),
        b'(' => Some(Token::LeftParen),
        b')' => Some(Token::RightParen),
        b']' => Some(Token::RightBracket),
        b'{' => Some(Token::LeftBrace),
        b'}' => Some(Token::RightBrace),
        _ => None,
    }
}

fn get_reserved_word_token<S>(word: &[u8]) -> Option<Token<S>> {
    match word {
        b"break" => Some(Token::Break),
        b"do" => Some(Token::Do),
        b"else" => Some(Token::Else),
        b"elseif" => Some(Token::ElseIf),
        b"end" => Some(Token::End),
        b"function" => Some(Token::Function),
        b"goto" => Some(Token::Goto),
        b"if" => Some(Token::If),
        b"in" => Some(Token::In),
        b"local" => Some(Token::Local),
        b"nil" => Some(Token::Nil),
        b"for" => Some(Token::For),
        b"while" => Some(Token::While),
        b"repeat" => Some(Token::Repeat),
        b"until" => Some(Token::Until),
        b"return" => Some(Token::Return),
        b"then" => Some(Token::Then),
        b"true" => Some(Token::True),
        b"false" => Some(Token::False),
        b"not" => Some(Token::Not),
        b"and" => Some(Token::And),
        b"or" => Some(Token::Or),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::interning::BasicInterner;
    use alloc::rc::Rc;
    use p8rs_macros::p8;

    use super::*;

    fn test_tokens(source: &str, tokens: &[Token<Rc<[u8]>>]) {
        let mut lexer = Lexer::new(source.as_bytes(), BasicInterner::default());
        let mut i = 0;
        while let Some(token) = lexer.read_token().unwrap() {
            assert!(i < tokens.len(), "too many tokens");
            assert_eq!(token, tokens[i], "tokens not equal");
            i += 1;
        }
        assert!(i == tokens.len(), "not enough tokens");
    }

    fn test_tokens_lines(source: &str, tokens: &[(Token<Rc<[u8]>>, u64)]) {
        let mut lexer = Lexer::new(source.as_bytes(), BasicInterner::default());
        let mut i = 0;
        loop {
            lexer.skip_whitespace().unwrap();
            let line_number = lexer.line_number().0;
            if let Some(token) = lexer.read_token().unwrap() {
                assert!(i < tokens.len(), "too many tokens");
                assert_eq!(token, tokens[i].0, "tokens not equal");
                assert_eq!(line_number, tokens[i].1, "line numbers do not match");
                i += 1;
            } else {
                break;
            }
        }
        assert!(i == tokens.len(), "not enough tokens");
    }

    fn str_token(s: &str) -> Token<Rc<[u8]>> {
        Token::String(p8scii::from_str(s).lossy().collect::<Vec<_>>().into_boxed_slice().into())
    }

    fn name_token(s: &str) -> Token<Rc<[u8]>> {
        Token::Name(p8scii::from_str(s).lossy().collect::<Vec<_>>().into_boxed_slice().into())
    }

    #[test]
    fn comments() {
        test_tokens_lines(
            r#"
                -- this is a comment
                -- this is also -- a comment
                --[[ long comment ]]
                --[==[ longer comment ]==]

                -- Real token
                -

                --[====[ longest comment
                    these shouldn't trigger the end of comments
                    ]=] ]==] ]===]
                ]====]

                -- Real token
                =
            "#,
            &[(Token::Minus, 7), (Token::Assign, 15)],
        );
    }

    #[test]
    fn long_string() {
        test_tokens(
            r#"
                [====[ [==[ this is a [[]] long string ]== ]==] ]====]
                [[ [=] [==] another long string [==] [=] ]]
                [[ \t\r\x escape codes are ignored \1\2\3 ]]
                [[ ⬆️⬇️⬅️➡️ PICO-8 symbols █▒░▤▥ ]]
            "#,
            &[
                str_token(" [==[ this is a [[]] long string ]== ]==] "),
                str_token(" [=] [==] another long string [==] [=] "),
                str_token(" \\t\\r\\x escape codes are ignored \\1\\2\\3 "),
                str_token(" ⬆️⬇️⬅️➡️ PICO-8 symbols █▒░▤▥ "),
            ],
        );

        test_tokens(
            "[==[\nfoo\nbar\rbaz\r\nbaf\rquux]==]",
            &[str_token("foo\nbar\nbaz\nbaf\nquux")],
        );
    }

    #[test]
    fn short_string() {
        test_tokens_lines(
            r#"
                "\\ \" '"
                '\n \t "'
                "begin \
end"
                "question\x3f"
                "exclaim\33"
                "\0\1\2\3"
                "⬆️⬇️⬅️➡️"
            "#,
            &[
                (str_token("\\ \" '"), 1),
                (str_token("\n \t \""), 2),
                (str_token("begin \nend"), 3),
                (str_token("question?"), 5),
                (str_token("exclaim!"), 6),
                (Token::String(vec![0, 1, 2, 3].into()), 7),
                (str_token("⬆️⬇️⬅️➡️"), 8),
            ],
        );
    }

    #[test]
    fn numerals() {
        test_tokens(
            r#"
                0xdead.beef
                0xdeadbeef
                12345
                12345.
                3.1415
            "#,
            &[
                Token::Number(p8!("dead.beef"hex)),
                Token::Number(p8!("beef.0000"hex)),
                Token::Number(p8!(12345)),
                Token::Number(p8!(12345)),
                Token::Number(p8!(3.1415)),
            ],
        );
    }

    #[test]
    fn words() {
        test_tokens(
            r#"
                break do else elseif end function goto if in local nil for while repeat until return
                then true false not and or
            "#,
            &[
                Token::Break,
                Token::Do,
                Token::Else,
                Token::ElseIf,
                Token::End,
                Token::Function,
                Token::Goto,
                Token::If,
                Token::In,
                Token::Local,
                Token::Nil,
                Token::For,
                Token::While,
                Token::Repeat,
                Token::Until,
                Token::Return,
                Token::Then,
                Token::True,
                Token::False,
                Token::Not,
                Token::And,
                Token::Or,
            ],
        );
    }

    #[test]
    fn names() {
        test_tokens(
            r#"
                custom names
                にほんこ゛
                ⬅️⬇️⬆️➡️
                █▒🐱😐
            "#,
            &[
                name_token("custom"),
                name_token("names"),
                name_token("にほんこ゛"),
                name_token("⬅️⬇️⬆️➡️"),
                name_token("█▒🐱😐"),
            ],
        );
    }
    
    #[test]
    fn ops() {
        test_tokens(
            r#"- + * / \ ^ , ; . .. ... < <= > >= == ~= != : :: # @ % $ ? ( ) [ ] { }"#,
            &[
                Token::Minus,
                Token::Add,
                Token::Mul,
                Token::Div,
                Token::IDiv,
                Token::Pow,
                Token::Comma,
                Token::SemiColon,
                Token::Dot,
                Token::Concat,
                Token::Dots,
                Token::LessThan,
                Token::LessEqual,
                Token::GreaterThan,
                Token::GreaterEqual,
                Token::Equal,
                Token::NotEqual,
                Token::NotEqual,
                Token::Colon,
                Token::DoubleColon,
                Token::Len,
                Token::Peek,
                Token::Peek2Mod,
                Token::Peek4,
                Token::Print,
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBracket,
                Token::RightBracket,
                Token::LeftBrace,
                Token::RightBrace,
            ],
        );
    }
    
    #[test]
    fn bit_ops() {
        test_tokens(
            r#"~ & | ^^ >> >>> << >>< <<>"#,
            &[
                Token::BitNotXor,
                Token::BitAnd,
                Token::BitOr,
                Token::BitXor,
                Token::ShiftRightArithmetic,
                Token::ShiftRightLogical,
                Token::ShiftLeft,
                Token::RotateRight,
                Token::RotateLeft,
            ],
        );
    }
    
    #[test]
    fn assigns() {
        test_tokens(
            r#"= += -= *= /= \= %= ^= &= |= ^^= >>= >>>= <<= >><= <<>= ..="#,
            &[
                Token::Assign,
                Token::AssignAdd,
                Token::AssignSub,
                Token::AssignMul,
                Token::AssignDiv,
                Token::AssignIDiv,
                Token::AssignMod,
                Token::AssignPow,
                Token::AssignBitAnd,
                Token::AssignBitOr,
                Token::AssignBitXor,
                Token::AssignShiftRightArithmetic,
                Token::AssignShiftRightLogical,
                Token::AssignShiftLeft,
                Token::AssignRotateRight,
                Token::AssignRotateLeft,
                Token::AssignConcat,
            ],
        );
    }
}
