#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub classification: &'static str,
    pub value: String,
}

impl Binding {
    pub fn content(value: &str) -> Self {
        Self {
            classification: "content-bound",
            value: value.into(),
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
