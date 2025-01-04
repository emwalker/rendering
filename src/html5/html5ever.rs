use super::Document;
use crate::types::Result;
use html5ever::{
    namespace_url, ns, parse_document, parse_fragment, tendril::TendrilSink, LocalName, ParseOpts,
    QualName,
};
use markup5ever_rcdom::RcDom;

fn context_name(context: &str) -> QualName {
    if let Some(cx) = context.strip_prefix("svg ") {
        QualName::new(None, ns!(svg), LocalName::from(cx))
    } else if let Some(cx) = context.strip_prefix("math ") {
        QualName::new(None, ns!(mathml), LocalName::from(cx))
    } else {
        QualName::new(None, ns!(html), LocalName::from(context))
    }
}

pub struct Dom {
    pub(crate) dom: RcDom,
    pub(crate) fragment: bool,
}

impl<'i> Document<'i, Dom> for Dom {
    fn parse_document(data: &'i str, scripting_enabled: bool) -> Result<Dom> {
        let mut opts: ParseOpts = Default::default();
        opts.tree_builder.scripting_enabled = scripting_enabled;
        let dom = parse_document(RcDom::default(), opts).one(data);
        Ok(Dom {
            dom,
            fragment: false,
        })
    }

    fn parse_fragment(data: &'i str, scripting_enabled: bool, context: &'i str) -> Result<Dom> {
        let mut opts: ParseOpts = Default::default();
        opts.tree_builder.scripting_enabled = scripting_enabled;
        let context = context_name(context);
        let dom = parse_fragment(RcDom::default(), opts, context, vec![]).one(data);
        Ok(Dom {
            dom,
            fragment: true,
        })
    }
}
