//! Untrusted native/portable evidence format and read-only inspection coverage.
//! Self-owned bytes and captured samples never establish source authority.

use serde_json::{Value, json};

use super::archive::{self, EvidenceArchiveError as ArchiveError};
use cockpit_core::Digest as ArchiveDigest;
use cockpit_protocol::ci_admission::{NativeExecutionEnvelope, UntrustedEvidenceArchiveManifest};
use cockpit_verification::VerificationReceipt as ArchiveNativeReceipt;

fn archive2a_native() -> Vec<u8> {
    // Exact captured native schema2 sample. The parser treats every identity
    // and success claim as UNTRUSTED; this is not a fresh test execution.
    r###"{"captureMode":"full_capture","contractDigest":"sha256:40e7ab0b9c454664c3a004ccb362c969eb5897beabfb25b6c8db78bd1c60b6ea","createdAt":"2026-10-31T05:42:18Z","evidenceSchemaVersion":2,"passed":true,"protocolVersion":1,"receipt":{"costObservation":{"advisoryOnly":true,"confidence":"complete","elapsedMs":44080,"executionElapsedMs":43583,"maxConcurrentProcesses":1,"nodesExecuted":1,"nodesPlanned":1,"nodesReused":0,"passed":true,"planningElapsedMs":0,"processesSpawned":1,"protectedNodesExecuted":0,"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","schemaVersion":1,"unknowns":[]},"elapsedMs":44080,"executionElapsedMs":43583,"executionRecords":[{"commandDigest":"sha256:dd3968def0dd92b8c1cabb1d5198ce956a5824b3ee296eb33f492523bd40965f","deadlineMs":1793426194838,"elapsedMs":43578,"exitCode":0,"nodeId":"project-command-0","passed":true,"spawned":true,"stderr":"7461736b2d617574686f72697a656420576f726b204974656d20766572696669636174696f6e20726571756972657320616e206578706c69636974202d2d7461736b2d63616c6c65722066696c650a63616e6469646174652052756e74696d6520707265666c69676874206661696c65642077697468206578697420310a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a63616e6469646174652052756e74696d65207265706f72747320667265736820766572696669636174696f6e2c206275742069747320666f726d616c2072656365697074206973206e6f74207265757361626c6520286578697374696e675f66726573685f726563656970745f6964656e746974795f6d69736d61746368290a776f726b737061636520726563656970742073796e632072656a65637465643a20666f726d616c20766572696669636174696f6e2072656365697074207265706f7369746f7279206964656e7469747920646966666572732066726f6d2052756e74696d65207374617475730a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a576f726b204974656d20494420697320756e7361666520666f72207468652063616e6f6e6963616c2065766964656e636520706174680a776f726b737061636520726563656970742073796e632072656a65637465643a2063616e6f6e6963616c20776f726b7370616365207265636569707420746172676574206973206e6f74206120726567756c6172206e6f6e2d73796d6c696e6b2066696c650a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a63616e6469646174652052756e74696d652061646d697473206e6569746865722072756e5f707265666c69676874206e6f722072756e5f766572696669636174696f6e0a63616e6469646174652052756e74696d65206e6f206c6f6e6765722061646d6974732072756e5f766572696669636174696f6e2061667465722073746174757320726566726573680a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652061646d697473206e6569746865722072756e5f707265666c69676874206e6f722072756e5f766572696669636174696f6e0a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a20457870656374696e672070726f7065727479206e616d6520656e636c6f73656420696e20646f75626c652071756f7465733a206c696e65203120636f6c756d6e20322028636861722031290a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a206475706c6963617465207374646f75742f65766964656e6365204a534f4e206669656c640a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a2052756e74696d65207374646f757420646f6573206e6f742070726f6a65637420746f2074686520666f726d616c20726563656970740a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a2052756e74696d65207374646f757420646f6573206e6f742070726f6a65637420746f2074686520666f726d616c20726563656970740a","stderrTruncated":false,"stdout":"776f726b7370616365207061636b61676520636f766572616765207061737365643a2031207061636b616765730a74696d656f75742072656772657373696f6e207061737365643a20626f756e6465643d312c2064656661756c743d312c20756e72656c617465643d312c20696e76616c69643d352c2052756e74696d652d7265667573616c3d310a70726f64756365722f636f6e73756d657220436172676f20636f6e746578742070726f6f66207061737365643a2031322063617365730a686f737465642052756e74696d6520766572696669636174696f6e206f726368657374726174696f6e2072656772657373696f6e207061737365640a","stdoutTruncated":false,"timedOut":false,"timeoutSeconds":900}],"filesHashed":5,"filesRead":5,"gitCalls":6,"maxConcurrentProcesses":1,"nodesExecuted":1,"nodesPlanned":1,"nodesReused":0,"passed":true,"planReceipt":{"affectedPaths":[],"assurance":"repository_verified","baseRevision":"efc9f309f1500e433240a76fe5dc76db64abb53b","dependencyConfidence":"unknown","escalations":[],"executedNodes":["project-command-0"],"executionElapsedMs":43583,"finalTier":"T0","initialTier":"T0","planningElapsedMs":0,"policyRefs":["wi-1114-bounded-release-verification-v1"],"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","repositorySnapshotDigest":"sha256:3ff877b2050551986531641a9573e9ecf332ea3da524b710f6e9a3ee38cb1150","requiredAssurance":"self_declared","requiredTier":"T0","reusedNodes":[],"savedExecutions":0,"schemaVersion":1,"selectionReasons":["effective:ai-cockpit-project-v1:wi-1114-bounded-release-verification-v1: Enforce the approved maximum of 900 seconds per actual verification command with single worker, finite supervised cleanup and at least 1 GiB free disk. Stage applicability does not authorize publication or production use; exact version, tag and assets require separate future owner approval. Keep all platform, source, evidence and approval requirements; the whole multi-command run has a separately declared finite aggregate budget."],"sharedNodes":[],"stage":"task","timeoutSeconds":900,"workItemId":"WI-1114-release-runtime-convergence"},"planningElapsedMs":0,"processSpawnFailures":0,"processesSpawned":1,"protectedNodesExecuted":0,"protectedNodesSkipped":0,"receiptCandidates":[],"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","rerunStale":0,"rerunUnknown":0,"results":[{"action":"execute","nodeId":"project-command-0","outputDigest":"sha256:3ed74a2373b13cd963aa5dc6b029c636f686b556ba635ac4b49d6f7f3d48634e","outputTruncated":false,"passed":true,"protected":false,"reason":"reuse_not_configured","receiptId":null,"reused":false,"satisfiedBy":"execution","state":"not_applicable","timedOut":false}],"runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","timeoutSeconds":900,"workItemId":"WI-1114-release-runtime-convergence"},"receiptDigest":"sha256:c82b48c8f1c733a39619b96d2ab77af6379cb329a03c1ba070546742cb3a6ff2","repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","repositorySnapshotDigest":"sha256:3ff877b2050551986531641a9573e9ecf332ea3da524b710f6e9a3ee38cb1150","runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","workItemId":"WI-1114-release-runtime-convergence"}"###.as_bytes().to_vec()
}

