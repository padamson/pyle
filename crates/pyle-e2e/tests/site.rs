//! pyle's own landing page in Chromium: the snapshots `scripts/site.sh`
//! builds, deployed with pyle-cli the way a site's workflow does it, and
//! served under `/pyle/` as on GitHub Pages.
//!
//! Every test needs the built snapshots and Chromium, so each is
//! `#[ignore]`d and fails, naming the build command, when run without
//! them. Install the browser once, build the site, then run:
//!   cargo run --manifest-path crates/pyle-e2e/Cargo.toml --example install-browsers
//!   ./scripts/site.sh
//!   cargo nextest run --manifest-path crates/pyle-e2e/Cargo.toml --config-file .config/nextest.toml --run-ignored all

mod common;

use anyhow::{Context, Result, ensure};
use axum::Router;
use playwright_rs::protocol::{ColorScheme, EmulateMediaOptions, Page};
use playwright_rs::{expect, expect_page};
use pyle::{DEFAULT_DEV_NOTICE, Prefix, Snapshot};
use tempfile::TempDir;
use tower_http::services::ServeDir;

/// Where the site is served, as `crates/site` says.
const PREFIX: &str = "/pyle";

/// Each snapshot `scripts/site.sh` builds, as its folder and the snapshot
/// it is: two releases, so one is older, and main's build.
const SNAPSHOTS: [(&str, &str); 3] = [("v0.1.0", "0.1.0"), ("v0.2.0", "0.2.0"), ("dev", "dev")];

/// How many choices the menu holds once the manifest has arrived: every
/// snapshot deployed.
const DEPLOYED: usize = SNAPSHOTS.len();

/// The default theme's colors, as computed styles: the site mounts
/// `PyleStyles` with no theme. Playwright's pages ask for light.
const PAGE: &str = "rgb(255, 255, 255)";
const SURFACE: &str = "rgb(246, 247, 248)";
const DARK_PAGE: &str = "rgb(13, 17, 23)";
const DARK_SURFACE: &str = "rgb(22, 27, 34)";

/// The site, deployed into a fresh folder with pyle-cli, which writes
/// `versions.json` and the root redirect.
fn deploy() -> Result<TempDir> {
    let site = TempDir::new().context("create the site folder")?;
    for (folder, snapshot) in SNAPSHOTS {
        let build = common::target_dir().join("site").join(folder);
        ensure!(
            build.join("index.html").exists(),
            "{} not built; run ./scripts/site.sh",
            build.display()
        );
        let snapshot: Snapshot = snapshot.parse()?;
        pyle_cli::add(&build, site.path(), &snapshot)?;
    }
    pyle_cli::update(site.path(), &Prefix::new(PREFIX), "pyle")?;
    Ok(site)
}

/// The site, deployed and served under [`PREFIX`]: its folder, which must
/// outlive the test, and the server's origin.
async fn serve_site() -> Result<(TempDir, String)> {
    let site = deploy()?;
    let origin =
        common::serve(Router::new().nest_service(PREFIX, ServeDir::new(site.path()))).await?;
    Ok((site, origin))
}

/// Runs `body` on the deployed site opened at `path`, once the version bar
/// is in the page. `body` is also given the server's origin. `init` runs in
/// the page before any of its own scripts.
async fn with_site(
    path: &str,
    init: &str,
    body: impl AsyncFnOnce(&Page, &str) -> Result<()>,
) -> Result<()> {
    let (_site, origin) = serve_site().await?;
    let url = format!("{origin}{path}");
    common::with_page(init, async |page: &Page| {
        page.goto(&url, None)
            .await
            .with_context(|| format!("open {url}"))?;
        expect(page.locator("#version-switcher"))
            .to_be_visible()
            .await
            .context("the page mounts the version bar")?;
        body(page, &origin).await
    })
    .await
}

