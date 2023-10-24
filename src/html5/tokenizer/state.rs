use super::{builder::Builder, NextChar, StartTag, Token};
use crate::types::AttributeMap;
use serde::Deserialize;
use NextChar::*;

#[derive(Debug, Copy, Clone, Deserialize, PartialEq)]
pub enum State {
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

impl State {
    pub fn next_char(self, c: NextChar, builder: &mut Builder) -> Self {
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
                fn anything(builder: &mut Builder) -> State {
                    builder.discard_and_rewind();
                    builder.push_str("</");
                    builder.push_str(&builder.buf.to_owned());
                    State::ScriptData
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
    fn todo(&self, c: NextChar, builder: &Builder) -> ! {
        todo!("char: [{:?}], state: {:?}, builder: {:?}", c, self, builder);
    }
}
