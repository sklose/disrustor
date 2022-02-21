#[test]
fn compile_time_safety_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cve/cve_2020_36470_1.rs");
    t.compile_fail("tests/cve/cve_2020_36470_2.rs");
}
