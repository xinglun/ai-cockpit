//! Deterministic, read-only descriptions of discoverable Runtime interfaces.
//!
//! This module is deliberately data-only. It does not inspect a repository,
//! read Work Item history, infer authority, or execute a command. CLI and MCP
//! adapters project the same facts through their respective transports.

use clap::{ArgAction, Args, Command, ValueEnum};
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
pub const WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID: &str = "workItemId";
pub const WORK_ITEM_OUTCOME_CANONICAL_DELIVERY: &str = "delivery";
pub const WORK_ITEM_OUTCOME_CANONICAL_JSON: &str = "json";
pub const WORK_ITEM_OUTCOME_CANONICAL_VIEW: &str = "view";
pub const WORK_ITEM_OUTCOME_CANONICAL_LANGUAGE: &str = "language";
pub const WORK_ITEM_OUTCOME_CANONICAL_DELIVERY_PROGRESS: &str = "deliveryProgress";
// Transport bindings are protocol-owned facts too.  Keeping the CLI spelling
// here prevents the derive parser from becoming a second interface registry.
pub const WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID: &str = "id";
pub const WORK_ITEM_OUTCOME_CLI_DELIVERY: &str = WORK_ITEM_OUTCOME_CANONICAL_DELIVERY;
pub const WORK_ITEM_OUTCOME_CLI_JSON: &str = WORK_ITEM_OUTCOME_CANONICAL_JSON;
pub const WORK_ITEM_OUTCOME_CLI_VIEW: &str = WORK_ITEM_OUTCOME_CANONICAL_VIEW;
pub const WORK_ITEM_OUTCOME_CLI_LANGUAGE: &str = WORK_ITEM_OUTCOME_CANONICAL_LANGUAGE;
pub const WORK_ITEM_OUTCOME_MCP_WORK_ITEM_ID: &str = WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID;
pub const WORK_ITEM_OUTCOME_MCP_DELIVERY: &str = WORK_ITEM_OUTCOME_CANONICAL_DELIVERY;
pub const WORK_ITEM_OUTCOME_MCP_VIEW: &str = WORK_ITEM_OUTCOME_CANONICAL_VIEW;
pub const WORK_ITEM_OUTCOME_MCP_LANGUAGE: &str = WORK_ITEM_OUTCOME_CANONICAL_LANGUAGE;
pub const WORK_ITEM_OUTCOME_MCP_DELIVERY_PROGRESS: &str =
    WORK_ITEM_OUTCOME_CANONICAL_DELIVERY_PROGRESS;
/// Conversation languages accepted by the human Outcome projections.
/// `zh-CN` is kept as a locale-compatible spelling and normalizes to the
/// canonical Simplified Chinese renderer.
pub const WORK_ITEM_OUTCOME_LANGUAGE_VALUES: &[&str] = &["en", "zh", "zh-CN", "ja"];

/// Actual CLI parser arguments for the discoverable Outcome query.
///
/// The CLI flattens this type into `work-item outcome`; interface discovery
/// reads the same Clap command model.  Keep transport-specific MCP-only
/// parameters outside this type.
#[derive(Clone, Debug, Args)]
pub struct WorkItemOutcomeQueryArgs {
    #[arg(long, help = WORK_ITEM_OUTCOME_ID_DESCRIPTION)]
    pub id: String,
    #[arg(
        long,
        action = ArgAction::SetTrue,
        default_value_t = false,
        help = WORK_ITEM_OUTCOME_DELIVERY_DESCRIPTION
    )]
    pub delivery: bool,
    #[arg(
        long,
        action = ArgAction::SetTrue,
        default_value_t = false,
        help = WORK_ITEM_OUTCOME_JSON_DESCRIPTION
    )]
    pub json: bool,
    #[arg(
        long,
        value_enum,
        default_value_t = WorkItemOutcomeView::Summary,
        help = WORK_ITEM_OUTCOME_VIEW_DESCRIPTION
    )]
    pub view: WorkItemOutcomeView,
    #[arg(long, value_enum, help = WORK_ITEM_OUTCOME_LANGUAGE_DESCRIPTION)]
    pub language: Option<WorkItemOutcomeLanguage>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum WorkItemOutcomeView {
    #[value(name = WORK_ITEM_OUTCOME_VIEW_SUMMARY)]
    Summary,
    #[value(name = WORK_ITEM_OUTCOME_VIEW_FULL)]
    Full,
}

impl WorkItemOutcomeView {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Summary => WORK_ITEM_OUTCOME_VIEW_SUMMARY,
            Self::Full => WORK_ITEM_OUTCOME_VIEW_FULL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum WorkItemOutcomeLanguage {
    #[value(name = "en")]
    En,
    #[value(name = "zh")]
    Zh,
    #[value(name = "zh-CN")]
    ZhCn,
    #[value(name = "ja")]
    Ja,
}

impl WorkItemOutcomeLanguage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
            Self::ZhCn => "zh-CN",
            Self::Ja => "ja",
        }
    }
}

