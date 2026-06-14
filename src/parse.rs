use std::fmt;

use lalrpop_util::ParseError as LalrParseError;

use crate::errors::{ParseError, TinyQttResult};
use crate::lexer::{Lexer, LexicalError, Token};
use crate::syntax::{Decl, Raw, ReplInput};

#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    dead_code,
    unreachable_pub
)]
mod parser_impl {
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);
}
use parser_impl::parser;

pub struct Parser {
    module: parser::ModuleParser,
    term: parser::TermParser,
    repl: parser::ReplParser,
}

impl fmt::Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parser").finish()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            module: parser::ModuleParser::new(),
            term: parser::TermParser::new(),
            repl: parser::ReplParser::new(),
        }
    }

    pub fn parse_module(&self, input: &str) -> TinyQttResult<Vec<Decl>> {
        let tokens = Lexer::new(input);
        self.module
            .parse(tokens)
            .map_err(|e| convert(e, input).into())
    }

    pub fn parse_term(&self, input: &str) -> TinyQttResult<Raw> {
        let tokens = Lexer::new(input);
        self.term
            .parse(tokens)
            .map_err(|e| convert(e, input).into())
    }

    pub fn parse_repl(&self, input: &str) -> TinyQttResult<ReplInput> {
        let tokens = Lexer::new(input);
        self.repl
            .parse(tokens)
            .map_err(|e| convert(e, input).into())
    }
}

fn convert(err: LalrParseError<usize, Token, LexicalError>, input: &str) -> ParseError {
    match err {
        LalrParseError::InvalidToken { location } => ParseError::InvalidToken { offset: location },
        LalrParseError::UnrecognizedEof { location, expected } => ParseError::UnexpectedEof {
            expected,
            offset: location.min(input.len()),
        },
        LalrParseError::UnrecognizedToken {
            token: (start, tok, end),
            expected,
        } => ParseError::UnexpectedToken {
            token: tok.to_string(),
            expected,
            span: (start, end),
        },
        LalrParseError::ExtraToken {
            token: (start, tok, end),
        } => ParseError::ExtraToken {
            token: tok.to_string(),
            span: (start, end),
        },
        LalrParseError::User { error } => ParseError::Lexical(error),
    }
}
