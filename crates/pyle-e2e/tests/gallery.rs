//! pyle's browser tests: what no native test can see. They serve the
//! theoria gallery built from pyle's stories, beside a site that pyle-cli
//! deploys for the test, and drive both in Chromium with playwright-rs.
//!
//! Every test needs the built gallery and Chromium, so each is `#[ignore]`d
//! and fails, naming the build command, when run without the gallery.
//! Install the browser once, build the gallery, then run:
//!   cargo run --manifest-path crates/pyle-e2e/Cargo.toml --example install-browsers
//!   ./scripts/gallery.sh build
//!   cargo nextest run --manifest-path crates/pyle-e2e/Cargo.toml --config-file .config/nextest.toml --run-ignored all

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use axum::Router;
use playwright_rs::protocol::{ColorScheme, EmulateMediaOptions, Page, Playwright};
use playwright_rs::{expect, expect_page};
use pyle_core::{Prefix, Snapshot};
use tempfile::TempDir;
use tower_http::services::ServeDir;

/// Where the stories' site is served. The `Version switcher` story reads
/// its manifest from under this prefix, viewing 0.14.0.
const PREFIX: &str = "/my-site";

/// The snapshots the test deploys, so the story's 0.14.0 is neither the
/// oldest nor the newest.
const SNAPSHOTS: [&str; 4] = ["0.13.0", "0.14.0", "0.15.0", "dev"];

/// The utility's color in the layering stories, as a computed style.
const UTILITY: &str = "rgb(255, 0, 255)";

/// The default theme's light surface (`#f6f7f8` in `pyle::Theme::default`),
/// as a computed style: what the bar has behind it when no site rule
/// reaches it. Playwright's pages ask for light.
const PYLE_SURFACE: &str = "rgb(246, 247, 248)";

/// The default theme's dark surface (`#161b22`), the same way.
const PYLE_DARK_SURFACE: &str = "rgb(22, 27, 34)";

/// The surface the story's site sets through `--pyle-surface`.
const SITE_SURFACE: &str = "rgb(0, 128, 128)";

fn gallery_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/theoria/gallery/pyle/dist")
}

/// A site deployed the way a site's workflow does it: each snapshot added
/// with pyle-cli, which writes `versions.json` and the root redirect. Each
/// snapshot's page names it in `#snapshot`.
fn deploy_site() -> Result<TempDir> {
    let site = TempDir::new().context("create the site folder")?;
    let build = TempDir::new().context("create the build folder")?;
    for name in SNAPSHOTS {
        let snapshot: Snapshot = name.parse()?;
        fs::write(
            build.path().join(pyle_cli::INDEX_FILE),
            format!(r#"<!doctype html><title>{name}</title><h1 id="snapshot">{name}</h1>"#),
        )
        .context("write the build's page")?;
        pyle_cli::add(build.path(), site.path(), &snapshot)?;
    }
    pyle_cli::update(site.path(), &Prefix::new(PREFIX), "fixture")?;
    Ok(site)
}

/// Runs `body` on a page showing the gallery, with the deployed site
/// served under [`PREFIX`] of the same origin, which `body` is also given.
/// `init` runs in the page before any of its own scripts.
///
/// Fails when the gallery is not built, and closes the browser whether or
/// not `body` passed.
async fn with_gallery(init: &str, body: impl AsyncFnOnce(&Page, &str) -> Result<()>) -> Result<()> {
    let gallery = gallery_dir();
    ensure!(
        gallery.join("index.html").exists(),
        "{} not built; run ./scripts/gallery.sh build",
        gallery.display()
    );
    let site = deploy_site()?;

    let app = Router::new()
        .nest_service(PREFIX, ServeDir::new(site.path()))
        .fallback_service(ServeDir::new(&gallery));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind the server")?;
    let origin = format!("http://{}", listener.local_addr().context("local addr")?);
    let _server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve the gallery");
    });

    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let result = async {
        let page = browser.new_page().await.context("new page")?;
        if !init.is_empty() {
            page.add_init_script(init)
                .await
                .context("add the init script")?;
        }
        page.goto(&origin, None).await.context("open the gallery")?;
        // `goto` returns at `load`, which does not wait for the gallery to
        // mount, and a count of the sidebar's entries does not retry.
        expect(page.locator(".theoria-nav button").first())
            .to_be_visible()
            .await
            .context("the gallery mounts its sidebar")?;
        body(&page, &origin).await
    }
    .await;
    let closed = browser.close().await.context("close browser");
    result.and(closed)
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

/// Shows the story whose bar fetches the deployed site's manifest, and
/// waits for the fetch: the menu holds two choices until it lands (dev and
/// the build being viewed) and four after.
async fn show_with_manifest(page: &Page) -> Result<()> {
    show(page, "Reading the site's manifest").await?;
    expect(page.locator("#version-select option"))
        .to_have_count(4)
        .await
        .context("the manifest's releases join the menu")
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn every_story_shows_one_bar_with_pyle_s_styles() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
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
            expect(page.locator(".theoria-stage #version-switcher"))
                .to_have_count(1)
                .await
                .with_context(|| format!("story {index}, {name:?}, shows one bar"))?;
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
async fn a_site_s_utility_outranks_pyle_once_the_site_names_the_order() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "A Tailwind site that names the order").await?;
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", UTILITY)
            .await
            .context("the bar takes the utility's background")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn a_site_s_reset_leaves_the_menu_its_border() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "A Tailwind site that names the order").await?;
        expect(page.locator("#version-select"))
            .to_have_css("border-top-width", "1px")
            .await
            .context("the menu keeps pyle's border over the reset's zero")
    })
    .await
}

