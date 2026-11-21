//! leptotes tests for the landing page frame: what its first render holds.

use leptos::prelude::*;

use super::LandingPage;
use crate::components::test_support::unruled_classes;

/// The page with two sections, marked so their order can be read.
fn page(class: &'static str) -> String {
    leptotes::render(move || {
        view! {
            <LandingPage class>
                <p id="first">"one"</p>
                <p id="second">"two"</p>
            </LandingPage>
        }
    })
}

#[test]
fn the_sections_appear_in_the_order_the_site_gives_them() {
    let html = page("");
    let first = html.find(r#"id="first""#);
    let second = html.find(r#"id="second""#);
    assert!(first.is_some() && first < second, "{html}");
}

#[test]
fn the_sections_sit_inside_the_frame() {
    let html = page("");
    assert!(
        html.starts_with(r#"<div class="pyle-page">"#) && html.ends_with("</div>"),
        "{html}"
    );
}

#[test]
fn a_site_s_classes_follow_pyle_s_on_the_frame() {
    let html = page("antialiased");
    assert_eq!(
        leptotes::count(&html, r#"class="pyle-page antialiased""#),
        1,
        "{html}"
    );
}

#[test]
fn every_class_the_frame_carries_has_a_rule_in_the_stylesheet() {
    let html = page("");
    assert_eq!(unruled_classes(&html), Vec::<String>::new(), "{html}");
}
