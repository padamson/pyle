//! What every browser test needs: a server for a folder, and a page in a
//! Chromium that is closed whether or not the test passed.

#![allow(dead_code, reason = "each test binary uses some of these")]

use std::path::PathBuf;

use anyhow::{Context, Result};
use axum::Router;
use playwright_rs::protocol::{Page, Playwright};

/// The repository's `target/`, where the gallery and the site are built.
pub fn target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target")
}

/// Serves `app` on a free port of 127.0.0.1 for as long as the test runs,
/// and returns its origin.
pub async fn serve(app: Router) -> Result<String> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind the server")?;
    let origin = format!("http://{}", listener.local_addr().context("local addr")?);
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    Ok(origin)
}

/// Runs `body` on a new page in Chromium, with `init` run in the page
/// before any of its own scripts, and closes the browser whether or not
/// `body` passed.
pub async fn with_page(init: &str, body: impl AsyncFnOnce(&Page) -> Result<()>) -> Result<()> {
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let result = async {
        let page = browser.new_page().await.context("new page")?;
        if !init.is_empty() {
            page.add_init_script(init)
                .await
                .context("add the init script")?;
        }
        body(&page).await
    }
    .await;
    let closed = browser.close().await.context("close browser");
    result.and(closed)
}