fn archive2a_crc(raw: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in raw {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320_u32 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn archive2a_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn archive2a_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

// Small classic, single-disk, stored ZIP writer for self-owned byte fixtures.
// ZIP fixtures use the same bounded format without a product ZIP writer dependency.
fn archive2a_zip(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, raw) in entries {
        let offset = out.len() as u32;
        let crc = archive2a_crc(raw);
        archive2a_u32(&mut out, 0x04034b50);
        for value in [20, 0, 0, 0, 0] {
            archive2a_u16(&mut out, value);
        }
        for value in [crc, raw.len() as u32, raw.len() as u32] {
            archive2a_u32(&mut out, value);
        }
        archive2a_u16(&mut out, name.len() as u16);
        archive2a_u16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(raw);
        archive2a_u32(&mut directory, 0x02014b50);
        for value in [0x0314, 20, 0, 0, 0, 0] {
            archive2a_u16(&mut directory, value);
        }
        for value in [crc, raw.len() as u32, raw.len() as u32] {
            archive2a_u32(&mut directory, value);
        }
        for value in [name.len() as u16, 0, 0, 0, 0] {
            archive2a_u16(&mut directory, value);
        }
        archive2a_u32(&mut directory, 0o100600 << 16);
        archive2a_u32(&mut directory, offset);
        directory.extend_from_slice(name.as_bytes());
    }
    let directory_start = out.len() as u32;
    out.extend_from_slice(&directory);
    archive2a_u32(&mut out, 0x06054b50);
    for value in [0, 0, entries.len() as u16, entries.len() as u16] {
        archive2a_u16(&mut out, value);
    }
    archive2a_u32(&mut out, directory.len() as u32);
    archive2a_u32(&mut out, directory_start);
    archive2a_u16(&mut out, 0);
    out
}

fn archive2a_bundle(native: Vec<u8>) -> Vec<u8> {
    let manifest = json!({"schemaVersion":1,"entries":[{
        "path":"receipts/verification.json","role":"native_verification_evidence",
        "bytes":native.len(),"digest":ArchiveDigest::sha256_bytes(&native)
    }]});
    archive2a_zip(&[
        ("manifest.json", serde_json::to_vec(&manifest).unwrap()),
        ("receipts/verification.json", native),
    ])
}

fn archive2a_reject(raw: &[u8], expected: ArchiveError) {
    let actual =
        archive::inspect_untrusted_evidence_archive(raw, &ArchiveDigest::sha256_bytes(raw));
    assert_eq!(actual.unwrap_err(), expected);
}

fn archive2a_central(raw: &[u8]) -> usize {
    let eocd = raw.len() - 22;
    u32::from_le_bytes(raw[eocd + 16..eocd + 20].try_into().unwrap()) as usize
}

// Self-owned synthetic archive bytes only. These helpers never execute or
// authenticate the declarations they encode.
fn archive2a_deflate(raw: &[u8]) -> Vec<u8> {
    use std::io::Write as _;
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}

fn archive2a_zip_encoded(
    entries: &[(&str, Vec<u8>)],
    deflate: bool,
    descriptor: Option<bool>,
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, raw) in entries {
        let offset = out.len() as u32;
        let crc = archive2a_crc(raw);
        let data = if deflate {
            archive2a_deflate(raw)
        } else {
            raw.clone()
        };
        let flags = if descriptor.is_some() { 8 } else { 0 };
        let method = if deflate { 8 } else { 0 };
        archive2a_u32(&mut out, 0x04034b50);
        for v in [20, flags, method, 0, 0] {
            archive2a_u16(&mut out, v);
        }
        for v in if descriptor.is_some() {
            [0, 0, 0]
        } else {
            [crc, data.len() as u32, raw.len() as u32]
        } {
            archive2a_u32(&mut out, v);
        }
        archive2a_u16(&mut out, name.len() as u16);
        archive2a_u16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&data);
        if let Some(signed) = descriptor {
            if signed {
                archive2a_u32(&mut out, 0x08074b50);
            }
            for v in [crc, data.len() as u32, raw.len() as u32] {
                archive2a_u32(&mut out, v);
            }
        }
        archive2a_u32(&mut directory, 0x02014b50);
        for v in [0x0314, 20, flags, method, 0, 0] {
            archive2a_u16(&mut directory, v);
        }
        for v in [crc, data.len() as u32, raw.len() as u32] {
            archive2a_u32(&mut directory, v);
        }
        for v in [name.len() as u16, 0, 0, 0, 0] {
            archive2a_u16(&mut directory, v);
        }
        archive2a_u32(&mut directory, 0o100600 << 16);
        archive2a_u32(&mut directory, offset);
        directory.extend_from_slice(name.as_bytes());
    }
    let start = out.len() as u32;
    out.extend_from_slice(&directory);
    archive2a_u32(&mut out, 0x06054b50);
    for v in [0, 0, entries.len() as u16, entries.len() as u16] {
        archive2a_u16(&mut out, v);
    }
    archive2a_u32(&mut out, directory.len() as u32);
    archive2a_u32(&mut out, start);
    archive2a_u16(&mut out, 0);
    out
}

fn archive2a_inventory(natives: &[(&str, Vec<u8>)]) -> Vec<(String, Vec<u8>)> {
    let entries = natives
        .iter()
        .map(|(path, bytes)| {
            json!({
                "path":path,"role":"native_verification_evidence","bytes":bytes.len(),
                "digest":ArchiveDigest::sha256_bytes(bytes)
            })
        })
        .collect::<Vec<_>>();
    let mut values = vec![(
        "manifest.json".to_owned(),
        serde_json::to_vec(&json!({"schemaVersion":1,"entries":entries})).unwrap(),
    )];
    values.extend(
        natives
            .iter()
            .map(|(path, raw)| ((*path).to_owned(), raw.clone())),
    );
    values
}

fn archive2a_bundle_encoded(
    natives: &[(&str, Vec<u8>)],
    deflate: bool,
    descriptor: Option<bool>,
) -> Vec<u8> {
    let entries = archive2a_inventory(natives);
    let borrowed = entries
        .iter()
        .map(|(path, bytes)| (path.as_str(), bytes.clone()))
        .collect::<Vec<_>>();
    archive2a_zip_encoded(&borrowed, deflate, descriptor)
}

// Insert bytes in the first local member; relocate central directory and later
// local-header offsets without changing any declared payload sizes.
fn archive2a_insert_first(raw: &[u8], at: usize, bytes: &[u8]) -> Vec<u8> {
    let old_central = archive2a_central(raw);
    let mut changed = raw.to_vec();
    changed.splice(at..at, bytes.iter().copied());
    let central = old_central + bytes.len();
    let eocd = changed.len() - 22;
    changed[eocd + 16..eocd + 20].copy_from_slice(&(central as u32).to_le_bytes());
    let count = u16::from_le_bytes(changed[eocd + 10..eocd + 12].try_into().unwrap());
    let mut entry = central;
    for _ in 0..count {
        let offset = u32::from_le_bytes(changed[entry + 42..entry + 46].try_into().unwrap());
        if offset as usize >= at {
            changed[entry + 42..entry + 46]
                .copy_from_slice(&(offset + bytes.len() as u32).to_le_bytes());
        }
        let name = u16::from_le_bytes(changed[entry + 28..entry + 30].try_into().unwrap()) as usize;
        let extra =
            u16::from_le_bytes(changed[entry + 30..entry + 32].try_into().unwrap()) as usize;
        let comment =
            u16::from_le_bytes(changed[entry + 32..entry + 34].try_into().unwrap()) as usize;
        entry += 46 + name + extra + comment;
    }
    changed
}

