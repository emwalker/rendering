use crate::types::{AttributeMap, Result};
use encoding_rs::{CoderResult, Decoder, Encoding};
use serde::Deserialize;
use std::{
    collections::VecDeque,
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
    initial_state: TokenizerState,
    _last_start_tag: Option<StartTag>,
    raw: InputStream<'s>,
}

impl<'s> Tokenizer<'s> {
    pub fn from_str(
        i: &'s str,
        initial_state: TokenizerState,
        last_start_tag: Option<StartTag>,
    ) -> Self {
        let raw = BufReader::new(i.as_bytes()).bytes().peekable();
        Self {
            raw,
            encoding: encoding_rs::UTF_8,
            initial_state,
            _last_start_tag: last_start_tag,
        }
    }

    pub fn iter(self) -> Result<TokenIterator<'s>> {
        let decoder = self.encoding.new_decoder_without_bom_handling();

        Ok(TokenIterator {
            builder: TokenBuilder::new(),
            decoder,
            dst: String::with_capacity(TokenIterator::BUFSIZE),
            index: 0,
            src: Vec::with_capacity(TokenIterator::BUFSIZE),
            state: self.initial_state,
            tokenizer: self,
        })
    }
}

#[derive(Default, Debug)]
struct TokenBuilder {
    attribute_name: String,
    attribute_value: String,
    buf: String,
    col: usize,
    emit_token: bool,
    errors: Vec<TokenError>,
    line: usize,
    reconsume_char: bool,
    token: Option<Token>,
    results: VecDeque<TokenResult>,
    last_start_tag: Option<String>,
}

impl TokenBuilder {
    fn new() -> Self {
        Self {
            line: 1,
            col: 0,
            ..Self::default()
        }
    }

    fn add_error(&mut self, code: &str) {
        self.errors.push(TokenError {
            code: code.into(),
            line: self.line,
            col: self.col,
        })
    }

    fn add_to_attribute_name(&mut self, c: char) {
        self.attribute_name.push(c);
    }

    fn add_to_attribute_value(&mut self, c: char) {
        self.attribute_value.push(c);
    }

    fn appropriate_end_tag(&self) -> bool {
        if let Some(Token::EndTag(name)) = &self.token {
            self.last_start_tag.as_ref() == Some(name)
        } else {
            false
        }
    }

    fn discard_and_rewind(&mut self) {
        let _ = self.token.take();
        if let Some(TokenResult::Ok { token, errors }) = self.results.pop_back() {
            self.token = Some(token);
            self.errors = errors;
        }
    }

    fn push_char(&mut self, c: char) {
        if let Some(Token::Character(string)) = &mut self.token {
            string.push(c);
        } else {
            self.push_token(Token::Character(c.to_string()));
        }
    }

    fn push_str(&mut self, s: &str) {
        if let Some(Token::Character(string)) = &mut self.token {
            string.push_str(s);
        } else {
            self.push_token(Token::Character(s.to_string()));
        }
    }

    fn push_token(&mut self, token: Token) {
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

    fn emit_token(&mut self) {
        assert!(self.token.is_some());
        self.push_result();
        self.emit_token = true;
    }

    fn finalize_attribute(&mut self) {
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

    fn flush(&mut self) {
        self.push_result();
        self.buf.clear();
        self.emit_token = false;
        self.reconsume_char = false;
        self.attribute_name.clear();
    }

    fn next_char(&mut self, c: NextChar) {
        if !self.reconsume_char {
            match c {
                Ch('\n') => self.line += 1,
                _ => self.col += 1,
            }
        }
        self.reconsume_char = false;
    }

    fn push_buf(&mut self, c: NextChar) {
        if let Ch(c) = c {
            self.buf.push(c);
        }
    }

    fn reconsume_char(&mut self) {
        self.reconsume_char = true;
    }
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq)]
pub enum TokenizerState {
    /// 8.2.4.36 After attribute name state
    #[serde(skip)]
    AfterAttributeName,

