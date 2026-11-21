//! The components, each as `<name>.rs` beside its rules (`<name>.css`), its
//! theoria stories (`<name>.stories.rs`, under the `stories` feature), and
//! its leptotes tests (`<name>.tests.rs`). The dotted names are not module
//! names, so each is declared with `#[path]`. `gallery.stories.rs` holds
//! what the stories share and gathers them.

mod version_bar;

pub use version_bar::{DEFAULT_DEV_NOTICE, VersionBar, VersionSwitcher};

/// Every component's rules, for the one stylesheet.
pub(crate) const CSS: &[&str] = &[version_bar::CSS];

#[cfg(feature = "stories")]
#[path = "gallery.stories.rs"]
pub(crate) mod gallery_stories;

#[cfg(feature = "stories")]
#[path = "version_bar.stories.rs"]
mod version_bar_stories;

#[cfg(feature = "stories")]
pub use gallery_stories::{StorySite, stories, stories_for};