fn archive2a_claims_native(count: usize, prefix: &str) -> Vec<u8> {
    // Synthetic internally consistent declarations for cardinality boundaries,
    // not new physical execution facts or authenticated producer evidence.
    let mut envelope: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    let mut result = envelope["receipt"]["results"][0].clone();
    let mut record = envelope["receipt"]["executionRecords"][0].clone();
    record["stdout"] = json!("");
    record["stderr"] = json!("");
    record["elapsedMs"] = json!(0);
    result["outputDigest"] = json!(ArchiveDigest::sha256_bytes(br#"{"success":true,"exitCode":0,"terminationSignal":null,"stdout":[],"stderr":[],"stdoutTruncated":false,"stderrTruncated":false,"timedOut":false}"#));
    let mut results = Vec::new();
    let mut records = Vec::new();
    let mut nodes = Vec::new();
    for index in 0..count {
        let node = format!("{prefix}-{index}");
        let mut next_result = result.clone();
        next_result["nodeId"] = json!(node);
        let mut next_record = record.clone();
        next_record["nodeId"] = json!(node);
        results.push(next_result);
        records.push(next_record);
        nodes.push(node);
    }
    envelope["receipt"]["results"] = json!(results);
    envelope["receipt"]["executionRecords"] = json!(records);
    for name in ["nodesPlanned", "nodesExecuted", "processesSpawned"] {
        envelope["receipt"][name] = json!(count);
    }
    envelope["receipt"]["planReceipt"]["executedNodes"] = json!(nodes);
    envelope["receipt"]
        .as_object_mut()
        .unwrap()
        .remove("costObservation");
    envelope["receiptDigest"] = json!(cockpit_protocol::digest_json(&envelope["receipt"]).unwrap());
    serde_json::to_vec(&envelope).unwrap()
}

#[test]
fn ci_admission_archive_raw_limit_boundaries_need_no_large_allocation() {
    assert!(archive::check_raw_archive_size(64 * 1024 * 1024).is_ok());
    assert_eq!(
        archive::check_raw_archive_size(64 * 1024 * 1024 + 1),
        Err(ArchiveError::RawSize)
    );

    let one = 64_u64 * 1024 * 1024;
    let compressed = one.div_ceil(100);
    assert!(archive::check_declared_budgets(&[(compressed, one), (compressed, one)]).is_ok());
    assert_eq!(
        archive::check_declared_budgets(&[(compressed, one), (compressed, one), (1, 1)]),
        Err(ArchiveError::ExpandedSize)
    );
    assert!(archive::check_declared_budgets(&[(1, 100), (0, 0)]).is_ok());
    assert_eq!(
        archive::check_declared_budgets(&[(1, 101)]),
        Err(ArchiveError::ExpansionRatio)
    );
    assert_eq!(
        archive::check_declared_budgets(&[(0, 1)]),
        Err(ArchiveError::ExpansionRatio)
    );
    assert_eq!(
        archive::check_declared_budgets(&[(u64::MAX, 0), (1, 0)]),
        Err(ArchiveError::ExpansionRatio)
    );
}

#[test]
fn ci_admission_archive_wrong_digest_never_enters_decoder() {
    let entered = std::cell::Cell::new(false);
    let actual = archive::inspect_with_decoder(
        b"not a ZIP",
        &ArchiveDigest::sha256_bytes(b"different"),
        |_, _| {
            entered.set(true);
            Err(ArchiveError::NativeReceipt)
        },
    );
    assert_eq!(actual.unwrap_err(), ArchiveError::ArchiveDigest);
    assert!(!entered.get());
}

#[test]
fn ci_admission_archive_manifest_dto_is_strict_and_cannot_claim_authentication() {
    let valid = json!({"schemaVersion":1,"entries":[]});
    assert!(serde_json::from_value::<UntrustedEvidenceArchiveManifest>(valid.clone()).is_ok());
    for key in ["trusted", "providerAuthenticated", "command", "endpoint"] {
        let mut changed = valid.clone();
        changed[key] = true.into();
        assert!(serde_json::from_value::<UntrustedEvidenceArchiveManifest>(changed).is_err());
    }
    let duplicate = br#"{"schemaVersion":1,"schemaVersion":1,"entries":[]}"#;
    assert!(serde_json::from_slice::<UntrustedEvidenceArchiveManifest>(duplicate).is_err());
}

#[test]
fn ci_admission_archive_native_dto_preserves_current_wire_without_authentication() {
    let native = archive2a_native();
    let parsed: NativeExecutionEnvelope<ArchiveNativeReceipt> =
        serde_json::from_slice(&native).unwrap();
    assert_eq!(parsed.evidence_schema_version, 2);
    assert!(parsed.receipt.passed);
    let mut changed: Value = serde_json::from_slice(&native).unwrap();
    changed["providerAuthenticated"] = true.into();
    assert!(
        serde_json::from_value::<NativeExecutionEnvelope<ArchiveNativeReceipt>>(changed).is_err()
    );
}

#[test]
fn ci_admission_archive_valid_stored_native_is_only_untrusted_structure() {
    let raw = archive2a_bundle(archive2a_native());
    let observed =
        archive::inspect_untrusted_evidence_archive(&raw, &ArchiveDigest::sha256_bytes(&raw))
            .unwrap();
    assert_eq!(observed.receipts.len(), 1);
    assert_eq!(observed.members.len(), 2);
    for gap in [
        "independent_execution_source_unavailable",
        "producer_runtime_unverified",
        "current_consumer_command_binding_unavailable",
        "native_coverage_manifest_missing",
    ] {
        assert!(
            observed.gaps.iter().any(|value| value == gap),
            "missing gap: {gap}"
        );
    }
    assert_eq!(
        observed
            .members
            .iter()
            .find(|member| member.path == "receipts/verification.json")
            .unwrap()
            .raw,
        archive2a_native()
    );

    for signed in [false, true] {
        let streamed = archive2a_bundle_encoded(
            &[("receipts/verification.json", archive2a_native())],
            false,
            Some(signed),
        );
        let parsed = archive::inspect_untrusted_evidence_archive(
            &streamed,
            &ArchiveDigest::sha256_bytes(&streamed),
        )
        .unwrap();
        assert_eq!(parsed.receipts.len(), 1);
        assert!(
            parsed
                .gaps
                .iter()
                .any(|gap| gap == "producer_runtime_unverified")
        );
        let central = archive2a_central(&streamed);
        let compressed =
            u32::from_le_bytes(streamed[central + 20..central + 24].try_into().unwrap()) as usize;
        let data = 30 + "manifest.json".len();
        let descriptor_crc = data + compressed + if signed { 4 } else { 0 };
        let mut bad = streamed;
        bad[descriptor_crc] ^= 1;
        archive2a_reject(&bad, ArchiveError::ZipStructure);
    }
}

#[test]
fn ci_admission_archive_valid_deflate_has_the_same_untrusted_semantics() {
    let raw = hex::decode("504b0304140000000800000021002f1252219d000000cb0000000d0000006d616e69666573742e6a736f6e4dcc410e82301484e1bbbc35515afa4ae5206e8c21a51da44681b40d8921dcddba733bf9fed929b9096f7b454c6199a9131561ce31205177db69b579a28e221cc29ad379430c637036177c7aa652541497178a99cbb8a1ff173db6e0313b14357cf2efd328cd15f9f040ca254a9395ac3ba58db88cc2b4239482b18d1d84646506a14766162c94765633d77a80af599a4642b7be6d517b739174dc8f2f504b0304140000000800000021000577229929090000112100001a00000072656365697074732f766572696669636174696f6e2e6a736f6eed59596fdcc811fe2bc43c8f0cb2bbd924b54fbb8a9318c1c686e4244002c3e8a35a62c421273c246b0dfff754159ba3198d6ccbc17a0363fd606bd847d55777b1f87ee5cc769c7af8b9f3b03a5d85a969dec6a5d57ae5ba76ec8d1bff505fc230e2fe706544ae4f550a85b1a9ad9cca95d6ca4993a6ca392bb57095aec0e6655558303658915bed4a6f8bd2facce9d46a3044ba073382ff91a88a54e8932c3d91d9eb343f55e2342bff8947e0a6f6d03ab87057b0317f877ea8bb76752ad6abad1906f0abd3b19f009ffa6eec5cd7ec4e64eb550f0eea2d127f8f420ce34b3b407f6346de7ebf32fea61ebafeee65dbdc2d5450d630f34344aedb6c1b184907d0982d32fb79589d2a959629aebc033711a9e77b5b322fe57ab531efcebad64d7d0fedf8aaef1c20d08111b5a8e1e1395f25e8cbcaabc6b4edfec2394c2c5b7a24269dacdbcb3dae290b3f33b9d89adb851069041cf2f9eb21d39414b3ed867a44e15ff87b8302a4c2a0058d50c297324d4bada4b05e7919ca4ca7224f7359644257652e0a91019412b2ac00997b2384453df5533bd61b78e82a3a14e80aaeb24a83434a260b36ab64d0852e0c522f54ee4a6dc0e442783c0065664a63759aa6129194f79477e65d65cfd267196e0c878e81824fed75dbddb6a89a7fbdf9f085b6db6d9d83eb7a4f34c879361bd3fa8752792f2b5d7a08a9f795b0a5cb9cb136f37956950eaa5c9bbc14ca4a005169b0520655895c48d4685ae93c20780fc637750b84202b2aa984ce2a85121fc29679412befeaf18c43349ddd848d8766fe379af924823c49570f7d66589c223e8e1efa1e6f164a6785d45678fc9be353897642eb1646e75a89342ff8d98a5455b89b6b2fd242eb9cce688dff24de53f83768c01d813b4807efd32f29529dd13afe2e8b543b3e8f7430cebdf07bbc910aee3aa28ba7892efe4e0daf8326fa8af9e48848207d609e9e9e0be68a77887aa14ba28e4fd94c8364280ae2a9cb19c7cc5f66489de491782a630aea9392457ea88d9c3978fc2bf99720143a2324b8cbf2e13fcdab440778859e2ac69d234ae4f29dffef9eff53bc9be288e88a42713ce939b2c89b3f8ad611aa88b35aee711c7b8ab3992adea0384de3293a037846459ec42343d91ca110658c1cc93c401779b8479287437ab857b16e590ee45fe18aa7dbc41fd7087d959a22e616a44a79422ef2ee23c33de227e30e6525c93a54d21c49f5c9bc744075d1ea6c2dd26ec5f639408d2b8aa931dd9df603e5c0233b49ce668af4f634cbe24e883932ea9d7520091963a45c9c7fd672d943af98cfefdd9705a117b36498c725e904910ae217f9027bbf64ef16d1121257b382fe524d50a979ac16a84aedfc87e523c9f53dee7d1ccc27503cccf99ef3b15eb41eb9a6acaff257f18e47383e89aae28c40517a141b1ce774be40693973cc7be8659ed178ae43405afab23af6edf9c3ef58aa0cf7fc2eb7022365248c894f63567cac3381887b39f3919cf5441c484d505f453e5ec4eee900dbe7f8f0794dd419d55e168b56d8e9fc0bea9562fdc25c53630748b54f7397577da2af8cfb473d65ac80df317cc7f0e5187e9368a5ec116b4f7ccfc83927ddc719778ec8258f5d63157163e43e5ebfd4fcd634af2de7e864a055ee082bc6984539e72aeab8af997154cc993a9c7ce9e5f8fd2c70c44bae9273bdcae99d886b66d00e4f78ba2905f77e941bb9d2e189eaebc8aac9479637c46847e6a162c71d8e7b056572a982825869a91eabaf83eed16e6f41f6488d9badb450e12a159e56ebbe7dfcab65baf0ba9f5a6778dc144c33cc53876ea2e1c9a35dd8fccbe2ff45ec1797788b2bdc1bcad9b7492a993dbc45bdf7c337b5d8ad7115a37e3f669107b4b4e018013debcc236dc79aa18c4011e38add2aeb52e8b9ff3b3c7f90d57035a7d507daf7b1aece6f588eee22ea59e31805dc1b09f4f7b983911c8b9ca9948c7d698871ca6f9345b967317c2b7b4c4f14c71c5772d6d1a18f1d79c7c77b85c07d53c9bae4deeac13bd627f5bcf806fac0b16fd060cfbf24efd85bc09317e0ba966670559a7e78b35e85ba81e1cf66b8a2bb797c3e07333f5dd6e399691a3caebffe0cf6fc7eb46c42e031eb2b335ef1cc71bdc2e3536fe629f2fdacf5ed0df475a891d07a65cd00e77053c76126045705995621cbd3149494023586c686dcbb427bab95b136979647865b6869447d77b63fac8e434f9a550fce343ce58e6820caca73605c7a6464f8e69343d150b7a6795d030d0d5fd37cb16eebb13e5cfac860ba6b6a77770e81f9ded6275996a913db4d28813fe9a101d4c3c9ac17c7984f6e3242f3f526d43bc2172d22bdea8ebe6bc810caa2c06e0489d12857e7129b88cc54e8dc50810b681d30d29b5c285b6429fa7a6524802c9dcdd07ecce43f53dd83ff71cf0f0668c25b0fae313d7bc072665f893dbbdc622754c3606ec03f5f2c33ebf478e08da4d19c3cb436c36c76f428f2cafa064e4d8d8676d7db7a3c590c7f939d3ed116a7c9f33674bd8364bc82c46c9102224a30bceacdb449ba90606c26c31ca8c916fac4b871324db24f27898e96dcd6e35532a0973490dc76fd35f4eb24903341324c7819e301893b44d24edb846e9831215c6392257faa7f4a420f90f87ab87e965c8ce69211a187195b37f57897f80e86a4edc6c44ce355d7d7bf40b29d6cb3c0e8fa04f1fb897595a0aa7f48e01de225b0a4cc75823467b618efe390441ba17c5bd363ca4ac2441fc5120c349294b5619a67c95f00106ed324180523aa6bb34e866e42adad93e5fbd54c36de58086f30390d3fb06a6faf3a54ca666ac67a89cb041365726586c4ec0034286374a14571e6f2b2874b0267277f09e3330a1f74e5fed091485be866a319ae578f26d8f58a2cf262840d47dc3f5eccfeb1f845fcfe82d85ad4d625679d0f9ff91ec55fa3fe68ea0675f6bf7da83adcbab8aeb7dbdd272c4ec067a8a7daa3f451ceaf98375003e8730d44f6f8f8b79873e785016d377f2f326e8c697d9665f599cf356889ed749c87c017ca082ca03693ce63393606cb81b6a9a81cf700a5b679ae2dbe08e5c629ab2a8f453a48afb06d57b0237c5c708f3f9db29277fb3de7112e5e18256f31a2def297d1cb69c95dac7c12a89d9a66495cf7cd1ec6db4095eea7bb9d164823ec8723f921915c62b781d5510f40e5fefff031f1578b8ba8a187d85d29ac2a5d1932476f60b2d25965b1f512c61685c1beafa89c95a232a9749935d842e54a638f878b46872056df7c65fced6dfae5e6fb2f504b01021403140000000800000021002f1252219d000000cb0000000d00000000000000000000008081000000006d616e69666573742e6a736f6e504b01021403140000000800000021000577229929090000112100001a00000000000000000000008081c800000072656365697074732f766572696669636174696f6e2e6a736f6e504b0506000000000200020083000000290a00000000").unwrap();
    let observed =
        archive::inspect_untrusted_evidence_archive(&raw, &ArchiveDigest::sha256_bytes(&raw))
            .unwrap();
    assert_eq!(observed.receipts.len(), 1);
    assert_eq!(
        observed
            .members
            .iter()
            .find(|member| member.path == "receipts/verification.json")
            .unwrap()
            .raw,
        archive2a_native()
    );

    for signed in [false, true] {
        let streamed = archive2a_bundle_encoded(
            &[("receipts/verification.json", archive2a_native())],
            true,
            Some(signed),
        );
        let parsed = archive::inspect_untrusted_evidence_archive(
            &streamed,
            &ArchiveDigest::sha256_bytes(&streamed),
        )
        .unwrap();
        assert_eq!(parsed.receipts.len(), 1);
        assert_eq!(
            parsed
                .members
                .iter()
                .find(|member| member.path == "receipts/verification.json")
                .unwrap()
                .raw,
            archive2a_native()
        );
    }
    let zipped = archive2a_bundle_encoded(
        &[("receipts/verification.json", archive2a_native())],
        true,
        None,
    );
    let central = archive2a_central(&zipped);
    let compressed = u32::from_le_bytes(zipped[central + 20..central + 24].try_into().unwrap());
    let data_end = 30 + "manifest.json".len() + compressed as usize;
    let mut trailing = archive2a_insert_first(&zipped, data_end, &[0]);
    let central = archive2a_central(&trailing);
    trailing[18..22].copy_from_slice(&(compressed + 1).to_le_bytes());
    trailing[central + 20..central + 24].copy_from_slice(&(compressed + 1).to_le_bytes());
    archive2a_reject(&trailing, ArchiveError::Checksum);
}

#[test]
fn ci_admission_archive_count_and_offsets_reject_before_decoder() {
    let valid = archive2a_bundle(archive2a_native());
    let eocd = valid.len() - 22;
    let mut count = valid.clone();
    count[eocd + 8..eocd + 12].copy_from_slice(&[65, 0, 65, 0]);
    let mut offset = valid.clone();
    offset[eocd + 16..eocd + 20].copy_from_slice(&(u32::MAX - 2).to_le_bytes());
    let mut length = valid.clone();
    length[eocd + 12..eocd + 16].copy_from_slice(&(u32::MAX - 1).to_le_bytes());
    for (raw, expected) in [
        (count, ArchiveError::DirectoryLimit),
        (offset, ArchiveError::DirectoryBounds),
        (length, ArchiveError::DirectoryBounds),
    ] {
        let entered = std::cell::Cell::new(false);
        let actual =
            archive::inspect_with_decoder(&raw, &ArchiveDigest::sha256_bytes(&raw), |_, _| {
                entered.set(true);
                Err(ArchiveError::NativeReceipt)
            });
        assert_eq!(actual.unwrap_err(), expected);
        assert!(!entered.get());
    }

    let names = (0..64)
        .map(|index| format!("entry-{index:03}.json"))
        .collect::<Vec<_>>();
    let entries = names
        .iter()
        .map(|name| (name.as_str(), b"{}".to_vec()))
        .collect::<Vec<_>>();
    let exact = archive2a_zip(&entries);
    let entered = std::cell::Cell::new(false);
    let result =
        archive::inspect_with_decoder(&exact, &ArchiveDigest::sha256_bytes(&exact), |_, _| {
            entered.set(true);
            Err(ArchiveError::NativeReceipt)
        });
    assert_eq!(result.unwrap_err(), ArchiveError::NativeReceipt);
    assert!(
        entered.get(),
        "exact64 directory bound must reach the injected decoder"
    );
}

#[test]
fn ci_admission_archive_zip64_multidisk_encryption_and_method_are_unsupported() {
    let valid = archive2a_bundle(archive2a_native());
    let eocd = valid.len() - 22;
    let central = archive2a_central(&valid);
    let mut zip64 = valid.clone();
    zip64[eocd + 8..eocd + 12].fill(0xff);
    let mut disk = valid.clone();
    disk[eocd + 4] = 1;
    let mut encrypted = valid.clone();
    encrypted[6] = 1;
    encrypted[central + 8] = 1;
    let mut method = valid.clone();
    method[8] = 12;
    method[central + 10] = 12;
    for raw in [zip64, disk, encrypted, method] {
        archive2a_reject(&raw, ArchiveError::UnsupportedFormat);
    }

    for id in [0x0001_u16, 0x7075, 0x6375, 0x9901] {
        let mut extra = Vec::new();
        archive2a_u16(&mut extra, id);
        archive2a_u16(&mut extra, 0);
        let mut changed = archive2a_insert_first(&valid, 30 + "manifest.json".len(), &extra);
        changed[28..30].copy_from_slice(&4_u16.to_le_bytes());
        archive2a_reject(&changed, ArchiveError::UnsupportedFormat);
    }
}

#[test]
fn ci_admission_archive_paths_aliases_duplicates_and_special_files_are_rejected() {
    for name in [
        "../receipt.json",
        "/receipt.json",
        "C:/receipt.json",
        "receipts\\receipt.json",
        "receipts//receipt.json",
        "receipts/./receipt.json",
        "receipts/../receipt.json",
        "receipt\0.json",
    ] {
        archive2a_reject(
            &archive2a_zip(&[(name, b"{}".to_vec())]),
            ArchiveError::Path,
        );
    }
    for name in ["manifest.json", "MANIFEST.JSON"] {
        archive2a_reject(
            &archive2a_zip(&[("manifest.json", b"{}".to_vec()), (name, b"{}".to_vec())]),
            ArchiveError::DuplicateEntry,
        );
    }
    let valid = archive2a_bundle(archive2a_native());
    let central = archive2a_central(&valid);
    for mode in [0o120777_u32, 0o010600, 0o020600, 0o140600] {
        let mut changed = valid.clone();
        changed[central + 38..central + 42].copy_from_slice(&(mode << 16).to_le_bytes());
        archive2a_reject(&changed, ArchiveError::SpecialFile);
    }

    for name in [
        "receipts/CON.json",
        "receipts/NUL.json",
        "receipts/COM1.json",
        "receipts/a.json.",
        "receipts/a%2e.json",
        "receipts/a：b.json",
    ] {
        archive2a_reject(
            &archive2a_zip(&[(name, b"{}".to_vec())]),
            ArchiveError::Path,
        );
    }
    for flag in [0x10_u32, 0x40, 0x400] {
        let mut changed = valid.clone();
        changed[central + 38..central + 42]
            .copy_from_slice(&((0o100600 << 16) | flag).to_le_bytes());
        archive2a_reject(&changed, ArchiveError::SpecialFile);
    }
}

#[test]
fn ci_admission_archive_truncation_crc_and_directory_local_name_conflict_are_rejected() {
    let valid = archive2a_bundle(archive2a_native());
    for cut in [1, 10, 22] {
        archive2a_reject(&valid[..valid.len() - cut], ArchiveError::ZipStructure);
    }
    let mut crc = valid.clone();
    crc[30 + "manifest.json".len()] ^= 1;
    archive2a_reject(&crc, ArchiveError::Checksum);
    let mut name = valid.clone();
    name[30] = b'X';
    archive2a_reject(&name, ArchiveError::ZipStructure);

    let central = archive2a_central(&valid);
    let old = u32::from_le_bytes(valid[central + 20..central + 24].try_into().unwrap());
    let mut overlapping = valid.clone();
    for offset in [18, 22, central + 20, central + 24] {
        overlapping[offset..offset + 4].copy_from_slice(&(old + 1).to_le_bytes());
    }
    let entered = std::cell::Cell::new(false);
    let result = archive::inspect_with_decoder(
        &overlapping,
        &ArchiveDigest::sha256_bytes(&overlapping),
        |_, _| {
            entered.set(true);
            Err(ArchiveError::NativeReceipt)
        },
    );
    assert_eq!(result.unwrap_err(), ArchiveError::ZipStructure);
    assert!(
        !entered.get(),
        "overlapping checked local/data spans must fail before decoding"
    );
}

#[test]
fn ci_admission_archive_declared_expansion_and_ratio_are_checked_before_decoder() {
    let valid = archive2a_bundle(archive2a_native());
    let central = archive2a_central(&valid);
    for (size, expected) in [
        (64 * 1024 * 1024_u32 + 1, ArchiveError::ExpandedSize),
        (8 * 1024 * 1024, ArchiveError::ExpansionRatio),
    ] {
        let mut changed = valid.clone();
        changed[central + 24..central + 28].copy_from_slice(&size.to_le_bytes());
        let entered = std::cell::Cell::new(false);
        let actual = archive::inspect_with_decoder(
            &changed,
            &ArchiveDigest::sha256_bytes(&changed),
            |_, _| {
                entered.set(true);
                Err(ArchiveError::NativeReceipt)
            },
        );
        assert_eq!(actual.unwrap_err(), expected);
        assert!(!entered.get());
    }

    let compressed = archive2a_deflate(b"hello");
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 5, 5).unwrap(),
        b"hello"
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 4, 5),
        Err(ArchiveError::ExpandedSize)
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 5, 4),
        Err(ArchiveError::ExpandedSize)
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 6, 6),
        Err(ArchiveError::Checksum)
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed[..compressed.len() - 1], 5, 5),
        Err(ArchiveError::Checksum)
    );
    let mut trailing = compressed;
    trailing.push(0);
    assert_eq!(
        archive::decode_deflate_exact(&trailing, 5, 5),
        Err(ArchiveError::Checksum)
    );
}