    /// 8.2.4.55 After DOCTYPE name state
    #[serde(skip)]
    AfterDOCTYPEName,

    /// 8.2.4.42 After attribute value (quoted) state
    #[serde(skip)]
    AfterAttributeValueQuoted,

    /// 8.2.4.35 Attribute name state
    #[serde(skip)]
    AttributeName,

    /// 8.2.4.38 Attribute value (double-quoted) state
    #[serde(skip)]
    AttributeValueDoubleQuoted,

    /// 8.2.4.39 Attribute value (single-quoted) state
    #[serde(skip)]
    AttributeValueSingleQuoted,

    /// 8.2.4.40 Attribute value (unquoted) state
    #[serde(skip)]
    AttributeValueUnquoted,

    /// 8.2.4.34 Before attribute name state
    #[serde(skip)]
    BeforeAttributeName,

    /// 8.2.4.37 Before attribute value state
    #[serde(skip)]
    BeforeAttributeValue,

    /// 8.2.4.53 Before DOCTYPE name state
    #[serde(skip)]
    BeforeDOCTYPEName,

    /// 8.2.4.44 Bogus comment state
    #[serde(skip)]
    BogusComment,

    /// 8.2.4.68 CDATA section state
    #[serde(rename = "CDATA section state")]
    CDATASection,

    /// 8.2.4.46 Comment start state
    #[serde(skip)]
    CommentStart,

    /// 8.2.4.1 Data state
    #[serde(rename = "Data state")]
    Data,

    /// 8.2.4.52 DOCTYPE state
    #[serde(skip)]
    DOCTYPE,

    /// 8.2.4.54 DOCTYPE name state
    #[serde(skip)]
    DOCTYPEName,

    /// 8.2.4.9 End tag open state
    #[serde(skip)]
    EndTagOpen,

    #[serde(skip)]
    Eof,

    /// 8.2.4.45 Markup declaration open state
    #[serde(skip)]
    MarkupDeclarationOpen,

    /// 8.2.4.7 PLAINTEXT state
    #[serde(rename = "PLAINTEXT state")]
    PLAINTEXT,

    /// 8.2.4.5 RAWTEXT state
    #[serde(rename = "RAWTEXT state")]
    RAWTEXT,

    /// 8.2.4.3 RCDATA state
    #[serde(rename = "RCDATA state")]
    RCDATA,

    /// 8.2.4.6 Script data state
    #[serde(rename = "Script data state")]
    ScriptData,

    /// 8.2.4.19 Script data end tag name state
    #[serde(skip)]
    ScriptDataEndTagName,

    /// 8.2.4.18 Script data end tag open state
    #[serde(skip)]
    ScriptDataEndTagOpen,

    /// 8.2.4.22 Script data escaped state
    #[serde(skip)]
    ScriptDataEscaped,

    /// 8.2.4.20 Script data escape start state
    #[serde(skip)]
    ScriptDataEscapeStart,

    /// 8.2.4.23 Script data escaped dash state
    #[serde(skip)]
    ScriptDataEscapedDash,

    /// 8.2.4.24 Script data escaped dash dash state
    #[serde(skip)]
    ScriptDataEscapeDashDash,

    /// 8.2.4.21 Script data escape start dash state
    #[serde(skip)]
    ScriptDataEscapeStartDash,

    /// 8.2.4.17 Script data less-than sign state
    #[serde(skip)]
    ScriptDataLessThanSign,

    /// 8.2.4.43 Self-closing start tag state
    #[serde(skip)]
    SelfClosingStartTag,

    /// 8.2.4.10 Tag name state
    #[serde(skip)]
    TagName,

