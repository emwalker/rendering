use super::{NextChar, StartTag, Token, TokenError, TokenResult};
use std::collections::VecDeque;

use NextChar::*;

#[derive(Default, Debug)]
pub struct Builder {
    attribute_name: String,
    attribute_value: String,
    pub buf: String,
    pub col: usize,
    pub emit_token: bool,
    errors: Vec<TokenError>,
    line: usize,
    pub reconsume_char: bool,
    pub token: Option<Token>,
    pub results: VecDeque<TokenResult>,
    last_start_tag: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            line: 1,
            col: 0,
            ..Self::default()
        }
    }

    pub fn add_error(&mut self, code: &str) {
        self.errors.push(TokenError {
            code: code.into(),
            line: self.line,
            col: self.col,
        })
    }

    pub fn add_to_attribute_name(&mut self, c: char) {
        self.attribute_name.push(c);
    }

    pub fn add_to_attribute_value(&mut self, c: char) {
        self.attribute_value.push(c);
    }

    pub fn appropriate_end_tag(&self) -> bool {
        if let Some(Token::EndTag(name)) = &self.token {
            self.last_start_tag.as_ref() == Some(name)
        } else {
            false
        }
    }

    pub fn discard_and_rewind(&mut self) {
        let _ = self.token.take();
        if let Some(TokenResult::Ok { token, errors }) = self.results.pop_back() {
            self.token = Some(token);
            self.errors = errors;
        }
    }

    pub fn push_char(&mut self, c: char) {
        if let Some(Token::Character(string)) = &mut self.token {
            string.push(c);
        } else {
            self.push_token(Token::Character(c.to_string()));
        }
    }

    pub fn push_str(&mut self, s: &str) {
        if let Some(Token::Character(string)) = &mut self.token {
            string.push_str(s);
        } else {
            self.push_token(Token::Character(s.to_string()));
        }
    }

    pub fn push_token(&mut self, token: Token) {
        match (&mut self.token, &token) {
            (Some(Token::Character(lhs)), Token::Character(rhs)) => {
                lhs.push_str(rhs);
            }

            _ => {
                self.push_result();
                self.token = Some(token);
            }
        }
    }

    fn push_result(&mut self) {
        if !self.attribute_name.is_empty() || !self.attribute_value.is_empty() {
            self.finalize_attribute();
        }

        if let Some(token) = self.token.take() {
            if let Token::StartTag(StartTag { name, .. }) = &token {
                self.last_start_tag = Some(name.into());
            }

            let mut errors = vec![];
            std::mem::swap(&mut self.errors, &mut errors);
            self.results.push_back(TokenResult::Ok { token, errors });
        }
    }

    pub fn emit_token(&mut self) {
        assert!(self.token.is_some());
        self.push_result();
        self.emit_token = true;
    }

    pub fn finalize_attribute(&mut self) {
        if self.attribute_name.is_empty() {
            return;
        }

        match &mut self.token {
            Some(Token::StartTag(StartTag { attributes, .. })) => {
                if attributes.contains_key(&self.attribute_name) {
                    self.add_error("duplicate-attribute")
                } else {
                    attributes.insert(
                        self.attribute_name.to_owned(),
                        self.attribute_value.to_owned(),
                    );
                }
            }

            Some(Token::EndTag(..)) => self.add_error("end-tag-with-attributes"),

            _ => {}
        }

        self.attribute_name.clear();
        self.attribute_value.clear();
    }

    pub fn flush(&mut self) {
        self.push_result();
        self.buf.clear();
        self.emit_token = false;
        self.reconsume_char = false;
        self.attribute_name.clear();
    }

    pub(crate) fn next_char(&mut self, c: NextChar) {
        if !self.reconsume_char {
            match c {
                Ch('\n') => self.line += 1,
                _ => self.col += 1,
            }
        }
        self.reconsume_char = false;
    }

    pub(crate) fn push_buf(&mut self, c: NextChar) {
        if let Ch(c) = c {
            self.buf.push(c);
        }
    }

    pub fn reconsume_char(&mut self) {
        self.reconsume_char = true;
    }
}
