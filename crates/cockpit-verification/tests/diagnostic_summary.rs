use cockpit_verification::{VerificationExecutionRecord, summarize_diagnostics};

fn record(
    node_id: &str,
    stderr: &[u8],
    passed: bool,
    exit_code: Option<i32>,
) -> VerificationExecutionRecord {
    VerificationExecutionRecord {
        node_id: node_id.into(),
        command_digest: format!("sha256:{}", "a".repeat(64)),
        timeout_seconds: 60,
        deadline_ms: 0,
        spawned: true,
        passed,
        exit_code,
        termination_signal: None,
        stdout_hex: String::new(),
        stderr_hex: stderr.iter().map(|byte| format!("{byte:02x}")).collect(),
        stdout_truncated: false,
        stderr_truncated: false,
        timed_out: false,
        elapsed_ms: 10,
    }
}

#[test]
fn repeated_clippy_roots_are_summarized_once_with_node_and_raw_evidence_retained() {
    let first_stderr = b"warning[clippy::needless_borrow]: this borrow is unnecessary\n  --> crates/core/src/lib.rs:10:5\n";
    let second_stderr = b"\x1b[33mwarning[clippy::needless_borrow]: this borrow is unnecessary\x1b[0m\n  --> crates/cli/src/main.rs:20:5\n";
    let distinct_stderr = b"error[clippy::unwrap_used]: used `unwrap()` on a `Result` value\n";
    let records = vec![
        record("cockpit-core", first_stderr, false, Some(101)),
        record("cockpit-cli", second_stderr, false, Some(101)),
        record("cockpit-mcp", distinct_stderr, false, Some(101)),
    ];
    let original_stderr = records
        .iter()
        .map(|record| record.stderr_hex.clone())
        .collect::<Vec<_>>();

    let summary = summarize_diagnostics(&records);

    assert_eq!(summary.len(), 2);
    let repeated = summary
        .iter()
        .find(|item| item.code.as_deref() == Some("clippy::needless_borrow"))
        .expect("repeated Clippy root");
    assert_eq!(repeated.severity, "warning");
    assert_eq!(repeated.message, "this borrow is unnecessary");
    assert_eq!(repeated.occurrences, 2);
    assert_eq!(repeated.node_ids, ["cockpit-cli", "cockpit-core"]);

    let distinct = summary
        .iter()
        .find(|item| item.code.as_deref() == Some("clippy::unwrap_used"))
        .expect("distinct Clippy root");
    assert_eq!(distinct.severity, "error");
    assert_eq!(distinct.occurrences, 1);
    assert_eq!(distinct.node_ids, ["cockpit-mcp"]);

    assert_eq!(
        records
            .iter()
            .map(|record| record.stderr_hex.clone())
            .collect::<Vec<_>>(),
        original_stderr,
        "summary generation must leave every node's raw stderr bytes untouched"
    );
    assert!(!records[0].passed);
    assert_eq!(records[0].exit_code, Some(101));
}

#[test]
fn diagnostic_summary_decodes_non_utf8_without_replacing_raw_bytes() {
    let stderr = b"warning: invalid byte \xff in diagnostic text\n";
    let record = record("cockpit-core", stderr, false, Some(1));

    let summary = summarize_diagnostics(std::slice::from_ref(&record));

    assert_eq!(summary.len(), 1);
    assert!(summary[0].message.contains('\u{fffd}'));
    assert_eq!(
        record.stderr_hex,
        stderr
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}

#[test]
fn rustc_style_lint_note_is_attached_to_its_human_readable_root() {
    let stderr = b"warning: this borrow is unnecessary\n  --> crates/core/src/lib.rs:10:5\n   = note: `#[warn(clippy::needless_borrow)]` on by default\n";
    let record = record("cockpit-core", stderr, false, Some(101));

    let summary = summarize_diagnostics(std::slice::from_ref(&record));

    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].code.as_deref(), Some("clippy::needless_borrow"));
    assert_eq!(summary[0].message, "this borrow is unnecessary");
}
