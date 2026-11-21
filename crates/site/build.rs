//! Passes `SITE_VERSION` (the snapshot this build is: `dev`, or a release
//! version with no `v`) to the page, `dev` when unset, so a plain build
//! and `trunk serve` need no setup. A value that is not a snapshot fails
//! the build here, where the page would only panic in the browser.

use pyle_core::Snapshot;

fn main() {
    println!("cargo:rerun-if-env-changed=SITE_VERSION");
    let version = std::env::var("SITE_VERSION").unwrap_or_else(|_| "dev".to_string());
    if let Err(err) = version.parse::<Snapshot>() {
        panic!(
            "SITE_VERSION={version:?} is not a snapshot (dev, or a version such as 0.1.0): {err}"
        );
    }
    println!("cargo:rustc-env=SITE_VERSION={version}");
}
