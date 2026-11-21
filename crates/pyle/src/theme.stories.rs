//! The theme playground: every field of a [`Theme`] as a control, over the
//! components that read it, with the theme written out as the Rust a site
//! pastes in. Compiled only under the `stories` feature.

use leptos::prelude::*;
use pyle_core::{Manifest, Prefix, Snapshot, Version};
use theoria::{Story, story};

use crate::components::gallery_stories::framed;
use crate::theme::DEFAULT;
use crate::{Palette, Theme, VersionBar};

/// A `Theme`'s fields are `&'static str`, so that a site's theme can be a
/// `const`. A control's text is leaked to become one. theoria runs the
/// whole story again on an edit to any control, so each edit leaks all
/// nine fields, a few hundred bytes, for as long as the gallery's tab is
/// open.
fn held(value: String) -> &'static str {
    value.leak()
}

/// A look to design by hand: each control is one field of the `Theme`, as
/// the CSS it is written into the stylesheet as, and `page` is the color of
/// the site's page behind the components. It starts as the default theme's
/// light palette.
///
/// The bar is on an older release, the state that draws with every field:
/// the label in `text_muted`, the menu on `surface_strong` with `border` and
/// `radius`, the notice in `accent`, and its link in `accent_hover` under
/// the pointer.
///
/// The theme has no `dark` palette, so the system's setting changes
/// nothing here. A site that follows the system designs two palettes, one
/// at a time, and puts the second in `dark`.
#[story(
    name = "Theme/Playground",
    page = "#ffffff",
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
    let older = Version::new(0, 14, 0);
    let newest = Version::new(0, 15, 0);
    view! {
        {framed(
            theme,
            &page,
            view! {
                <VersionBar
                    current=Snapshot::Release(older)
                    prefix=Prefix::new("/my-site")
                    manifest=Signal::stored(Some(Manifest::new([older, newest])))
                    // Picking a version stays in the playground.
                    on_select=Callback::new(|_: Snapshot| ())
                />
            },
        )}
        // `{:#?}` of a `Theme` is the Rust that builds it.
        <pre id="theme-source" style="margin: 0; padding: 1rem 1.5rem; font: 0.8125rem ui-monospace, monospace">
            {format!("{theme:#?}")}
        </pre>
    }
}

pub fn stories() -> Vec<Story> {
    vec![playground()]
}
