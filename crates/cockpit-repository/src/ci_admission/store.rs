//! Bounded immutable portable-record storage. Returned bytes have no source authority.
//! RED scaffold only: production I/O follows after a real assertion RED.

use crate::ObserverError;
use std::path::Path;

pub(super) const MAX_PORTABLE_INPUT_BYTES: u64 = 64 * 1024 * 1024;
pub(super) const MAX_PORTABLE_MARKER_BYTES: u64 = 128 * 1024;

/// Cold storage returns bytes, never a verified source capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PortableStoredRecord {
    pub(super) input: Vec<u8>,
    pub(super) marker: Vec<u8>,
    pub(super) record_ref: String,
}

pub(super) fn publish_portable_record(
    root: &Path,
    wi: &str,
    record_id: &str,
    input: &[u8],
    marker: &[u8],
) -> Result<PortableStoredRecord, ObserverError> {
    let _ = (
        wi,
        record_id,
        input,
        marker,
        MAX_PORTABLE_INPUT_BYTES,
        MAX_PORTABLE_MARKER_BYTES,
    );
    Err(ObserverError::State {
        path: root.join(".ai/evidence"),
        message: "portable_store_not_implemented (RED scaffold)".into(),
    })
}

pub(super) fn load_portable_record(
    root: &Path,
    wi: &str,
    record_id: &str,
) -> Result<Option<PortableStoredRecord>, ObserverError> {
    let _ = (
        wi,
        record_id,
        MAX_PORTABLE_INPUT_BYTES,
        MAX_PORTABLE_MARKER_BYTES,
    );
    Err(ObserverError::State {
        path: root.join(".ai/evidence"),
        message: "portable_store_not_implemented (RED scaffold)".into(),
    })
}
