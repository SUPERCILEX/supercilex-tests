#[test]
fn fmt() {
    supercilex_tests::fmt();
}

#[test]
fn clippy() {
    supercilex_tests::clippy();
}

#[test]
#[cfg(feature = "api")]
fn api() {
    supercilex_tests::api();
}
