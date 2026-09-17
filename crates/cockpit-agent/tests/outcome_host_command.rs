use std::fs;
use std::path::Path;
use std::process::Command;

use cockpit_agent::{CommandOutcomeMessageHost, deliver_outcome};
use cockpit_core::Digest;
use cockpit_protocol::{OutcomeDelivery, OutcomeDeliverySegment};

fn delivery(work_item_id: &str) -> OutcomeDelivery {
    let first = format!("Outcome: 🟢 {work_item_id}\nWhat was completed\n");
    let second = "Evidence and next action\n".to_owned();
    let body = format!("{first}{second}");
    let archive = Digest::sha256_bytes(format!("archive:{work_item_id}").as_bytes());
    let delivery_id = Digest::sha256_bytes(format!("delivery:{work_item_id}").as_bytes());
    let segments = [first, second]
        .into_iter()
        .enumerate()
        .map(|(index, body)| OutcomeDeliverySegment {
            schema_version: 1,
            delivery_id: delivery_id.clone(),
            work_item_id: work_item_id.into(),
            archive_identity: archive.clone(),
            language: "en".into(),
            part: index as u32 + 1,
            total_parts: 2,
            body_digest: Digest::sha256_bytes(body.as_bytes()),
            body,
        })
        .collect();
    OutcomeDelivery {
        schema_version: 1,
        delivery_id,
        work_item_id: work_item_id.into(),
        archive_identity: archive,
        language: "en".into(),
        view: "full".into(),
        body: body.clone(),
        body_digest: Digest::sha256_bytes(body.as_bytes()),
        body_summary: format!("Outcome: 🟢 {work_item_id}"),
        segments,
        delivery_state: "prepared".into(),
        host_confirmation: "unknown".into(),
        next_action: "forward".into(),
        outcome: None,
        legacy_outcome: None,
        error: None,
    }
}

fn compile_host_fixture(directory: &Path) -> std::path::PathBuf {
    let source = directory.join("host.rs");
    let binary = directory.join(if cfg!(windows) { "host.exe" } else { "host" });
    fs::write(
        &source,
        r#"
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut file = OpenOptions::new().create(true).append(true).open(&args[1]).unwrap();
    file.write_all(input.as_bytes()).unwrap();
    file.write_all(b"\n").unwrap();
    println!("{{\"deliveryId\":\"{}\",\"part\":{},\"segmentDigest\":\"{}\",\"accepted\":true,\"displayed\":true}}", args[2], args[3], args[4]);
}
"#,
    )
    .expect("write host fixture");
    assert!(
        Command::new("rustc")
            .args(["--edition", "2021"])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .status()
            .expect("compile host fixture")
            .success()
    );
    binary
}

#[test]
fn command_host_emits_and_confirms_real_assistant_message_events() {
    let directory = tempfile::tempdir().expect("tempdir");
    let event_log = directory.path().join("assistant-events.jsonl");
    let binary = compile_host_fixture(directory.path());
    let first = delivery("WI-COMMAND-FIRST");
    let second = delivery("WI-COMMAND-SECOND");
    let args = vec![
        event_log.clone().into_os_string(),
        "{deliveryId}".into(),
        "{part}".into(),
        "{segmentDigest}".into(),
    ];
    let mut host = CommandOutcomeMessageHost::new(binary).with_args(args);
    let first_report = deliver_outcome(&first, &mut host, None).expect("first delivery");
    let second_report = deliver_outcome(&second, &mut host, None).expect("second delivery");
    assert_eq!(first_report.delivery_state, "display_confirmed");
    assert_eq!(second_report.delivery_state, "display_confirmed");

    let events = fs::read_to_string(event_log).expect("assistant event log");
    assert_eq!(events.lines().count(), 4);
    assert!(events.contains("WI-COMMAND-FIRST"));
    assert!(events.contains("WI-COMMAND-SECOND"));
    assert!(events.contains("assistant_message"));
    assert!(events.contains("What was completed"));
    assert!(events.contains("Evidence and next action"));
}
