mod builder;
mod state;

pub use self::state::State;
use crate::types::AttributeMap;
use builder::Builder;
use encoding_rs::{CoderResult, Decoder, Encoding};
use serde::Deserialize;
use std::{
    io::{BufReader, Read},
    iter::Peekable,
};

type InputStream<'s> = Peekable<std::io::Bytes<BufReader<&'s [u8]>>>;

#[derive(Debug, Deserialize, PartialEq)]
pub struct TokenError {
    pub code: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenResult {
    Ok {
        token: Token,
        errors: Vec<TokenError>,
    },

    Errors(Vec<TokenError>),
}

pub struct Tokenizer<'s> {
    encoding: &'static Encoding,
    initial_state: State,
    _last_start_tag: Option<StartTag>,
    raw: InputStream<'s>,
}

impl<'s> Tokenizer<'s> {
    pub fn from_str(i: &'s str, initial_state: State, last_start_tag: Option<StartTag>) -> Self {
        let raw = BufReader::new(i.as_bytes()).bytes().peekable();
        Self {
            raw,
            encoding: encoding_rs::UTF_8,
            initial_state,
            _last_start_tag: last_start_tag,
        }
    }

    pub fn iter(self) -> TokenIterator<'s> {
        let decoder = self.encoding.new_decoder_without_bom_handling();

        TokenIterator {
            builder: Builder::new(),
            decoder,
            dst: String::with_capacity(TokenIterator::BUFSIZE),
            index: 0,
            src: Vec::with_capacity(TokenIterator::BUFSIZE),
            state: self.initial_state,
            tokenizer: self,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StartTag {
    pub name: String,
    pub attributes: AttributeMap,
    pub self_closing: bool,
}

impl From<&String> for StartTag {
    fn from(name: &String) -> Self {
        Self {
            name: name.to_owned(),
            attributes: AttributeMap::new(),
            self_closing: false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Character(String),

    Comment(String),

    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        // When correctness: true, force-quirks: false and vice-versa
        correctness: bool,
    },

    EndTag(String),

    Eof,

    StartTag(StartTag),
}

// Result of the attempt to fetch another character from the decoder
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NextChar {
    Ch(char),
    Eof,
    Stop,
}

use NextChar::*;

pub struct TokenIterator<'s> {
    builder: Builder,
    decoder: Decoder,
    dst: String,
    index: usize,
    src: Vec<u8>,
    state: State,
    tokenizer: Tokenizer<'s>,
}

impl Iterator for TokenIterator<'_> {
    type Item = TokenResult;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.builder.results.pop_front();
        if result.is_some() {
            return result;
        }

        while let c @ (Ch(..) | Eof) = self.next_char() {
            self.builder.next_char(c);

            let mut state = self.state.next_char(c, &mut self.builder);
            if self.builder.reconsume_char {
                state = state.next_char(c, &mut self.builder);
            }
            self.state = state;

            if self.builder.emit_token {
                self.builder.flush();
                if let Some(result) = self.builder.results.pop_front() {
                    return Some(result);
                }
            }

            if c == Eof {
                break;
            }
        }

        self.builder.flush();
        self.builder.results.pop_front()
    }
}

impl TokenIterator<'_> {
    const BUFSIZE: usize = 2048;

    fn next_char(&mut self) -> NextChar {
        let mut last = false;

        if !self.dst.is_empty() {
            return self.consume();
        }

        if self.src.is_empty() {
            while self.src.len() < Self::BUFSIZE {
                if let Some(result) = self.tokenizer.raw.next() {
                    if let Ok(c) = result {
                        self.src.push(c);
                    } else {
                        todo!("result not ok: {:?}", result);
                        // self.errors.push(..)
                    }
                } else {
                    last = true;
                    break;
                }
            }
            self.index = 0;
        }

        let (result, total_read, had_errors) =
            self.decoder
                .decode_to_string(&self.src, &mut self.dst, last);

        if had_errors {
            todo!("unicode decoding errors");
        }

        if self.dst.is_empty() {
            match result {
                CoderResult::InputEmpty => return Stop,

                CoderResult::OutputFull => {
                    if total_read == 0 {
                        return Stop;
                    }
                    todo!("output full")
                    // self.errors.push(..)
                    // return None;
                }
            }
        }

        self.consume()
    }

    fn consume(&mut self) -> NextChar {
        let res = self.dst.as_bytes().get(self.index);
        if let Some(&c) = res {
            self.index += 1;
            return Ch(c as _);
        }
        Eof
    }
}

#[cfg(test)]
mod tests {
    use super::{state::State, *};

    fn iter(i: &str, state: State) -> TokenIterator<'_> {
        Tokenizer::from_str(i, state, None).iter()
    }

    #[test]
    fn good() {
        // domjs.test, line 267
        let mut iter = iter("<!DOCTYPE html ", State::Data);
        let TokenResult::Ok { token, errors } = iter.next().unwrap() else {
            panic!()
        };

        assert!(matches!(
            iter.next(),
            Some(TokenResult::Ok {
                token: Token::Eof,
                ..
            }),
        ));

        assert!(iter.next().is_none());

        assert_eq!(
            token,
            Token::Doctype {
                name: Some("html".into()),
                public_id: None,
                system_id: None,
                correctness: false,
            }
        );

        assert!(!errors.is_empty());
        let error = &errors[0];

        assert_eq!(
            error,
            &TokenError {
                code: "eof-in-doctype".into(),
                line: 1,
                col: 16,
            }
        );
    }

    #[test]
    fn empty_end_tag() {
        // tests1.test, line 38
        let mut iter = iter("</>", State::Data);
        let result = iter.next().unwrap();
        let TokenResult::Ok { token, errors } = result else {
            panic!("result: {:?}", result);
        };

        assert_eq!(token, Token::Eof);

        let error = &errors[0];

        assert_eq!(
            error,
            &TokenError {
                code: "missing-end-tag-name".into(),
                line: 1,
                col: 3,
            }
        );
    }

    #[test]
    fn empty_start_tag() {
        // tests1.test, line 45
        let mut iter = iter("<>", State::Data);
        let result = iter.next().unwrap();

        let TokenResult::Ok { token, errors } = result else {
            panic!("result: {:?}", result);
        };

        if let Token::Character(string) = token {
            assert_eq!(string, "<>");
        } else {
            panic!("{:?}", token);
        };

        let error = &errors[0];

        assert_eq!(
            error,
            &TokenError {
                code: "invalid-first-character-of-tag-name".into(),
                line: 1,
                col: 2,
            }
        );
    }
}
