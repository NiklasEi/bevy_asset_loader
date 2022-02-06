#[cfg(feature = "render")]
#[test]
fn ui_with_render_feature() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_with_render_feature/*.rs");
}

#[cfg(not(feature = "render"))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
