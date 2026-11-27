//! The theme playground: every field of a [`Theme`] as a control, over the
//! components that read it, with the theme written out as the Rust a site
//! pastes in. Compiled only under the `stories` feature.

use leptos::prelude::*;
use theoria::{Bounded, Choice, Color, Story, story};

use crate::components::gallery_stories::{StorySite, sample_sections};
use crate::theme::DEFAULT;
use crate::{Palette, PyleStyles, Theme};

/// The radius control, in rem, starting at the default theme's radius.
fn radius_control() -> Bounded {
    Bounded::new(0.25).min(0.0).max(1.5).step(0.0625)
}

/// The font control: a few stacks, starting at the default theme's.
fn font_control() -> Choice<&'static str> {
    Choice::labelled([
        ("System sans", DEFAULT.font),
        ("Serif", "ui-serif, Georgia, serif"),
        ("Monospace", "ui-monospace, monospace"),
    ])
}

/// `text` as the `&'static str` a `Theme` field holds, so that a site's
/// theme can be a `const`. Each edit of the field leaks its new text, a few
/// bytes, for as long as the gallery's tab is open.
fn held(text: impl Fn() -> String + Send + Sync + 'static) -> Memo<&'static str> {
    Memo::new(move |_| &*text().leak())
}

/// A look to design by hand: each control is one field of the `Theme`. It
/// starts as the default theme's light palette, and draws a page of pyle's
/// sections with it. An edit writes the stylesheet again and the theme's
/// Rust above the page, and redraws nothing else.
///
/// The page is drawn on `page`. The bar is on an older release, the state
/// that draws with every field: the bar on `surface`, the label in
/// `text_muted`, the menu on `surface_strong` with `border` and `radius`,
/// the notice in `accent`, and its link in `accent_hover` under the pointer.
/// The hero's title is in `text`, its tagline in `text_muted`, and its
/// links in `accent`.
///
/// `radius` is in rem. The theme has no `dark` palette, so the system's
/// setting changes nothing here. A site that follows the system designs
/// two palettes, one at a time, and puts the second in `dark`.
#[story(
    name = "Theme/Playground",
    page = DEFAULT.palette.page,
    surface = DEFAULT.palette.surface,
    surface_strong = DEFAULT.palette.surface_strong,
    border = DEFAULT.palette.border,
    text = DEFAULT.palette.text,
    text_muted = DEFAULT.palette.text_muted,
    accent = DEFAULT.palette.accent,
    accent_hover = DEFAULT.palette.accent_hover,
    radius = radius_control(),
    font = font_control()
)]
fn playground(
    page: Signal<Color>,
    surface: Signal<Color>,
    surface_strong: Signal<Color>,
    border: Signal<Color>,
    text: Signal<Color>,
    text_muted: Signal<Color>,
    accent: Signal<Color>,
    accent_hover: Signal<Color>,
    radius: Signal<Bounded>,
    font: Signal<Choice<&'static str>>,
) -> impl IntoView {
    let color = |field: Signal<Color>| held(move || field.get().to_string());
    let [
        page,
        surface,
        surface_strong,
        border,
        text,
        text_muted,
        accent,
        accent_hover,
    ] = [
        page,
        surface,
        surface_strong,
        border,
        text,
        text_muted,
        accent,
        accent_hover,
    ]
    .map(color);
    let radius = held(move || rem(radius.get().value()));
    let theme = Signal::derive(move || Theme {
        palette: Palette {
            page: page.get(),
            surface: surface.get(),
            surface_strong: surface_strong.get(),
            border: border.get(),
            text: text.get(),
            text_muted: text_muted.get(),
            accent: accent.get(),
            accent_hover: accent_hover.get(),
        },
        dark: None,
        radius: radius.get(),
        font: font.get().value(),
    });
    designing(theme)
}

/// A length of `value` rem, as the CSS a `Theme`'s `radius` holds.
fn rem(value: f64) -> String {
    format!("{value}rem")
}

/// The page the playground draws for `theme`: the stylesheet, the theme as
/// Rust, and pyle's sections. Only the stylesheet and the Rust read
/// `theme`, so a change to it redraws nothing else.
fn designing(theme: Signal<Theme>) -> impl IntoView {
    let StorySite {
        prefix, dev_notice, ..
    } = StorySite::default();
    view! {
        <PyleStyles theme />
        // `{:#?}` of a `Theme` is the Rust that builds it. It goes above the
        // page, which is at least as tall as the window.
        <pre id="theme-source" style="margin: 0; padding: 1rem 1.5rem; font: 0.8125rem ui-monospace, monospace">
            {move || format!("{:#?}", theme.get())}
        </pre>
        {sample_sections(prefix, dev_notice)}
    }
}

pub fn stories() -> Vec<Story> {
    vec![playground()]
}

#[cfg(test)]
mod tests {
    use leptotes::Dom;

    use super::*;
    use crate::components::test_support::{ONE_LOOK, styles_of, stylesheets};

    /// The playground's page, built for the default theme and then given
    /// `ONE_LOOK` before it is drawn: what it draws with is read from the theme
    /// when drawn, not when built.
    fn drawn_with_one_look() -> Dom {
        leptotes::dom(|| {
            let theme = RwSignal::new(DEFAULT);
            let view = designing(theme.into());
            theme.set(ONE_LOOK);
            view
        })
    }

    #[test]
    fn the_radius_starts_as_the_default_theme_s() {
        assert_eq!(rem(radius_control().value()), DEFAULT.radius);
    }

    #[test]
    fn the_font_starts_as_the_default_theme_s() {
        assert_eq!(font_control().value(), DEFAULT.font);
    }

    #[test]
    fn the_page_is_styled_by_the_theme_s_stylesheet_alone() {
        let dom = drawn_with_one_look();
        assert_eq!(stylesheets(&dom), [styles_of(&ONE_LOOK)], "{dom}");
    }

    #[test]
    fn the_rust_above_the_page_builds_the_theme_the_page_is_drawn_with() {
        let dom = drawn_with_one_look();
        assert_eq!(
            dom.find("#theme-source").text_content(),
            format!("{ONE_LOOK:#?}"),
            "{dom}"
        );
    }
}
