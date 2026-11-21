//! pyle's own landing page, built only from pyle's components: the first
//! site to use them, and the one the browser tests in crates/pyle-e2e
//! deploy and drive.

use leptos::prelude::*;
use pyle::{Hero, LandingPage, Prefix, PyleStyles, Snapshot, VersionSwitcher};

/// The snapshot this build is, from `SITE_VERSION` at build time.
const SITE_VERSION: &str = env!("SITE_VERSION");

/// Where the site is served: GitHub project Pages for padamson/pyle.
const PREFIX: &str = "/pyle";

#[component]
fn App() -> impl IntoView {
    let current: Snapshot = SITE_VERSION.parse().expect("build.rs checked SITE_VERSION");
    view! {
        <PyleStyles />
        <LandingPage>
            <VersionSwitcher current prefix=Prefix::new(PREFIX) />
            <Hero
                title="pyle"
                tagline="The Leptos components my project landing pages are built from, so a fix lands once for every site."
            >
                <a href="https://github.com/padamson/pyle">"Source"</a>
            </Hero>
        </LandingPage>
    }
}

fn main() {
    leptos::mount::mount_to_body(App);
}
