//! Deterministic, read-only descriptions of discoverable Runtime interfaces.
//!
//! This module is deliberately data-only. It does not inspect a repository,
//! read Work Item history, infer authority, or execute a command. CLI and MCP
//! adapters project the same facts through their respective transports.

use serde::{Deserialize, Serialize};

pub const INTERFACE_DESCRIPTION_SCHEMA_VERSION: u32 = 1;
pub const WORK_ITEM_OUTCOME_SURFACE: &str = "work-item-outcome";
pub const WORK_ITEM_OUTCOME_VIEW_SUMMARY: &str = "summary";
pub const WORK_ITEM_OUTCOME_VIEW_FULL: &str = "full";
pub const WORK_ITEM_OUTCOME_VIEW_VALUES: &[&str] =
    &[WORK_ITEM_OUTCOME_VIEW_SUMMARY, WORK_ITEM_OUTCOME_VIEW_FULL];
pub const WORK_ITEM_OUTCOME_DEFAULT_VIEW: &str = WORK_ITEM_OUTCOME_VIEW_SUMMARY;
pub const WORK_ITEM_OUTCOME_DEFAULT_DELIVERY: bool = false;
pub const WORK_ITEM_OUTCOME_DEFAULT_JSON: bool = false;
pub const WORK_ITEM_OUTCOME_ID_DESCRIPTION: &str = "Canonical Work Item identifier.";
pub const WORK_ITEM_OUTCOME_DELIVERY_DESCRIPTION: &str =
    "Request the immutable full archive delivery payload.";
pub const WORK_ITEM_OUTCOME_JSON_DESCRIPTION: &str =
    "Emit machine-readable JSON instead of the human handoff.";
pub const WORK_ITEM_OUTCOME_VIEW_DESCRIPTION: &str =
    "Select the reader-first summary or complete human view.";
pub const WORK_ITEM_OUTCOME_LANGUAGE_DESCRIPTION: &str = "Active conversation language for the human Outcome; adapters should pass it explicitly, with locale fallback only when omitted.";
/// Conversation languages accepted by the human Outcome projections.
/// `zh-CN` is kept as a locale-compatible spelling and normalizes to the
/// canonical Simplified Chinese renderer.
pub const WORK_ITEM_OUTCOME_LANGUAGE_VALUES: &[&str] = &["en", "zh", "zh-CN", "ja"];

pub fn normalize_work_item_outcome_language(value: &str) -> &'static str {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.starts_with("zh") {
        "zh"
    } else if normalized.starts_with("ja") {
        "ja"
    } else {
        "en"
    }
}

pub fn work_item_outcome_language_is_valid(value: &str) -> bool {
    WORK_ITEM_OUTCOME_LANGUAGE_VALUES.contains(&value)
}

pub const CAPABILITY_SHOW_SURFACE: &str = WORK_ITEM_OUTCOME_SURFACE;
pub const CAPABILITY_SHOW_FORMAT_JSON: &str = "json";
pub const CAPABILITY_SHOW_FORMAT_MARKDOWN: &str = "markdown";
pub const CAPABILITY_SHOW_FORMAT_VALUES: &[&str] =
    &[CAPABILITY_SHOW_FORMAT_JSON, CAPABILITY_SHOW_FORMAT_MARKDOWN];
pub const CAPABILITY_SHOW_DEFAULT_FORMAT: &str = CAPABILITY_SHOW_FORMAT_JSON;
pub const CAPABILITY_SHOW_DEFAULT_LANGUAGE: &str = "en";
pub const CAPABILITY_SHOW_LANGUAGE_VALUES: &[&str] = WORK_ITEM_OUTCOME_LANGUAGE_VALUES;
pub const CAPABILITY_SHOW_SURFACE_DESCRIPTION: &str =
    "Optional read-only interface description surface.";
pub const CAPABILITY_SHOW_FORMAT_DESCRIPTION: &str =
    "Description encoding; JSON is language-neutral.";
