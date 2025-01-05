use super::Document;
use crate::types::{Error, Result};
use html5ever::interface::create_element;
use html5ever::tendril::fmt::UTF8;
use html5ever::tendril::Tendril;
use html5ever::tokenizer::{Doctype, TokenSink};
use html5ever::{
    namespace_url, ns,
    tendril::{fmt::Slice, StrTendril},
    tokenizer::{
        Tag,
        TagKind::{EndTag, StartTag},
        Token, TokenSinkResult,
    },
    tree_builder::{TreeBuilder, TreeBuilderOpts},
    Attribute, QualName,
};
use markup5ever::LocalName;
use markup5ever_rcdom::{Node, RcDom};
use quick_xml::events::attributes::Attributes;
use quick_xml::name::QName;
use quick_xml::{events::Event, reader::Reader};
use std::rc::Rc;
use tracing::{event, Level};

fn str_tendril(bytes: &[u8]) -> Result<Tendril<UTF8>> {
    let tendril = StrTendril::try_from_byte_slice(bytes.as_bytes())
        .map_err(|_| Error::General("failed to create StrTendril".into()))?;
    Ok(tendril)
}

fn start_tag(name: QName<'_>, attributes: Attributes<'_>, self_closing: bool) -> Result<Token> {
    let name = match name.decompose() {
        (name, None) => LocalName::from(std::str::from_utf8(name.as_ref())?),
        (name, Some(prefix)) => {
            let name = std::str::from_utf8(name.as_ref())?;
            let prefix = std::str::from_utf8(prefix.as_ref())?;
            let combined = format!("{prefix}:{name}");
            LocalName::from(combined)
        }
    };
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

    Ok(Token::TagToken(tag))
}

fn build_tree(mut builder: TreeBuilder<Rc<Node>, RcDom>, mut reader: Reader<&[u8]>) -> Result<Dom> {
    macro_rules! break_early {
        ($result:ident) => {
            if !matches!($result, TokenSinkResult::Continue) {
                break;
            }
        };
    }

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let token = start_tag(e.name(), e.html_attributes(), false)?;
                let result = builder.process_token(token, 0);
                break_early!(result);
            }

            Ok(Event::End(e)) => {
                let name = match e.name().decompose() {
                    (name, None) => LocalName::from(std::str::from_utf8(name.as_ref())?),
                    (name, Some(prefix)) => {
                        let name = std::str::from_utf8(name.as_ref())?;
                        let prefix = std::str::from_utf8(prefix.as_ref())?;
                        LocalName::from(format!("{prefix}:{name}"))
                    }
                };

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
                let token = start_tag(e.name(), e.html_attributes(), true)?;
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

            Ok(Event::CData(e)) => {
                let buf = e.as_ref();
                let buf = String::from_utf8_lossy(buf.as_bytes());
                let cdata = format!("[CDATA[{buf}]]");
                let token = Token::CommentToken(StrTendril::from(cdata));
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

            Ok(Event::PI(_) | Event::Decl(_)) => {
                continue;
            }

            Ok(Event::Eof) => {
                let _ = builder.process_token(Token::EOFToken, 0);
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

fn make_reader(data: &str) -> Reader<&[u8]> {
    let mut reader = Reader::from_str(data);
    let config = reader.config_mut();
    config.check_end_names = false;
    config.allow_unmatched_ends = true;
    config.trim_text_start = false;
    config.trim_text_end = false;
    reader
}

pub struct Dom {
    pub(crate) dom: RcDom,
    pub(crate) fragment: bool,
}

impl Document<'_, Dom> for Dom {
    fn parse_document(data: &str, scripting_enabled: bool) -> Result<Dom> {
        let reader = make_reader(data);
        let opts = TreeBuilderOpts {
            scripting_enabled,
            ..TreeBuilderOpts::default()
        };
        let builder = TreeBuilder::new(RcDom::default(), opts);
        build_tree(builder, reader)
    }

    fn parse_fragment(data: &'_ str, scripting_enabled: bool, context: &'_ str) -> Result<Dom> {
        let reader = make_reader(data);
        let opts = TreeBuilderOpts {
            scripting_enabled,
            ..TreeBuilderOpts::default()
        };

        let mut sink = RcDom::default();
        let local = LocalName::from(context);
        let name = QualName {
            prefix: None,
            ns: ns!(html),
            local,
        };
        let context = create_element(&mut sink, name, vec![]);
        let builder = TreeBuilder::new_for_fragment(sink, context, None, opts);

        build_tree(builder, reader)
    }
}
