//! The theoria gallery built from pyle's stories, in Chromium: that every
//! story renders with pyle's styles, how a story stands in for the
//! navigation a site would do, and what an edit in a playground redraws.
//! A site sets its theme and the bar's classes once, so only a playground
//! changes them on an open page, and only the gallery shows it.
//!
//! Every test needs the built gallery and Chromium, so each is `#[ignore]`d
//! and fails, naming the build command, when run without the gallery.
//! Install the browser once, build the gallery, then run:
//!   cargo run --manifest-path crates/pyle-e2e/Cargo.toml --example install-browsers
//!   cargo theoria build
//!   cargo nextest run --manifest-path crates/pyle-e2e/Cargo.toml --config-file .config/nextest.toml --run-ignored all

mod common;

use anyhow::{Context, Result, ensure};
use axum::Router;
use playwright_rs::protocol::{Locator, Page};
use playwright_rs::{expect, expect_page};
use tower_http::services::ServeDir;

/// Runs `body` on a page showing the gallery, which `body` is given the
/// origin of. Fails when the gallery is not built.
async fn with_gallery(body: impl AsyncFnOnce(&Page, &str) -> Result<()>) -> Result<()> {
    let gallery = common::target_dir().join("theoria/gallery/pyle/dist");
    ensure!(
        gallery.join("index.html").exists(),
        "{} not built; run cargo theoria build",
        gallery.display()
    );
    let origin = common::serve(Router::new().fallback_service(ServeDir::new(&gallery))).await?;
    common::with_page("", async |page: &Page| {
        page.goto(&origin, None).await.context("open the gallery")?;
        // `goto` returns at `load`, which does not wait for the gallery to
        // mount, and a count of the sidebar's entries does not retry.
        expect(page.locator(".theoria-nav a").first())
            .to_be_visible()
            .await
            .context("the gallery mounts its sidebar")?;
        body(page, &origin).await
    })
    .await
}

