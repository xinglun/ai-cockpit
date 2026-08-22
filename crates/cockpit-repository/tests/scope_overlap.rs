use cockpit_repository::{
    WorkItemStartOptions, attach, set_work_item_intelligence, start_work_item_with_options,
    work_item_compatibility,
};
use std::process::Command;

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    directory
}

fn start_item(root: &std::path::Path, id: &str, scope: &str) {
    start_work_item_with_options(
        root,
        id,
        id,
        "scope compatibility",
        &[scope.into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
}

fn declare_parallel(root: &std::path::Path, id: &str) {
    set_work_item_intelligence(root, id, Vec::new(), Vec::new(), true)
        .expect("declare parallel Work Item");
}

#[test]
fn nested_glob_and_exact_path_overlap_is_not_parallel_compatible() {
    let directory = repository();
    start_item(directory.path(), "WI-PARENT", "src/**");
    start_item(directory.path(), "WI-CHILD", "src/main.rs");
    declare_parallel(directory.path(), "WI-PARENT");

    let compatibility =
        work_item_compatibility(directory.path(), "WI-PARENT").expect("compatibility");
    assert!(!compatibility.compatible);
    assert_eq!(compatibility.conflicts, vec!["WI-CHILD"]);
    assert!(
        compatibility
            .reasons
            .contains(&"scope_overlap:WI-CHILD".into())
    );
}

#[test]
fn nested_glob_prefixes_are_not_parallel_compatible() {
    let directory = repository();
    start_item(directory.path(), "WI-SOURCE", "src/**");
    start_item(directory.path(), "WI-TEST", "src/test/**");
    declare_parallel(directory.path(), "WI-SOURCE");

    let compatibility =
        work_item_compatibility(directory.path(), "WI-SOURCE").expect("compatibility");
    assert!(!compatibility.compatible);
    assert_eq!(compatibility.conflicts, vec!["WI-TEST"]);
}

#[test]
fn windows_separator_scope_overlap_is_normalized() {
    let directory = repository();
    start_item(directory.path(), "WI-WINDOWS", r"src\**");
    start_item(directory.path(), "WI-POSIX", "src/main.rs");
    declare_parallel(directory.path(), "WI-WINDOWS");

    let compatibility =
        work_item_compatibility(directory.path(), "WI-WINDOWS").expect("compatibility");
    assert!(!compatibility.compatible);
    assert!(
        compatibility
            .reasons
            .contains(&"scope_overlap:WI-POSIX".into())
    );
}

#[test]
fn disjoint_prefixes_remain_parallel_compatible() {
    let directory = repository();
    start_item(directory.path(), "WI-SOURCE", "src/**");
    start_item(directory.path(), "WI-DOCS", "docs/**");
    declare_parallel(directory.path(), "WI-SOURCE");

    let compatibility =
        work_item_compatibility(directory.path(), "WI-SOURCE").expect("compatibility");
    assert!(compatibility.compatible);
    assert!(compatibility.conflicts.is_empty());
    assert!(compatibility.reasons.is_empty());
}

#[test]
fn unsupported_glob_shape_fails_closed_as_unknown() {
    let directory = repository();
    start_item(directory.path(), "WI-GLOB", "src/*/generated/**");
    start_item(directory.path(), "WI-EXACT", "src/main/generated/output.rs");
    declare_parallel(directory.path(), "WI-GLOB");

    let compatibility =
        work_item_compatibility(directory.path(), "WI-GLOB").expect("compatibility");
    assert!(!compatibility.compatible);
    assert!(compatibility.conflicts.is_empty());
    assert!(
        compatibility
            .reasons
            .contains(&"scope_overlap_unknown:WI-EXACT".into())
    );
}

#[test]
fn absolute_or_parent_scope_fails_closed_as_unknown() {
    for (id, scope) in [("WI-ABSOLUTE", "/src/**"), ("WI-PARENT", "../src/**")] {
        let directory = repository();
        start_item(directory.path(), id, scope);
        start_item(directory.path(), "WI-EXACT", "src/main.rs");
        declare_parallel(directory.path(), id);

        let compatibility = work_item_compatibility(directory.path(), id).expect("compatibility");
        assert!(!compatibility.compatible, "scope {scope} must fail closed");
        assert!(compatibility.conflicts.is_empty());
        assert!(
            compatibility
                .reasons
                .contains(&"scope_overlap_unknown:WI-EXACT".into())
        );
    }
}
