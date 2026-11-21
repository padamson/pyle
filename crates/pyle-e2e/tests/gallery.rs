//! The theoria gallery built from pyle's stories, in Chromium: that every
//! story renders with pyle's styles, and how a story stands in for the
//! navigation a site would do.
//!
//! Every test needs the built gallery and Chromium, so each is `#[ignore]`d
//! and fails, naming the build command, when run without the gallery.
//! Install the browser once, build the gallery, then run:
//!   cargo run --manifest-path crates/pyle-e2e/Cargo.toml --example install-browsers
//!   ./scripts/gallery.sh build
//!   cargo nextest run --manifest-path crates/pyle-e2e/Cargo.toml --config-file .config/nextest.toml --run-ignored all

mod common;

use anyhow::{Context, Result, ensure};
use axum::Router;
use playwright_rs::protocol::Page;
use playwright_rs::{expect, expect_page};
use tower_http::services::ServeDir;

/// Runs `body` on a page showing the gallery, which `body` is given the
/// origin of. Fails when the gallery is not built.
async fn with_gallery(body: impl AsyncFnOnce(&Page, &str) -> Result<()>) -> Result<()> {
    let gallery = common::target_dir().join("theoria/gallery/pyle/dist");
    ensure!(
        gallery.join("index.html").exists(),
        "{} not built; run ./scripts/gallery.sh build",
        gallery.display()
    );
    let origin = common::serve(Router::new().fallback_service(ServeDir::new(&gallery))).await?;
    common::with_page("", async |page: &Page| {
        page.goto(&origin, None).await.context("open the gallery")?;
        // `goto` returns at `load`, which does not wait for the gallery to
        // mount, and a count of the sidebar's entries does not retry.
        expect(page.locator(".theoria-nav button").first())
            .to_be_visible()
            .await
            .context("the gallery mounts its sidebar")?;
        body(page, &origin).await
    })
    .await
}

/// Shows the story whose sidebar entry reads `leaf`, the part of its name
/// after the last `/`, and waits for the sidebar to mark it, so that what a
/// test reads next is not the story shown before.
async fn show(page: &Page, leaf: &str) -> Result<()> {
    page.locator(".theoria-nav")
        .get_by_text(leaf, true)
        .click(None)
        .await
        .with_context(|| format!("select the story {leaf:?}"))?;
    expect(page.locator(".theoria-nav button.active"))
        .to_have_text(leaf)
        .await
        .with_context(|| format!("the sidebar marks the story {leaf:?}"))
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn every_story_carries_pyle_s_styles_once() -> Result<()> {
    with_gallery(async |page: &Page, _| {
        let entries = page
            .locator(".theoria-nav button")
            .all()
            .await
            .context("list the stories")?;
        ensure!(!entries.is_empty(), "the gallery lists no stories");
        for (index, entry) in entries.iter().enumerate() {
            let name = entry.inner_text().await.context("read a story's name")?;
            entry
                .click(None)
                .await
                .with_context(|| format!("select story {index}, {name:?}"))?;
            expect(page.locator(".theoria-stage #pyle-styles"))
                .to_have_count(1)
                .await
                .with_context(|| format!("story {index}, {name:?}, carries pyle's styles"))?;
        }
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn a_story_draws_the_bar_of_the_release_a_visitor_picks_and_stays() -> Result<()> {
    with_gallery(async |page: &Page, origin: &str| {
        show(page, "On an older release").await?;
        page.locator("#version-select")
            .select_option("0.15.0", None)
            .await
            .context("pick 0.15.0")?;
        expect(page.locator(".story-lands"))
            .to_contain_text("/my-site/v0.15.0/")
            .await
            .context("the story says where the visitor would land")?;
        expect(page.locator("#version-select"))
            .to_have_value("0.15.0")
            .await
            .context("the story draws that release's bar")?;
        expect_page(page)
            .to_have_url(&format!("{origin}/"))
            .await
            .context("the browser stays in the gallery")
    })
    .await
}

/// The page story's bar is the same one: its notice links to an address
/// the gallery does not serve.
#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn following_the_notice_s_link_in_the_page_story_stays_in_the_gallery() -> Result<()> {
    with_gallery(async |page: &Page, origin: &str| {
        show(page, "A page from pyle's sections").await?;
        page.locator("#version-switcher a")
            .click(None)
            .await
            .context("follow the link to the newest release")?;
        expect(page.locator("#version-select"))
            .to_have_value("0.15.0")
            .await
            .context("the story draws the newest release's bar")?;
        expect_page(page)
            .to_have_url(&format!("{origin}/"))
            .await
            .context("the browser stays in the gallery")
    })
    .await
}