/// Waits for the bar to read the deployed manifest: until then the menu
/// lists only main's build and the one being viewed.
async fn manifest_read(page: &Page) -> Result<()> {
    expect(page.locator("#version-select option"))
        .to_have_count(DEPLOYED)
        .await
        .context("every deployed snapshot joins the menu")
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_site_s_root_sends_a_visitor_to_the_newest_release() -> Result<()> {
    let (_site, origin) = serve_site().await?;
    common::with_page("", async |page: &Page| {
        page.goto(&format!("{origin}{PREFIX}/"), None)
            .await
            .context("open the site's root")?;
        expect_page(page)
            .to_have_url(&format!("{origin}{PREFIX}/v0.2.0/"))
            .await
            .context("the redirect lands on v0.2.0")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_bar_lists_every_snapshot_the_deploy_holds() -> Result<()> {
    with_site("/pyle/dev/", "", async |page: &Page, _| {
        manifest_read(page).await?;
        let labels = page
            .locator("#version-select option")
            .all_text_contents()
            .await
            .context("read the menu")?;
        ensure!(
            labels == ["dev (main)", "v0.2.0", "v0.1.0"],
            "the menu lists {labels:?}"
        );
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn main_s_build_says_it_is_unreleased() -> Result<()> {
    with_site("/pyle/dev/", "", async |page: &Page, _| {
        expect(page.locator(".pyle-version-notice"))
            .to_contain_text(DEFAULT_DEV_NOTICE)
            .await
            .context("the bar says main's build is unreleased")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn main_s_build_links_to_the_newest_release() -> Result<()> {
    with_site("/pyle/dev/", "", async |page: &Page, _| {
        expect(page.locator("#version-switcher a"))
            .to_have_attribute("href", "/pyle/v0.2.0/")
            .await
            .context("the notice links to v0.2.0 under the prefix")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn an_older_release_links_to_the_newest_one() -> Result<()> {
    with_site("/pyle/v0.1.0/", "", async |page: &Page, _| {
        expect(page.locator("#version-switcher a"))
            .to_have_attribute("href", "/pyle/v0.2.0/")
            .await
            .context("the notice links to v0.2.0")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_newest_release_shows_no_notice() -> Result<()> {
    with_site("/pyle/v0.2.0/", "", async |page: &Page, _| {
        manifest_read(page).await?;
        expect(page.locator(".pyle-version-notice"))
            .to_have_count(0)
            .await
            .context("no notice on the newest release")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn picking_a_release_opens_its_snapshot() -> Result<()> {
    with_site("/pyle/dev/", "", async |page: &Page, origin: &str| {
        manifest_read(page).await?;
        page.locator("#version-select")
            .select_option("0.1.0", None)
            .await
            .context("pick 0.1.0")?;
        expect_page(page)
            .to_have_url(&format!("{origin}/pyle/v0.1.0/"))
            .await
            .context("the browser goes to the snapshot's folder")?;
        // The page there is the build made as 0.1.0, which picks itself.
        expect(page.locator("#version-select"))
            .to_have_value("0.1.0")
            .await
            .context("the snapshot's bar shows 0.1.0")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_build_being_viewed_stays_picked_when_the_manifest_arrives() -> Result<()> {
    with_site("/pyle/v0.1.0/", "", async |page: &Page, _| {
        manifest_read(page).await?;
        expect(page.locator("#version-select"))
            .to_have_value("0.1.0")
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
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn a_page_the_browser_brings_back_shows_its_own_build_again() -> Result<()> {
    with_site("/pyle/v0.2.0/", "", async |page: &Page, _| {
        manifest_read(page).await?;
        let menu = page.locator("#version-select");
        // Setting the value from a script fires no `change`, so nothing
        // navigates.
        let left_on: String = menu
            .evaluate(
                "(menu) => { menu.value = '0.1.0'; return menu.value; }",
                None::<()>,
            )
            .await
            .context("leave the menu on 0.1.0")?;
        ensure!(left_on == "0.1.0", "the menu was left on {left_on:?}");
        page.evaluate_expression(
            "window.dispatchEvent(new PageTransitionEvent('pageshow', { persisted: true }))",
        )
        .await
        .context("send the event a restore sends")?;
        expect(menu)
            .to_have_value("0.2.0")
            .await
            .context("the menu is back on the build being viewed")
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
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_bar_is_never_in_the_page_without_its_styles() -> Result<()> {
    with_site(
        "/pyle/dev/",
        WATCH_FOR_AN_UNSTYLED_BAR,
        async |page: &Page, _| {
            manifest_read(page).await?;
            let unstyled: bool = page
                .evaluate("window.__pyleBarUnstyled", None::<&()>)
                .await
                .context("read what the observer saw")?;
            ensure!(!unstyled, "the bar was in the page before pyle's styles");
            Ok(())
        },
    )
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_page_is_the_theme_s_page_color() -> Result<()> {
    with_site("/pyle/v0.2.0/", "", async |page: &Page, _| {
        expect(page.locator(".pyle-page"))
            .to_have_css("background-color", PAGE)
            .await
            .context("the page is the default theme's page color")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_browser_s_margin_around_the_page_is_gone() -> Result<()> {
    with_site("/pyle/v0.2.0/", "", async |page: &Page, _| {
        expect(page.locator("body"))
            .to_have_css("margin-top", "0px")
            .await
            .context("the body that holds the page has no margin")
    })
    .await
}

/// The browser paints the window with the body's color wherever the page
/// does not reach: past its ends on a scroll, and behind a phone's toolbars.
#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_window_behind_the_page_is_the_page_s_color() -> Result<()> {
    with_site("/pyle/v0.2.0/", "", async |page: &Page, _| {
        expect(page.locator("body"))
            .to_have_css("background-color", PAGE)
            .await
            .context("the body that holds the page has its color")
    })
    .await
}

/// The default theme has a dark palette, so the scrollbars and the menu's
/// list, which the browser draws, are told they may go dark with it.
#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn the_browser_s_own_parts_follow_the_system_on_a_page_with_a_dark_palette() -> Result<()> {
    with_site("/pyle/v0.2.0/", "", async |page: &Page, _| {
        expect(page.locator("html"))
            .to_have_css("color-scheme", "light dark")
            .await
            .context("the root's scheme is light or dark")
    })
    .await
}

#[tokio::test]
#[ignore = "needs the built site and Chromium; run with --run-ignored all after ./scripts/site.sh"]
async fn a_visitor_who_asks_for_dark_gets_the_dark_palette() -> Result<()> {
    with_site("/pyle/v0.2.0/", "", async |page: &Page, _| {
        expect(page.locator("#version-switcher"))
            .to_have_css("background-color", SURFACE)
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
            .to_have_css("background-color", DARK_SURFACE)
            .await
            .context("the bar moves to the dark surface")?;
        expect(page.locator(".pyle-page"))
            .to_have_css("background-color", DARK_PAGE)
            .await
            .context("the page moves to the dark page color")
    })
    .await
}
