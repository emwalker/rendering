use super::{ParseError, ScriptMode, Test, Tests};
use crate::testing::FIXTURE_DIR;
use crate::types::Result;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, mem};

fn parse_errors(s: &str) -> Vec<ParseError> {
    s.lines()
        .map(|s| s.parse::<ParseError>())
        .collect::<Result<Vec<_>>>()
        .unwrap()
}

// From https://github.com/servo/html5ever/blob/8415d500150d3232036bd2fb9681e7820fd7ecea/rcdom/tests/html-tree-builder.rs#L33
fn parse_tests(s: &str) -> Vec<HashMap<String, String>> {
    let mut lines = s.lines();
    let mut tests = vec![];
    let mut test = HashMap::new();
    let mut key: Option<String> = None;
    let mut val = String::new();

    macro_rules! finish_val ( () => (
        match key.take() {
            None => (),
            Some(key) => {
                assert!(test.insert(key, mem::take(&mut val)).is_none());
            }
        }
    ));

    macro_rules! finish_test ( () => (
        if !test.is_empty() {
            tests.push(mem::take(&mut test));
        }
    ));

    loop {
        match lines.next() {
            None => break,
            Some(line) => {
                if let Some(rest) = line.strip_prefix('#') {
                    finish_val!();
                    if line == "#data" {
                        finish_test!();
                    }
                    key = Some(rest.to_owned());
                } else {
                    val.push_str(line);
                    val.push('\n');
                }
            }
        }
    }

    finish_val!();
    finish_test!();
    tests
}

fn make_test(test: HashMap<String, String>) -> Test {
    let data = test
        .get("data")
        .unwrap()
        .lines()
        .map(|l| l.trim_end_matches("\n"))
        .join("\n");

    let errors = test
        .get("errors")
        .map(|s| parse_errors(s))
        .unwrap_or_default();
    let new_errors = test
        .get("new-errors")
        .map(|s| parse_errors(s))
        .unwrap_or_default();

    let script_mode = if test.contains_key("script-on") {
        ScriptMode::ScriptOn
    } else if test.contains_key("script-off") {
        ScriptMode::ScriptOff
    } else {
        ScriptMode::Both
    };

    let document_fragment = test
        .get("document-fragment")
        .map(|s| s.trim_end_matches("\n").to_owned());
    let document = test
        .get("document")
        .unwrap()
        .trim_end_matches("\n")
        .to_string();

    Test {
        data,
        errors,
        new_errors,
        script_mode,
        document_fragment,
        document,
    }
}

pub fn fixture_from_path(path: &PathBuf) -> Result<Tests> {
    let s = fs::read_to_string(path)?;
    let tests: Vec<_> = parse_tests(&s).into_iter().map(make_test).collect();

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
