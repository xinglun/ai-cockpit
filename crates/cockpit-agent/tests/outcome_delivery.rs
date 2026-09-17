use cockpit_agent::{
    AgentError, HostDeliveryCapabilities, OutcomeDeliveryProgress, OutcomeMessageHost,
    OutcomeMessageReceipt, deliver_outcome, ensure_same_outcome_delivery,
};
use cockpit_core::Digest;
use cockpit_protocol::{OutcomeDelivery, OutcomeDeliverySegment};

fn delivery() -> OutcomeDelivery {
    delivery_named("WI-DELIVERY")
}

fn delivery_named(work_item_id: &str) -> OutcomeDelivery {
    let first = format!("Outcome: 🟢 {work_item_id}\nWhat was completed\n");
    let second = "Evidence\n";
    let body = format!("{first}{second}");
    let archive = Digest::sha256_bytes(format!("archive:{work_item_id}").as_bytes());
    let id = Digest::sha256_bytes(format!("delivery:{work_item_id}").as_bytes());
    let segment = OutcomeDeliverySegment {
        schema_version: 1,
        delivery_id: id.clone(),
        work_item_id: work_item_id.into(),
        archive_identity: archive.clone(),
        language: "en".into(),
        part: 1,
        total_parts: 2,
        body_digest: Digest::sha256_bytes(first.as_bytes()),
        body: first,
    };
    let second_segment = OutcomeDeliverySegment {
        schema_version: 1,
        delivery_id: id.clone(),
        work_item_id: work_item_id.into(),
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
        work_item_id: work_item_id.into(),
        archive_identity: archive,
        language: "en".into(),
        view: "full".into(),
        body_digest: Digest::sha256_bytes(body.as_bytes()),
        body_summary: format!("Outcome: 🟢 {work_item_id}"),
        body,
        segments: vec![segment, second_segment],
        delivery_state: "prepared".into(),
        host_confirmation: "unknown".into(),
        next_action: "forward".into(),
        outcome: None,
        legacy_outcome: None,
        error: None,
    }
}

struct Host {
    capabilities: HostDeliveryCapabilities,
    messages: Vec<String>,
    fail_after: Option<usize>,
    accepted: Option<bool>,
    displayed: Option<bool>,
}

impl OutcomeMessageHost for Host {
    fn capabilities(&self) -> HostDeliveryCapabilities {
        self.capabilities
    }

    fn send_message(
        &mut self,
        message: &OutcomeDeliverySegment,
    ) -> Result<OutcomeMessageReceipt, AgentError> {
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
        Ok(OutcomeMessageReceipt::for_segment(
            message,
            self.accepted,
            self.displayed,
        ))
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
        accepted: Some(true),
        displayed: None,
    };
    let report = deliver_outcome(&delivery(), &mut host, None).expect("delivery");
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
        accepted: None,
        displayed: None,
    };
    let error = deliver_outcome(&payload, &mut host, None).expect_err("interruption");
    let progress = match error {
        AgentError::DeliveryFailed {
            sent_parts,
            progress,
            ..
        } => {
            assert_eq!(sent_parts, 1);
            *progress
        }
        other => panic!("unexpected error: {other:?}"),
    };
    // The same prepared payload is resumed; no Runtime verification/archive
    // call exists in this adapter boundary, and the host sees the exact body.
    host.fail_after = None;
    let report = deliver_outcome(&payload, &mut host, Some(&progress)).expect("retry");
    assert!(report.complete);
    assert_eq!(report.delivery_state, "unknown");
    assert!(report.duplicate_risk);
    assert_eq!(host.messages.len(), 3);
}

#[test]
fn display_capability_does_not_promote_missing_per_message_confirmation() {
    let mut host = Host {
        capabilities: HostDeliveryCapabilities {
            acceptance_confirmation: true,
            display_confirmation: true,
            idempotency: true,
        },
        messages: Vec::new(),
        fail_after: None,
        accepted: Some(true),
        displayed: None,
    };
    let report = deliver_outcome(&delivery(), &mut host, None).expect("delivery");
    assert_eq!(report.delivery_state, "host_accepted");
    assert_eq!(report.host_confirmation, "accepted");
}

#[test]
fn non_contiguous_progress_cannot_skip_to_the_end_without_receipts() {
    let payload = delivery();
    let mut progress = OutcomeDeliveryProgress::new(&payload);
    progress.confirmed.push(OutcomeMessageReceipt::for_segment(
        &payload.segments[1],
        Some(true),
        Some(true),
    ));
    let mut host = Host {
        capabilities: HostDeliveryCapabilities::default(),
        messages: Vec::new(),
        fail_after: None,
        accepted: Some(true),
        displayed: Some(true),
    };
    let error = deliver_outcome(&payload, &mut host, Some(&progress)).expect_err("stale progress");
    assert!(
        error
            .to_string()
            .contains("contiguous accepted receipt chain")
    );
    assert!(host.messages.is_empty());
}

#[test]
fn consecutive_work_items_keep_delivery_identity_separate() {
    let first = delivery_named("WI-FIRST");
    let second = delivery_named("WI-SECOND");
    assert_ne!(first.delivery_id, second.delivery_id);
    assert_ne!(first.work_item_id, second.work_item_id);
    let mut host = Host {
        capabilities: HostDeliveryCapabilities::default(),
        messages: Vec::new(),
        fail_after: None,
        accepted: Some(true),
        displayed: Some(true),
    };
    let first_report = deliver_outcome(&first, &mut host, None).expect("first delivery");
    let second_report = deliver_outcome(&second, &mut host, None).expect("second delivery");
    assert_eq!(first_report.work_item_id, "WI-FIRST");
    assert_eq!(second_report.work_item_id, "WI-SECOND");
    assert_eq!(host.messages.len(), 4);
}

#[test]
fn stale_delivery_identity_is_rejected_before_host_message() {
    let expected = delivery();
    let mut candidate = delivery();
    candidate.archive_identity = Digest::sha256_bytes(b"new-archive");
    let error = ensure_same_outcome_delivery(&expected, &candidate).expect_err("stale");
    assert!(error.to_string().contains("stale or mismatched"));
}

#[test]
fn empty_progress_cannot_claim_completion_without_attempting_the_first_segment() {
    let payload = delivery();
    let mut host = Host {
        capabilities: HostDeliveryCapabilities::default(),
        messages: Vec::new(),
        fail_after: Some(0),
        accepted: Some(true),
        displayed: Some(true),
    };
    let error = deliver_outcome(
        &payload,
        &mut host,
        Some(&OutcomeDeliveryProgress::new(&payload)),
    )
    .expect_err("an empty progress chain must still attempt part one");
    match error {
        AgentError::DeliveryFailed {
            sent_parts,
            progress,
            ..
        } => {
            assert_eq!(sent_parts, 0);
            assert!(progress.confirmed.is_empty());
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert!(host.messages.is_empty());
}

#[test]
fn return_only_adapter_reports_segments_without_host_confirmation() {
    let payload = delivery();
    let mut host = cockpit_agent::ReturnOnlyOutcomeHost::default();
    let report = deliver_outcome(&payload, &mut host, None).expect("return-only handoff");
    assert!(report.complete);
    assert_eq!(report.delivery_state, "unknown");
    assert_eq!(report.host_confirmation, "unknown");
    assert_eq!(host.returned_segment_count(), payload.segments.len());
}
