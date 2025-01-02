use super::TestSerialization;
use crate::html5::lol_html;

impl TestSerialization for lol_html::Dom {
    fn serialize(&mut self) -> String {
        String::from_utf8(self.as_bytes().clone()).expect("failed to convert to UTF8")
    }
}