pub const CAPABILITY_SHOW_LANGUAGE_DESCRIPTION: &str =
    "Markdown labels only; structured facts remain unchanged.";

pub fn work_item_outcome_view_is_valid(value: &str) -> bool {
    WORK_ITEM_OUTCOME_VIEW_VALUES.contains(&value)
}

pub fn capability_show_format_is_valid(value: &str) -> bool {
    CAPABILITY_SHOW_FORMAT_VALUES.contains(&value)
}

pub fn capability_show_language_is_valid(value: &str) -> bool {
    CAPABILITY_SHOW_LANGUAGE_VALUES.contains(&value)
}

const fn bool_default_text(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InterfaceDescription {
    pub schema_version: u32,
    pub name: String,
    pub runtime_version: String,
    pub surfaces: Vec<InterfaceSurface>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InterfaceSurface {
    pub name: String,
    pub transport: String,
    pub parameters: Vec<InterfaceParameter>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InterfaceParameter {
    pub name: String,
    pub wire_type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    pub enum_values: Vec<String>,
    pub aliases: Vec<String>,
    pub description: String,
}

/// The protocol-owned facts for one discoverable parameter.
///
/// Adapters consume this table rather than copying defaults, enums, or
/// descriptions into a second schema.  The table is deliberately transport
/// aware only at the surface level; it does not encode authorization or
/// lifecycle behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterfaceParameterSpec {
    pub name: &'static str,
    pub wire_type: &'static str,
    pub required: bool,
    pub default: Option<&'static str>,
    pub enum_values: &'static [&'static str],
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}

const OUTCOME_DELIVERY_PARAMETER: InterfaceParameterSpec = InterfaceParameterSpec {
    name: "delivery",
    wire_type: "boolean",
    required: false,
    default: Some(bool_default_text(WORK_ITEM_OUTCOME_DEFAULT_DELIVERY)),
    enum_values: &[],
    aliases: &[],
    description: WORK_ITEM_OUTCOME_DELIVERY_DESCRIPTION,
};

const OUTCOME_VIEW_PARAMETER: InterfaceParameterSpec = InterfaceParameterSpec {
    name: "view",
    wire_type: "enum",
    required: false,
    default: Some(WORK_ITEM_OUTCOME_DEFAULT_VIEW),
    enum_values: WORK_ITEM_OUTCOME_VIEW_VALUES,
    aliases: &[],
    description: WORK_ITEM_OUTCOME_VIEW_DESCRIPTION,
};

const OUTCOME_LANGUAGE_PARAMETER: InterfaceParameterSpec = InterfaceParameterSpec {
    name: "language",
    wire_type: "enum",
    required: false,
    default: None,
    enum_values: WORK_ITEM_OUTCOME_LANGUAGE_VALUES,
    aliases: &[],
    description: WORK_ITEM_OUTCOME_LANGUAGE_DESCRIPTION,
};

static CLI_WORK_ITEM_OUTCOME_PARAMETERS: &[InterfaceParameterSpec] = &[
    InterfaceParameterSpec {
        name: "id",
        wire_type: "string",
        required: true,
        default: None,
        enum_values: &[],
        aliases: &[],
        description: WORK_ITEM_OUTCOME_ID_DESCRIPTION,
    },
    OUTCOME_DELIVERY_PARAMETER,
    InterfaceParameterSpec {
        name: "json",
        wire_type: "boolean",
        required: false,
        default: Some(bool_default_text(WORK_ITEM_OUTCOME_DEFAULT_JSON)),
        enum_values: &[],
        aliases: &[],
        description: WORK_ITEM_OUTCOME_JSON_DESCRIPTION,
    },
    OUTCOME_VIEW_PARAMETER,
    OUTCOME_LANGUAGE_PARAMETER,
];

static MCP_WORK_ITEM_OUTCOME_PARAMETERS: &[InterfaceParameterSpec] = &[
    InterfaceParameterSpec {
        name: "workItemId",
        wire_type: "string",
        required: true,
        default: None,
        enum_values: &[],
        aliases: &["id"],
        description: "Canonical Work Item identifier; `id` is a deprecated alias.",
    },
    OUTCOME_LANGUAGE_PARAMETER,
    OUTCOME_VIEW_PARAMETER,
    OUTCOME_DELIVERY_PARAMETER,
    InterfaceParameterSpec {
        name: "deliveryProgress",
        wire_type: "object",
        required: false,
        default: None,
        enum_values: &[],
        aliases: &[],
        description: "Identity-bound accepted-segment progress for an interrupted delivery.",
    },
];

static CAPABILITY_SHOW_PARAMETERS: &[InterfaceParameterSpec] = &[
    InterfaceParameterSpec {
        name: "surface",
        wire_type: "enum",
        required: false,
        default: None,
        enum_values: &[CAPABILITY_SHOW_SURFACE],
        aliases: &[],
        description: CAPABILITY_SHOW_SURFACE_DESCRIPTION,
    },
    InterfaceParameterSpec {
        name: "format",
        wire_type: "enum",
        required: false,
        default: Some(CAPABILITY_SHOW_DEFAULT_FORMAT),
        enum_values: CAPABILITY_SHOW_FORMAT_VALUES,
        aliases: &[],
        description: CAPABILITY_SHOW_FORMAT_DESCRIPTION,
    },
    InterfaceParameterSpec {
        name: "language",
        wire_type: "enum",
        required: false,
        default: Some(CAPABILITY_SHOW_DEFAULT_LANGUAGE),
        enum_values: CAPABILITY_SHOW_LANGUAGE_VALUES,
        aliases: &[],
        description: CAPABILITY_SHOW_LANGUAGE_DESCRIPTION,
    },
];

/// Return the protocol-owned parameter table for a discoverable surface.
pub fn work_item_outcome_interface_specs(
    surface: &str,
) -> Option<&'static [InterfaceParameterSpec]> {
    match surface {
        "cli" => Some(CLI_WORK_ITEM_OUTCOME_PARAMETERS),
        "mcp" => Some(MCP_WORK_ITEM_OUTCOME_PARAMETERS),
        _ => None,
    }
}

