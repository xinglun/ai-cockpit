use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::ReleaseError;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Issuer {
    pub repository: String,
    #[serde(rename = "workflowRef")]
    pub workflow_ref: String,
    pub run_id: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    pub repository: String,
    #[serde(rename = "baseRef")]
    pub base_ref: String,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseBinding {
    pub tag: String,
    pub commit: String,
    #[serde(rename = "providerReleaseId")]
    pub provider_release_id: u64,
    #[serde(rename = "manifestSha256")]
    pub manifest_sha256: String,
    #[serde(rename = "formulaSha256")]
    pub formula_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandoffDocument {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub issuer: Issuer,
    pub destination: Destination,
    pub release: ReleaseBinding,
    #[serde(rename = "authorizedAction")]
    pub authorized_action: String,
    #[serde(rename = "issuedAt")]
    pub issued_at: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}

#[derive(Serialize)]
struct HandoffWithoutRequestId<'a> {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    issuer: &'a Issuer,
    destination: &'a Destination,
    release: &'a ReleaseBinding,
    #[serde(rename = "authorizedAction")]
    authorized_action: &'a str,
    #[serde(rename = "issuedAt")]
    issued_at: &'a str,
    #[serde(rename = "expiresAt")]
    expires_at: &'a str,
}

impl HandoffDocument {
    pub fn new(
        issuer: Issuer,
        destination: Destination,
        release: ReleaseBinding,
        authorized_action: String,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, ReleaseError> {
        let issued_at = format_timestamp(issued_at);
        let expires_at = format_timestamp(expires_at);
        let document = Self {
            schema_version: 1,
            request_id: String::new(),
            issuer,
            destination,
            release,
            authorized_action,
            issued_at,
            expires_at,
        };
        document.validate_static()?;
        validate_window(&document.issued_at, &document.expires_at)?;
        let request_id = document.recompute_request_id()?;
        Ok(Self {
            request_id,
            ..document
        })
    }

    pub fn parse_str(input: &str) -> Result<Self, ReleaseError> {
        let document: Self = serde_json::from_str(input)?;
        document.validate_static()?;
        validate_window(&document.issued_at, &document.expires_at)?;
        if document.request_id != document.recompute_request_id()? {
            return Err(ReleaseError::Invalid(
                "handoff request identity mismatch".into(),
            ));
        }
        Ok(document)
    }

    pub fn recompute_request_id(&self) -> Result<String, ReleaseError> {
        let canonical = HandoffWithoutRequestId {
            schema_version: self.schema_version,
            issuer: &self.issuer,
            destination: &self.destination,
            release: &self.release,
            authorized_action: &self.authorized_action,
            issued_at: &self.issued_at,
            expires_at: &self.expires_at,
        };
        let bytes = serde_json::to_vec(&canonical)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ReleaseError> {
        let mut bytes = serde_json::to_vec(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), ReleaseError> {
        self.validate_static()?;
        if self.request_id != self.recompute_request_id()? {
            return Err(ReleaseError::Invalid(
                "handoff request identity mismatch".into(),
            ));
        }
        let issued = parse_timestamp(&self.issued_at)?;
        let expires = parse_timestamp(&self.expires_at)?;
        validate_window(&self.issued_at, &self.expires_at)?;
        if now < issued - chrono::Duration::minutes(5) {
            return Err(ReleaseError::Invalid(
                "handoff issuance is outside clock skew".into(),
            ));
        }
        if now > expires + chrono::Duration::minutes(5) {
            return Err(ReleaseError::Invalid("handoff is expired".into()));
        }
        Ok(())
    }

    fn validate_static(&self) -> Result<(), ReleaseError> {
        if self.schema_version != 1 {
            return Err(ReleaseError::Invalid("unsupported handoff schema".into()));
        }
        let workflow_commit = self.issuer.workflow_ref.rsplit('@').next();
        if self.issuer.repository != "xinglun/ai-cockpit"
            || !self
                .issuer
                .workflow_ref
                .starts_with("xinglun/ai-cockpit/.github/workflows/release.yml@")
            || !workflow_commit.is_some_and(is_commit)
        {
            return Err(ReleaseError::Invalid(
                "invalid handoff issuer identity".into(),
            ));
        }
        if self.destination.repository != "xinglun/homebrew-tap"
            || self.destination.base_ref != "main"
            || self.destination.path != "Formula/ai-cockpit.rb"
        {
            return Err(ReleaseError::Invalid("invalid handoff destination".into()));
        }
        if self.authorized_action != "open_pull_request" {
            return Err(ReleaseError::Invalid(
                "unsupported handoff authorized action".into(),
            ));
        }
        let version = self
            .release
            .tag
            .strip_prefix('v')
            .and_then(|value| semver::Version::parse(value).ok())
            .ok_or_else(|| ReleaseError::Invalid("invalid handoff release tag".into()))?;
        if workflow_commit != Some(self.release.commit.as_str()) {
            return Err(ReleaseError::Invalid(
                "handoff workflow commit must match release commit".into(),
            ));
        }
        if format!("v{version}") != self.release.tag
            || !is_commit(&self.release.commit)
            || self.release.provider_release_id == 0
        {
            return Err(ReleaseError::Invalid(
                "invalid handoff release identity".into(),
            ));
        }
        validate_digest(&self.release.manifest_sha256, "manifest")?;
        validate_digest(&self.release.formula_sha256, "Formula")?;
        parse_timestamp(&self.issued_at)?;
        parse_timestamp(&self.expires_at)?;
        Ok(())
    }
}

fn format_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, ReleaseError> {
    let parsed = DateTime::parse_from_rfc3339(value)
        .map_err(|error| ReleaseError::Invalid(format!("invalid RFC3339 timestamp: {error}")))?
        .with_timezone(&Utc);
    if format_timestamp(parsed) != value {
        return Err(ReleaseError::Invalid(
            "handoff timestamps must use UTC RFC3339 seconds".into(),
        ));
    }
    Ok(parsed)
}

fn validate_window(issued_at: &str, expires_at: &str) -> Result<(), ReleaseError> {
    let issued = parse_timestamp(issued_at)?;
    let expires = parse_timestamp(expires_at)?;
    if expires <= issued {
        return Err(ReleaseError::Invalid(
            "handoff expiry must follow issuance".into(),
        ));
    }
    if expires - issued > chrono::Duration::hours(24) {
        return Err(ReleaseError::Invalid(
            "handoff expiry cannot exceed 24 hours".into(),
        ));
    }
    Ok(())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_digest(value: &str, name: &str) -> Result<(), ReleaseError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ReleaseError::Invalid(format!("invalid {name} digest")));
    }
    Ok(())
}
