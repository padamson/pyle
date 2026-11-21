//! What every component's stories share, and the two galleries they are
//! gathered into: pyle's own, and a site's. Compiled only under the
//! `stories` feature.

use leptos::prelude::*;
use theoria::Stories;

use super::{DEFAULT_DEV_NOTICE, version_bar_stories};
use crate::{PyleStyles, Theme};

/// A site that mounts pyle's components, for stories that show them as the
/// site's visitors see them. A site's own gallery passes one to
/// [`stories_for`], built from the theme it mounts.
///
/// The default is a site that has chosen nothing: the default theme, light
/// or dark with the system, over the browser's own page color. Build one
/// with `..StorySite::default()`, so that a field a later component adds
/// does not break the site's gallery.
#[derive(Clone, Copy, Debug)]
pub struct StorySite {
    /// The path the site is served under, as [`pyle_core::Prefix::new`]
    /// takes it.
    pub prefix: &'static str,
    /// The CSS color of the page behind the components.
    pub page: &'static str,
    /// What the version bar says on main's build.
    pub dev_notice: &'static str,
    /// The site's look.
    pub theme: Theme,
}

impl Default for StorySite {
    fn default() -> Self {
        Self {
            prefix: "/my-site",
            page: "Canvas",
            dev_notice: DEFAULT_DEV_NOTICE,
            theme: Theme::default(),
        }
    }
}

/// pyle's styles under `theme`, then `children` over a page of the color
/// `page`: the frame every story draws its component in.
pub(crate) fn framed(theme: Theme, page: &str, children: impl IntoView) -> impl IntoView {
    view! {
        <PyleStyles theme />
        <div style=format!("background: {page}; min-height: 7rem")>{children}</div>
    }
}

/// Every component's stories, for pyle's own theoria gallery.
pub fn stories() -> Stories {
    version_bar_stories::stories()
        .into_iter()
        .chain(crate::theme::stories::stories())
        .collect()
}

/// Every component in each state a visitor can see it in, as `site` shows
/// it, for the site's own theoria gallery:
///
/// ```
/// use pyle::{StorySite, Theme};
///
/// pub fn all_stories() -> theoria::Stories {
///     pyle::stories_for(StorySite {
///         prefix: "/cqa-grader",
///         page: "#1b2a24",
///         // The `Theme` the site hands to `PyleStyles`.
///         theme: Theme::default(),
///         ..StorySite::default()
///     })
/// }
/// # let names: Vec<&str> = all_stories().into_iter().map(|story| story.name()).collect();
/// # assert_eq!(names.len(), 5, "one story a state: {names:?}");
/// # assert!(names.iter().all(|name| name.starts_with("Version bar/")), "{names:?}");
/// ```
///
/// Chain the site's own stories onto it:
/// `pyle::stories_for(site).into_iter().chain(my_stories()).collect()`.
pub fn stories_for(site: StorySite) -> Stories {
    version_bar_stories::for_site(site).into()
}
