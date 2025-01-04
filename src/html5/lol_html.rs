use super::Document;
use crate::types::Result;
use lol_html::{HtmlRewriter, Settings};

#[derive(Debug)]
pub struct Dom(pub(crate) Vec<u8>);

impl Dom {
    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.0
    }
}

impl<'i> Document<'i, Dom> for Dom {
    fn parse_document(input: &'i str) -> Result<Dom> {
        let mut output = vec![];
        let mut rewriter =
            HtmlRewriter::new(Settings::new(), |c: &[u8]| output.extend_from_slice(c));
        rewriter.write(input.as_bytes())?;
        rewriter.end()?;
        Ok(Dom(output))
    }
}
