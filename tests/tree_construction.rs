use rendering::html5::html5ever;
use rendering::html5::lol_html;
use rendering::html5::quick_xml;
use rendering::html5::tl;
use rendering::testing::tree_construction::fixture_from_filename;
use test_case::test_case;

macro_rules! parses {
    ($type:ty, $func:ident, [$($filename:expr),*]) => {
        $(
            #[test_case($filename)]
        )*
        fn $func(filename: &str) {
            let tests = fixture_from_filename(filename).expect("error loading fixture");

            for test in tests.iter() {
                println!("running {}", test.data);

                let _ = test.parse::<$type>(false).unwrap();
            }
        }
    };
}

macro_rules! passes {
    ($type:ty, $func:ident, [$($filename:expr),*]) => {
        $(
            #[test_case($filename)]
        )*
        fn $func(filename: &str) {
            let tests = fixture_from_filename(filename).expect("error loading fixture");

            for test in tests.iter() {
                println!("running {}", test.data);

                let results = test.results::<$type>().unwrap();

                for mut result in results {
                    let (actual, expected) = result.run();
                    assert_eq!(actual, expected);
                }
            }
        }
    };
}

parses!(
    tl::Dom,
    test_tl_dom_parses_fragments,
    [
        "adoption01.dat",
        "adoption02.dat",
        "blocks.dat",
        "comments01.dat",
        "doctype01.dat",
        "domjs-unsafe.dat",
        "entities01.dat",
        "entities02.dat",
        "foreign-fragment.dat",
        "html5test-com.dat",
        "inbody01.dat",
        "isindex.dat",
        "main-element.dat",
        "math.dat",
        "menuitem-element.dat",
        "namespace-sensitivity.dat",
        "noscript01.dat",
        "pending-spec-changes.dat",
        "pending-spec-changes-plain-text-unsafe.dat",
        "plain-text-unsafe.dat",
        "quirks01.dat",
        "ruby.dat",
        "scriptdata01.dat",
        "search-element.dat",
        "svg.dat",
        "tables01.dat",
        "template.dat",
        "tests10.dat",
        "tests11.dat",
        "tests12.dat",
        "tests14.dat",
        "tests15.dat",
        "tests16.dat",
        "tests17.dat",
        "tests18.dat",
        "tests19.dat",
        "tests1.dat",
        "tests20.dat",
        "tests21.dat",
        "tests22.dat",
        "tests23.dat",
        "tests24.dat",
        "tests25.dat",
        "tests26.dat",
        "tests2.dat",
        "tests3.dat",
        "tests4.dat",
        "tests5.dat",
        "tests6.dat",
        "tests7.dat",
        "tests8.dat",
        "tests9.dat",
        "tests_innerHTML_1.dat",
        "tricky01.dat",
        "webkit01.dat",
        "webkit02.dat"
    ]
);

parses!(
    lol_html::Dom,
    test_lol_html_dom_parses_fragments,
    [
        "adoption01.dat",
        "adoption02.dat",
        "blocks.dat",
        "comments01.dat",
        "doctype01.dat",
        "domjs-unsafe.dat",
        "entities01.dat",
        "entities02.dat",
        "foreign-fragment.dat",
        "html5test-com.dat",
        "inbody01.dat",
        "isindex.dat",
        "main-element.dat",
        "math.dat",
        "menuitem-element.dat",
        "namespace-sensitivity.dat",
        "noscript01.dat",
        "pending-spec-changes.dat",
        "pending-spec-changes-plain-text-unsafe.dat",
        "plain-text-unsafe.dat",
        "quirks01.dat",
        "ruby.dat",
        "scriptdata01.dat",
        "search-element.dat",
        "svg.dat",
        "tables01.dat",
        "template.dat",
        "tests10.dat",
        "tests11.dat",
        "tests12.dat",
        "tests14.dat",
        "tests15.dat",
        "tests16.dat",
        "tests17.dat",
        // "tests18.dat",
        "tests19.dat",
        "tests1.dat",
        "tests20.dat",
        "tests21.dat",
        "tests22.dat",
        "tests23.dat",
        "tests24.dat",
        "tests25.dat",
        "tests26.dat",
        "tests2.dat",
        "tests3.dat",
        "tests4.dat",
        "tests5.dat",
        "tests6.dat",
        "tests7.dat",
        "tests8.dat",
        "tests9.dat",
        "tests_innerHTML_1.dat",
        "tricky01.dat",
        "webkit01.dat",
        "webkit02.dat"
    ]
);

