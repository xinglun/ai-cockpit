use cockpit_git::normalize_changed_paths;

#[test]
fn changed_paths_are_normalized_and_sorted_once() {
    let paths = normalize_changed_paths(["./src/../src/lib.rs", "tests/b.rs", "tests/a.rs"]);
    assert_eq!(paths, vec!["src/lib.rs", "tests/a.rs", "tests/b.rs"]);
}
