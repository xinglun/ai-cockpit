use crate::{error::ReleaseError, manifest::ReleaseManifest};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormulaSource {
    Production { release_origin: String },
    Fixture { base_url: String },
}

pub fn render_formula(
    manifest: &ReleaseManifest,
    source: FormulaSource,
) -> Result<String, ReleaseError> {
    manifest.validate()?;
    let base_url = match &source {
        FormulaSource::Production { release_origin } => {
            const PRODUCTION_ORIGIN: &str =
                "https://github.com/xinglun/ai-cockpit/releases/download/";
            if release_origin != PRODUCTION_ORIGIN {
                return Err(ReleaseError::Invalid(
                    "production Formula requires the fixed HTTPS GitHub origin".into(),
                ));
            }
            release_origin.clone()
        }
        FormulaSource::Fixture { base_url } => {
            if !is_loopback_http_origin(base_url) {
                return Err(ReleaseError::Invalid(
                    "fixture Formula requires a loopback HTTP origin".into(),
                ));
            }
            base_url.clone()
        }
    };
    let arm = manifest
        .artifacts()
        .iter()
        .find(|artifact| artifact.target == "aarch64-apple-darwin")
        .ok_or_else(|| ReleaseError::Invalid("missing macOS ARM64 artifact".into()))?;
    let intel = manifest
        .artifacts()
        .iter()
        .find(|artifact| artifact.target == "x86_64-apple-darwin")
        .ok_or_else(|| ReleaseError::Invalid("missing macOS Intel artifact".into()))?;
    let url = |filename: &str| format!("{}{}/{}", base_url, manifest.tag, filename);
    let fixture_comment = match source {
        FormulaSource::Production { .. } => "",
        FormulaSource::Fixture { .. } => "  # TEST-ONLY staged fixture Formula\n",
    };
    Ok(format!(
        "class AiCockpit < Formula\n  desc \"AI Cockpit governance runtime\"\n  homepage \"https://github.com/xinglun/ai-cockpit\"\n  license \"MIT\"\n  version \"{}\"\n{}\n  on_macos do\n    on_arm do\n      url \"{}\"\n      sha256 \"{}\"\n    end\n    on_intel do\n      url \"{}\"\n      sha256 \"{}\"\n    end\n  end\n\n  on_linux do\n    odie \"ai-cockpit Homebrew Formula currently supports macOS only\"\n  end\n\n  def install\n    bin.install \"ai-cockpit\"\n  end\n\n  test do\n    assert_match version.to_s, shell_output(\"#{{bin}}/ai-cockpit --version\")\n    system \"#{{bin}}/ai-cockpit\", \"--help\"\n  end\nend\n",
        manifest.version,
        fixture_comment,
        url(&arm.archive.filename),
        arm.archive.sha256,
        url(&intel.archive.filename),
        intel.archive.sha256,
    ))
}

fn is_loopback_http_origin(origin: &str) -> bool {
    let Some(port) = origin
        .strip_prefix("http://127.0.0.1:")
        .and_then(|value| value.strip_suffix('/'))
    else {
        return false;
    };
    !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit())
}