#[test]
fn ci_admission_archive_exact_manifest_inventory_and_member_digest_are_mandatory() {
    let native = archive2a_native();
    let valid = json!({"schemaVersion":1,"entries":[{"path":"receipts/verification.json","role":"native_verification_evidence","bytes":native.len(),"digest":ArchiveDigest::sha256_bytes(&native)}]});
    for (key, value, expected) in [
        ("schemaVersion", json!(2), ArchiveError::Manifest),
        ("trusted", json!(true), ArchiveError::Manifest),
    ] {
        let mut m = valid.clone();
        m[key] = value;
        archive2a_reject(
            &archive2a_zip(&[
                ("manifest.json", serde_json::to_vec(&m).unwrap()),
                ("receipts/verification.json", native.clone()),
            ]),
            expected,
        );
    }
    // Equal duplicate values are invalid syntax too. A Value-first parser
    // would silently retain one bytes field and falsely accept this member.
    let duplicate_nested = serde_json::to_string(&valid).unwrap().replacen(
        &format!("\"bytes\":{}", native.len()),
        &format!("\"bytes\":{},\"bytes\":{}", native.len(), native.len()),
        1,
    );
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", duplicate_nested.into_bytes()),
            ("receipts/verification.json", native.clone()),
        ]),
        ArchiveError::Manifest,
    );
    let mut bad_digest = valid.clone();
    bad_digest["entries"][0]["digest"] = json!(ArchiveDigest::sha256_bytes(b"different"));
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", serde_json::to_vec(&bad_digest).unwrap()),
            ("receipts/verification.json", native.clone()),
        ]),
        ArchiveError::EntryDigest,
    );
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", serde_json::to_vec(&valid).unwrap()),
            ("receipts/verification.json", native),
            ("extra.json", b"{}".to_vec()),
        ]),
        ArchiveError::Manifest,
    );
    archive2a_reject(
        &archive2a_zip(&[("other-manifest.json", b"{}".to_vec())]),
        ArchiveError::Manifest,
    );

    let native = archive2a_native();
    let entry = json!({"path":"receipts/verification.json","role":"native_verification_evidence","bytes":native.len(),"digest":ArchiveDigest::sha256_bytes(&native)});
    let duplicated = json!({"schemaVersion":1,"entries":[entry.clone(),entry]});
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", serde_json::to_vec(&duplicated).unwrap()),
            ("receipts/verification.json", native),
        ]),
        ArchiveError::Manifest,
    );
}

