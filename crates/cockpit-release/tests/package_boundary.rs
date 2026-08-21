use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn root_license_has_the_approved_mit_notice() {
    let license = fs::read_to_string(repository_root().join("LICENSE"))
        .expect("root LICENSE must exist before release packaging");
    assert!(license.starts_with("MIT License\n"));
    assert!(license.contains("Copyright (c) 2026 Ray\n"));
    assert!(license.contains("The above copyright notice and this permission notice"));
}
