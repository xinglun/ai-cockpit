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
pub const WORK_ITEM_OUTCOME_LANGUAGE_VALUES: &[&str] = &["en", "zh", "ja"];

pub fn work_item_outcome_view_is_valid(value: &str) -> bool {
    WORK_ITEM_OUTCOME_VIEW_VALUES.contains(&value)
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

fn parameter(
    name: &str,
    wire_type: &str,
    required: bool,
    default: Option<&str>,
    enum_values: &[&str],
    aliases: &[&str],
    description: &str,
) -> InterfaceParameter {
    InterfaceParameter {
        name: name.into(),
        wire_type: wire_type.into(),
        required,
        default: default.map(str::to_owned),
        enum_values: enum_values.iter().map(|value| (*value).into()).collect(),
        aliases: aliases.iter().map(|value| (*value).into()).collect(),
        description: description.into(),
    }
}

/// Return the one protocol-owned description for the trial interface.
///
/// The CLI and MCP surfaces intentionally expose their transport-specific
/// names, while shared facts such as `view`, `delivery`, and their defaults
/// are constructed from the same literals here.
pub fn work_item_outcome_interface_description() -> InterfaceDescription {
    InterfaceDescription {
        schema_version: INTERFACE_DESCRIPTION_SCHEMA_VERSION,
        name: WORK_ITEM_OUTCOME_SURFACE.into(),
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        surfaces: vec![
            InterfaceSurface {
                name: "cli".into(),
                transport: "argv".into(),
                parameters: vec![
                    parameter(
                        "id",
                        "string",
                        true,
                        None,
                        &[],
                        &[],
                        "Canonical Work Item identifier.",
                    ),
                    parameter(
                        "delivery",
                        "boolean",
                        false,
                        Some(if WORK_ITEM_OUTCOME_DEFAULT_DELIVERY {
                            "true"
                        } else {
                            "false"
                        }),
                        &[],
                        &[],
                        "Request the immutable full archive delivery payload.",
                    ),
                    parameter(
                        "json",
                        "boolean",
                        false,
                        Some(if WORK_ITEM_OUTCOME_DEFAULT_JSON {
                            "true"
                        } else {
                            "false"
                        }),
                        &[],
                        &[],
                        "Emit machine-readable JSON instead of the human handoff.",
                    ),
                    parameter(
                        "view",
                        "enum",
                        false,
                        Some(WORK_ITEM_OUTCOME_DEFAULT_VIEW),
                        WORK_ITEM_OUTCOME_VIEW_VALUES,
                        &[],
                        "Select the reader-first summary or complete human view.",
                    ),
                ],
            },
            InterfaceSurface {
                name: "mcp".into(),
                transport: "json-rpc".into(),
                parameters: vec![
                    parameter(
                        "workItemId",
                        "string",
                        true,
                        None,
                        &[],
                        &["id"],
                        "Canonical Work Item identifier; `id` is a deprecated alias.",
                    ),
                    parameter(
                        "language",
                        "enum",
                        false,
                        None,
                        WORK_ITEM_OUTCOME_LANGUAGE_VALUES,
                        &[],
                        "Presentation language; localization does not change facts.",
                    ),
                    parameter(
                        "view",
                        "enum",
                        false,
                        Some(WORK_ITEM_OUTCOME_DEFAULT_VIEW),
                        WORK_ITEM_OUTCOME_VIEW_VALUES,
                        &[],
                        "Select the reader-first summary or complete human view.",
                    ),
                    parameter(
                        "delivery",
                        "boolean",
                        false,
                        Some(if WORK_ITEM_OUTCOME_DEFAULT_DELIVERY {
                            "true"
                        } else {
                            "false"
                        }),
                        &[],
                        &[],
                        "Request the immutable full archive delivery payload.",
                    ),
                    parameter(
                        "deliveryProgress",
                        "object",
                        false,
                        None,
                        &[],
                        &[],
                        "Identity-bound accepted-segment progress for an interrupted delivery.",
                    ),
                ],
            },
        ],
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
