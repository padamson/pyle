//! The frame a landing page is built in: the page's background, text color,
//! and font from the theme, with the page's sections stacked in it in the
//! order the site gives them.

use leptos::prelude::*;

use super::classes;

/// The frame's rules, which [`crate::stylesheet`] puts in `@layer pyle`.
/// One reaches outside pyle's own elements, to a `<body>` that holds the
/// page, mounted on the body or inside an element of the site's. It zeroes
/// the browser's default margin, which would otherwise frame the page in
/// white, and gives the body the page's color, which the browser paints the
/// rest of the window with: what shows past the page's ends on a scroll, and
/// what a phone's toolbars take their color from. A body that holds no pyle
/// page is left alone.
pub(crate) const CSS: &str = include_str!("landing_page.css");

/// A landing page: the sections passed as children, top to bottom, on the
/// theme's page color.
///
/// It lays out nothing inside a section. Each section sets its own width
/// and spacing, so a section can run the full width of the page (the
/// version bar does) or sit in a column (the hero does). The crate's
/// documentation shows a page built with it.
#[component]
pub fn LandingPage(
    /// The page's sections, top to bottom.
    children: Children,
    /// Classes for the page, after pyle's own.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    view! { <div class=classes("pyle-page", &class)>{children()}</div> }
}

#[cfg(test)]
#[path = "landing_page.tests.rs"]
mod tests;
