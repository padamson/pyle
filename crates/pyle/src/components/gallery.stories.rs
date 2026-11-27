//! What every component's stories share, and the two galleries they are
//! gathered into: pyle's own, and a site's. Compiled only under the
//! `stories` feature.

use leptos::prelude::*;
use pyle_core::{Manifest, Prefix, Snapshot, Version};
use theoria::Stories;

use super::{
    DEFAULT_DEV_NOTICE, Hero, LandingPage, hero_stories, landing_page_stories, version_bar_stories,
};
use crate::{PyleStyles, Theme};

/// A site that mounts pyle's components, for stories that show them as the
/// site's visitors see them. A site's own gallery passes one to
/// [`stories_for`], built from the theme it mounts.
///
/// The default is a site that has chosen nothing: the default theme, light
/// or dark with the system. Build one with `..StorySite::default()`, so
/// that a field a later component adds does not break the site's gallery.
#[derive(Clone, Copy, Debug)]
pub struct StorySite {
    /// The path the site is served under, as [`pyle_core::Prefix::new`]
    /// takes it.
    pub prefix: &'static str,
    /// What the version bar says on main's build.
    pub dev_notice: &'static str,
    /// The site's look.
    pub theme: Theme,
}

impl Default for StorySite {
    fn default() -> Self {
        Self {
            prefix: "/my-site",
            dev_notice: DEFAULT_DEV_NOTICE,
            theme: Theme::default(),
        }
    }
}

/// pyle's styles under `theme`, then `children` on the theme's page color:
/// the frame every story draws a section in. A story of a whole page uses
/// [`crate::LandingPage`] instead.
pub(crate) fn framed(theme: Theme, children: impl IntoView) -> impl IntoView {
    view! {
        <PyleStyles theme />
        <div style="min-height: 7rem; background: var(--pyle-page); color: var(--pyle-text); font-family: var(--pyle-font)">
            {children}
        </div>
    }
}

/// A whole page as `site` shows it, under the site's theme: the page story.
pub(crate) fn sample_page(site: StorySite) -> impl IntoView {
    view! {
        <PyleStyles theme=site.theme />
        {sample_sections(site.prefix, site.dev_notice)}
    }
}

/// pyle's sections in the order a site would give them, for a site served
/// under `prefix` whose bar says `dev_notice` on main's build, with no
/// stylesheet of their own: what [`sample_page`] and the theme playground
/// draw. A section added to pyle joins it here.
///
/// The bar starts on an older release, the state that draws with the most
/// of the theme, and stays in the gallery as [`version_bar_stories`] has it.
pub(crate) fn sample_sections(prefix: &'static str, dev_notice: &'static str) -> impl IntoView {
    let older = Version::new(0, 14, 0);
    let newest = Version::new(0, 15, 0);
    let bar = version_bar_stories::staying_bar(
        RwSignal::new(Snapshot::Release(older)),
        Prefix::new(prefix),
        Some(Manifest::new([older, newest])),
        dev_notice.to_string().into(),
        String::new().into(),
    );
    view! {
        <LandingPage>
            {bar}
            <Hero title="Project name" tagline="What the project does, in a line or two.">
                <a href="#source">"Source"</a>
                <a href="#docs">"Docs"</a>
            </Hero>
        </LandingPage>
    }
    // Erased, as the bar's own stories are, for rustc's query depth limit.
    .into_any()
}

/// Every component's stories, for pyle's own theoria gallery.
pub fn stories() -> Stories {
    landing_page_stories::for_site(StorySite::default())
        .into_iter()
        .chain(version_bar_stories::stories())
        .chain(hero_stories::for_site(StorySite::default()))
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
///         // The `Theme` the site hands to `PyleStyles`.
///         theme: Theme::default(),
///         ..StorySite::default()
///     })
/// }
/// # let names: Vec<&str> = all_stories().into_iter().map(|story| story.name()).collect();
/// # let under = |group: &str| names.iter().filter(|name| name.starts_with(group)).count();
/// # assert_eq!(
/// #     (under("Landing page/"), under("Version bar/"), under("Hero/"), names.len()),
/// #     (1, 5, 2, 8),
/// #     "{names:?}"
/// # );
/// ```
///
/// Chain the site's own stories onto it:
/// `pyle::stories_for(site).into_iter().chain(my_stories()).collect()`.
pub fn stories_for(site: StorySite) -> Stories {
    landing_page_stories::for_site(site)
        .into_iter()
        .chain(version_bar_stories::for_site(site))
        .chain(hero_stories::for_site(site))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::{ONE_LOOK, styles_of, stylesheets};

    #[test]
    fn the_page_story_is_styled_by_the_site_s_theme_alone() {
        let dom = leptotes::dom(|| {
            sample_page(StorySite {
                theme: ONE_LOOK,
                ..StorySite::default()
            })
        });
        assert_eq!(stylesheets(&dom), [styles_of(&ONE_LOOK)], "{dom}");
    }
}
