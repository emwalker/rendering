use super::Document;
use crate::types::Result;
use html5ever::{
    parse_document, serialize, tendril::TendrilSink, tree_builder::TreeBuilderOpts, ParseOpts,
};
use markup5ever_rcdom::{RcDom, SerializableHandle};

pub struct Dom(pub(crate) RcDom);

impl Dom {
    pub fn serialize(&self) -> String {
        let document: SerializableHandle = self.0.document.clone().into();
        let mut bytes = vec![];
        serialize(&mut bytes, &document, Default::default()).expect("serialization failed");
        String::from_utf8(bytes).expect("failed to convert to UTF8")
    }
}

impl<'i> Document<'i, Dom> for Dom {
    fn parse(input: &'i str) -> Result<Dom> {
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
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
