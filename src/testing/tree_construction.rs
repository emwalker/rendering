// See https://github.com/html5lib/html5lib-tests/tree/master/tree-construction
use crate::html5::Document;
use crate::types::{Error, Result};
use regex::Regex;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::LazyLock;

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

impl FromStr for ParseError {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        static LINE_COL1: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^*?\s*\((\d+)(?:,|:)\s*(\d+)\):? (.+)$").unwrap());
        static LINE_COL2: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^Line:? (\d+) Col:? (\d+) (.+)$").unwrap());
        static LINE_COL3: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^(\d+):(\d+): (.+)$").unwrap());
        static LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+): (.+)$").unwrap());

        if let Some(caps) = LINE_COL1.captures(s) {
            let (_, [line, col, message]) = caps.extract();
            let pos = Position {
                line: line.parse::<usize>().unwrap(),
                col: col.parse::<usize>().unwrap(),
            };

            return Ok(Self::Location {
                pos,
                message: message.to_owned(),
            });
        }

        if let Some(caps) = LINE_COL2.captures(s) {
            let (_, [line, col, message]) = caps.extract();
            let pos = Position {
                line: line.parse::<usize>().unwrap(),
                col: col.parse::<usize>().unwrap(),
            };

            return Ok(Self::Location {
                pos,
                message: message.to_owned(),
            });
        }

        if let Some(caps) = LINE_COL3.captures(s) {
            let (_, [line, col, message]) = caps.extract();
            let pos = Position {
                line: line.parse::<usize>().unwrap(),
                col: col.parse::<usize>().unwrap(),
            };

            return Ok(Self::Location {
                pos,
                message: message.to_owned(),
            });
        }

        if let Some(caps) = LINE.captures(s) {
            let (_, [line, message]) = caps.extract();
            return Ok(Self::Line {
                line: line.parse::<usize>().unwrap(),
                message: message.to_owned(),
            });
        }

        Ok(ParseError::Message(s.into()))
    }
}

#[derive(Clone, Copy, Debug)]
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
