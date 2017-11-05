use std::fmt;
use std::error::Error;
use token::{Token, TokenKind};

pub use token::InputPosition;

#[derive(Debug, PartialEq)]
pub struct AST(pub Vec<ASTNode>);

impl AST {
    pub fn from_tokens(tokens: &Vec<Token>) -> Result<AST, SyntaxError> {
        let mut ops = vec![];
        let mut ts = tokens.clone();
        ts.reverse();

        while let Some(t) = ts.pop() {
            if let Some(op) = try_parse_scalar(&t) {
                ops.push(op);
            } else if t.kind == TokenKind::StartLoop {
                ops.push(parse_loop(&mut ts, &t.pos)?);
            } else if t.kind == TokenKind::EndLoop {
                return Err(SyntaxError { pos: t.pos, kind: ErrorKind::UnopenedLoop });
            }
        }

        Ok(AST(ops))
    }
}

fn parse_loop(ts: &mut Vec<Token>, start_pos: &InputPosition) -> Result<ASTNode, SyntaxError> {
    let mut ops = vec![];

    while let Some(t) = ts.pop() {
        if let Some(op) = try_parse_scalar(&t) {
            ops.push(op);
        } else if t.kind == TokenKind::StartLoop {
            ops.push(parse_loop(ts, &t.pos)?);
        } else if t.kind == TokenKind::EndLoop {
            return Ok(ASTNode {
                kind: ASTNodeKind::Loop,
                pos: start_pos.clone(),
                ops: Some(ops)
            });
        }
    }

    Err(SyntaxError { pos: start_pos.clone(), kind: ErrorKind::UnclosedLoop })
}

#[derive(Debug, PartialEq)]
pub struct SyntaxError {
    pub pos: InputPosition,
    pub kind: ErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    UnclosedLoop,
    UnopenedLoop,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} ({}:{})", self.description(), self.pos.line, self.pos.pos)
    }
}

impl Error for SyntaxError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::UnopenedLoop => "Unopened loop",
            ErrorKind::UnclosedLoop => "Unclosed loop",
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ASTNode {
    pub kind: ASTNodeKind,
    pub pos: InputPosition,
    pub ops: Option<Vec<ASTNode>>,
}

#[cfg(test)]
impl ASTNode {
    fn new_scalar(kind: ASTNodeKind, line: usize, pos: usize) -> ASTNode {
        ASTNode {
            kind: kind,
            pos: InputPosition {
                line: line,
                pos: pos,
            },
            ops: None
        }
    }

    fn new_loop(line: usize, pos: usize, ops: Vec<ASTNode>) -> ASTNode {
        ASTNode {
            kind: ASTNodeKind::Loop,
            pos: InputPosition {
                line: line,
                pos: pos,
            },
            ops: Some(ops),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ASTNodeKind {
    Loop, 
    IncTape,
    DecTape,
    IncVal,
    DecVal,
    Read,
    Write,
}

fn try_parse_scalar(t: &Token) -> Option<ASTNode> {
    Some(ASTNode {
        kind: match t.kind {
            TokenKind::IncTape => ASTNodeKind::IncTape,
            TokenKind::DecTape => ASTNodeKind::DecTape,
            TokenKind::IncVal => ASTNodeKind::IncVal,
            TokenKind::DecVal => ASTNodeKind::DecVal,
            TokenKind::Read => ASTNodeKind::Read,
            TokenKind::Write => ASTNodeKind::Write,
            _ => return None,
        },
        pos: t.pos.clone(),
        ops: None,
    })
}

#[cfg(test)]
mod test {
    use token::tokenize;
    use super::{AST, ASTNode, ASTNodeKind, SyntaxError, ErrorKind, InputPosition};

    #[test]
    fn empty() {
        let raw = "";
        let val = AST::from_tokens(&tokenize(raw));
        let expect = Ok(AST(vec![]));

        assert_eq!(val, expect);
    }

    #[test]
    fn scalar() {
        let raw = "><+-,.";
        let val = AST::from_tokens(&tokenize(raw));
        let expect = Ok(AST(vec![
            ASTNode::new_scalar(ASTNodeKind::IncTape, 1, 1),
            ASTNode::new_scalar(ASTNodeKind::DecTape, 1, 2),
            ASTNode::new_scalar(ASTNodeKind::IncVal, 1, 3),
            ASTNode::new_scalar(ASTNodeKind::DecVal, 1, 4),
            ASTNode::new_scalar(ASTNodeKind::Read, 1, 5),
            ASTNode::new_scalar(ASTNodeKind::Write, 1, 6),
        ]));

        assert_eq!(val, expect);
    }

    #[test]
    fn empty_loop() {
        let raw = "[]";
        let val = AST::from_tokens(&tokenize(raw));
        let expect = Ok(AST(vec![ASTNode::new_loop(1, 1, vec![])]));

        assert_eq!(val, expect);
    }

    #[test]
    fn simple_loop() {
        let raw = "+[-]";
        let val = AST::from_tokens(&tokenize(raw));
        let expect = Ok(AST(vec![
            ASTNode::new_scalar(ASTNodeKind::IncVal, 1, 1),
            ASTNode::new_loop(1, 2, vec![
                ASTNode::new_scalar(ASTNodeKind::DecVal, 1, 3),
            ]),
        ]));

        assert_eq!(val, expect);
    }

    #[test]
    fn nested_loop() {
        let raw = "+[+[-]-]";
        let val = AST::from_tokens(&tokenize(raw));
        let expect = Ok(AST(vec![
            ASTNode::new_scalar(ASTNodeKind::IncVal, 1, 1),
            ASTNode::new_loop(1, 2, vec![
                ASTNode::new_scalar(ASTNodeKind::IncVal, 1, 3),
                ASTNode::new_loop(1, 4, vec![
                    ASTNode::new_scalar(ASTNodeKind::DecVal, 1, 5),
                ]),
                ASTNode::new_scalar(ASTNodeKind::DecVal, 1, 7),
            ]),
        ]));

        assert_eq!(val, expect);
    }

    #[test]
    fn unopened_loop() {
        let raw = "]";
        let val = AST::from_tokens(&tokenize(raw));
        let expect = Err(SyntaxError {
            pos: InputPosition { line: 1, pos: 1 },
            kind: ErrorKind::UnopenedLoop,
        });

        assert_eq!(val, expect);
    }

    #[test]
    fn unclosed_loop() {
        let raw = "[";
        let val = AST::from_tokens(&tokenize(raw));
        let expect = Err(SyntaxError {
            pos: InputPosition { line: 1, pos: 1 },
            kind: ErrorKind::UnclosedLoop,
        });

        assert_eq!(val, expect);
    }
}