/// Why the site has to name the order: pyle's stylesheet arrives after the
/// site's, and a layer declared later outranks one declared earlier.
#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn with_no_order_named_pyle_s_rule_beats_the_utility() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "A Tailwind site that names no order").await?;
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", PYLE_SURFACE)
            .await
            .context("the bar keeps the default theme's light surface")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn a_site_s_own_value_for_a_pyle_variable_replaces_the_theme_s() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "A site that sets one of pyle's variables").await?;
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", SITE_SURFACE)
            .await
            .context("the bar draws on the surface the site set outside any layer")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn a_visitor_who_asks_for_dark_gets_the_dark_palette() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "On the newest release").await?;
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", PYLE_SURFACE)
            .await
            .context("the bar starts on the light surface")?;
        page.emulate_media(
            EmulateMediaOptions::builder()
                .color_scheme(ColorScheme::Dark)
                .build(),
        )
        .await
        .context("ask for dark")?;
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", PYLE_DARK_SURFACE)
            .await
            .context("the bar moves to the default theme's dark surface")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn a_site_that_handles_the_pick_itself_is_told_and_the_page_stays() -> Result<()> {
    with_gallery("", async |page: &Page, origin: &str| {
        show(page, "On an older release").await?;
        page.locator("#version-select")
            .select_option("0.15.0", None)
            .await
            .context("pick 0.15.0")?;
        expect(page.locator(".story-lands"))
            .to_contain_text("/my-site/v0.15.0/")
            .await
            .context("the story's callback is handed the pick")?;
        expect_page(page)
            .to_have_url(&format!("{origin}/"))
            .await
            .context("the browser stays in the gallery")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn a_site_with_its_own_layers_ranks_pyle_between_them() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "A site with its own layers that names the order").await?;
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
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn the_bar_lists_the_releases_a_deploy_wrote() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show_with_manifest(page).await?;
        let labels = page
            .locator("#version-select option")
            .all_text_contents()
            .await
            .context("read the menu")?;
        ensure!(
            labels == ["dev (main)", "v0.15.0", "v0.14.0", "v0.13.0"],
            "the menu lists {labels:?}"
        );
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn the_build_being_viewed_stays_picked_when_the_manifest_arrives() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show_with_manifest(page).await?;
        expect(page.locator("#version-select"))
            .to_have_value("0.14.0")
            .await
            .context("the menu still shows the build being viewed")
    })
    .await
}

/// A page the browser brings back from its back/forward cache is as the
/// visitor left it, with the version they went to still picked. Playwright
/// runs Chromium with that cache off, so the test leaves the menu on another
/// version the way a restored page would be, and sends the event a restore
/// sends.
#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn a_page_the_browser_brings_back_shows_its_own_build_again() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "On the newest release").await?;
        let menu = page.locator("#version-select");
        expect(menu.clone())
            .to_have_value("0.15.0")
            .await
            .context("the menu starts on the build being viewed")?;
        // Setting the value from a script fires no `change`, so nothing
        // navigates.
        let left_on: String = menu
            .evaluate(
                "(menu) => { menu.value = '0.13.0'; return menu.value; }",
                None::<()>,
            )
            .await
            .context("leave the menu on 0.13.0")?;
        ensure!(left_on == "0.13.0", "the menu was left on {left_on:?}");
        page.evaluate_expression(
            "window.dispatchEvent(new PageTransitionEvent('pageshow', { persisted: true }))",
        )
        .await
        .context("send the event a restore sends")?;
        expect(menu)
            .to_have_value("0.15.0")
            .await
            .context("the menu is back on the build being viewed")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn an_older_release_links_to_the_newest_one_deployed() -> Result<()> {
    with_gallery("", async |page: &Page, _| {
        show(page, "Reading the site's manifest").await?;
        expect(page.locator("#version-switcher a"))
            .to_have_attribute("href", "/my-site/v0.15.0/")
            .await
            .context("the notice links to 0.15.0 under the prefix")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn picking_a_release_opens_its_snapshot() -> Result<()> {
    with_gallery("", async |page: &Page, origin: &str| {
        show_with_manifest(page).await?;
        page.locator("#version-select")
            .select_option("0.13.0", None)
            .await
            .context("pick 0.13.0")?;
        expect_page(page)
            .to_have_url(&format!("{origin}/my-site/v0.13.0/"))
            .await
            .context("the browser goes to the snapshot's folder")?;
        expect(page.locator("#snapshot"))
            .to_have_text("0.13.0")
            .await
            .context("the snapshot deployed as 0.13.0 is the page shown")
    })
    .await
}

/// Records, each time the page's elements change, whether the bar is in
/// the page while pyle's rules are not. A change the browser could paint
/// between is one this sees.
const WATCH_FOR_AN_UNSTYLED_BAR: &str = r"
    window.__pyleBarUnstyled = false;
    new MutationObserver(() => {
        const styles = document.getElementById('pyle-styles');
        const styled = styles && styles.textContent.includes('.pyle-version-bar');
        if (document.getElementById('version-switcher') && !styled) {
            window.__pyleBarUnstyled = true;
        }
    }).observe(document, { childList: true, subtree: true });
";

#[tokio::test]
#[ignore = "needs the built gallery and Chromium; run with --run-ignored all after ./scripts/gallery.sh build"]
async fn the_bar_is_never_in_the_page_without_its_styles() -> Result<()> {
    with_gallery(WATCH_FOR_AN_UNSTYLED_BAR, async |page: &Page, _| {
        expect(page.locator("#version-switcher"))
            .to_have_count(1)
            .await
            .context("the first story shows the bar")?;
        show_with_manifest(page).await?;
        let unstyled: bool = page
            .evaluate("window.__pyleBarUnstyled", None::<&()>)
            .await
            .context("read what the observer saw")?;
        ensure!(!unstyled, "the bar was in the page before pyle's styles");
        Ok(())
    })
    .await
}
