use super::{Document, Dom};
use crate::types::Result;
use tl::{ParserOptions, VDom};

impl<'i> Document<'i, VDom<'i>> for VDom<'i> {
    fn parse(input: &'i str) -> Result<Dom<VDom<'i>>> {
        let dom = tl::parse(input, ParserOptions::default())?;
        Ok(Dom(dom))
    }
}