#[test]
fn ci_admission_archive_native_failure_timeout_truncation_and_accounting_are_not_success() {
    let original: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    for pointer in [
        "/receipt/results/0/passed",
        "/receipt/executionRecords/0/passed",
        "/receipt/executionRecords/0/timedOut",
        "/receipt/results/0/timedOut",
        "/receipt/results/0/outputTruncated",
        "/receipt/executionRecords/0/stderrTruncated",
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = json!(!pointer.ends_with("passed"));
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for pointer in [
        "/receipt/nodesPlanned",
        "/receipt/nodesExecuted",
        "/receipt/processesSpawned",
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = json!(2);
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }

    for (field, invalid) in [
        ("exitCode", json!(1)),
        ("exitCode", Value::Null),
        ("terminationSignal", json!(15)),
        ("spawned", json!(false)),
        ("stdoutTruncated", json!(true)),
        ("timeoutSeconds", json!(0)),
        ("deadlineMs", json!(0)),
        ("elapsedMs", json!(900001)),
        ("stdout", json!("ff")),
        ("stderr", json!("nothex")),
        ("commandDigest", json!("sha256:bad")),
    ] {
        let mut changed = original.clone();
        changed["receipt"]["executionRecords"][0][field] = invalid;
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for (field, invalid) in [
        ("schemaVersion", json!(2)),
        ("advisoryOnly", json!(false)),
        ("confidence", json!("unknown")),
        ("unknowns", json!(["forged_complete_cost"])),
        (
            "repositoryId",
            json!(ArchiveDigest::sha256_bytes(b"foreign-repository")),
        ),
        (
            "runtimeDigest",
            json!(ArchiveDigest::sha256_bytes(b"foreign-runtime")),
        ),
        ("nodesExecuted", json!(2)),
        ("elapsedMs", json!(44081)),
        ("providerAuthenticated", json!(true)),
    ] {
        let mut changed = original.clone();
        changed["receipt"]["costObservation"][field] = invalid;
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for (pointer, invalid) in [
        ("/receipt/results/0/state", json!("fresh")),
        ("/receipt/results/0/reason", json!("fresh_exact_binding")),
        ("/receipt/planReceipt/stage", json!("unknown_stage")),
        (
            "/receipt/planReceipt/reusedNodes",
            json!(["project-command-0"]),
        ),
        (
            "/receipt/planReceipt/sharedNodes",
            json!(["project-command-0"]),
        ),
        ("/receipt/planReceipt/savedExecutions", json!(1)),
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = invalid;
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for pointer in ["", "/receipt", "/receipt/planReceipt"] {
        let mut changed = original.clone();
        changed.pointer_mut(pointer).unwrap()["qualityGateReceipt"] = json!({"passed":true});
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for invalid_version in ["", "1", "v1.0.1", "01.0.1", "1.0.1-01", "1.0.1+"] {
        let mut changed = original.clone();
        // Keep all identities equal so this exercises version structure itself.
        changed["runtimeVersion"] = json!(invalid_version);
        changed["receipt"]["runtimeVersion"] = json!(invalid_version);
        changed["receipt"]["costObservation"]["runtimeVersion"] = json!(invalid_version);
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
}

#[test]
fn ci_admission_archive_receipt_digest_and_identity_mismatch_are_rejected() {
    let original: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    for key in [
        "workItemId",
        "repositoryId",
        "runtimeVersion",
        "runtimeDigest",
    ] {
        let mut changed = original.clone();
        changed[key] = json!("different");
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    let mut changed = original;
    changed["receiptDigest"] = json!(ArchiveDigest::sha256_bytes(b"different"));
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
        ArchiveError::NativeReceipt,
    );
}

#[test]
fn ci_admission_archive_aggregate_expansion_and_node_claim_limits_are_not_ignored() {
    let mut raw = archive2a_zip(&[
        ("a.json", vec![0]),
        ("b.json", vec![0]),
        ("c.json", vec![0]),
    ]);
    let central = archive2a_central(&raw);
    for index in 0..3 {
        let start = central + index * (46 + "a.json".len());
        raw[start + 24..start + 28].copy_from_slice(&(64 * 1024 * 1024_u32).to_le_bytes());
    }
    // Total metadata must be bounded before allocating/decompressing entries;
    // aggregate bound takes precedence over later ratio/span checks.
    archive2a_reject(&raw, ArchiveError::ExpandedSize);
    let mut envelope: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    let result = envelope["receipt"]["results"][0].clone();
    envelope["receipt"]["results"] = Value::Array(vec![result; 4097]);
    envelope["receiptDigest"] = json!(cockpit_protocol::digest_json(&envelope["receipt"]).unwrap());
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&envelope).unwrap()),
        ArchiveError::NodeLimit,
    );

    let exact = archive2a_bundle(archive2a_claims_native(4096, "exact"));
    let parsed =
        archive::inspect_untrusted_evidence_archive(&exact, &ArchiveDigest::sha256_bytes(&exact))
            .unwrap();
    assert_eq!(parsed.receipts[0].receipt.results.len(), 4096);
    let aggregate = archive2a_bundle_encoded(
        &[
            ("receipts/a.json", archive2a_claims_native(2048, "a")),
            ("receipts/b.json", archive2a_claims_native(2048, "b")),
        ],
        false,
        None,
    );
    assert_eq!(
        archive::inspect_untrusted_evidence_archive(
            &aggregate,
            &ArchiveDigest::sha256_bytes(&aggregate)
        )
        .unwrap()
        .receipts
        .len(),
        2
    );
    let overflow = archive2a_bundle_encoded(
        &[
            ("receipts/a.json", archive2a_claims_native(2048, "a")),
            ("receipts/b.json", archive2a_claims_native(2049, "b")),
        ],
        false,
        None,
    );
    archive2a_reject(&overflow, ArchiveError::NodeLimit);
    let mut counter: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    counter["receipt"]["nodesPlanned"] = json!(u64::MAX);
    counter["receiptDigest"] = json!(cockpit_protocol::digest_json(&counter["receipt"]).unwrap());
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&counter).unwrap()),
        ArchiveError::NodeLimit,
    );
}

