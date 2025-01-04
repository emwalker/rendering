// See https://github.com/html5lib/html5lib-tests/tree/master/tree-construction
use crate::html5::Document;
use crate::types::Result;
use nom::lib::std::fmt::Debug;
use std::path::PathBuf;

#[cfg(feature = "html5ever")]
mod html5ever;
#[cfg(feature = "lol_html")]
mod lol_html;
mod parser;
#[cfg(feature = "quick-xml")]
mod quick_xml;
#[cfg(feature = "tl")]
mod tl;

pub use parser::fixture_from_filename;

#[derive(Debug, PartialEq)]
pub struct Position {
    line: usize,
    col: usize,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Message(String),

    Line {
        line: usize,
        message: String,
    },

    Location {
        pos: Position,
        message: String,
    },

    Span {
        start: Position,
        end: Position,
        message: String,
    },
}

#[derive(Debug)]
pub enum ScriptMode {
    ScriptOn,
    ScriptOff,
    Both,
}

pub trait TestSerialization {
    fn serialize(&mut self) -> String;
}

#[derive(Debug)]
pub struct Test {
    pub data: String,
    pub errors: Vec<ParseError>,
    pub new_errors: Vec<ParseError>,
    pub document_fragment: Option<String>,
    pub script_mode: ScriptMode,
    pub document: String,
}

impl<'i, S: TestSerialization> TreeConstructionResult<'i, S> {
    pub fn run(&'i mut self) -> (String, String) {
        let actual = self.dom.serialize();
        let expected = self.test.document.to_owned();
        (actual, expected)
    }
}

impl Test {
    pub fn results<'i, T>(&'i self) -> Result<Vec<TreeConstructionResult<'i, T>>>
    where
        T: Document<'i, T>,
    {
        let mut results = vec![];

        match self.script_mode {
            ScriptMode::ScriptOn => results.push(self.parse(true)?),
            ScriptMode::ScriptOff => results.push(self.parse(false)?),
            ScriptMode::Both => {
                results.push(self.parse(true)?);
                results.push(self.parse(false)?);
            }
        }

        Ok(results)
    }

    pub fn parse<'i, T>(&'i self, scripting_enabled: bool) -> Result<TreeConstructionResult<'i, T>>
    where
        T: Document<'i, T>,
    {
        let dom = if let Some(ref context) = self.document_fragment {
            T::parse_fragment(&self.data, scripting_enabled, context)?
        } else {
            T::parse_document(&self.data, scripting_enabled)?
        };

        Ok(TreeConstructionResult { dom, test: self })
    }
}

pub struct TreeConstructionResult<'i, T> {
    dom: T,
    test: &'i Test,
}

pub struct Tests {
    pub path: PathBuf,
    pub tests: Vec<Test>,
}

impl Tests {
    pub fn iter(&self) -> impl Iterator<Item = &Test> {
        self.tests.iter()
    }
}
