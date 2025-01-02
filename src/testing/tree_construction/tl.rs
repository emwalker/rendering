use super::TreeConstructionResult;
use crate::html5::tl;

impl<'i> tl::Dom<'i> {
    fn serialize(&'i self) -> String {
        self.0.outer_html()
    }
}

impl<'i> TreeConstructionResult<'i, tl::Dom<'i>> {
    pub fn expected(&self) -> &str {
        &self.test.document
    }

    pub fn actual(&'i self) -> String {
        self.dom.serialize()
    }
}