    /// 8.2.4.8 Tag open state
    #[serde(skip)]
    TagOpen,
}

impl TokenizerState {
    fn next_char(self, c: NextChar, builder: &mut TokenBuilder) -> Self {
        builder.push_buf(c);

        match self {
            Self::BogusComment
            | Self::CDATASection
            | Self::CommentStart
            | Self::PLAINTEXT
            | Self::RAWTEXT
            | Self::RCDATA => self.todo(c, builder),

            Self::AfterAttributeName => self.todo(c, builder),

            Self::AfterAttributeValueQuoted => match c {
                Ch(c) if c.is_ascii_whitespace() => {
                    builder.finalize_attribute();
                    Self::BeforeAttributeName
                }
                Ch('/') => {
                    builder.finalize_attribute();
                    Self::SelfClosingStartTag
                }
                Ch('>') => {
                    builder.finalize_attribute();
                    builder.emit_token();
                    Self::Data
                }
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                _ => {
                    builder.reconsume_char();
                    Self::BeforeAttributeName
                }
            },

            Self::AfterDOCTYPEName => match c {
                Eof | Stop => {
                    builder.add_error("eof-in-doctype");
                    if let Some(Token::Doctype { correctness, .. }) = &mut builder.token {
                        *correctness = false;
                    } else {
                        self.todo(Eof, builder);
                    }
                    Self::Data
                }
                _ => self.todo(c, builder),
            },

            Self::AttributeName => match c {
                Ch(c) if c.is_ascii_whitespace() => Self::AfterAttributeName,
                Ch('/') => Self::SelfClosingStartTag,
                Ch('=') => Self::BeforeAttributeValue,
                Ch('>') => {
                    builder.emit_token();
                    Self::Data
                }
                Ch(c @ 'A'..='Z') => {
                    builder.add_to_attribute_name(c.to_ascii_lowercase());
                    self
                }
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.add_to_attribute_name(c);
                    self
                }
            },

            Self::AttributeValueDoubleQuoted => self.todo(c, builder),

            Self::AttributeValueSingleQuoted => match c {
                Ch('\'') => Self::AfterAttributeValueQuoted,
                Ch('&') => self.todo(c, builder),
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.add_to_attribute_value(c);
                    self
                }
            },

            Self::AttributeValueUnquoted => match c {
                Ch(c) if c.is_ascii_whitespace() => {
                    builder.finalize_attribute();
                    Self::BeforeAttributeName
                }
                Ch('>') => {
                    builder.finalize_attribute();
                    Self::Data
                }
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.add_to_attribute_value(c);
                    self
                }
            },

