use crate::types::Result;

#[cfg(feature = "html5ever")]
pub mod html5ever;
#[cfg(feature = "quick-xml")]
pub mod quick_xml;

pub trait Document<'i, T> {
    fn parse_document(data: &'i str, scripting_enabled: bool) -> Result<T>;

    #[allow(unused_variables)]
    fn parse_fragment(data: &'i str, scripting_enabled: bool, context: &'i str) -> Result<T> {
        Self::parse_document(data, scripting_enabled)
    }
}
