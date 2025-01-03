use crate::types::Result;

#[cfg(feature = "html5ever")]
pub mod html5ever;
#[cfg(feature = "lol_html")]
pub mod lol_html;
#[cfg(feature = "quick-xml")]
pub mod quick_xml;
#[cfg(feature = "tl")]
pub mod tl;

pub trait Document<'i, T> {
    fn parse_document(data: &'i str) -> Result<T>;

    fn parse_fragment(data: &'i str, _context: &'i str, _scripting_enabled: bool) -> Result<T> {
        Self::parse_document(data)
    }
}