#[test]
fn ci_admission_archive_duplicate_json_missing_records_and_producer_reuse_remain_rejected() {
    let original = String::from_utf8(archive2a_native()).unwrap();
    let duplicate = original.replacen("\"passed\":true", "\"passed\":true,\"passed\":true", 1);
    assert_ne!(original, duplicate);
    archive2a_reject(
        &archive2a_bundle(duplicate.into_bytes()),
        ArchiveError::NativeReceipt,
    );
    // Use a valid real native retention schema, then duplicate an equal
    // persistence key at depth3; outer deny_unknown_fields cannot catch it.
    let parsed: Value = serde_json::from_str(&original).unwrap();
    let retention = json!({"schemaVersion":1,"repositoryId":parsed["repositoryId"],
        "workItemId":parsed["workItemId"],"createdAt":parsed["createdAt"],
        "retention":{"classification":"internal","persistence":"full_capture",
            "retentionDays":30,"expiresAt":null,"disposalAction":"retain"}});
    let policy: cockpit_protocol::EvidenceRetentionPolicy =
        serde_json::from_value(retention.clone()).unwrap();
    assert!(cockpit_protocol::validate_evidence_retention(&policy.retention).is_ok());
    let mut with_retention = parsed.clone();
    with_retention["retention"] = retention.clone();
    let raw_with_retention = archive2a_bundle(serde_json::to_vec(&with_retention).unwrap());
    let inspected_with_retention = archive::inspect_untrusted_evidence_archive(
        &raw_with_retention,
        &ArchiveDigest::sha256_bytes(&raw_with_retention),
    )
    .unwrap();
    assert_eq!(inspected_with_retention.receipts.len(), 1);
    assert_eq!(
        inspected_with_retention.receipts[0].retention,
        Some(retention.clone())
    );
    for (pointer, invalid) in [
        ("/retention/schemaVersion", json!(2)),
        ("/retention/workItemId", json!("WI-FOREIGN")),
        (
            "/retention/repositoryId",
            json!(ArchiveDigest::sha256_bytes(b"foreign-retention")),
        ),
        ("/retention/createdAt", json!("not-a-timestamp")),
        ("/retention/retention/retentionDays", json!(0)),
        ("/retention/retention/retentionDays", json!(u64::MAX)),
        ("/retention/retention/persistence", json!("digest_only")),
        (
            "/retention/retention/classification",
            json!("secret_prohibited"),
        ),
        ("/retention/retention/disposalAction", json!("")),
    ] {
        let mut changed = with_retention.clone();
        *changed.pointer_mut(pointer).unwrap() = invalid;
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    let mut unknown_retention = with_retention;
    unknown_retention["retention"]["retention"]["providerAuthenticated"] = json!(true);
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&unknown_retention).unwrap()),
        ArchiveError::NativeReceipt,
    );
    let duplicate_retention = serde_json::to_string(&retention).unwrap().replacen(
        "\"persistence\":\"full_capture\"",
        "\"persistence\":\"full_capture\",\"persistence\":\"full_capture\"",
        1,
    );
    let deep_duplicate =
        original.replacen('{', &format!("{{\"retention\":{duplicate_retention},"), 1);
    archive2a_reject(
        &archive2a_bundle(deep_duplicate.into_bytes()),
        ArchiveError::NativeReceipt,
    );
    let mut missing: Value = serde_json::from_str(&original).unwrap();
    missing["receipt"]
        .as_object_mut()
        .unwrap()
        .remove("executionRecords");
    missing["receiptDigest"] = json!(cockpit_protocol::digest_json(&missing["receipt"]).unwrap());
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&missing).unwrap()),
        ArchiveError::NativeReceipt,
    );
    let mut reused: Value = serde_json::from_str(&original).unwrap();
    reused["receipt"]["results"][0]["reused"] = json!(true);
    reused["receiptDigest"] = json!(cockpit_protocol::digest_json(&reused["receipt"]).unwrap());
    // No authenticated upstream proof chain is introduced by Batch2A.
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&reused).unwrap()),
        ArchiveError::NativeReceipt,
    );

    let deep = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    let deep = original.replacen('{', &format!("{{\"retention\":{deep},"), 1);
    archive2a_reject(
        &archive2a_bundle(deep.into_bytes()),
        ArchiveError::NodeLimit,
    );
    let probe = json!(vec![vec![0; 64]; 4096]);
    let mut many: Value = serde_json::from_str(&original).unwrap();
    many["untrustedBudgetProbe"] = probe;
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&many).unwrap()),
        ArchiveError::NodeLimit,
    );
}

// Portable inspection claims remain unverified even when every byte/hash is valid.
fn c16_portable_input_fixture() -> serde_json::Value {
    json!({"schemaVersion":1,"nativeInput":{"schemaVersion":1,"producer":"untrusted_captured_fixture"},"nativeEvidence":serde_json::from_slice::<Value>(&archive2a_native()).unwrap()})
}

