use crate::types::{AttributeMap, Result};
use encoding_rs::{Decoder, Encoding};
use serde::Deserialize;
use std::{
    io::{BufReader, Read},
    iter::Peekable,
};

type InputStream<'s> = Peekable<std::io::Bytes<BufReader<&'s [u8]>>>;

pub struct Tokenizer<'s> {
    encoding: &'static Encoding,
    raw: InputStream<'s>,
    last_start_tag: Option<StartTag>,
}

impl<'s> Tokenizer<'s> {
    pub fn from_str(i: &'s str, last_start_tag: Option<StartTag>) -> Self {
        let raw = BufReader::new(i.as_bytes()).bytes().peekable();
        Self {
            raw,
            encoding: encoding_rs::UTF_8,
            last_start_tag,
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq)]
pub enum TokenizerState {
    #[serde(rename = "CDATA section state")]
    CData,
    #[serde(rename = "Data state")]
    Data,
    #[serde(rename = "PLAINTEXT state")]
    PlainText,
    #[serde(rename = "RAWTEXT state")]
    RawText,
    #[serde(rename = "RCDATA state")]
    RCData,
    #[serde(rename = "Script data state")]
    ScriptData,
    #[serde(skip)]
    TagOpen,
}

impl<'s> IntoIterator for Tokenizer<'s> {
    type Item = Result<Token>;

    type IntoIter = TokenIterator<'s>;

    fn into_iter(self) -> Self::IntoIter {
        let decoder = self.encoding.new_decoder_without_bom_handling();

        TokenIterator {
            _tokenizer: self,
            src: Vec::new(),
            dst: Vec::new(),
            state: TokenizerState::Data,
            decoder,
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

    StartTag(StartTag),
}

pub struct TokenIterator<'s> {
    _tokenizer: Tokenizer<'s>,
    src: Vec<u8>,
    dst: Vec<u8>,
    decoder: Decoder,
    state: TokenizerState,
}

impl Iterator for TokenIterator<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Ok(_c) = self.next_char()? {
                self.state = TokenizerState::TagOpen;
            } else {
                // Log error
                break;
            }
        }

        None
    }
}

impl TokenIterator<'_> {
    fn next_char(&mut self) -> Option<Result<char>> {
        let last = false;
        let (_result, _total_read, _total_written, _had_errors) =
            self.decoder
                .decode_to_utf8(&mut self.src, &mut self.dst, last);
        self.dst.pop().map(|c| Ok(c as _))
    }
}
