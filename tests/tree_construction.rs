use rendering::testing::tree_construction::{fixture_from_filename, fixtures};
use test_case::test_case;

#[test]
fn parsing_of_fixtures() {
    let _ = fixtures().expect("failed to load fixtures");
}

#[test_case("tests1.dat")]
fn test(filename: &str) {
    let tests = fixture_from_filename(filename).expect("error loading fixture");

    for test in tests.iter() {
        println!("running {}", test.data);
        // Make sure we don't panic
        let _ = test.parse().expect("failed to parse");
    }
}
