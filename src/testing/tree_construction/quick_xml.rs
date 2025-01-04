use super::html5ever::serialize_dom;
use super::TestSerialization;
use crate::html5::quick_xml;

impl TestSerialization for quick_xml::Dom {
    fn serialize(&mut self) -> String {
        serialize_dom(&self.dom, self.fragment)
    }
}
