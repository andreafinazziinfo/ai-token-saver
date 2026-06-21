use std::fs;

fn find_fixtures_dir() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    loop {
        let candidate = dir.join("fixtures");
        if candidate.exists() && candidate.is_dir() {
            return candidate;
        }
        if !dir.pop() {
            panic!("Could not find fixtures directory in parent tree");
        }
    }
}

#[test]
fn test_golden_fixtures_git_status() {
    let fixtures_dir = find_fixtures_dir();
    let input = fs::read_to_string(fixtures_dir.join("git_status/input.txt")).unwrap();
    let expected = fs::read_to_string(fixtures_dir.join("git_status/expected.txt")).unwrap();

    let filtered = rtk_filters::git_status::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}

#[test]
fn test_golden_fixtures_git_diff() {
    let fixtures_dir = find_fixtures_dir();
    let input = fs::read_to_string(fixtures_dir.join("git_diff/input.txt")).unwrap();
    let expected = fs::read_to_string(fixtures_dir.join("git_diff/expected.txt")).unwrap();

    let filtered = rtk_filters::git_diff::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}

#[test]
fn test_golden_fixtures_cargo_build() {
    let fixtures_dir = find_fixtures_dir();
    let input = fs::read_to_string(fixtures_dir.join("cargo_build/input.txt")).unwrap();
    let expected = fs::read_to_string(fixtures_dir.join("cargo_build/expected.txt")).unwrap();

    let filtered = rtk_filters::cargo_build::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}

#[test]
fn test_golden_fixtures_cargo_test() {
    let fixtures_dir = find_fixtures_dir();
    let input = fs::read_to_string(fixtures_dir.join("cargo_test/input.txt")).unwrap();
    let expected = fs::read_to_string(fixtures_dir.join("cargo_test/expected.txt")).unwrap();

    let filtered = rtk_filters::cargo_test::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}
