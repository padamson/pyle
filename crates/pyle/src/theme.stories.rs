//! The theme playground: every field of a [`Theme`] as a control, over the
//! components that read it, with the theme written out as the Rust a site
//! pastes in. Compiled only under the `stories` feature.

use leptos::prelude::*;
use theoria::{Story, story};

use crate::components::gallery_stories::{StorySite, sample_page};
use crate::theme::DEFAULT;
use crate::{Palette, Theme};

/// A `Theme`'s fields are `&'static str`, so that a site's theme can be a
/// `const`. A control's text is leaked to become one. theoria runs the
/// whole story again on an edit to any control, so each edit leaks all
/// ten fields, a few hundred bytes, for as long as the gallery's tab is
/// open.
fn held(value: String) -> &'static str {
    value.leak()
}

/// A look to design by hand: each control is one field of the `Theme`, as
/// the CSS it is written into the stylesheet as. It starts as the default
/// theme's light palette, and draws a page of pyle's sections with it.
///
/// The page is drawn on `page`. The bar is on an older release, the state
/// that draws with every field: the bar on `surface`, the label in
/// `text_muted`, the menu on `surface_strong` with `border` and `radius`,
/// the notice in `accent`, and its link in `accent_hover` under the pointer.
/// The hero's title is in `text`, its tagline in `text_muted`, and its
/// links in `accent`.
///
/// The theme has no `dark` palette, so the system's setting changes
/// nothing here. A site that follows the system designs two palettes, one
/// at a time, and puts the second in `dark`.
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
    radius = DEFAULT.radius,
    font = DEFAULT.font
)]
fn playground(
    page: String,
    surface: String,
    surface_strong: String,
    border: String,
    text: String,
    text_muted: String,
    accent: String,
    accent_hover: String,
    radius: String,
    font: String,
) -> impl IntoView {
    let theme = Theme {
        palette: Palette {
            page: held(page),
            surface: held(surface),
            surface_strong: held(surface_strong),
            border: held(border),
            text: held(text),
            text_muted: held(text_muted),
            accent: held(accent),
            accent_hover: held(accent_hover),
        },
        dark: None,
        radius: held(radius),
        font: held(font),
    };
    view! {
        // `{:#?}` of a `Theme` is the Rust that builds it. It goes above the
        // page, which is at least as tall as the window.
        <pre id="theme-source" style="margin: 0; padding: 1rem 1.5rem; font: 0.8125rem ui-monospace, monospace">
            {format!("{theme:#?}")}
        </pre>
        {sample_page(StorySite {
            theme,
            ..StorySite::default()
        })}
    }
}

pub fn stories() -> Vec<Story> {
    vec![playground()]
}
