// See https://github.com/html5lib/html5lib-tests/tree/master/tree-construction
use super::FIXTURE_DIR;
use crate::types::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_until1, take_while1},
    character::complete::{multispace0, multispace1},
    combinator::{all_consuming, eof, map, opt},
    multi::{many0, many1},
    sequence::{delimited, preceded, tuple},
    Finish, IResult,
};
use std::{fs, path::PathBuf};

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

#[derive(Debug)]
pub struct Test {
    pub data: String,
    pub errors: Vec<ParseError>,
    pub new_errors: Vec<ParseError>,
    pub document_fragment: Option<String>,
    pub script_mode: ScriptMode,
    pub document: String,
}

pub enum TreeConstructionResult {
    Success,
    Error,
}

impl Test {
    pub fn parse(&self) -> Result<TreeConstructionResult> {
        Ok(TreeConstructionResult::Success)
    }
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

fn data(i: &str) -> IResult<&str, &str> {
    preceded(tag("#data\n"), preceded(multispace0, take_until("#errors")))(i)
}

fn error_1(i: &str) -> IResult<&str, ParseError> {
    let location = map(
        tuple((
            tag("("),
            nom::character::complete::u64,
            tag(","),
            nom::character::complete::u64,
            tag(")"),
        )),
        |(_, line, _, col, _): (&str, u64, &str, u64, &str)| (line as usize, col as usize),
    );

    map(
        tuple((location, tag(": "), take_until1("\n"))),
        |((line, col), _, message)| ParseError::Location {
            pos: Position { line, col },
            message: message.trim().into(),
        },
    )(i)
}

fn error_2(i: &str) -> IResult<&str, ParseError> {
    let location = map(
        tuple((
            tag("("),
            nom::character::complete::u64,
            tag(":"),
            nom::character::complete::u64,
            tag(")"),
        )),
        |(_, line, _, col, _): (&str, u64, &str, u64, &str)| (line as usize, col as usize),
    );

    map(
        tuple((location, tag(" "), take_until1("\n"))),
        |((line, col), _, message)| ParseError::Location {
            pos: Position { line, col },
            message: message.trim().into(),
        },
    )(i)
}

fn error_3(i: &str) -> IResult<&str, ParseError> {
    let location = map(
        tuple((
            nom::character::complete::u64,
            tag(":"),
            nom::character::complete::u64,
        )),
        |(line, _, col): (u64, &str, u64)| (line as usize, col as usize),
    );

    map(
        tuple((location, tag(": "), take_until1("\n"))),
        |((line, col), _, message)| ParseError::Location {
            pos: Position { line, col },
            message: message.trim().into(),
        },
    )(i)
}

fn error_4(i: &str) -> IResult<&str, ParseError> {
    let location = map(
        tuple((
            alt((tag(" * ("), tag("* ("))),
            nom::character::complete::u64,
            tag(","),
            nom::character::complete::u64,
            tag(")"),
        )),
        |(_, line, _, col, _): (&str, u64, &str, u64, &str)| (line as usize, col as usize),
    );

    map(
        tuple((location, tag(" "), take_until1("\n"))),
        |((line, col), _, message)| ParseError::Location {
            pos: Position { line, col },
            message: message.trim().into(),
        },
    )(i)
}

fn error_5(i: &str) -> IResult<&str, ParseError> {
    map(
        tuple((nom::character::complete::u64, tag(": "), take_until1("\n"))),
        |(line, _, message): (u64, &str, &str)| ParseError::Line {
            line: line as _,
            message: message.trim().into(),
        },
    )(i)
}

// (1:44-1:49) non-void-html-element-start-tag-with-trailing-solidus
fn error_6(i: &str) -> IResult<&str, ParseError> {
    let span = map(
        tuple((
            tag("("),
            nom::character::complete::u64,
            tag(":"),
            nom::character::complete::u64,
            tag("-"),
            nom::character::complete::u64,
            tag(":"),
            nom::character::complete::u64,
            tag(")"),
        )),
        |(_, line1, _, col1, _, line2, _, col2, _): (
            &str,
            u64,
            &str,
            u64,
            &str,
            u64,
            &str,
            u64,
            &str,
        )| {
            (
                Position {
                    line: line1 as _,
                    col: col1 as _,
                },
                Position {
                    line: line2 as _,
                    col: col2 as _,
                },
            )
        },
    );

    map(
        tuple((span, tag(" "), take_until1("\n"))),
        |((start, end), _, message): ((Position, Position), &str, &str)| ParseError::Span {
            start,
            end,
            message: message.into(),
        },
    )(i)
}

fn error_messages(i: &str) -> IResult<&str, Vec<ParseError>> {
    map(take_until1("#"), |string: &str| {
        string
            .lines()
            .map(|s| ParseError::Message(s.into()))
            .collect::<Vec<_>>()
    })(i)
}

fn old_errors(i: &str) -> IResult<&str, Vec<ParseError>> {
    delimited(
        tuple((multispace0, tag("#errors\n"))),
        map(
            opt(alt((
                many1(delimited(
                    multispace0,
                    alt((error_1, error_2, error_3, error_4, error_5)),
                    tag("\n"),
                )),
                error_messages,
            ))),
            |errors| errors.unwrap_or_default(),
        ),
        multispace0,
    )(i)
}

fn new_errors(i: &str) -> IResult<&str, Vec<ParseError>> {
    delimited(
        tuple((multispace0, tag("#new-errors\n"))),
        many0(delimited(multispace0, alt((error_2, error_6)), tag("\n"))),
        multispace0,
    )(i)
}

fn document_line(i: &str) -> IResult<&str, String> {
    map(
        tuple((multispace0, tag("|"), take_until("\n"))),
        |(_, pipe, rest): (&str, &str, &str)| pipe.to_owned() + rest,
    )(i)
}

fn bare_line(i: &str) -> IResult<&str, String> {
    map(
        preceded(multispace0, take_while1(|c| c != '\n' && c != '#')),
        |s: &str| s.to_owned(),
    )(i)
}

fn document(i: &str) -> IResult<&str, String> {
    map(
        delimited(
            tuple((multispace0, tag("#document\n"))),
            many1(alt((document_line, bare_line))),
            multispace1,
        ),
        |doc: Vec<String>| doc.join("\n") + "\n",
    )(i)
}

fn document_fragment(i: &str) -> IResult<&str, &str> {
    preceded(tag("#document-fragment\n"), take_until1("#"))(i)
}

fn test(i: &str) -> IResult<&str, Test> {
    map(
        tuple((
            data,
            old_errors,
            opt(new_errors),
            opt(tag("#script-on\n")),
            opt(tag("#script-off\n")),
            opt(document_fragment),
            document,
            multispace0,
        )),
        |(data, errors, new_errors, script_on, script_off, document_fragment, document, _)| {
            let script_mode = match (&script_on, &script_off) {
                (Some("#script-on\n"), None) => ScriptMode::ScriptOn,
                (None, Some("#script-off\n")) => ScriptMode::ScriptOff,
                (Some(_), Some(_)) => unreachable!(),
                _ => ScriptMode::Both,
            };

            Test {
                data: data.into(),
                errors,
                new_errors: new_errors.unwrap_or_default(),
                script_mode,
                document_fragment: document_fragment.map(str::to_string),
                document,
            }
        },
    )(i)
}

fn parse_str(i: &str) -> Result<Vec<Test>> {
    let file = map(tuple((many1(test), eof)), |(tests, _)| tests);
    let (_, tests) = all_consuming(file)(i.trim_start())
        .finish()
        .map_err(|err| Error::TreeConstruction(format!("{}", err)))?;
    Ok(tests)
}

pub fn fixture_from_path(path: &PathBuf) -> Result<Tests> {
    let s = fs::read_to_string(path)?;
    let tests: Vec<Test> = parse_str(&s)?;

    Ok(Tests {
        path: path.into(),
        tests,
    })
}

pub fn fixture_from_filename(filename: &str) -> Result<Tests> {
    let path = PathBuf::from(FIXTURE_DIR)
        .join("tree-construction")
        .join(filename);
    fixture_from_path(&path)
}

pub fn fixtures() -> Result<Vec<Tests>> {
    let fixtures = PathBuf::from(FIXTURE_DIR).join("tree-construction");
    let mut tests = vec![];

    for entry in fs::read_dir(&fixtures).unwrap() {
        let path = entry.unwrap().path();

        if !path.is_file() || path.extension().expect("file ending") != "dat" {
            continue;
        }

        println!("loading test file {:?}", path);
        let fixtures: Tests = fixture_from_path(&path)?;
        tests.push(fixtures);
    }

    Ok(tests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_data() {
        assert_eq!(
            data("#data\n         Test \n#errors"),
            Ok(("#errors", "Test \n")),
        );

        assert_eq!(
            data(
                "#data\n<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Frameset//EN\"
                \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd\"><p><table>\n#errors",
            ),
            Ok(("#errors",  "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Frameset//EN\"\n                \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd\"><p><table>\n"))
        )
    }

    #[test]
    fn line() {
        let (_, line) = document_line("            | <html>\n").unwrap();
        assert_eq!(line, "| <html>");
    }

    #[test]
    fn parse_document() {
        let (_, doc) = document(
            r#"
#document
| <html>
|   <head>
|   <body>
|     "Test"
"#
            .trim_start(),
        )
        .unwrap();
        assert_eq!(doc, "| <html>\n|   <head>\n|   <body>\n|     \"Test\"\n");
    }

    #[test]
    fn tests1_dat_1() {
        let (_, test) = test(
            r#"
#data
Test
#errors
(1,0): expected-doctype-but-got-chars
#document
| <html>
|   <head>
|   <body>
|     "Test"
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.data, "Test\n");
        assert_eq!(
            test.errors,
            &[ParseError::Location {
                pos: Position { line: 1, col: 0 },
                message: "expected-doctype-but-got-chars".into(),
            }]
        );
        assert_eq!(
            test.document,
            "| <html>\n|   <head>\n|   <body>\n|     \"Test\"\n"
        );
    }

    #[test]
    fn quirks01_dat_1() {
        let (_, test) = test(
            r#"
#data
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Frameset//EN"
"http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd"><p><table>
#errors
(2,54): unknown-doctype
(2,64): eof-in-table
#document
| <!DOCTYPE html "-//W3C//DTD XHTML 1.0 Frameset//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd">
| <html>
|   <head>
|   <body>
|     <p>
|     <table>
"#.trim_start()
        ).unwrap();

        assert_eq!(test.errors.len(), 2);
    }

    #[test]
    fn quirks01_dat_15() {
        let (_, test) = test(
            r#"
#data
<!DOCTYPE html SYSTEM "http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd"><p><table>
#errors
(1,83): unknown-doctype
(1,93): eof-in-table
#document
| <!DOCTYPE html "" "http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd">
| <html>
|   <head>
|   <body>
|     <p>
|       <table>
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.errors.len(), 2);
    }

    #[test]
    fn comments01_dat_13() {
        let (_, test) = test(
            r#"
#data
FOO<!-- BAR --!>BAZ
#errors
(1,3): expected-doctype-but-got-chars
(1,15): unexpected-bang-after-double-dash-in-comment
#new-errors
(1:16) incorrectly-closed-comment
#document
| <html>
|   <head>
|   <body>
|     "FOO"
|     <!--  BAR  -->
|     "BAZ"
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.errors.len(), 2);
        assert_eq!(test.new_errors.len(), 1);
    }

    #[test]
    fn comments01_dat_42() {
        let (_, test) = test(
            r#"
#data
FOO<!-- BAR --!
>BAZ
#errors
(1,3): expected-doctype-but-got-chars
(2:5) eof-in-comment
#new-errors
(2:5) eof-in-comment
#document
| <html>
|   <head>
|   <body>
|     "FOO"
|     <!--  BAR --!
>BAZ -->
"#
            .trim_start(),
        )
        .unwrap();

        assert!(test.document.ends_with(">BAZ -->\n"));
    }

    #[test]
    fn tables01_dat_288() {
        let (_, test) = test(
            r#"
#data
<div><table><svg><foreignObject><select><table><s>
#errors
1:1: Expected a doctype token
1:13: 'svg' tag isn't allowed here. Currently open tags: html, body, div, table.
1:33: 'select' tag isn't allowed here. Currently open tags: html, body, div, table, svg, foreignobject.
1:41: 'table' tag isn't allowed here. Currently open tags: html, body, div, table, svg, foreignobject, select.
1:41: 'table' tag isn't allowed here. Currently open tags: html, body, div, table, svg, foreignobject.
1:48: 's' tag isn't allowed here. Currently open tags: html, body, div, table.
1:51: Premature end of file. Currently open tags: html, body, div, table, s.
#document
| <html>
|   <head>
|   <body>
|     <div>
|       <svg svg>
|         <svg foreignObject>
|           <select>
|       <table>
|       <s>
|       <table>
"#.trim_start(),
        ).unwrap();

        assert_eq!(test.errors.len(), 7);
    }

    #[test]
    fn template_dat_61() {
        let (_, test) = test(
            r#"
#data
<div><template><div><span></template><b>
#errors
 * (1,6) missing DOCTYPE
 * (1,38) mismatched template end tag
 * (1,41) unexpected end of file
#document
| <html>
|   <head>
|   <body>
|     <div>
|       <template>
|         content
|           <div>
|             <span>
|       <b>
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.errors.len(), 3);
    }

    #[test]
    fn template_dat_1659() {
        let (_, test) = test(
            r#"            
#data
<!DOCTYPE HTML><template><tr><td>cell</td></tr>a</template>
#errors
(1,59): foster-parenting-character
#document
| <!DOCTYPE html>
| <html>
|   <head>
|     <template>
|       content
|         <tr>
|           <td>
|             "cell"
|         "a"
|   <body>
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.errors.len(), 1);
    }

    #[test]
    fn template_data_148() {
        let (_, test) = test(
            r#"
#data
<table><template></template><div></div>
#errors
no doctype
bad div in table
bad /div in table
eof in table
#document
| <html>
|   <head>
|   <body>
|     <div>
|     <table>
|       <template>
|         content
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.errors.len(), 4);
    }

    #[test]
    fn template_dat_1613() {
        let (_, test) = test(
            r#"
            #data
<template><form><input name="q"></form><div>second</div></template>
#errors
#document-fragment
template
#document
| <template>
|   content
|     <form>
|       <input>
|         name="q"
|     <div>
|       "second"
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.document_fragment, Some("template\n".into()));
    }

    #[test]
    fn webkit02_dat_13() {
        let (_, test) = test(
            r#"
#data
<p id="status"><noscript><strong>A</strong></noscript><span>B</span></p>
#errors
(1,15): expected-doctype-but-got-start-tag
#script-on
#document
| <html>
|   <head>
|   <body>
|     <p>
|       id="status"
|       <noscript>
|         "<strong>A</strong>"
|       <span>
|         "B"
"#
            .trim_start(),
        )
        .unwrap();

        assert!(matches!(test.script_mode, ScriptMode::ScriptOn));
    }

    #[test]
    fn webkit02_dat_29() {
        let (_, test) = test(
            r#"
#data
<p id="status"><noscript><strong>A</strong></noscript><span>B</span></p>
#errors
(1,15): expected-doctype-but-got-start-tag
#script-off
#document
| <html>
|   <head>
|   <body>
|     <p>
|       id="status"
|       <noscript>
|         <strong>
|           "A"
|       <span>
|         "B"
"#
            .trim_start(),
        )
        .unwrap();

        assert!(matches!(test.script_mode, ScriptMode::ScriptOff));
    }

    #[test]
    fn tests08_dat_1() {
        let (_, test) = test(
            r#"
#data
<div>
<div></div>
</span>x
#errors
(1,5): expected-doctype-but-got-start-tag
(3,7): unexpected-end-tag
(3,8): expected-closing-tag-but-got-eof
#document
| <html>
|   <head>
|   <body>
|     <div>
|       "
"
|       <div>
|       "
x"
"#
            .trim_start(),
        )
        .unwrap();

        assert_eq!(test.errors.len(), 3);
        assert!(test.document.ends_with("\"\nx\"\n"));
    }

    #[test]
    fn parse_error_5() {
        assert_eq!(
            error_5("52: End of file seen and there were open elements.\n"),
            Ok((
                "\n",
                ParseError::Line {
                    line: 52,
                    message: "End of file seen and there were open elements.".into(),
                }
            )),
        )
    }

    #[test]
    fn parse_error_6() {
        assert_eq!(
            error_6("(1:44-1:49) non-void-html-element-start-tag-with-trailing-solidus\n"),
            Ok((
                "\n",
                ParseError::Span {
                    start: Position { line: 1, col: 44 },
                    end: Position { line: 1, col: 49 },
                    message: "non-void-html-element-start-tag-with-trailing-solidus".into(),
                }
            )),
        )
    }

    #[test]
    fn foreign_fragment_dat_169() {
        let (_, test) = test(
            r#"
#data
<b></b><mglyph/><i></i><malignmark/><u></u><ms/>X
#errors
51: Self-closing syntax (“/>”) used on a non-void HTML element. Ignoring the slash and treating as a start tag.
52: End of file seen and there were open elements.
#new-errors
(1:44-1:49) non-void-html-element-start-tag-with-trailing-solidus
#document-fragment
math ms
#document
| <b>
| <math mglyph>
| <i>
| <math malignmark>
| <u>
| <ms>
|   "X"
"#.trim_start(),
        ).unwrap();

        assert_eq!(test.errors.len(), 2);
        let error = test.errors.first().unwrap();
        assert!(matches!(error, ParseError::Line { .. }));

        assert_eq!(test.new_errors.len(), 1);
        let error = test.new_errors.first().unwrap();
        assert!(matches!(error, ParseError::Span { .. }));
    }
}
