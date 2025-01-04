use super::Document;
use crate::types::Result;
use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    reader::Reader,
    Writer,
};
use std::io::Cursor;

pub struct Dom<'i>(pub(crate) Reader<&'i [u8]>);

impl<'i> Document<'i, Dom<'i>> for Dom<'i> {
    fn parse_document(input: &str) -> Result<Dom<'_>> {
        let mut reader = Reader::from_str(input);
        reader.config_mut().trim_text(true);
        Ok(Dom(reader))
    }
}

impl Dom<'_> {
    pub fn outer_html(&mut self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        loop {
            match self.0.read_event() {
                Ok(Event::Start(e)) if e.name().as_ref() == b"this_tag" => {
                    // crates a new element ... alternatively we could reuse `e` by calling
                    // `e.into_owned()`
                    let mut elem = BytesStart::new("my_elem");

                    // collect existing attributes
                    elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                    // copy existing attributes, adds a new my-key="some value" attribute
                    elem.push_attribute(("my-key", "some value"));

                    // writes the event to the writer
                    assert!(writer.write_event(Event::Start(elem)).is_ok());
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"this_tag" => {
                    assert!(writer
                        .write_event(Event::End(BytesEnd::new("my_elem")))
                        .is_ok());
                }
                Ok(Event::Eof) => break,
                // we can either move or borrow the event to write, depending on your use-case
                Ok(e) => assert!(writer.write_event(e).is_ok()),
                Err(e) => panic!("Error at position {}: {:?}", self.0.error_position(), e),
            }
        }

        let bytes = writer.into_inner().into_inner();
        String::from_utf8(bytes).expect("failed to convert to UTF8")
    }
}
