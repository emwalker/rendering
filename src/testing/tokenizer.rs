use super::FIXTURE_DIR;
use crate::{
    html5::tokenizer::{StartTag, Token, TokenError, TokenResult, Tokenizer, TokenizerState},
    types::{AttributeMap, Result},
};
use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use serde_json::Value;
use std::{fs, path::PathBuf};

struct OutputVisitor;

impl<'de> Visitor<'de> for OutputVisitor {
    type Value = Token;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Assertion variant")
    }
}

impl<'de> Deserialize<'de> for Token {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values: Vec<Value> = Deserialize::deserialize(deserializer)?;
        let kind: &str = values[0].as_str().unwrap();

        fn attributes(value: &Value) -> AttributeMap {
            value
                .as_object()
                .unwrap()
                .into_iter()
                .filter_map(|(name, value)| {
                    if value.is_null() {
                        None
                    } else {
                        Some((name.to_owned(), value.as_str().unwrap().to_owned()))
                    }
                })
                .collect::<AttributeMap>()
        }

        match values.len() {
            2 => match kind {
                "Character" => Ok(Token::Character(values[1].as_str().unwrap().to_owned())),

                "Comment" => Ok(Token::Comment(values[1].as_str().unwrap().to_owned())),

                "EndTag" => Ok(Token::EndTag(values[1].as_str().unwrap().to_owned())),

                _ => Err(D::Error::invalid_value(
                    Unexpected::Str(kind),
                    &"Character, Comment or EndTag",
                )),
            },

            3 => match kind {
                "StartTag" => Ok(Token::StartTag(StartTag {
                    name: values[1].as_str().unwrap().to_owned(),
                    attributes: attributes(&values[2]),
                    self_closing: false,
                })),

                _ => Err(D::Error::invalid_value(Unexpected::Str(kind), &"StartTag")),
            },

            4 => match kind {
                "StartTag" => Ok(Token::StartTag(StartTag {
                    name: values[1].as_str().unwrap().to_owned(),
                    attributes: attributes(&values[2]),
                    self_closing: values[1].as_bool().unwrap_or_default(),
                })),

                _ => Err(D::Error::invalid_value(Unexpected::Str(kind), &"StartTag")),
            },

            5 => match kind {
                "DOCTYPE" => Ok(Token::Doctype {
                    name: values[1].as_str().map(str::to_owned),
                    public_id: values[2].as_str().map(str::to_owned),
                    system_id: values[3].as_str().map(str::to_owned),
                    correctness: values[4].as_bool().unwrap_or_default(),
                }),

                _ => Err(D::Error::invalid_value(Unexpected::Str(kind), &"DOCTYPE")),
            },

            _ => Err(D::Error::invalid_length(
                values.len(),
                &"an array of length 2, 3, 4 or 5",
            )),
        }
    }
}

pub struct TestResult {
    output: Vec<TokenResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonTest {
    pub description: String,
    #[serde(default = "Vec::new")]
    pub errors: Vec<TokenError>,
    #[serde(default = "Vec::new")]
    initial_states: Vec<TokenizerState>,
    pub input: String,
    pub last_start_tag: Option<String>,
    pub output: Vec<Token>,
    pub double_escaped: Option<bool>,
}

impl JsonTest {
    pub fn pump_tokenizer(&self, initial_state: TokenizerState) -> Result<TestResult> {
        let last_start_tag = self.last_start_tag.as_ref().map(StartTag::from);
        let tokenizer = Tokenizer::from_str(&self.input, initial_state, last_start_tag);
        let output = tokenizer.iter()?.collect::<Vec<_>>();
        Ok(TestResult { output })
    }

    pub fn assert_valid(&self) {
        for state in self.initial_states() {
            let TestResult { output } = self.pump_tokenizer(state).unwrap();

            for (expected, actual) in self.output.iter().zip(output) {
                if let TokenResult::Ok { token: actual, .. } = actual {
                    assert_eq!(expected, &actual);
                }
            }
        }
    }