/// Return one protocol-owned parameter fact for adapter schema generation.
pub fn work_item_outcome_parameter_spec(
    surface: &str,
    name: &str,
) -> Option<&'static InterfaceParameterSpec> {
    work_item_outcome_interface_specs(surface)?
        .iter()
        .find(|spec| spec.name == name)
}

/// Return the protocol-owned parameter facts for the read-only capability
/// discovery surface. Adapters must project this table instead of repeating
/// property names, defaults, enums, or descriptions.
pub fn capability_show_interface_specs() -> &'static [InterfaceParameterSpec] {
    CAPABILITY_SHOW_PARAMETERS
}

pub fn capability_show_parameter_spec(name: &str) -> Option<&'static InterfaceParameterSpec> {
    CAPABILITY_SHOW_PARAMETERS
        .iter()
        .find(|spec| spec.name == name)
}

fn materialize_parameter(spec: &InterfaceParameterSpec) -> InterfaceParameter {
    InterfaceParameter {
        name: spec.name.into(),
        wire_type: spec.wire_type.into(),
        required: spec.required,
        default: spec.default.map(str::to_owned),
        enum_values: spec
            .enum_values
            .iter()
            .map(|value| (*value).into())
            .collect(),
        aliases: spec.aliases.iter().map(|value| (*value).into()).collect(),
        description: spec.description.into(),
    }
}

