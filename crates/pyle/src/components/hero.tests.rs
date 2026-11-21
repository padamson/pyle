//! leptotes tests for the hero: what its first render holds.

use leptos::prelude::*;

use super::Hero;
use crate::components::test_support::unruled_classes;

/// A hero with every part filled in.
fn full() -> String {
    leptotes::render(|| {
        view! {
            <Hero title="pyle" tagline="Landing pages from components.">
                <a href="/source">"Source"</a>
            </Hero>
        }
    })
}

/// A hero with only its title.
fn bare() -> String {
    leptotes::render(|| view! { <Hero title="pyle" /> })
}

#[test]
fn the_title_is_the_page_s_heading() {
    let html = full();
    assert_eq!(
        leptotes::count(&html, r#"<h1 class="pyle-hero-title">pyle</h1>"#),
        1,
        "{html}"
    );
}

#[test]
fn the_tagline_follows_the_title() {
    let html = full();
    let title = html.find("<h1");
    let tagline = html.find("Landing pages from components.");
    assert!(title.is_some() && title < tagline, "{html}");
}

#[test]
fn the_site_s_links_go_in_the_row_under_the_tagline() {
    let html = full();
    let row = html
        .split_once(r#"class="pyle-hero-actions""#)
        .map(|(_, row)| row);
    assert_eq!(
        row.map(|row| leptotes::count(row, r#"href="/source""#)),
        Some(1),
        "{html}"
    );
}

#[test]
fn a_hero_with_only_a_title_draws_no_tagline_and_no_row() {
    let html = bare();
    assert_eq!(
        (
            leptotes::count(&html, "pyle-hero-tagline"),
            leptotes::count(&html, "pyle-hero-actions")
        ),
        (0, 0),
        "{html}"
    );
}

#[test]
fn a_site_s_classes_follow_pyle_s_on_the_section() {
    let html = leptotes::render(|| view! { <Hero title="pyle" class="tall" /> });
    assert_eq!(
        leptotes::count(&html, r#"<section class="pyle-hero tall">"#),
        1,
        "{html}"
    );
}

#[test]
fn every_class_the_hero_carries_has_a_rule_in_the_stylesheet() {
    let html = full();
    assert_eq!(unruled_classes(&html), Vec::<String>::new(), "{html}");
}
