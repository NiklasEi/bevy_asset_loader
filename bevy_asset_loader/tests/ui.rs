#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
    // Todo
    // fail with unknown asset attributes e.g. texture_atlas = "bla"
    // or asset(paths = "kg")
}