/// Return the one protocol-owned description for the trial interface.
///
/// The CLI and MCP surfaces intentionally expose transport-specific names;
/// all parameter facts come from the protocol-owned tables above.
pub fn work_item_outcome_interface_description() -> InterfaceDescription {
    InterfaceDescription {
        schema_version: INTERFACE_DESCRIPTION_SCHEMA_VERSION,
        name: WORK_ITEM_OUTCOME_SURFACE.into(),
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        surfaces: [
            ("cli", "argv", CLI_WORK_ITEM_OUTCOME_PARAMETERS),
            ("mcp", "json-rpc", MCP_WORK_ITEM_OUTCOME_PARAMETERS),
        ]
        .into_iter()
        .map(|(name, transport, specs)| InterfaceSurface {
            name: name.into(),
            transport: transport.into(),
            parameters: specs.iter().map(materialize_parameter).collect(),
        })
        .collect(),
    }
}

fn localized_labels(language: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match language {
        "zh" | "zh-CN" => ("接口事实", "传输", "参数", "类型"),
        "ja" => ("インターフェース事実", "トランスポート", "パラメータ", "型"),
        _ => ("Interface facts", "Transport", "Parameters", "Type"),
    }
}

fn localized_required(language: &str) -> &'static str {
    match language {
        "zh" | "zh-CN" => "必填",
        "ja" => "必須",
        _ => "Required",
    }
}

fn localized_default(language: &str) -> &'static str {
    match language {
        "zh" | "zh-CN" => "默认",
        "ja" => "既定値",
        _ => "Default",
    }
}

fn localized_enum(language: &str) -> &'static str {
    match language {
        "zh" | "zh-CN" => "枚举",
        "ja" => "列挙",
        _ => "Enum",
    }
}

fn localized_aliases(language: &str) -> &'static str {
    match language {
        "zh" | "zh-CN" => "别名",
        "ja" => "別名",
        _ => "Aliases",
    }
}

fn localized_invariant_note(language: &str) -> &'static str {
    match language {
        "zh" | "zh-CN" => "参数名、类型、必填性、默认值和枚举值是结构化事实；本描述不授予权限。",
        "ja" => {
            "パラメータ名、型、必須性、既定値、列挙値は構造化された事実であり、この説明は権限を与えません。"
        }
        _ => {
            "Names, types, requiredness, defaults, and enum values are structured facts; this description grants no authority."
        }
    }
}

/// Render the description as a deterministic generated Markdown region.
pub fn render_interface_description_markdown(
    description: &InterfaceDescription,
    language: &str,
) -> String {
    let language = normalize_work_item_outcome_language(language);
    let (title, transport_label, parameters_label, type_label) = localized_labels(language);
    let required_label = localized_required(language);
    let default_label = localized_default(language);
    let enum_label = localized_enum(language);
    let aliases_label = localized_aliases(language);
    let mut output = String::new();
    output.push_str("<!-- AI_COCKPIT_INTERFACE_FACTS:BEGIN work-item-outcome -->\n");
    output.push_str(&format!("### {}: `{}`\n\n", title, description.name));
    output.push_str(&format!(
        "- Schema: `v{}`\n- Runtime: `{}`\n- {}\n\n",
        description.schema_version,
        description.runtime_version,
        localized_invariant_note(language)
    ));
    for surface in &description.surfaces {
        output.push_str(&format!(
            "#### `{}` · {}: `{}`\n\n",
            surface.name, transport_label, surface.transport
        ));
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n| --- | --- | --- | --- | --- | --- |\n",
            parameters_label, type_label, required_label, default_label, enum_label, aliases_label
        ));
        for parameter in &surface.parameters {
            let default = parameter.default.as_deref().unwrap_or("—");
            let enum_values = if parameter.enum_values.is_empty() {
                "—".into()
            } else {
                parameter.enum_values.join(" | ")
            };
            let aliases = if parameter.aliases.is_empty() {
                "—".into()
            } else {
                parameter.aliases.join(" | ")
            };
            output.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
                parameter.name,
                parameter.wire_type,
                if parameter.required { "yes" } else { "no" },
                default,
                enum_values,
                aliases
            ));
        }
        output.push('\n');
    }
    output.push_str("<!-- AI_COCKPIT_INTERFACE_FACTS:END work-item-outcome -->\n");
    output
}