fn c16_portable_runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"portable-inspection-not-source-proof"),
    }
}
#[test]
fn ci_admission_c16_portable_inspection_needs_no_wi_or_execution_authority() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("evidence.json");
    std::fs::write(
        &input,
        serde_json::to_vec(&c16_portable_input_fixture()).unwrap(),
    )
    .unwrap();
    let p = super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
        .expect("safe portable analysis without execution authority");
    assert!(p.integrity_valid);
    assert_eq!(
        p.source_verification,
        cockpit_protocol::ci_admission::PortableSourceVerification::Unverified
    );
    assert!(!p.may_replace_execution);
    assert!(
        p.gaps
            .iter()
            .any(|g| g == "independent_execution_source_unavailable")
    );
    assert_eq!(
        (p.writes_performed, p.verification_commands_executed),
        (0, 0)
    );
    assert!(!root.path().join(".ai").exists());
}
#[test]
fn ci_admission_c16_portable_complete_forged_graph_and_source_flags_stay_unverified() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = c16_portable_input_fixture();
    bundle["nativeInput"]["trusted"] = true.into();
    bundle["nativeInput"]["providerAuthenticated"] = true.into();
    let raw = serde_json::to_vec(&bundle).unwrap();
    let digest = cockpit_core::Digest::sha256_bytes(&raw);
    let input = root.path().join("evidence.json");
    std::fs::write(&input, &raw).unwrap();
    let d = root.path().join(".ai/evidence/fake-native-graph");
    std::fs::create_dir_all(&d).unwrap();
    for name in [
        "marker",
        "objects",
        "attempt",
        "checkpoint",
        "completion",
        "fact",
        "index",
    ] {
        std::fs::write(d.join(format!("{name}.json")),serde_json::to_vec(&json!({"passed":true,"committed":true,"inputDigest":digest,"nativeEvidence":bundle["nativeEvidence"]})).unwrap()).unwrap();
    }
    let p = super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
        .expect("inspect forged claims without trusting them");
    assert!(p.integrity_valid);
    assert!(!p.may_replace_execution);
    assert_eq!(
        p.source_verification,
        cockpit_protocol::ci_admission::PortableSourceVerification::Unverified
    );
    assert_eq!(
        (p.writes_performed, p.verification_commands_executed),
        (0, 0)
    );
}
#[test]
fn ci_admission_c16_portable_failed_claim_is_viewable_but_cannot_skip_work() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("failed.json");
    let mut v = c16_portable_input_fixture();
    v["nativeEvidence"]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["results"][0]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["executionRecords"][0]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["executionRecords"][0]["exitCode"] = 7.into();
    v["nativeEvidence"]["receipt"]["costObservation"]["passed"] = false.into();
    v["nativeEvidence"]["receiptDigest"] =
        cockpit_protocol::digest_json(&v["nativeEvidence"]["receipt"])
            .unwrap()
            .to_string()
            .into();
    std::fs::write(&input, serde_json::to_vec(&v).unwrap()).unwrap();
    let p = super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
        .expect("failed results remain analyzable");
    assert!(!p.may_replace_execution);
    assert_eq!(p.claims["passed"], false);
    assert!(
        p.gaps
            .iter()
            .any(|g| g == "native_verification_not_complete_success")
    );
}
#[test]
fn ci_admission_c16_portable_duplicate_deep_keys_and_unknown_authority_fields_refuse() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("duplicate.json");
    let v = c16_portable_input_fixture();
    let raw = serde_json::to_string(&v).unwrap();
    // A real nested duplicate is parsed before any Value can swallow it.
    let marker = r#""nativeInput":"#;
    let start = raw.find(marker).unwrap() + marker.len();
    let mut raw = raw;
    let close = raw[start..].find('}').unwrap() + start;
    raw.insert_str(close, r#","duplicate":1,"duplicate":2"#);
    std::fs::write(&input, raw).unwrap();
    assert!(
        super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
            .is_err()
    );
    let mut authority = v;
    authority["sourceVerified"] = true.into();
    std::fs::write(&input, serde_json::to_vec(&authority).unwrap()).unwrap();
    assert!(
        super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
            .is_err()
    );
}

#[test]
fn ci_admission_c16_portable_view_rejects_nonregular_and_oversize_before_read() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let runtime = c16_portable_runtime();
    assert!(super::plan_portable_evidence(root, None, root, &runtime, None).is_err());
    let input = root.join("oversize.json");
    let file = std::fs::File::create(&input).unwrap();
    file.set_len(super::MAX_PORTABLE_INPUT_BYTES + 1).unwrap();
    let error = super::plan_portable_evidence(root, None, &input, &runtime, None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("portable_input_not_bounded_regular_file")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let target = root.join("real.json");
        std::fs::write(
            &target,
            serde_json::to_vec(&c16_portable_input_fixture()).unwrap(),
        )
        .unwrap();
        let link = root.join("link.json");
        symlink(&target, &link).unwrap();
        let error = super::plan_portable_evidence(root, None, &link, &runtime, None).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("portable_input_not_bounded_regular_file")
        );
        let fifo = root.join("fifo.json");
        let path = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
        // No writer or worker thread is needed: metadata must reject this FIFO
        // before open, and production's raced-open path additionally uses NONBLOCK.
        assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
        let error = super::plan_portable_evidence(root, None, &fifo, &runtime, None).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("portable_input_not_bounded_regular_file")
        );
    }
}

#[test]
fn ci_admission_c16_portable_plan_digest_binds_exact_input_and_runtime() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("evidence.json");
    let raw = serde_json::to_vec(&c16_portable_input_fixture()).unwrap();
    std::fs::write(&input, &raw).unwrap();
    let runtime = c16_portable_runtime();
    let first = super::plan_portable_evidence(root.path(), None, &input, &runtime, None).unwrap();
    assert_eq!(first.input_digest, cockpit_core::Digest::sha256_bytes(&raw));
    let same = super::plan_portable_evidence(root.path(), None, &input, &runtime, None).unwrap();
    assert_eq!(first.plan_digest, same.plan_digest);
    let mut different_runtime = runtime.clone();
    different_runtime.runtime_digest =
        cockpit_core::Digest::sha256_bytes(b"another actual Runtime");
    let changed =
        super::plan_portable_evidence(root.path(), None, &input, &different_runtime, None).unwrap();
    assert_ne!(first.plan_digest, changed.plan_digest);
    let mut raw_with_space = raw;
    raw_with_space.push(b'\n');
    std::fs::write(&input, raw_with_space).unwrap();
    let changed = super::plan_portable_evidence(root.path(), None, &input, &runtime, None).unwrap();
    assert_ne!(first.input_digest, changed.input_digest);
    assert_ne!(first.plan_digest, changed.plan_digest);
    assert!(changed.integrity_valid);
    assert!(!root.path().join(".ai").exists());
}

// C18: self-owned offline archive bytes only. A successful parse cannot mint
// a source capability, provider observation, execution witness or acceptance.
fn c18_portable_archive_fixture() -> Vec<u8> {
    let mut raw = serde_json::to_vec(&c16_portable_input_fixture()).unwrap();
    raw.push(b'\n'); // Preserve exact packet bytes, including insignificant space.
    raw
}

fn c18_portable_archive_reject(raw: &[u8], portable: &[u8], expected: ArchiveError) {
    let result = archive::inspect_untrusted_portable_archive(
        raw,
        &ArchiveDigest::sha256_bytes(raw),
        &ArchiveDigest::sha256_bytes(portable),
    );
    assert_eq!(result.unwrap_err(), expected);
}

#[test]
fn ci_admission_c18_portable_archive_preserves_single_member_for_each_supported_encoding() {
    let portable = c18_portable_archive_fixture();
    for deflate in [false, true] {
        for descriptor in [None, Some(false), Some(true)] {
            let raw = archive2a_zip_encoded(
                &[("portable-native-evidence.json", portable.clone())],
                deflate,
                descriptor,
            );
            let inspected = archive::inspect_untrusted_portable_archive(
                &raw,
                &ArchiveDigest::sha256_bytes(&raw),
                &ArchiveDigest::sha256_bytes(&portable),
            )
            .expect("strict single-file portable evidence inspection");
            assert_eq!(inspected.archive_digest, ArchiveDigest::sha256_bytes(&raw));
            assert_eq!(
                inspected.portable_digest,
                ArchiveDigest::sha256_bytes(&portable)
            );
            assert_eq!(inspected.portable_raw, portable);
            // The original manifest-based profile remains independently strict.
            assert_eq!(
                archive::inspect_untrusted_evidence_archive(
                    &raw,
                    &ArchiveDigest::sha256_bytes(&raw)
                )
                .unwrap_err(),
                ArchiveError::Manifest
            );
        }
    }
    let legacy = archive2a_bundle(archive2a_native());
    assert!(
        archive::inspect_untrusted_evidence_archive(&legacy, &ArchiveDigest::sha256_bytes(&legacy))
            .is_ok()
    );
}

#[test]
fn ci_admission_c18_portable_archive_binds_archive_and_exact_portable_byte_digests() {
    let portable = c18_portable_archive_fixture();
    let raw = archive2a_zip(&[("portable-native-evidence.json", portable.clone())]);
    assert_eq!(
        archive::inspect_untrusted_portable_archive(
            &raw,
            &ArchiveDigest::sha256_bytes(b"another archive"),
            &ArchiveDigest::sha256_bytes(&portable),
        )
        .unwrap_err(),
        ArchiveError::ArchiveDigest
    );
    assert_eq!(
        archive::inspect_untrusted_portable_archive(
            &raw,
            &ArchiveDigest::sha256_bytes(&raw),
            &ArchiveDigest::sha256_bytes(b"another portable packet"),
        )
        .unwrap_err(),
        ArchiveError::EntryDigest
    );
    let mut changed = portable.clone();
    changed.push(b' ');
    let changed_archive = archive2a_zip(&[("portable-native-evidence.json", changed)]);
    // CRC and archive SHA are correct; a semantically equivalent packet still
    // has to match the independent expected hash of its original raw bytes.
    c18_portable_archive_reject(&changed_archive, &portable, ArchiveError::EntryDigest);
}

