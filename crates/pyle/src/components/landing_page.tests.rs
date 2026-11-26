//! leptotes tests for the landing page frame: what its first render holds.

use leptos::prelude::*;
use leptotes::Dom;

use super::LandingPage;
use crate::components::test_support::unruled_classes;

/// The page with two sections, marked so their order can be read.
fn page(class: &'static str) -> Dom {
    leptotes::dom(move || {
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
    let dom = page("");
    assert_eq!(dom.attrs("p", "id"), ["first", "second"], "{dom}");
}

#[test]
fn the_sections_sit_inside_the_frame() {
    let dom = page("");
    assert_eq!(dom.count(".pyle-page > p"), dom.count("p"), "{dom}");
}

#[test]
fn nothing_is_drawn_outside_the_frame() {
    let dom = page("");
    assert_eq!(
        dom.count("*"),
        1 + dom.find(".pyle-page").count("*"),
        "{dom}"
    );
}

#[test]
fn a_site_s_classes_follow_pyle_s_on_the_frame() {
    let dom = page("antialiased");
    assert_eq!(
        dom.find(".pyle-page").attr("class"),
        Some("pyle-page antialiased"),
        "{dom}"
    );
}

#[test]
fn every_class_the_frame_carries_has_a_rule_in_the_stylesheet() {
    let dom = page("");
    assert_eq!(unruled_classes(&dom), Vec::<String>::new(), "{dom}");
}
