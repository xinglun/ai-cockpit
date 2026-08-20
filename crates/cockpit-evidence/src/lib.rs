#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvidenceBinding {
    Content {
        digest: String,
    },
    Diff {
        base_commit: String,
        head_commit: String,
        changed_paths_digest: String,
    },
    Environment {
        fingerprint: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub classification: &'static str,
    pub value: String,
    pub evidence_binding: EvidenceBinding,
}

impl Binding {
    pub fn content(value: &str) -> Self {
        Self {
            classification: "content-bound",
            value: value.into(),
            evidence_binding: EvidenceBinding::Content {
                digest: value.into(),
            },
        }
    }

    pub fn diff(base_commit: &str, head_commit: &str, changed_paths_digest: &str) -> Self {
        Self {
            classification: "diff-bound",
            value: changed_paths_digest.into(),
            evidence_binding: EvidenceBinding::Diff {
                base_commit: base_commit.into(),
                head_commit: head_commit.into(),
                changed_paths_digest: changed_paths_digest.into(),
            },
        }
    }

    pub fn environment(fingerprint: &str) -> Self {
        Self {
            classification: "environment-bound",
            value: fingerprint.into(),
            evidence_binding: EvidenceBinding::Environment {
                fingerprint: fingerprint.into(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReuseInput {
    pub content_digest: Option<String>,
    pub protected: bool,
    pub expired: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReuseAction {
    Reuse,
    Execute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReuseDecision {
    pub action: ReuseAction,
    pub reason: &'static str,
}

pub fn decide_reuse(binding: &Binding, input: &ReuseInput) -> ReuseDecision {
    if input.protected {
        return ReuseDecision {
            action: ReuseAction::Execute,
            reason: "protected_node",
        };
    }
    if input.expired {
        return ReuseDecision {
            action: ReuseAction::Execute,
            reason: "expired",
        };
    }
    if input.content_digest.as_deref() == Some(binding.value.as_str()) {
        ReuseDecision {
            action: ReuseAction::Reuse,
            reason: "fresh_exact_binding",
        }
    } else {
        ReuseDecision {
            action: ReuseAction::Execute,
            reason: "binding_mismatch",
        }
    }
}
