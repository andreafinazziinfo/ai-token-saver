mod common;

/// Integration tests — run with: cargo test --ignored
/// Requires: cargo install --path . (rtk in PATH)

#[test]
#[ignore]
fn rtk_binary_is_in_path() {
    assert!(std::process::Command::new("rtk")
        .arg("--version")
        .output()
        .expect("rtk not found — run `cargo install --path .`")
        .status
        .success());
}

#[test]
#[ignore]
fn rewrite_git_status_exit_0() {
    let out = std::process::Command::new("rtk")
        .args(["rewrite", "git status"])
        .output()
        .expect("rtk not found");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "rtk git status"
    );
}

#[test]
#[ignore]
fn rewrite_unknown_cmd_exit_1() {
    let status = std::process::Command::new("rtk")
        .args(["rewrite", "python manage.py runserver"])
        .status()
        .expect("rtk not found");
    assert_eq!(status.code(), Some(1));
}

#[test]
#[ignore]
fn rewrite_force_push_exit_2() {
    let status = std::process::Command::new("rtk")
        .args(["rewrite", "git push origin main --force"])
        .status()
        .expect("rtk not found");
    assert_eq!(status.code(), Some(2));
}

#[test]
#[ignore]
fn rewrite_git_push_exit_3() {
    let out = std::process::Command::new("rtk")
        .args(["rewrite", "git push"])
        .output()
        .expect("rtk not found");
    assert_eq!(out.status.code(), Some(3));
    assert!(!out.stdout.is_empty());
}

/// Token savings on real fixtures (populate tests/fixtures/ first via fixture_baseline.sh)
#[test]
#[ignore]
fn git_status_token_savings_gte_60pct() {
    let fixture = "tests/fixtures/git_status_raw.txt";
    if !std::path::Path::new(fixture).exists() {
        eprintln!("SKIP: {fixture} not found — run scripts/bench/fixture_baseline.sh first");
        return;
    }
    let input = std::fs::read_to_string(fixture).unwrap();
    let out = std::process::Command::new("rtk")
        .args(["git", "status"])
        .output()
        .expect("rtk not found");
    let filtered = String::from_utf8_lossy(&out.stdout).to_string();
    let (savings, passes) = common::token_savings(&input, &filtered);
    assert!(passes, "git status savings {savings:.1}% < 60%");
}