/// Opens the story named `name`, its full name with every `/`, from its
/// sidebar entry, and waits for the sidebar to mark it, so that what a test
/// reads next is that story. Returns the story's link, which the gallery
/// keeps in the address bar while the story is shown. The only place a test
/// opens a story by name.
async fn show(page: &Page, origin: &str, name: &str) -> Result<String> {
    // The link as the gallery writes it, form-encoded but with each `/`
    // left as it is, so it picks out the one entry whose `href` it is.
    let encoded: String = form_urlencoded::byte_serialize(name.as_bytes()).collect();
    let query = format!("?story={}", encoded.replace("%2F", "/"));
    page.locator(format!(r#".theoria-nav a[href="{query}"]"#))
        .click(None)
        .await
        .with_context(|| format!("open the story {name:?}"))?;
    expect(page.locator(".theoria-nav [aria-current=page]"))
        .to_have_attribute("href", &query)
        .await
        .with_context(|| format!("the sidebar marks the story {name:?}"))?;
    Ok(format!("{origin}/{query}"))
}

/// The text box of the playground's control for the arg `name`.
fn control(page: &Page, name: &str) -> Locator {
    page.locator(format!(
        r#".theoria-controls input[type=text][aria-label="{name}"]"#
    ))
}

/// Picks `version` in the bar's menu and waits for the story to draw that
/// release's bar.
async fn pick(page: &Page, version: &str) -> Result<()> {
    page.locator("#version-select")
        .select_option(version, None)
        .await
        .with_context(|| format!("pick {version}"))?;
    expect(page.locator(".story-lands"))
        .to_contain_text(&format!("/my-site/v{version}/"))
        .await
        .with_context(|| format!("the story draws {version}'s bar"))
}

/// Waits for the next frame and the task after it, so that a change the
/// last edit set off for later has been made.
async fn after_the_next_frame(page: &Page) -> Result<()> {
    page.evaluate::<(), ()>(
        "new Promise((done) => requestAnimationFrame(() => setTimeout(done)))",
        None,
    )
    .await
    .context("wait for the next frame")
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after cargo theoria build"]
async fn every_story_carries_pyle_s_styles_once() -> Result<()> {
    with_gallery(async |page: &Page, _| {
        let entries = page
            .locator(".theoria-nav a")
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
            expect(entry.clone())
                .to_have_attribute("aria-current", "page")
                .await
                .with_context(|| format!("the sidebar marks story {index}, {name:?}"))?;
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
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after cargo theoria build"]
async fn a_story_draws_the_bar_of_the_release_a_visitor_picks_and_stays() -> Result<()> {
    with_gallery(async |page: &Page, origin: &str| {
        let link = show(page, origin, "Version bar/On an older release").await?;
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
            .to_have_url(&link)
            .await
            .context("the browser stays in the gallery")
    })
    .await
}

/// The page story's bar is the same one: its notice links to an address
/// the gallery does not serve.
#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after cargo theoria build"]
async fn following_the_notice_s_link_in_the_page_story_stays_in_the_gallery() -> Result<()> {
    with_gallery(async |page: &Page, origin: &str| {
        let link = show(page, origin, "Landing page/A page from pyle's sections").await?;
        page.locator("#version-switcher a")
            .click(None)
            .await
            .context("follow the link to the newest release")?;
        expect(page.locator("#version-select"))
            .to_have_value("0.15.0")
            .await
            .context("the story draws the newest release's bar")?;
        expect_page(page)
            .to_have_url(&link)
            .await
            .context("the browser stays in the gallery")
    })
    .await
}

/// Records, in `window.__redrawn`, each element of the stage whose children
/// or text change, other than the stylesheet and the theme's Rust above the
/// page.
const WATCH_FOR_A_REDRAW: &str = r"
    window.__redrawn = [];
    new MutationObserver((records) => {
        for (const record of records) {
            const node = record.target.nodeType === Node.ELEMENT_NODE
                ? record.target
                : record.target.parentElement;
            if (!node) {
                window.__redrawn.push('text taken out of the page');
            } else if (!node.closest('#pyle-styles, #theme-source')) {
                window.__redrawn.push(node.outerHTML.slice(0, 120));
            }
        }
    }).observe(document.querySelector('.theoria-stage'), {
        childList: true,
        characterData: true,
        subtree: true,
    });
";

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after cargo theoria build"]
async fn an_edit_to_the_theme_writes_the_stylesheet_again_and_redraws_nothing_else() -> Result<()> {
    with_gallery(async |page: &Page, origin: &str| {
        show(page, origin, "Theme/Playground").await?;
        page.evaluate_expression(WATCH_FOR_A_REDRAW)
            .await
            .context("watch the stage")?;
        control(page, "accent")
            .fill("#ff0000", None)
            .await
            .context("make the accent red")?;
        expect(page.locator(".theoria-stage #pyle-styles"))
            .to_contain_text("#ff0000")
            .await
            .context("the stylesheet is written again with the new accent")?;
        after_the_next_frame(page).await?;
        let redrawn: Vec<String> = page
            .evaluate("window.__redrawn", None::<&()>)
            .await
            .context("read what the observer saw")?;
        ensure!(redrawn.is_empty(), "the edit redrew {redrawn:#?}");
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after cargo theoria build"]
async fn an_edit_to_the_bar_s_classes_keeps_the_version_picked_in_it() -> Result<()> {
    with_gallery(async |page: &Page, origin: &str| {
        show(page, origin, "Version bar/Playground").await?;
        pick(page, "0.15.0").await?;
        control(page, "class")
            .fill("sticky", None)
            .await
            .context("give the bar a class")?;
        expect(page.locator(".theoria-stage .pyle-version-bar.sticky"))
            .to_be_visible()
            .await
            .context("the bar is drawn again with the class")?;
        expect(page.locator("#version-select"))
            .to_have_value("0.15.0")
            .await
            .context("the bar drawn again is 0.15.0's")
    })
    .await
}

/// The notice is drawn only on main's build, so on 0.15.0 the edit changes
/// nothing to wait on; the test waits a frame instead.
#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after cargo theoria build"]
async fn an_edit_to_the_bar_s_notice_keeps_the_version_picked_in_it() -> Result<()> {
    with_gallery(async |page: &Page, origin: &str| {
        show(page, origin, "Version bar/Playground").await?;
        pick(page, "0.15.0").await?;
        control(page, "dev_notice")
            .fill("Not released yet.", None)
            .await
            .context("give the bar other words for main's build")?;
        after_the_next_frame(page).await?;
        expect(page.locator("#version-select"))
            .to_have_value("0.15.0")
            .await
            .context("the bar is still 0.15.0's")
    })
    .await
}
