#[cfg(all(feature = "2d", feature = "3d"))]
#[test]
fn ui_with_2d_3d_features() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_with_2d_3d_features/*.rs");
}

#[cfg(all(not(feature = "2d"), not(feature = "3d")))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
