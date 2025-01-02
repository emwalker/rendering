use super::TestSerialization;
use crate::html5::quick_xml;

impl TestSerialization for quick_xml::Dom<'_> {
    fn serialize(&mut self) -> String {
        self.outer_html()
    }
}