/// Return the actual Clap command fragment shared by query parsing and
/// interface description generation.
pub fn work_item_outcome_query_command() -> Command {
    WorkItemOutcomeQueryArgs::augment_args(Command::new("outcome"))
}

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

fn cli_parameter_from_query_argument(argument: &clap::Arg) -> InterfaceParameter {
    let name = argument.get_id().as_str();
    let boolean = matches!(
        argument.get_action(),
        ArgAction::SetTrue | ArgAction::SetFalse
    );
    let enum_values = if boolean {
        Vec::new()
    } else {
        argument
            .get_possible_values()
            .into_iter()
            .map(|value| value.get_name().to_owned())
            .collect()
    };
    InterfaceParameter {
        name: name.into(),
        wire_type: if boolean {
            "boolean".into()
        } else if enum_values.is_empty() {
            "string".into()
        } else {
            "enum".into()
        },
        required: argument.is_required_set(),
        default: argument
            .get_default_values()
            .first()
            .map(|value| value.to_string_lossy().into_owned()),
        enum_values,
        aliases: Vec::new(),
        description: argument
            .get_help()
            .map(ToString::to_string)
            .unwrap_or_default(),
    }
}

fn cli_outcome_parameters_from_query_parser() -> Vec<InterfaceParameter> {
    work_item_outcome_query_command()
        .get_arguments()
        .map(cli_parameter_from_query_argument)
        .collect()
}

/// The protocol-owned facts for one discoverable parameter.
///
/// Adapters consume this table rather than copying defaults, enums, or
/// descriptions into a second schema.  The table is deliberately transport
/// aware only at the surface level; it does not encode authorization or
/// lifecycle behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterfaceParameterSpec {
    /// Stable logical parameter identity. Surface bindings may expose a
    /// transport-specific name or alias, but all wire facts come from this
    /// canonical definition.
    pub canonical_name: &'static str,
    pub name: &'static str,
    pub wire_type: &'static str,
    pub required: bool,
    pub default: Option<&'static str>,
    pub enum_values: &'static [&'static str],
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}

/// A transport projection plus parser-derived facts for one Outcome parameter.
///
/// Defaults, enum values, types, and descriptions are materialized from the
/// actual shared Clap parser. This structure represents the request schema
/// consumed by an adapter and by interface discovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeInterfaceParameterSpec {
    pub canonical_name: String,
    pub name: String,
    pub wire_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub enum_values: Vec<String>,
    pub aliases: Vec<String>,
    pub description: String,
}

fn cli_outcome_parameter_specs() -> Vec<OutcomeInterfaceParameterSpec> {
    cli_outcome_parameters_from_query_parser()
        .into_iter()
        .map(|parameter| OutcomeInterfaceParameterSpec {
            canonical_name: if parameter.name == WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID {
                WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID.into()
            } else {
                parameter.name.clone()
            },
            name: parameter.name,
            wire_type: parameter.wire_type,
            required: parameter.required,
            default: parameter.default,
            enum_values: parameter.enum_values,
            aliases: Vec::new(),
            description: parameter.description,
        })
        .collect()
}

fn mcp_outcome_parameter_from_cli(
    parameter: &OutcomeInterfaceParameterSpec,
) -> Option<OutcomeInterfaceParameterSpec> {
    if parameter.name == WORK_ITEM_OUTCOME_CLI_JSON {
        return None;
    }
    let identity = parameter.name == WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID;
    Some(OutcomeInterfaceParameterSpec {
        canonical_name: parameter.canonical_name.clone(),
        name: if identity {
            WORK_ITEM_OUTCOME_MCP_WORK_ITEM_ID.into()
        } else {
            parameter.name.clone()
        },
        wire_type: parameter.wire_type.clone(),
        required: parameter.required,
        default: parameter.default.clone(),
        enum_values: parameter.enum_values.clone(),
        aliases: if identity {
            vec![WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID.into()]
        } else {
            Vec::new()
        },
        description: parameter.description.clone(),
    })
}

fn mcp_only_outcome_delivery_progress_parameter() -> OutcomeInterfaceParameterSpec {
    OutcomeInterfaceParameterSpec {
        canonical_name: WORK_ITEM_OUTCOME_CANONICAL_DELIVERY_PROGRESS.into(),
        name: WORK_ITEM_OUTCOME_MCP_DELIVERY_PROGRESS.into(),
        wire_type: "object".into(),
        required: false,
        default: None,
        enum_values: Vec::new(),
        aliases: Vec::new(),
        description: "Identity-bound accepted-segment progress for an interrupted delivery.".into(),
    }
}

