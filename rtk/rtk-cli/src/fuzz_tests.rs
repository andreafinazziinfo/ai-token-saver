#![cfg(test)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_cargo_build_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::cargo_build::filter(&s);
    }

    #[test]
    fn test_cargo_test_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::cargo_test::filter(&s);
    }

    #[test]
    fn test_git_diff_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::git_diff::filter(&s);
    }

    #[test]
    fn test_git_log_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::git_log::filter(&s);
    }

    #[test]
    fn test_git_status_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::git_status::filter(&s);
    }

    #[test]
    fn test_pytest_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::pytest_filter::filter(&s);
    }

    #[test]
    fn test_go_test_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::go_test::filter(&s);
    }

    #[test]
    fn test_gradle_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::gradle::filter(&s);
    }

    #[test]
    fn test_ls_filter_no_panic(s in any::<String>()) {
        let _ = rtk_filters::ls_filter::filter(&s);
    }

    #[test]
    fn test_dlp_redact_no_panic(s in any::<String>()) {
        let _ = rtk_db::dlp::redact(&s);
    }
}
