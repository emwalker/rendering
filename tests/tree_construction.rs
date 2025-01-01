use rendering::html5::{lol_html, tl};
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

        #[cfg(feature = "tl")]
        let _ = test.parse::<tl::Dom>().expect("failed to parse");
        // assert_eq!(result.expected(), result.actual());

        #[cfg(feature = "lol_html")]
        let _ = test.parse::<lol_html::Dom>().expect("failed to parse");
        // assert_eq!(result.expected(), result.actual());
    }
}
