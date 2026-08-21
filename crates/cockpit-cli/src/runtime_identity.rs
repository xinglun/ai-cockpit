use anyhow::{Context, Result};
use cockpit_core::Digest;
use cockpit_protocol::{PROTOCOL_VERSION, RuntimeContext};
use sha2::{Digest as ShaDigest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn load(path: &Path) -> Result<RuntimeContext> {
    let mut executable = std::fs::File::open(path)
        .with_context(|| format!("open runtime executable {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes_read = executable
            .read(&mut buffer)
            .with_context(|| format!("read runtime executable {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let runtime_digest = format!("sha256:{}", hex::encode(hasher.finalize()))
        .parse::<Digest>()
        .context("validate runtime executable digest")?;
    Ok(RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest,
    })
}

pub fn load_current() -> Result<RuntimeContext> {
    let executable = std::env::current_exe().context("resolve current runtime executable")?;
    load(&executable)
}

#[cfg(test)]
mod tests {
    #[test]
    fn missing_executable_path_fails_closed() {
        let missing = std::env::temp_dir().join(format!(
            "ai-cockpit-missing-runtime-identity-{}",
            std::process::id()
        ));
        let error = super::load(&missing).expect_err("missing executable must fail");
        assert!(error.to_string().contains("open runtime executable"));
    }
}
