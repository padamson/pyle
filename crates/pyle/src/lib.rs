//! The Leptos components my project landing pages are built from, so a fix
//! lands once for every site.
//!
//! A site mounts [`PyleStyles`] once with its [`Theme`], then builds its
//! page from pyle's sections inside a [`LandingPage`], in the order it
//! wants them:
//!
//! ```
//! use leptos::prelude::*;
//! use pyle::{Hero, LandingPage, Prefix, PyleStyles, Snapshot, Theme, VersionSwitcher};
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     let current: Snapshot = "0.15.0".parse().expect("the build's SITE_VERSION");
//!     view! {
//!         <PyleStyles theme=Theme::default() />
//!         <LandingPage>
//!             <VersionSwitcher current prefix=Prefix::new("/cqa-grader") />
//!             <Hero title="cqa-grader" tagline="What it does, in a line.">
//!                 <a href="https://github.com/padamson/cqa-grader">"Source"</a>
//!             </Hero>
//!         </LandingPage>
//!     }
//! }
//! ```
//!
//! pyle turns on no Leptos runtime. The site picks `csr`, `ssr`, or
//! `hydrate` through the feature of the same name.
//!
//! Styling is pyle's own CSS, in `@layer pyle`, with every class named
//! `pyle-*` and every value a `--pyle-*` variable the [`Theme`] sets. A
//! site's stylesheet ranks the layer below its utilities, so a class passed
//! to a component's `class` wins.

mod components;
mod theme;
mod versions;

pub use components::{DEFAULT_DEV_NOTICE, Hero, LandingPage, VersionBar, VersionSwitcher};
pub use pyle_core::{Manifest, Prefix, Snapshot, Version};
pub use theme::{Palette, PyleStyles, Theme, stylesheet};
pub use versions::{Notice, VersionOption, notice, version_options};

/// Every component's stories, for pyle's own theoria gallery
/// (`theoria.toml`).
#[cfg(feature = "stories")]
pub use components::stories as all_stories;
#[cfg(feature = "stories")]
pub use components::{StorySite, stories_for};
