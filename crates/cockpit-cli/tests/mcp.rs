use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn mcp_command_serves_json_rpc_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("mcp");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(
            br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
"#,
        )
        .expect("request");
    let output = child.wait_with_output().expect("response");
    assert!(output.status.success());
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(response["result"]["serverInfo"]["name"], "ai-cockpit");
}
