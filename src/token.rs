#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: InputPosition,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputPosition {
    pub line: usize,
    pub pos: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    StartLoop,
    EndLoop,
    IncTape,
    DecTape,
    IncVal,
    DecVal,
    Read,
    Write,
    Comment,
}

impl From<u8> for TokenKind {
    fn from(d: u8) -> TokenKind {
        TokenKind::from(d as char)
    }
}

impl From<char> for TokenKind {
    fn from(c: char) -> TokenKind {
        match c {
            '[' => TokenKind::StartLoop,
            ']' => TokenKind::EndLoop,
            '>' => TokenKind::IncTape,
            '<' => TokenKind::DecTape,
            '+' => TokenKind::IncVal,
            '-' => TokenKind::DecVal,
            ',' => TokenKind::Read,
            '.' => TokenKind::Write,
            _ => TokenKind::Comment,
        }
    }
}

pub fn tokenize(s: &str) -> Vec<Token> {
    let mut ret = vec![];
    let mut line = 0;
    let mut pos;

    for l in s.lines() {
        line += 1;
        pos = 0;
        for c in l.chars() {
            pos += 1;
            ret.push(Token {
                kind: TokenKind::from(c),
                pos: InputPosition { line: line, pos: pos },
            });
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use super::{Token, TokenKind, InputPosition};

    #[test]
    fn start_loop() {
        assert_eq!(TokenKind::StartLoop, TokenKind::from('['));
    }

    #[test]
    fn end_loop() {
        assert_eq!(TokenKind::EndLoop, TokenKind::from(']'));
    }

    #[test]
    fn inc_tape() {
        assert_eq!(TokenKind::IncTape, TokenKind::from('>'));
    }

    #[test]
    fn dec_tape() {
        assert_eq!(TokenKind::DecTape, TokenKind::from('<'));
    }

    #[test]
    fn inc_val() {
        assert_eq!(TokenKind::IncVal, TokenKind::from('+'));
    }

    #[test]
    fn dec_val() {
        assert_eq!(TokenKind::DecVal, TokenKind::from('-'));
    }

    #[test]
    fn read() {
        assert_eq!(TokenKind::Read, TokenKind::from(','));
    }

    #[test]
    fn write() {
        assert_eq!(TokenKind::Write, TokenKind::from('.'));
    }

    #[test]
    fn comment() {
        for d in 0..255u8 {
            let c = d as char;
            if !"[]><+-,.".contains(c) {
                assert_eq!(TokenKind::Comment, TokenKind::from(c));
            } else {
                assert_ne!(TokenKind::Comment, TokenKind::from(c));
            }
        }
    }

    #[test]
    fn tokenize() {
        let s = "[>+,\n .-<]";
        let tokens = super::tokenize(s);
        let expect: Vec<Token> = vec![
            Token { kind: TokenKind::StartLoop, pos: InputPosition { line: 1, pos: 1 } },
            Token { kind: TokenKind::IncTape, pos: InputPosition { line: 1, pos: 2 } },
            Token { kind: TokenKind::IncVal, pos: InputPosition { line: 1, pos: 3 } },
            Token { kind: TokenKind::Read, pos: InputPosition { line: 1, pos: 4 } },
            Token { kind: TokenKind::Comment, pos: InputPosition { line: 2, pos: 1 } },
            Token { kind: TokenKind::Write, pos: InputPosition { line: 2, pos: 2 } },
            Token { kind: TokenKind::DecVal, pos: InputPosition { line: 2, pos: 3 } },
            Token { kind: TokenKind::DecTape, pos: InputPosition { line: 2, pos: 4 } },
            Token { kind: TokenKind::EndLoop, pos: InputPosition { line: 2, pos: 5 } },
        ];
        assert_eq!(tokens, expect);
    }
}
