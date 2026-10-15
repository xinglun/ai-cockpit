use std::process::Command;

#[test]
fn explicit_source_plan_and_record_are_discoverable() {
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(
        help.contains("legacy-closeout-source-plan"),
        "explicit read-only source plan is missing: {help}"
    );
    assert!(
        help.contains("legacy-closeout-source-record"),
        "explicit source record is missing: {help}"
    );
}