parses!(
    quick_xml::Dom,
    test_quick_xml_dom_parses_fragments,
    [
        "adoption01.dat",
        "adoption02.dat",
        "blocks.dat",
        "comments01.dat",
        "doctype01.dat",
        "domjs-unsafe.dat",
        "entities01.dat",
        "entities02.dat",
        "foreign-fragment.dat",
        "html5test-com.dat",
        "inbody01.dat",
        "isindex.dat",
        "main-element.dat",
        "math.dat",
        "menuitem-element.dat",
        "namespace-sensitivity.dat",
        "noscript01.dat",
        "pending-spec-changes.dat",
        "pending-spec-changes-plain-text-unsafe.dat",
        "plain-text-unsafe.dat",
        "quirks01.dat",
        "ruby.dat",
        "scriptdata01.dat",
        "search-element.dat",
        "svg.dat",
        "tables01.dat",
        "template.dat",
        "tests10.dat",
        "tests11.dat",
        "tests12.dat",
        "tests14.dat",
        "tests15.dat",
        "tests16.dat",
        "tests17.dat",
        "tests18.dat",
        "tests19.dat",
        "tests1.dat",
        "tests20.dat",
        "tests21.dat",
        "tests22.dat",
        "tests23.dat",
        "tests24.dat",
        "tests25.dat",
        "tests26.dat",
        "tests2.dat",
        "tests3.dat",
        "tests4.dat",
        "tests5.dat",
        "tests6.dat",
        "tests7.dat",
        "tests8.dat",
        "tests9.dat",
        "tests_innerHTML_1.dat",
        "tricky01.dat",
        "webkit01.dat",
        "webkit02.dat"
    ]
);

parses!(
    html5ever::Dom,
    test_html5ever_dom_parses_fragments,
    [
        "adoption01.dat",
        "adoption02.dat",
        "blocks.dat",
        "comments01.dat",
        "doctype01.dat",
        "domjs-unsafe.dat",
        "entities01.dat",
        "entities02.dat",
        "foreign-fragment.dat",
        "html5test-com.dat",
        "inbody01.dat",
        "isindex.dat",
        "main-element.dat",
        "math.dat",
        "menuitem-element.dat",
        "namespace-sensitivity.dat",
        "noscript01.dat",
        "pending-spec-changes.dat",
        "pending-spec-changes-plain-text-unsafe.dat",
        "plain-text-unsafe.dat",
        "quirks01.dat",
        "ruby.dat",
        "scriptdata01.dat",
        "search-element.dat",
        "svg.dat",
        "tables01.dat",
        "template.dat",
        "tests10.dat",
        "tests11.dat",
        "tests12.dat",
        "tests14.dat",
        "tests15.dat",
        "tests16.dat",
        "tests17.dat",
        "tests18.dat",
        "tests19.dat",
        "tests1.dat",
        "tests20.dat",
        "tests21.dat",
        "tests22.dat",
        "tests23.dat",
        "tests24.dat",
        "tests25.dat",
        "tests26.dat",
        "tests2.dat",
        "tests3.dat",
        "tests4.dat",
        "tests5.dat",
        "tests6.dat",
        "tests7.dat",
        "tests8.dat",
        "tests9.dat",
        "tests_innerHTML_1.dat",
        "tricky01.dat",
        "webkit01.dat",
        "webkit02.dat"
    ]
);

passes!(
    html5ever::Dom,
    test_html5ever_dom_passes_test,
    [
        "adoption02.dat",
        "blocks.dat",
        "comments01.dat",
        "doctype01.dat",
        "domjs-unsafe.dat",
        "entities01.dat",
        "entities02.dat",
        "html5test-com.dat",
        "inbody01.dat",
        "isindex.dat",
        "main-element.dat",
        "menuitem-element.dat",
        "namespace-sensitivity.dat",
        "pending-spec-changes.dat",
        "pending-spec-changes-plain-text-unsafe.dat",
        "plain-text-unsafe.dat",
        "quirks01.dat",
        "ruby.dat",
        "scriptdata01.dat",
        "search-element.dat",
        "tables01.dat",
        "tests10.dat",
        "tests11.dat",
        "tests12.dat",
        "tests14.dat",
        "tests15.dat",
        "tests17.dat",
        "tests19.dat",
        "tests1.dat",
        "tests20.dat",
        "tests21.dat",
        "tests22.dat",
        "tests23.dat",
        "tests24.dat",
        "tests25.dat",
        "tests2.dat",
        "tests3.dat",
        "tests8.dat",
        "tests9.dat"
    ]
);
