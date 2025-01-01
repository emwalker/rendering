use crate::types::Result;

#[cfg(feature = "lol_html")]
pub mod lol_html;
#[cfg(feature = "quick-xml")]
pub mod quick_xml;
#[cfg(feature = "tl")]
pub mod tl;

pub trait Document<'i, T> {
    fn parse(input: &'i str) -> Result<T>;
}