    pub fn initial_states(&self) -> Vec<TokenizerState> {
        if self.initial_states.is_empty() {
            return vec![TokenizerState::Data];
        }
        self.initial_states.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct XmlValidationFixtures {
    #[serde(rename = "xmlViolationTests")]
    pub tests: Vec<JsonTest>,
}

#[derive(Debug, Deserialize)]
pub struct JsonFixtures {
    pub tests: Vec<JsonTest>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FixtureFile {
    JsonFixtures(JsonFixtures),
    XmlFixtures(XmlValidationFixtures),
}

pub struct Tests {
    pub path: PathBuf,
    pub fixtures: FixtureFile,
}

pub fn parse_str(i: &str) -> Result<FixtureFile> {
    let fixtures: FixtureFile = serde_json::from_str(i)?;
    Ok(fixtures)
}

pub fn fixture_from_path(path: &PathBuf) -> Result<Tests> {
    let s = fs::read_to_string(path)?;
    let fixtures: FixtureFile = parse_str(&s)?;

    Ok(Tests {
        path: path.into(),
        fixtures,
    })
}

pub fn fixture_from_filename(filename: &str) -> Result<Tests> {
    let path = PathBuf::from(FIXTURE_DIR).join("tokenizer").join(filename);
    fixture_from_path(&path)
}

pub fn fixtures() -> Result<Vec<Tests>> {
    let fixtures = PathBuf::from(FIXTURE_DIR).join("tokenizer");
    let mut tests = vec![];

    for entry in fs::read_dir(&fixtures).unwrap() {
        let path = entry.unwrap().path();

        if !path.is_file() || path.extension().expect("file ending") != "test" {
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
    use std::collections::HashMap;

    use super::*;

    fn parse(i: &str) -> JsonTest {
        serde_json::from_str(i).expect("error parsing")
    }

    #[test]
    fn entities1_test_3() {
        let test = parse(
            r#"{
                "description": "Undefined named entity in a double-quoted attribute value ending in semicolon and whose name starts with a known entity name.",
                "input":"<h a=\"&noti;\">",
                "output": [["StartTag", "h", {"a": "&noti;"}]]
            }"#,
        );

        assert!(test.description.starts_with("Undefined"));
        assert_eq!(test.input, "<h a=\"&noti;\">");
        assert_eq!(
            test.output,
            &[Token::StartTag(StartTag {
                name: "h".into(),
                attributes: HashMap::from([("a".into(), "&noti;".into())]),
                self_closing: false,
            })],
        );
    }

    #[test]
    fn domjs_test_3() {
        let test = parse(
            r#"{
                "description":"CR in bogus comment state",
                "input":"<?\u000d",
                "output":[["Comment", "?\u000a"]],
                "errors":[
                    { "code": "unexpected-question-mark-instead-of-tag-name", "line": 1, "col": 2 }
                ]
            }"#,
        );

        assert_eq!(test.description, "CR in bogus comment state");
    }

    #[test]
    fn domjs_test_267() {
        let test = parse(
            r#"{
                "description":"space EOF after doctype ",
                "input":"<!DOCTYPE html ",
                "output":[["DOCTYPE", "html", null, null , false]],
                "errors":[
                    { "code": "eof-in-doctype", "line": 1, "col": 16 }
                ]
            }"#,
        );

        assert_eq!(test.description, "space EOF after doctype ");

        if let Token::Doctype { name, .. } = &test.output[0] {
            assert_eq!(name, &Some("html".into()));
        } else {
            panic!();
        };

        let error = &test.errors[0];
        assert_eq!(
            error,
            &TokenError {
                code: "eof-in-doctype".into(),
                line: 1,
                col: 16
            }
        );
    }

    #[test]
    fn xml_violation_tests() {
        let input = r#"
        {"xmlViolationTests": [
            {"description":"Non-XML character",
            "input":"a\uFFFFb",
            "output":[["Character","a\uFFFDb"]]},

            {"description":"Non-XML space",
            "input":"a\u000Cb",
            "output":[["Character","a b"]]},

            {"description":"Double hyphen in comment",
            "input":"<!-- foo -- bar -->",
            "output":[["Comment"," foo - - bar "]]},

            {"description":"FF between attributes",
            "input":"<a b=''\u000Cc=''>",
            "output":[["StartTag","a",{"b":"","c":""}]]}
        ]}"#;

        let fixtures: XmlValidationFixtures = serde_json::from_str(input).expect("failed to parse");
        assert_eq!(fixtures.tests.len(), 4);
    }

    #[test]
    fn test2_test_3() {
        let input = r#"
        {"description":"DOCTYPE without name",
        "input":"<!DOCTYPE>",
        "output":[["DOCTYPE", null, null, null, false]],
        "errors":[
            { "code": "missing-doctype-name", "line": 1, "col": 10 }
        ]}"#;

        let test: JsonTest = serde_json::from_str(input).expect("failed to parse");

        let output = &test.output[0];
        assert!(matches!(output, Token::Doctype { .. }));
    }

    #[test]
    fn double_escaped() {
        let input = r#"{
            "description":"NUL in CDATA section",
            "doubleEscaped":true,
            "initialStates":["CDATA section state"],
            "input":"\\u0000]]>",
            "output":[["Character", "\\u0000"]]
        }"#;

        let test: JsonTest = serde_json::from_str(input).expect("failed to parse");
        assert_eq!(test.double_escaped, Some(true));
    }
}
