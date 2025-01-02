use super::Document;
use crate::types::Result;
use html5ever::{parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts, ParseOpts};
use markup5ever_rcdom::RcDom;

pub struct Dom(pub(crate) RcDom);

impl<'i> Document<'i, Dom> for Dom {
    fn parse(input: &'i str) -> Result<Dom> {
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut input.as_bytes())
            .unwrap();
        Ok(Dom(dom))
    }
}
