//! leptotes tests for the version bar: what its first render holds. What
//! happens after it (picking a version, the manifest arriving) needs a
//! browser.

use leptos::prelude::*;
use leptotes::Dom;
use pyle_core::{Manifest, Prefix, Snapshot, Version};

use super::{BAR, DEFAULT_DEV_NOTICE, NOTICE_WORDS, VersionBar, VersionSwitcher, message};
use crate::components::test_support::unruled_classes;
use crate::versions::Notice;

const V13: Version = Version::new(0, 13, 0);
const V14: Version = Version::new(0, 14, 0);
const V15: Version = Version::new(0, 15, 0);

fn releases() -> Option<Manifest> {
    Some(Manifest::new([V13, V14, V15]))
}

/// The bar of a site under `/cqa-grader`, viewing `current`.
fn bar(current: Snapshot, manifest: Option<Manifest>) -> Dom {
    leptotes::dom(move || {
        view! {
            <VersionBar
                current
                prefix=Prefix::new("/cqa-grader")
                manifest=Signal::stored(manifest.clone())
            />
        }
    })
}

#[test]
fn the_menu_lists_dev_then_each_release_in_the_manifest() {
    let dom = bar(Snapshot::Dev, releases());
    assert_eq!(
        dom.attrs("option", "value"),
        ["dev", "0.15.0", "0.14.0", "0.13.0"],
        "{dom}"
    );
}

#[test]
fn the_build_being_viewed_is_the_one_selected_choice() {
    let dom = bar(Snapshot::Release(V14), releases());
    assert_eq!(
        (
            dom.attrs("option[selected]", "value"),
            dom.texts("option[selected]")
        ),
        (vec!["0.14.0"], vec![String::from("v0.14.0")]),
        "{dom}"
    );
}

#[test]
fn the_bar_keeps_the_ids_a_site_s_tests_find_it_by() {
    let dom = bar(Snapshot::Dev, releases());
    assert_eq!(
        (
            dom.count("div#version-switcher"),
            dom.count("select#version-select")
        ),
        (1, 1),
        "{dom}"
    );
}

#[test]
fn main_s_build_says_it_is_unreleased() {
    let dom = bar(Snapshot::Dev, releases());
    assert_eq!(dom.find(NOTICE_WORDS).text(), DEFAULT_DEV_NOTICE, "{dom}");
}

#[test]
fn a_site_s_own_words_replace_the_dev_notice() {
    let dom = leptotes::dom(|| {
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
            dom.find(NOTICE_WORDS).text(),
            dom.find(BAR).text().contains(DEFAULT_DEV_NOTICE)
        ),
        (String::from("The trace format may still change."), false),
        "{dom}"
    );
}

#[test]
fn the_notice_links_to_the_newest_release_under_the_prefix_and_names_it() {
    let dom = bar(Snapshot::Dev, releases());
    let link = dom.find(".pyle-version-notice a");
    assert_eq!(
        (link.attr("href"), link.text().contains("v0.15.0")),
        (Some("/cqa-grader/v0.15.0/"), true),
        "{dom}"
    );
}

#[test]
fn main_s_build_before_the_first_release_links_nowhere() {
    let dom = bar(Snapshot::Dev, Some(Manifest::default()));
    assert_eq!(
        (dom.count(".pyle-version-notice"), dom.count("a")),
        (1, 0),
        "{dom}"
    );
}

#[test]
fn an_older_release_says_which_one_it_is_and_links_to_the_newest() {
    let dom = bar(Snapshot::Release(V13), releases());
    assert_eq!(
        (
            dom.find(NOTICE_WORDS).text().contains("v0.13.0"),
            dom.find(".pyle-version-notice a").attr("href")
        ),
        (true, Some("/cqa-grader/v0.15.0/")),
        "{dom}"
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

// Main's build with a release to go to draws every part of the bar: the
// notice and its link too.
#[test]
fn every_class_the_bar_carries_has_a_rule_in_the_stylesheet() {
    let dom = bar(Snapshot::Dev, releases());
    assert_eq!(
        (dom.count(".pyle-version-link"), unruled_classes(&dom)),
        (1, Vec::<String>::new()),
        "{dom}"
    );
}

#[test]
fn the_newest_release_shows_no_notice() {
    let dom = bar(Snapshot::Release(V15), releases());
    assert_eq!(dom.count(".pyle-version-notice"), 0, "{dom}");
}

#[test]
fn a_bar_with_no_classes_from_the_site_has_only_pyle_s() {
    let dom = bar(Snapshot::Dev, None);
    assert_eq!(
        dom.find(BAR).attr("class"),
        Some("pyle-version-bar"),
        "{dom}"
    );
}

#[test]
fn a_site_s_classes_follow_pyle_s_on_the_bar() {
    let dom = leptotes::dom(|| {
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
        dom.find(BAR).attr("class"),
        Some("pyle-version-bar sticky top-0"),
        "{dom}"
    );
}

#[test]
fn the_switcher_first_renders_dev_and_the_build_being_viewed() {
    let dom = leptotes::dom(|| {
        view! { <VersionSwitcher current=Snapshot::Release(V13) prefix=Prefix::new("/x") /> }
    });
    assert_eq!(dom.attrs("option", "value"), ["dev", "0.13.0"], "{dom}");
}

#[test]
fn the_switcher_hands_the_site_s_words_and_classes_to_the_bar() {
    let dom = leptotes::dom(|| {
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
        (dom.find(NOTICE_WORDS).text(), dom.find(BAR).attr("class")),
        (
            String::from("APIs may change."),
            Some("pyle-version-bar sticky")
        ),
        "{dom}"
    );
}
