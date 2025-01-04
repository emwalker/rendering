use super::Document;
use crate::types::Result;
use tl::{ParserOptions, VDom};

#[derive(Debug)]
pub struct Dom<'i>(pub(crate) VDom<'i>);

impl Dom<'_> {
    pub fn outer_html(&self) -> String {
        self.0.outer_html()
    }
}

impl<'i> Document<'i, Dom<'i>> for Dom<'i> {
    fn parse_document(input: &'i str) -> Result<Dom<'i>> {
        let dom = tl::parse(input, ParserOptions::default())?;
        Ok(Dom(dom))
    }
}
