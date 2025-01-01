use crate::types::Result;
use nom::lib::std::fmt::Debug;

#[cfg(feature = "tl")]
pub mod tl;

#[cfg(feature = "lol_html")]
pub mod lol_html;

pub trait Document<'i, T: Debug> {
    fn parse(input: &'i str) -> Result<T>;
}
