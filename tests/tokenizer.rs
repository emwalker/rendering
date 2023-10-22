use rendering::testing::tokenizer::{fixture_from_filename, fixtures, FixtureFile, JsonFixtures};
use test_case::test_case;

#[test]
fn parsing_of_fixtures() {
    let _ = fixtures().expect("failed to load fixtures");
}

#[test_case("test1.test")]
fn tokenization(filename: &str) {
    let tests = fixture_from_filename(filename).expect("failed to open fixtures");

    let tests = if let FixtureFile::JsonFixtures(JsonFixtures { tests }) = &tests.fixtures {
        tests
    } else {
        panic!("expected json fixtures");
    };

    for test in tests.iter() {
        if test.description == "Entity with trailing semicolon (1)" {
            break;
        }

        println!("running test: {}", test.description);
        test.assert_valid();
    }
}
