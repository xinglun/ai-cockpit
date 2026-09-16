use std::process::Command;

#[test]
fn retirement_command_is_exposed_in_work_item_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "--help"])
        .output()
        .expect("run ai-cockpit help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("retire"),
        "help did not expose retire: {stdout}"
    );
    assert!(stdout.contains("integrated") || stdout.contains("replaced"));
}
