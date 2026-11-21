//! The cascade between a site's stylesheet and pyle's, in Chromium, on
//! static pages: the site's stylesheet, then pyle's (which arrives after
//! the site's, as `PyleStyles` does), then the version bar with a utility
//! class from the site on it, and a hero.
//!
//! The Tailwind cases read stylesheets built by Tailwind 4.3.5, the release
//! the sites use, and committed beside this file
//! (`./scripts/tailwind-fixtures.sh` rebuilds them). Each opens with
//! `@layer properties;`, as a site's does once it uses a utility such as a
//! shadow, so Tailwind's lowest layer is named before the site's line.
//! Needs Chromium; run:
//!   cargo nextest run --manifest-path crates/pyle-e2e/Cargo.toml --config-file .config/nextest.toml --run-ignored all

mod common;

use std::fs;

use anyhow::{Context, Result, ensure};
use leptos::prelude::*;
use playwright_rs::expect;
use playwright_rs::protocol::Page;
use pyle::{Hero, Prefix, Snapshot, Theme, Version, VersionBar};

/// The site's utility on the bar, in Tailwind's arbitrary-value form so
/// that its computed color is plain.
const UTILITY_CLASS: &str = "bg-[rgb(255,0,255)]";

/// The utility's color, as a computed style.
const UTILITY: &str = "rgb(255, 0, 255)";

/// The default theme's surface, as a computed style: what the bar has
/// behind it when no rule of the site's reaches it.
const PYLE_SURFACE: &str = "rgb(246, 247, 248)";

/// A site that writes its own `base` and `utilities` layers, as asbuilt
/// does, with the line naming pyle's place: a reset that zeroes every
/// element's border, and the same utility.
const OWN_LAYERS: &str = r"@layer base, pyle, utilities;
@layer base {
  *, ::after, ::before { box-sizing: border-box; border: 0 solid; margin: 0; padding: 0; }
}
@layer utilities {
  .bg-\[rgb\(255\,0\,255\)\] { background-color: rgb(255 0 255); }
}
";

/// A site setting one of pyle's variables outside any layer, which
/// outranks every layer.
const OWN_SURFACE: &str = ":root { --pyle-surface: rgb(0 128 128); }";

/// The surface that site sets, as a computed style.
const SITE_SURFACE: &str = "rgb(0, 128, 128)";

/// The stylesheet Tailwind built from `<name>.input.css`.
fn tailwind(name: &str) -> Result<String> {
    let path = format!("{}/tests/fixtures/{name}.css", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path)
        .with_context(|| format!("read {path}; run ./scripts/tailwind-fixtures.sh"))
}

/// A page holding `site_css`, then pyle's stylesheet, then the bar with
/// `class` on it and a hero with a link.
fn page_with(site_css: &str, class: &'static str) -> String {
    let sections = leptotes::render(move || {
        view! {
            <VersionBar
                current=Snapshot::Release(Version::new(0, 14, 0))
                prefix=Prefix::new("/my-site")
                manifest=Signal::stored(None)
                class
            />
            <Hero title="Project name">
                <a href="#source">"Source"</a>
            </Hero>
        }
    });
    format!(
        "<!doctype html><style>{site_css}</style><style id=\"pyle-styles\">{}</style>{sections}",
        pyle::stylesheet(&Theme::default())
    )
}

/// How wide the element `selector` finds is drawn, padding and border
/// included, in CSS pixels.
async fn drawn_width(page: &Page, selector: &str) -> Result<f64> {
    page.locator(selector)
        .evaluate("(el) => el.getBoundingClientRect().width", None::<()>)
        .await
        .with_context(|| format!("measure {selector}"))
}

/// Runs `body` on a page set to `html`.
async fn on(html: String, body: impl AsyncFnOnce(&Page) -> Result<()>) -> Result<()> {
    common::with_page("", async |page: &Page| {
        page.set_content(&html, None)
            .await
            .context("set the page's content")?;
        body(page).await
    })
    .await
}

#[tokio::test]
#[ignore = "needs Chromium; run with --run-ignored all"]
async fn a_tailwind_site_s_utility_outranks_pyle_once_it_names_the_order() -> Result<()> {
    let html = page_with(&tailwind("tailwind-ordered")?, UTILITY_CLASS);
    on(html, async |page: &Page| {
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", UTILITY)
            .await
            .context("the bar takes the utility's background")
    })
    .await
}

#[tokio::test]
#[ignore = "needs Chromium; run with --run-ignored all"]
async fn tailwind_s_reset_leaves_the_menu_its_border() -> Result<()> {
    let html = page_with(&tailwind("tailwind-ordered")?, UTILITY_CLASS);
    on(html, async |page: &Page| {
        expect(page.locator("#version-select"))
            .to_have_css("border-top-width", "1px")
            .await
            .context("the menu keeps pyle's border over the reset's zero")
    })
    .await
}

// Tailwind's reset has links inherit their decoration, which is none.
#[tokio::test]
#[ignore = "needs Chromium; run with --run-ignored all"]
async fn tailwind_s_reset_leaves_the_hero_s_links_underlined() -> Result<()> {
    let html = page_with(&tailwind("tailwind-ordered")?, "");
    on(html, async |page: &Page| {
        expect(page.locator(".pyle-hero-actions a"))
            .to_have_css("text-decoration-line", "underline")
            .await
            .context("the link keeps pyle's underline over the reset's none")
    })
    .await
}

/// A section's column is 64rem with its padding inside, whichever way the
/// site's reset has boxes sized. This page has no reset, so the browser's
/// own sizing, which would put the padding outside, is what pyle's rule is
/// up against. The window is wider than the column.
#[tokio::test]
#[ignore = "needs Chromium; run with --run-ignored all"]
async fn a_section_s_column_holds_its_padding_on_a_site_with_no_reset() -> Result<()> {
    on(page_with("", ""), async |page: &Page| {
        for column in [".pyle-version-bar-inner", ".pyle-hero-inner"] {
            let width = drawn_width(page, column).await?;
            ensure!(
                (width - 1024.0).abs() < 0.5,
                "{column} is {width}px wide, not 64rem (1024px)"
            );
        }
        Ok(())
    })
    .await
}

/// Why the site has to name the order: pyle's stylesheet arrives after the
/// site's, and a layer named later outranks one named earlier.
#[tokio::test]
#[ignore = "needs Chromium; run with --run-ignored all"]
async fn with_no_order_named_pyle_outranks_a_tailwind_utility() -> Result<()> {
    let html = page_with(&tailwind("tailwind-unordered")?, UTILITY_CLASS);
    on(html, async |page: &Page| {
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", PYLE_SURFACE)
            .await
            .context("the bar keeps the default theme's surface")
    })
    .await
}

#[tokio::test]
#[ignore = "needs Chromium; run with --run-ignored all"]
async fn a_site_with_its_own_layers_ranks_pyle_between_them() -> Result<()> {
    on(page_with(OWN_LAYERS, UTILITY_CLASS), async |page: &Page| {
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", UTILITY)
            .await
            .context("the utility outranks pyle")?;
        expect(page.locator("#version-select"))
            .to_have_css("border-top-width", "1px")
            .await
            .context("pyle outranks the reset")
    })
    .await
}

#[tokio::test]
#[ignore = "needs Chromium; run with --run-ignored all"]
async fn a_site_s_own_value_for_a_pyle_variable_replaces_the_theme_s() -> Result<()> {
    on(page_with(OWN_SURFACE, ""), async |page: &Page| {
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", SITE_SURFACE)
            .await
            .context("the bar draws on the surface the site set")
    })
    .await
}
