use cockpit_agent::{
    AgentError, HostDeliveryCapabilities, OutcomeMessageHost, deliver_outcome,
    ensure_same_outcome_delivery,
};
use cockpit_core::Digest;
use cockpit_protocol::{OutcomeDelivery, OutcomeDeliverySegment};

fn delivery() -> OutcomeDelivery {
    let first = "Outcome: 🟢 WI-DELIVERY\nWhat was completed\n";
    let second = "Evidence\n";
    let body = format!("{first}{second}");
    let archive = Digest::sha256_bytes(b"archive");
    let id = Digest::sha256_bytes(b"delivery");
    let segment = OutcomeDeliverySegment {
        schema_version: 1,
        delivery_id: id.clone(),
        work_item_id: "WI-DELIVERY".into(),
        archive_identity: archive.clone(),
        language: "en".into(),
        part: 1,
        total_parts: 2,
        body_digest: Digest::sha256_bytes(first.as_bytes()),
        body: first.into(),
    };
    let second_segment = OutcomeDeliverySegment {
        schema_version: 1,
        delivery_id: id.clone(),
        work_item_id: "WI-DELIVERY".into(),
        archive_identity: archive.clone(),
        language: "en".into(),
        part: 2,
        total_parts: 2,
        body_digest: Digest::sha256_bytes(second.as_bytes()),
        body: second.into(),
    };
    OutcomeDelivery {
        schema_version: 1,
        delivery_id: id,
        work_item_id: "WI-DELIVERY".into(),
        archive_identity: archive,
        language: "en".into(),
        view: "full".into(),
        body_digest: Digest::sha256_bytes(body.as_bytes()),
        body_summary: "Outcome: 🟢 WI-DELIVERY".into(),
        body,
        segments: vec![segment, second_segment],
        delivery_state: "prepared".into(),
        host_confirmation: "unknown".into(),
        next_action: "forward".into(),
        outcome: None,
        error: None,
    }
}

struct Host {
    capabilities: HostDeliveryCapabilities,
    messages: Vec<String>,
    fail_after: Option<usize>,
}

impl OutcomeMessageHost for Host {
    fn capabilities(&self) -> HostDeliveryCapabilities {
        self.capabilities
    }

    fn send_message(&mut self, message: &OutcomeDeliverySegment) -> Result<(), AgentError> {
        if self
            .fail_after
            .is_some_and(|limit| self.messages.len() >= limit)
        {
            return Err(AgentError::State {
                path: "host".into(),
                message: "simulated interruption".into(),
            });
        }
        self.messages.push(message.body.clone());
        Ok(())
    }
}

#[test]
fn host_acceptance_is_not_display_confirmation_and_messages_are_real_events() {
    let mut host = Host {
        capabilities: HostDeliveryCapabilities {
            acceptance_confirmation: true,
            display_confirmation: false,
            idempotency: true,
        },
        messages: Vec::new(),
        fail_after: None,
    };
    let report = deliver_outcome(&delivery(), &mut host, 0).expect("delivery");
    assert!(report.complete);
    assert_eq!(report.delivery_state, "host_accepted");
    assert_eq!(report.host_confirmation, "accepted");
    assert_eq!(host.messages.len(), 2);
    assert!(host.messages[0].contains("What was completed"));
}

#[test]
fn interrupted_delivery_resumes_same_body_without_new_lifecycle_execution() {
    let payload = delivery();
    let mut host = Host {
        capabilities: HostDeliveryCapabilities::default(),
        messages: Vec::new(),
        fail_after: Some(1),
    };
    let error = deliver_outcome(&payload, &mut host, 0).expect_err("interruption");
    match error {
        AgentError::DeliveryFailed { sent_parts, .. } => assert_eq!(sent_parts, 1),
        other => panic!("unexpected error: {other:?}"),
    }
    // The same prepared payload is resumed; no Runtime verification/archive
    // call exists in this adapter boundary, and the host sees the exact body.
    host.fail_after = None;
    let report = deliver_outcome(&payload, &mut host, 1).expect("retry");
    assert!(report.complete);
    assert_eq!(report.delivery_state, "unknown");
    assert!(report.duplicate_risk);
    assert_eq!(host.messages.len(), 2);
}

#[test]
fn stale_delivery_identity_is_rejected_before_host_message() {
    let expected = delivery();
    let mut candidate = delivery();
    candidate.archive_identity = Digest::sha256_bytes(b"new-archive");
    let error = ensure_same_outcome_delivery(&expected, &candidate).expect_err("stale");
    assert!(error.to_string().contains("stale or mismatched"));
}
