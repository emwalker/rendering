use crate::types::Result;
use nom::lib::std::fmt::Debug;

#[cfg(feature = "tl")]
pub mod tl;

pub struct Dom<T>(pub(crate) T);

pub trait Document<'i, T: Debug> {
    fn parse(input: &'i str) -> Result<Dom<T>>;
}
