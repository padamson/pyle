//! The components, each as `<name>.rs` beside its rules (`<name>.css`), its
//! theoria stories (`<name>.stories.rs`, under the `stories` feature), and
//! its leptotes tests (`<name>.tests.rs`). The dotted names are not module
//! names, so each is declared with `#[path]`. `gallery.stories.rs` holds
//! what the stories share and gathers them.

mod hero;
mod landing_page;
mod version_bar;

pub use hero::Hero;
pub use landing_page::LandingPage;
pub use version_bar::{DEFAULT_DEV_NOTICE, VersionBar, VersionSwitcher};

/// Every component's rules, for the one stylesheet.
pub(crate) const CSS: &[&str] = &[landing_page::CSS, version_bar::CSS, hero::CSS];

/// A component's own class, then the site's `extra` classes after it.
fn classes(own: &str, extra: &str) -> String {
    format!("{own} {extra}").trim_end().to_string()
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::collections::BTreeSet;

    use leptotes::Dom;

    use crate::{Palette, Theme};

    /// A palette unlike the default's in every color.
    pub(crate) const PLAIN: Palette = Palette {
        page: "p",
        surface: "s",
        surface_strong: "ss",
        border: "b",
        text: "t",
        text_muted: "tm",
        accent: "a",
        accent_hover: "ah",
    };

    /// A theme unlike the default in every field, with no dark palette.
    pub(crate) const ONE_LOOK: Theme = Theme {
        palette: PLAIN,
        dark: None,
        radius: "2px",
        font: "serif",
    };

    /// The `<style>` element `PyleStyles` draws for `theme`.
    pub(crate) fn styles_of(theme: &Theme) -> String {
        format!(
            r#"<style id="pyle-styles">{}</style>"#,
            crate::stylesheet(theme)
        )
    }

    /// How many `<style>` elements `html` holds, and how many of them are
    /// `theme`'s.
    pub(crate) fn stylesheets(html: &str, theme: &Theme) -> (usize, usize) {
        (
            leptotes::count(html, "<style"),
            leptotes::count(html, &styles_of(theme)),
        )
    }

    /// Every class an element of `dom` carries that has no rule in pyle's stylesheet. A
    /// class renamed in a component and not in its rules leaves that part
    /// unstyled, and no other native test would say so.
    pub(crate) fn unruled_classes(dom: &Dom) -> Vec<String> {
        let css = crate::stylesheet(&crate::Theme::default());
        let classes: BTreeSet<&str> = dom
            .attrs("[class]", "class")
            .into_iter()
            .flat_map(str::split_whitespace)
            .collect();
        classes
            .into_iter()
            .filter(|class| !css.contains(&format!(".{class} {{")))
            .map(str::to_string)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::classes;

    #[test]
    fn a_component_with_no_classes_from_the_site_has_only_its_own() {
        assert_eq!(classes("pyle-hero", ""), "pyle-hero");
    }

    #[test]
    fn the_site_s_classes_follow_the_component_s_own() {
        assert_eq!(classes("pyle-hero", "mt-8 wide"), "pyle-hero mt-8 wide");
    }
}

#[cfg(any(test, feature = "stories"))]
#[path = "gallery.stories.rs"]
pub(crate) mod gallery_stories;

#[cfg(any(test, feature = "stories"))]
#[path = "version_bar.stories.rs"]
mod version_bar_stories;

#[cfg(any(test, feature = "stories"))]
#[path = "hero.stories.rs"]
mod hero_stories;

#[cfg(any(test, feature = "stories"))]
#[path = "landing_page.stories.rs"]
mod landing_page_stories;

#[cfg(any(test, feature = "stories"))]
pub use gallery_stories::{StorySite, stories, stories_for};
