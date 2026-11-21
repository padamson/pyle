//! leptotes tests for the version bar: what its first render holds. What
//! happens after it (picking a version, the manifest arriving) needs a
//! browser.

use std::collections::BTreeSet;

use leptos::prelude::*;
use pyle_core::{Manifest, Prefix, Snapshot, Version};

use super::{DEFAULT_DEV_NOTICE, VersionBar, VersionSwitcher, message};
use crate::versions::Notice;

const V13: Version = Version::new(0, 13, 0);
const V14: Version = Version::new(0, 14, 0);
const V15: Version = Version::new(0, 15, 0);

fn releases() -> Option<Manifest> {
    Some(Manifest::new([V13, V14, V15]))
}

/// The bar of a site under `/cqa-grader`, viewing `current`.
fn bar(current: Snapshot, manifest: Option<Manifest>) -> String {
    leptotes::render(move || {
        view! {
            <VersionBar
                current
                prefix=Prefix::new("/cqa-grader")
                manifest=Signal::stored(manifest.clone())
            />
        }
    })
}

/// Each choice's value, in the order the menu lists them.
fn choices(html: &str) -> Vec<&str> {
    html.split(r#"<option value=""#)
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect()
}

/// The notice, from where it opens to the end of the bar.
fn notice_of(html: &str) -> &str {
    html.split_once("pyle-version-notice")
        .map_or("", |(_, notice)| notice)
}

/// The words of the bar's one link.
fn link_words(html: &str) -> &str {
    html.split_once("<a ")
        .and_then(|(_, link)| link.split_once('>'))
        .and_then(|(_, rest)| rest.split_once("</a>"))
        .map_or("", |(words, _)| words)
}

#[test]
fn the_menu_lists_dev_then_each_release_in_the_manifest() {
    let html = bar(Snapshot::Dev, releases());
    assert_eq!(
        choices(&html),
        ["dev", "0.15.0", "0.14.0", "0.13.0"],
        "{html}"
    );
}

#[test]
fn the_build_being_viewed_is_the_one_selected_choice() {
    let html = bar(Snapshot::Release(V14), releases());
    assert_eq!(
        (
            leptotes::count(&html, r#"<option value="0.14.0" selected>v0.14.0</option>"#),
            leptotes::count(&html, " selected")
        ),
        (1, 1),
        "{html}"
    );
}

#[test]
fn the_bar_keeps_the_ids_a_site_s_tests_find_it_by() {
    let html = bar(Snapshot::Dev, releases());
    assert_eq!(
        (
            leptotes::count(&html, r#"<div id="version-switcher""#),
            leptotes::count(&html, r#"<select id="version-select""#)
        ),
        (1, 1),
        "{html}"
    );
}

#[test]
fn main_s_build_says_it_is_unreleased() {
    let html = bar(Snapshot::Dev, releases());
    assert_eq!(
        leptotes::count(notice_of(&html), DEFAULT_DEV_NOTICE),
        1,
        "{html}"
    );
}

#[test]
fn a_site_s_own_words_replace_the_dev_notice() {
    let html = leptotes::render(|| {
        view! {
            <VersionBar
                current=Snapshot::Dev
                prefix=Prefix::new("/")
                manifest=Signal::stored(None)
                dev_notice="The trace format may still change."
            />
        }
    });
    assert_eq!(
        (
            leptotes::count(notice_of(&html), "The trace format may still change."),
            leptotes::count(&html, DEFAULT_DEV_NOTICE)
        ),
        (1, 0),
        "{html}"
    );
}

#[test]
fn the_notice_links_to_the_newest_release_under_the_prefix_and_names_it() {
    let html = bar(Snapshot::Dev, releases());
    assert_eq!(
        (
            leptotes::count(&html, r#"href="/cqa-grader/v0.15.0/""#),
            leptotes::count(link_words(&html), "v0.15.0")
        ),
        (1, 1),
        "{html}"
    );
}

#[test]
fn main_s_build_before_the_first_release_links_nowhere() {
    let html = bar(Snapshot::Dev, Some(Manifest::default()));
    assert_eq!(
        (
            leptotes::count(&html, "pyle-version-notice"),
            leptotes::count(&html, "<a ")
        ),
        (1, 0),
        "{html}"
    );
}

#[test]
fn an_older_release_says_which_one_it_is_and_links_to_the_newest() {
    let html = bar(Snapshot::Release(V13), releases());
    let notice = notice_of(&html);
    assert_eq!(
        (
            leptotes::count(notice, "v0.13.0"),
            leptotes::count(notice, r#"href="/cqa-grader/v0.15.0/""#)
        ),
        (1, 1),
        "{html}"
    );
}

#[test]
fn the_dev_notice_is_the_site_s_words_and_nothing_else() {
    assert_eq!(
        message(
            &Notice::Unreleased { latest: Some(V15) },
            "APIs may change."
        ),
        "APIs may change."
    );
}

// A class renamed in the component and not in its rules leaves that part
// of the bar unstyled, and no other native test would say so. Main's build
// with a release to go to draws every part: the notice and its link too.
#[test]
fn every_class_the_bar_carries_has_a_rule_in_the_stylesheet() {
    let html = bar(Snapshot::Dev, releases());
    let css = crate::stylesheet(&crate::Theme::default());
    let classes: BTreeSet<&str> = html
        .split("class=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .flat_map(str::split_whitespace)
        .collect();
    let unruled: Vec<&str> = classes
        .iter()
        .copied()
        .filter(|class| !css.contains(&format!(".{class} {{")))
        .collect();
    assert_eq!(
        (classes.contains("pyle-version-link"), unruled),
        (true, Vec::new()),
        "{html}\n{css}"
    );
}

#[test]
fn the_newest_release_shows_no_notice() {
    let html = bar(Snapshot::Release(V15), releases());
    assert_eq!(leptotes::count(&html, "pyle-version-notice"), 0, "{html}");
}

#[test]
fn a_bar_with_no_classes_from_the_site_has_only_pyle_s() {
    let html = bar(Snapshot::Dev, None);
    assert_eq!(
        leptotes::count(&html, r#"class="pyle-version-bar""#),
        1,
        "{html}"
    );
}

#[test]
fn a_site_s_classes_follow_pyle_s_on_the_bar() {
    let html = leptotes::render(|| {
        view! {
            <VersionBar
                current=Snapshot::Dev
                prefix=Prefix::new("/")
                manifest=Signal::stored(None)
                class="sticky top-0"
            />
        }
    });
    assert_eq!(
        leptotes::count(&html, r#"class="pyle-version-bar sticky top-0""#),
        1,
        "{html}"
    );
}

#[test]
fn the_switcher_first_renders_dev_and_the_build_being_viewed() {
    let html = leptotes::render(|| {
        view! { <VersionSwitcher current=Snapshot::Release(V13) prefix=Prefix::new("/x") /> }
    });
    assert_eq!(choices(&html), ["dev", "0.13.0"], "{html}");
}

#[test]
fn the_switcher_hands_the_site_s_words_and_classes_to_the_bar() {
    let html = leptotes::render(|| {
        view! {
            <VersionSwitcher
                current=Snapshot::Dev
                prefix=Prefix::new("/x")
                dev_notice="APIs may change."
                class="sticky"
            />
        }
    });
    assert_eq!(
        (
            leptotes::count(notice_of(&html), "APIs may change."),
            leptotes::count(&html, r#"class="pyle-version-bar sticky""#)
        ),
        (1, 1),
        "{html}"
    );
}