            Self::BeforeAttributeName => match c {
                Ch(c) if c.is_ascii_whitespace() => self,
                Ch('/') => Self::SelfClosingStartTag,
                Ch('>') => {
                    builder.emit_token();
                    Self::Data
                }
                Ch(c @ 'A'..='Z') => {
                    builder.finalize_attribute();
                    builder.add_to_attribute_name(c.to_ascii_lowercase());
                    Self::AttributeName
                }
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.finalize_attribute();
                    builder.add_to_attribute_name(c);
                    Self::AttributeName
                }
            },

            Self::BeforeAttributeValue => match c {
                Ch('\'') => Self::AttributeValueSingleQuoted,
                Ch('"') => Self::AttributeValueDoubleQuoted,
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.add_to_attribute_value(c);
                    Self::AttributeValueUnquoted
                }
            },

            Self::BeforeDOCTYPEName => match c {
                Ch(c) if c.is_whitespace() => self,
                Ch(c @ 'A'..='Z') => {
                    builder.push_token(Token::Doctype {
                        name: Some(c.to_ascii_lowercase().into()),
                        public_id: None,
                        system_id: None,
                        correctness: true,
                    });
                    Self::DOCTYPEName
                }
                Eof | Stop => self.todo(c, builder),
                Ch(c) => {
                    builder.push_token(Token::Doctype {
                        name: Some(c.into()),
                        public_id: None,
                        system_id: None,
                        correctness: true,
                    });
                    Self::DOCTYPEName
                }
            },

            Self::Data => match c {
                Ch('<') => Self::TagOpen,
                Eof | Stop => {
                    builder.push_token(Token::Eof);
                    Self::Eof
                }
                Ch(c) => {
                    builder.push_char(c);
                    builder.add_error("invalid-first-character-of-tag-name");
                    self
                }
            },

            Self::DOCTYPE => match c {
                Ch(' ') => Self::BeforeDOCTYPEName,
                Eof | Stop => self.todo(c, builder),
                Ch(_) => self.todo(c, builder),
            },

            Self::DOCTYPEName => match c {
                Ch(' ') => Self::AfterDOCTYPEName,
                Ch('>') => {
                    builder.emit_token();
                    Self::Data
                }
                Ch(ch @ 'A'..='Z') => {
                    if let Some(Token::Doctype {
                        name: Some(name), ..
                    }) = &mut builder.token
                    {
                        name.push(ch.to_ascii_lowercase());
                        self
                    } else {
                        self.todo(c, builder);
                    }
                }
                Eof | Stop => {
                    builder.add_error("eof-in-doctype");
                    if let Some(Token::Doctype { correctness, .. }) = &mut builder.token {
                        *correctness = false;
                    } else {
                        self.todo(Eof, builder);
                    }
                    Self::Data
                }
                Ch(ch) => {
                    if let Some(Token::Doctype {
                        name: Some(name), ..
                    }) = &mut builder.token
                    {
                        name.push(ch);
                        self
                    } else {
                        self.todo(c, builder);
                    }
                }
            },

            Self::EndTagOpen => match c {
                Ch(c @ ('A'..='Z' | 'a'..='z')) => {
                    builder.push_token(Token::EndTag(c.to_string()));
                    Self::TagName
                }
                Ch('>') => {
                    builder.add_error("missing-end-tag-name");
                    Self::Data
                }
                _ => Self::BogusComment,
            },

            Self::Eof => self,

            Self::MarkupDeclarationOpen => {
                if builder.buf == "--" {
                    Self::CommentStart
                } else if builder.buf.to_ascii_lowercase() == "<!doctype" {
                    Self::DOCTYPE
                } else {
                    self
                }
            }

            Self::ScriptData => match c {
                Ch('<') => Self::ScriptDataLessThanSign,
                Eof | Stop => {
                    builder.push_token(Token::Eof);
                    builder.emit_token();
                    Self::Eof
                }
                Ch(c) => {
                    builder.push_char(c);
                    self
                }
            },

            Self::ScriptDataEndTagName => {
                fn anything(builder: &mut TokenBuilder) -> TokenizerState {
                    builder.discard_and_rewind();
                    builder.push_str("</");
                    builder.push_str(&builder.buf.to_owned());
                    TokenizerState::ScriptData
                }

                match c {
                    Ch('>') => {
                        if builder.appropriate_end_tag() {
                            builder.emit_token();
                            Self::Data
                        } else {
                            anything(builder)
                        }
                    }
                    Ch(ch @ 'a'..='z') => {
                        if let Some(Token::EndTag(name)) = &mut builder.token {
                            name.push(ch);
                        } else {
                            self.todo(c, builder);
                        }
                        self
                    }
                    _ => anything(builder),
                }
            }

            Self::ScriptDataEndTagOpen => match c {
                Ch('A'..='Z') => self.todo(c, builder),
                Ch(c @ 'a'..='z') => {
                    builder.push_token(Token::EndTag(c.into()));
                    Self::ScriptDataEndTagName
                }
                _ => {
                    builder.push_str("</");
                    builder.reconsume_char();
                    Self::ScriptData
                }
            },

            Self::ScriptDataEscaped => match c {
                Ch(c @ '-') => {
                    builder.push_char(c);
                    Self::ScriptDataEscapedDash
                }
                Ch('<') => Self::ScriptDataLessThanSign,
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.push_char(c);
                    self
                }
            },

            Self::ScriptDataEscapeStart => match c {
                Ch(c @ '-') => {
                    builder.push_char(c);
                    Self::ScriptDataEscapeStartDash
                }
                _ => {
                    builder.reconsume_char();
                    Self::ScriptData
                }
            },

            Self::ScriptDataEscapedDash => match c {
                Ch(c @ '-') => {
                    builder.push_char(c);
                    Self::ScriptDataEscapeDashDash
                }
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.push_char(c);
                    Self::ScriptDataEscaped
                }
            },

            Self::ScriptDataEscapeDashDash => match c {
                Ch(c @ '-') => {
                    builder.push_char(c);
                    self
                }
                Ch('<') => Self::ScriptDataLessThanSign,
                Ch(c @ '>') => {
                    builder.push_char(c);
                    Self::ScriptData
                }
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(c) => {
                    builder.push_char(c);
                    Self::ScriptDataEscaped
                }
            },

            Self::ScriptDataEscapeStartDash => match c {
                Ch(c @ '-') => {
                    builder.push_char(c);
                    Self::ScriptDataEscapeDashDash
                }
                _ => {
                    builder.reconsume_char();
                    Self::ScriptData
                }
            },

            Self::ScriptDataLessThanSign => match c {
                Ch('/') => {
                    builder.buf.clear();
                    Self::ScriptDataEndTagOpen
                }
                Ch('!') => {
                    builder.push_str("<!");
                    Self::ScriptDataEscapeStart
                }
                _ => {
                    builder.push_char('<');
                    builder.reconsume_char();
                    Self::ScriptData
                }
            },

            Self::SelfClosingStartTag => match c {
                Ch('>') => {
                    if let Some(Token::StartTag(element)) = &mut builder.token {
                        element.self_closing = true;
                        builder.emit_token();
                        Self::Data
                    } else {
                        builder.col -= 1;
                        builder.add_error("missing-end-tag-name");
                        Self::Data
                    }
                }
                _ => {
                    builder.reconsume_char();
                    Self::BeforeAttributeName
                }
            },

            Self::TagName => match c {
                Ch(c) if c.is_ascii_whitespace() => Self::BeforeAttributeName,
                Ch('>') => {
                    builder.emit_token();
                    Self::Data
                }
                Eof | Stop => {
                    builder.reconsume_char();
                    Self::Data
                }
                Ch(ch) => {
                    if let Some(Token::StartTag(StartTag { name, .. })) = &mut builder.token {
                        name.push(ch);
                        self
                    } else {
                        self.todo(c, builder);
                    }
                }
            },

            Self::TagOpen => match c {
                Ch('!') => Self::MarkupDeclarationOpen,
                Ch('/') => Self::EndTagOpen,
                Ch(c @ ('a'..='z' | 'A'..='Z')) => {
                    builder.push_token(Token::StartTag(StartTag {
                        name: c.to_lowercase().to_string(),
                        attributes: AttributeMap::new(),
                        self_closing: false,
                    }));
                    Self::TagName
                }
                _ => {
                    builder.reconsume_char();
                    builder.push_char('<');
                    Self::Data
                }
            },
        }
    }

    // For development only
    fn todo(&self, c: NextChar, builder: &TokenBuilder) -> ! {
        todo!("char: [{:?}], state: {:?}, builder: {:?}", c, self, builder);
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
enum NextChar {
    Ch(char),
    Eof,
    Stop,
}

use NextChar::*;

pub struct TokenIterator<'s> {
    builder: TokenBuilder,
    decoder: Decoder,
    dst: String,
    index: usize,
    src: Vec<u8>,
    state: TokenizerState,
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
    use super::*;

    fn iter(i: &str, state: TokenizerState) -> TokenIterator<'_> {
        Tokenizer::from_str(i, state, None)
            .iter()
            .expect("failed to get an iterator")
    }

    #[test]
    fn good() {
        // domjs.test, line 267
        let mut iter = iter("<!DOCTYPE html ", TokenizerState::Data);
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
        let mut iter = iter("</>", TokenizerState::Data);
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
        let mut iter = iter("<>", TokenizerState::Data);
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
