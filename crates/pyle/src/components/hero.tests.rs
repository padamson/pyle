//! leptotes tests for the hero: what its first render holds.

use leptos::prelude::*;
use leptotes::Dom;

use super::Hero;
use crate::components::test_support::unruled_classes;

/// A hero with every part filled in.
fn full() -> Dom {
    leptotes::dom(|| {
        view! {
            <Hero title="pyle" tagline="Landing pages from components.">
                <a href="/source">"Source"</a>
            </Hero>
        }
    })
}

/// A hero with only its title.
fn bare() -> Dom {
    leptotes::dom(|| view! { <Hero title="pyle" /> })
}

#[test]
fn the_title_is_the_page_s_heading() {
    let dom = full();
    assert_eq!(dom.find("h1.pyle-hero-title").text(), "pyle", "{dom}");
}

#[test]
fn the_parts_come_title_then_tagline_then_the_row() {
    let dom = full();
    let parts: Vec<_> = dom
        .find_all(".pyle-hero-inner > *")
        .iter()
        .map(|part| part.attr("class"))
        .collect();
    assert_eq!(
        parts,
        [
            Some("pyle-hero-title"),
            Some("pyle-hero-tagline"),
            Some("pyle-hero-actions")
        ],
        "{dom}"
    );
}

#[test]
fn the_site_s_links_go_in_the_row_under_the_tagline() {
    let dom = full();
    assert_eq!(
        dom.attrs(".pyle-hero-actions a", "href"),
        ["/source"],
        "{dom}"
    );
}

#[test]
fn a_hero_with_only_a_title_draws_no_tagline_and_no_row() {
    let dom = bare();
    assert_eq!(
        (
            dom.count(".pyle-hero-tagline"),
            dom.count(".pyle-hero-actions")
        ),
        (0, 0),
        "{dom}"
    );
}

#[test]
fn a_site_s_classes_follow_pyle_s_on_the_section() {
    let dom = leptotes::dom(|| view! { <Hero title="pyle" class="tall" /> });
    assert_eq!(
        dom.find(":scope > section").attr("class"),
        Some("pyle-hero tall"),
        "{dom}"
    );
}

#[test]
fn every_class_the_hero_carries_has_a_rule_in_the_stylesheet() {
    let dom = full();
    assert_eq!(unruled_classes(&dom), Vec::<String>::new(), "{dom}");
}