#[test]
fn ci_admission_c18_portable_archive_requires_only_the_exact_regular_root_member() {
    let portable = c18_portable_archive_fixture();
    for name in [
        "manifest.json",
        "PORTABLE-NATIVE-EVIDENCE.JSON",
        "nested/portable-native-evidence.json",
    ] {
        c18_portable_archive_reject(
            &archive2a_zip(&[(name, portable.clone())]),
            &portable,
            ArchiveError::Manifest,
        );
    }
    c18_portable_archive_reject(&archive2a_zip(&[]), &portable, ArchiveError::Manifest);
    c18_portable_archive_reject(
        &archive2a_zip(&[
            ("portable-native-evidence.json", portable.clone()),
            ("extra.json", b"{}".to_vec()),
        ]),
        &portable,
        ArchiveError::Manifest,
    );
    c18_portable_archive_reject(
        &archive2a_bundle(archive2a_native()),
        &portable,
        ArchiveError::Manifest,
    );
    c18_portable_archive_reject(
        &archive2a_zip(&[
            ("portable-native-evidence.json", portable.clone()),
            ("PORTABLE-NATIVE-EVIDENCE.JSON", portable.clone()),
        ]),
        &portable,
        ArchiveError::DuplicateEntry,
    );
    for name in [
        "../portable-native-evidence.json",
        "/portable-native-evidence.json",
    ] {
        c18_portable_archive_reject(
            &archive2a_zip(&[(name, portable.clone())]),
            &portable,
            ArchiveError::Path,
        );
    }
    let raw = archive2a_zip(&[("portable-native-evidence.json", portable.clone())]);
    let central = archive2a_central(&raw);
    for mode in [0o120777_u32, 0o010600, 0o020600, 0o140600] {
        let mut changed = raw.clone();
        changed[central + 38..central + 42].copy_from_slice(&(mode << 16).to_le_bytes());
        c18_portable_archive_reject(&changed, &portable, ArchiveError::SpecialFile);
    }
}

#[test]
fn ci_admission_c18_portable_archive_rejects_duplicate_json_and_invalid_native_semantics() {
    let original = c16_portable_input_fixture();
    let mut failures = Vec::new();
    let mut changed = original.clone();
    changed["schemaVersion"] = 2.into();
    failures.push(serde_json::to_vec(&changed).unwrap());
    let mut changed = original.clone();
    changed["sourceVerified"] = true.into();
    failures.push(serde_json::to_vec(&changed).unwrap());
    let mut changed = original.clone();
    changed["nativeEvidence"]["passed"] = false.into();
    failures.push(serde_json::to_vec(&changed).unwrap());
    let mut changed = original.clone();
    changed["nativeEvidence"]["receiptDigest"] = ArchiveDigest::sha256_bytes(b"other receipt")
        .to_string()
        .into();
    failures.push(serde_json::to_vec(&changed).unwrap());
    let mut changed = original;
    changed["nativeEvidence"]["receipt"]
        .as_object_mut()
        .unwrap()
        .remove("executionRecords");
    changed["nativeEvidence"]["receiptDigest"] =
        json!(cockpit_protocol::digest_json(&changed["nativeEvidence"]["receipt"]).unwrap());
    failures.push(serde_json::to_vec(&changed).unwrap());
    let mut duplicate =
        br#"{"schemaVersion":1,"nativeInput":{"nested":{"nonce":0,"nonce":1}},"nativeEvidence":"#
            .to_vec();
    duplicate.extend_from_slice(&archive2a_native());
    duplicate.push(b'}');
    failures.push(duplicate);
    let mut trailing = c18_portable_archive_fixture();
    trailing.extend_from_slice(b"{}");
    failures.push(trailing);
    for portable in failures {
        let raw = archive2a_zip(&[("portable-native-evidence.json", portable.clone())]);
        // All ZIP CRCs and both expected byte hashes are internally correct.
        c18_portable_archive_reject(&raw, &portable, ArchiveError::NativeReceipt);
    }
}

#[test]
fn ci_admission_c18_portable_archive_keeps_checked_zip_crc_offsets_descriptors_and_stream_end() {
    let portable = c18_portable_archive_fixture();
    let valid = archive2a_zip(&[("portable-native-evidence.json", portable.clone())]);
    let central = archive2a_central(&valid);
    let eocd = valid.len() - 22;
    let mut failures = Vec::new();
    let mut count = valid.clone();
    count[eocd + 8..eocd + 12].copy_from_slice(&[65, 0, 65, 0]);
    failures.push((count, ArchiveError::DirectoryLimit));
    let mut offset = valid.clone();
    offset[eocd + 16..eocd + 20].copy_from_slice(&(u32::MAX - 2).to_le_bytes());
    failures.push((offset, ArchiveError::DirectoryBounds));
    let mut encrypted = valid.clone();
    encrypted[6] = 1;
    encrypted[central + 8] = 1;
    failures.push((encrypted, ArchiveError::UnsupportedFormat));
    let mut method = valid.clone();
    method[8] = 12;
    method[central + 10] = 12;
    failures.push((method, ArchiveError::UnsupportedFormat));
    let mut crc = valid.clone();
    crc[30 + "portable-native-evidence.json".len()] ^= 1;
    failures.push((crc, ArchiveError::Checksum));
    let mut name = valid.clone();
    name[30] = b'X';
    failures.push((name, ArchiveError::ZipStructure));
    failures.push((
        valid[..valid.len() - 1].to_vec(),
        ArchiveError::ZipStructure,
    ));
    let mut trailing = valid;
    trailing.push(0);
    failures.push((trailing, ArchiveError::ZipStructure));
    let mut descriptor = archive2a_zip_encoded(
        &[("portable-native-evidence.json", portable.clone())],
        false,
        Some(true),
    );
    descriptor[30 + "portable-native-evidence.json".len() + portable.len() + 4] ^= 1;
    failures.push((descriptor, ArchiveError::ZipStructure));
    let overlap = archive2a_zip(&[
        ("portable-native-evidence.json", portable.clone()),
        ("extra.json", b"{}".to_vec()),
    ]);
    let central = archive2a_central(&overlap);
    let mut overlap = overlap;
    for offset in [18, 22, central + 20, central + 24] {
        overlap[offset..offset + 4].copy_from_slice(&((portable.len() + 1) as u32).to_le_bytes());
    }
    failures.push((overlap, ArchiveError::ZipStructure));
    let deflated = archive2a_zip_encoded(
        &[("portable-native-evidence.json", portable.clone())],
        true,
        None,
    );
    let central = archive2a_central(&deflated);
    let compressed = u32::from_le_bytes(deflated[central + 20..central + 24].try_into().unwrap());
    let end = 30 + "portable-native-evidence.json".len() + compressed as usize;
    let mut tail = archive2a_insert_first(&deflated, end, &[0]);
    let central = archive2a_central(&tail);
    tail[18..22].copy_from_slice(&(compressed + 1).to_le_bytes());
    tail[central + 20..central + 24].copy_from_slice(&(compressed + 1).to_le_bytes());
    failures.push((tail, ArchiveError::Checksum));
    for (raw, expected) in failures {
        c18_portable_archive_reject(&raw, &portable, expected);
    }
}

#[test]
fn ci_admission_c18_portable_archive_retains_raw_expansion_ratio_and_json_budgets() {
    let portable = c18_portable_archive_fixture();
    {
        let oversized = vec![0_u8; archive::MAX_EVIDENCE_ARCHIVE_BYTES as usize + 1];
        assert_eq!(
            archive::inspect_untrusted_portable_archive(
                &oversized,
                &ArchiveDigest::sha256_bytes(b"must reject raw size before hashing"),
                &ArchiveDigest::sha256_bytes(&portable),
            )
            .unwrap_err(),
            ArchiveError::RawSize
        );
    }
    let valid = archive2a_zip(&[("portable-native-evidence.json", portable.clone())]);
    let central = archive2a_central(&valid);
    for (expanded, expected) in [
        (64 * 1024 * 1024_u32 + 1, ArchiveError::ExpandedSize),
        (
            portable.len() as u32 * 100 + 1,
            ArchiveError::ExpansionRatio,
        ),
    ] {
        let mut changed = valid.clone();
        changed[central + 24..central + 28].copy_from_slice(&expanded.to_le_bytes());
        c18_portable_archive_reject(&changed, &portable, expected);
    }
    let aggregate = archive2a_zip(&[
        ("portable-native-evidence.json", portable.clone()),
        ("extra-a.json", b"{}".to_vec()),
        ("extra-b.json", b"{}".to_vec()),
    ]);
    let mut position = archive2a_central(&aggregate);
    let mut aggregate = aggregate;
    for size in [64 * 1024 * 1024_u32, 64 * 1024 * 1024, 1] {
        aggregate[position + 24..position + 28].copy_from_slice(&size.to_le_bytes());
        let length = usize::from(u16::from_le_bytes(
            aggregate[position + 28..position + 30].try_into().unwrap(),
        ));
        position += 46 + length;
    }
    c18_portable_archive_reject(&aggregate, &portable, ArchiveError::ExpandedSize);
    let mut deep = json!(null);
    for _ in 0..66 {
        deep = json!([deep]);
    }
    let mut depth = c16_portable_input_fixture();
    depth["nativeInput"] = deep;
    let mut array = c16_portable_input_fixture();
    array["nativeInput"] = json!({"items":vec![0;4097]});
    let mut nodes = c16_portable_input_fixture();
    nodes["nativeInput"] = json!({"rows":vec![vec![0;4096];64]});
    for packet in [depth, array, nodes] {
        let portable = serde_json::to_vec(&packet).unwrap();
        let raw = archive2a_zip(&[("portable-native-evidence.json", portable.clone())]);
        c18_portable_archive_reject(&raw, &portable, ArchiveError::NodeLimit);
    }
}