/// Return the actual MCP request schema for `work_item_outcome`.
///
/// Every shared parameter is projected from the executable CLI query parser.
/// `deliveryProgress` is the one MCP-only request field and is appended here,
/// so the MCP handler and discovery projection consume the same definition.
pub fn work_item_outcome_mcp_request_parameter_specs() -> Vec<OutcomeInterfaceParameterSpec> {
    let mut specs = cli_outcome_parameter_specs()
        .iter()
        .filter_map(mcp_outcome_parameter_from_cli)
        .collect::<Vec<_>>();
    let mcp_order = [
        WORK_ITEM_OUTCOME_MCP_WORK_ITEM_ID,
        WORK_ITEM_OUTCOME_MCP_LANGUAGE,
        WORK_ITEM_OUTCOME_MCP_VIEW,
        WORK_ITEM_OUTCOME_MCP_DELIVERY,
    ];
    specs.sort_by_key(|spec| {
        mcp_order
            .iter()
            .position(|name| *name == spec.name)
            .unwrap_or(mcp_order.len())
    });
    specs.push(mcp_only_outcome_delivery_progress_parameter());
    specs
}

static CAPABILITY_SHOW_PARAMETERS: &[InterfaceParameterSpec] = &[
    InterfaceParameterSpec {
        canonical_name: "surface",
        name: "surface",
        wire_type: "enum",
        required: false,
        default: None,
        enum_values: &[CAPABILITY_SHOW_SURFACE],
        aliases: &[],
        description: CAPABILITY_SHOW_SURFACE_DESCRIPTION,
    },
    InterfaceParameterSpec {
        canonical_name: "format",
        name: "format",
        wire_type: "enum",
        required: false,
        default: Some(CAPABILITY_SHOW_DEFAULT_FORMAT),
        enum_values: CAPABILITY_SHOW_FORMAT_VALUES,
        aliases: &[],
        description: CAPABILITY_SHOW_FORMAT_DESCRIPTION,
    },
    InterfaceParameterSpec {
        canonical_name: "language",
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
) -> Option<Vec<OutcomeInterfaceParameterSpec>> {
    match surface {
        "cli" => Some(cli_outcome_parameter_specs()),
        "mcp" => Some(work_item_outcome_mcp_request_parameter_specs()),
        _ => None,
    }
}

/// Return one protocol-owned parameter fact for adapter schema generation.
pub fn work_item_outcome_parameter_spec(
    surface: &str,
    name: &str,
) -> Option<OutcomeInterfaceParameterSpec> {
    work_item_outcome_interface_specs(surface)?
        .iter()
        .find(|spec| spec.name == name)
        .cloned()
}

/// Return a transport binding by its protocol-owned canonical identity.
///
/// The binding may expose a different transport name (for example the CLI's
/// `id` versus MCP's `workItemId`) or aliases, but adapters must resolve that
/// difference from this table rather than repeating the mapping in runtime
/// handlers.
pub fn work_item_outcome_parameter_spec_by_canonical(
    surface: &str,
    canonical_name: &str,
) -> Option<OutcomeInterfaceParameterSpec> {
    work_item_outcome_interface_specs(surface)?
        .iter()
        .find(|spec| spec.canonical_name == canonical_name)
        .cloned()
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

fn materialize_outcome_parameter(spec: &OutcomeInterfaceParameterSpec) -> InterfaceParameter {
    InterfaceParameter {
        name: spec.name.clone(),
        wire_type: spec.wire_type.clone(),
        required: spec.required,
        default: spec.default.clone(),
        enum_values: spec.enum_values.clone(),
        aliases: spec.aliases.clone(),
        description: spec.description.clone(),
    }
}

/// Return the one protocol-owned description for the trial interface.
///
/// The CLI and MCP surfaces intentionally expose transport-specific names;
/// all shared parameter facts come from the executable parser above.
pub fn work_item_outcome_interface_description() -> InterfaceDescription {
    InterfaceDescription {
        schema_version: INTERFACE_DESCRIPTION_SCHEMA_VERSION,
        name: WORK_ITEM_OUTCOME_SURFACE.into(),
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        surfaces: [("cli", "argv"), ("mcp", "json-rpc")]
            .into_iter()
            .map(|(name, transport)| {
                let parameters = work_item_outcome_interface_specs(name)
                    .expect("known Outcome interface surface")
                    .iter()
                    .map(materialize_outcome_parameter)
                    .collect();
                InterfaceSurface {
                    name: name.into(),
                    transport: transport.into(),
                    parameters,
                }
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
