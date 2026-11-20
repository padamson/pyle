#[path = "../build/build_id.rs"]
mod build_id;

use build_id::build_id;

#[test]
fn a_build_without_git_prints_the_bare_version() {
    assert_eq!(build_id("0.1.0", None, &[]), "0.1.0");
}

#[test]
fn a_commit_off_any_tag_prints_its_sha() {
    assert_eq!(build_id("0.1.0", Some("abc1234"), &[]), "0.1.0 (abc1234)");
}

#[test]
fn the_release_tag_prints_the_bare_version() {
    assert_eq!(build_id("0.1.0", Some("abc1234"), &["v0.1.0"]), "0.1.0");
}

#[test]
fn the_release_tag_among_others_prints_the_bare_version() {
    assert_eq!(
        build_id("0.1.0", Some("abc1234"), &["latest", "v0.1.0"]),
        "0.1.0"
    );
}

#[test]
fn a_tag_for_another_version_still_prints_the_sha() {
    assert_eq!(
        build_id("0.1.0", Some("abc1234"), &["v0.0.9"]),
        "0.1.0 (abc1234)"
    );
}
