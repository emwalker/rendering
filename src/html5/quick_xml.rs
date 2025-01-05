use super::Document;
use crate::types::{Error, Result};
use html5ever::tendril::fmt::UTF8;
use html5ever::tendril::Tendril;
use html5ever::tokenizer::{Doctype, TokenSink};
use html5ever::{
    namespace_url, ns,
    tendril::{fmt::Slice, StrTendril},
    tokenizer::{
        Tag,
        TagKind::{EndTag, StartTag},
        Token,
    },
    tree_builder::{TreeBuilder, TreeBuilderOpts},
    Attribute, QualName,
};
use markup5ever::LocalName;
use markup5ever_rcdom::RcDom;
use quick_xml::events::attributes::Attributes;
use quick_xml::name::QName;
use quick_xml::{events::Event, reader::Reader};
use tracing::{event, Level};

fn str_tendril(bytes: &[u8]) -> Result<Tendril<UTF8>> {
    let tendril = StrTendril::try_from_byte_slice(bytes.as_bytes())
        .map_err(|_| Error::General("failed to create StrTendril".into()))?;
    Ok(tendril)
}

fn start_tag(name: QName<'_>, attributes: Attributes<'_>, self_closing: bool) -> Token {
    let name = name.local_name();
    let name = LocalName::from(std::str::from_utf8(name.as_ref()).unwrap());
    let mut attrs = vec![];

    for result in attributes {
        let Ok(attr) = result else {
            continue;
        };

        let local = std::str::from_utf8(attr.key.into_inner()).unwrap();
        let name = QualName {
            prefix: None,
            ns: ns!(html),
            local: local.into(),
        };
        let value = StrTendril::try_from_byte_slice(attr.value.as_bytes()).unwrap();
        let attribute = Attribute { name, value };
        attrs.push(attribute);
    }

    let tag = Tag {
        kind: StartTag,
        name,
        self_closing,
        attrs,
    };

    Token::TagToken(tag)
}

pub struct Dom {
    pub(crate) dom: RcDom,
    pub(crate) fragment: bool,
}

impl Document<'_, Dom> for Dom {
    fn parse_document(input: &str, _parsing_enabled: bool) -> Result<Dom> {
        let mut reader = Reader::from_str(input);
        let config = reader.config_mut();
        config.trim_text(true);
        config.check_end_names = false;
        config.allow_unmatched_ends = true;

        let opts = TreeBuilderOpts::default();
        let mut builder = TreeBuilder::new(RcDom::default(), opts);

        macro_rules! break_early {
            ($result:ident) => {
                if !matches!($result, html5ever::tokenizer::TokenSinkResult::Continue) {
                    break;
                }
            };
        }

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let token = start_tag(e.name(), e.html_attributes(), false);
                    let result = builder.process_token(token, 0);
                    break_early!(result);
                }

                Ok(Event::End(e)) => {
                    let name = e.name().local_name();
                    let buf: &[u8] = name.as_ref();
                    let name = std::str::from_utf8(buf)?;
                    let name = LocalName::from(name);

                    let tag = Tag {
                        kind: EndTag,
                        name,
                        self_closing: false,
                        attrs: vec![],
                    };

                    let token = Token::TagToken(tag);
                    let result = builder.process_token(token, 0);
                    break_early!(result);
                }

                Ok(Event::Empty(e)) => {
                    let token = start_tag(e.name(), e.html_attributes(), true);
                    let result = builder.process_token(token, 0);
                    break_early!(result);
                }

                Ok(Event::Text(e)) => {
                    let buf = e.as_ref();
                    let token = Token::CharacterTokens(str_tendril(buf)?);
                    let result = builder.process_token(token, 0);
                    break_early!(result);
                }

                Ok(Event::Comment(e)) => {
                    let buf = e.as_ref();
                    let token = Token::CommentToken(str_tendril(buf)?);
                    let result = builder.process_token(token, 0);
                    break_early!(result);
                }

                Ok(Event::DocType(e)) => {
                    let buf = e.as_ref();
                    let mut name = str_tendril(buf)?;
                    name.make_ascii_lowercase();

                    let doc_type = Doctype {
                        name: Some(name),
                        public_id: None,
                        system_id: None,
                        force_quirks: false,
                    };

                    let token = Token::DoctypeToken(doc_type);
                    let result = builder.process_token(token, 0);
                    break_early!(result);
                }

                Ok(Event::CData(_) | Event::PI(_) | Event::Decl(_)) => {
                    continue;
                }

                Ok(Event::Eof) => {
                    builder.end();
                    break;
                }

                Err(e) => {
                    event!(Level::WARN, "error parsing document: {e}");
                }
            }
        }

        Ok(Dom {
            dom: builder.sink,
            fragment: false,
        })
    }
}
