//! Real CLI/MCP observers for isolated native fixtures. No authority crosses the process boundary.
#![cfg(test)]
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(super) fn binary() -> PathBuf {
    let path = std::env::var_os("AI_COCKPIT_INTERFACE_DESCRIPTION_BIN")
        .map(PathBuf::from)
        .expect("runner supplies the same-input CLI binary");
    assert!(
        path.is_absolute() && path.is_file(),
        "real CLI binary is required: {}",
        path.display()
    );
    path
}
pub(super) fn runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: cockpit_core::Digest::sha256_bytes(&std::fs::read(binary()).unwrap()),
    }
}
pub(super) fn mcp_request(root: &Path, request: &serde_json::Value) -> serde_json::Value {
    let mut child = Command::new(binary())
        .args(["mcp", "--repo"])
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real MCP stdio process");
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{}", serde_json::to_string(request).unwrap()).unwrap();
    drop(stdin);
    let output = child.wait_with_output().expect("real MCP process terminal");
    assert!(
        output.status.success(),
        "MCP transport failed before target validation: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<_> = text.lines().filter(|s| !s.trim().is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "one request must have one actual response: {text}"
    );
    let response: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(response["id"], request["id"]);
    response
}
