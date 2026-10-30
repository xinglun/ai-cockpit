use std::process::Command;

#[test]
fn github_plan_help_explains_exact_attempt_and_non_admission() {
    let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["evidence", "github", "plan", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    for text in [
        "--repo",
        "--work-item",
        "--run-id",
        "--attempt",
        "--job-id",
        "--artifact-id",
        "never accepts evidence",
        "latest attempt",
    ] {
        assert!(help.contains(text), "missing public help {text}: {help}");
    }
    for private in ["__verification-supervisor", "__composition-supervisor"] {
        assert!(!help.contains(private));
    }
}

#[test]
fn github_plan_rejects_zero_selector_and_fake_authority_before_repository_io() {
    for (flag, value) in [
        ("--run-id", "0"),
        ("--provider-authenticated", "true"),
        ("--endpoint", "https://evil.invalid"),
    ] {
        let mut args = vec![
            "evidence",
            "github",
            "plan",
            "--repo",
            "/nonexistent-plan-fixture",
            "--work-item",
            "WI-TEST",
            "--run-id",
            "1",
            "--attempt",
            "1",
            "--job-id",
            "2",
            "--artifact-id",
            "3",
        ];
        if flag == "--run-id" {
            args[8] = value;
        } else {
            args.extend([flag, value]);
        }
        let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        let error = String::from_utf8(out.stderr).unwrap();
        assert!(error.contains(if flag == "--run-id" {
            "invalid value"
        } else {
            "unexpected argument"
        }));
    }
}
